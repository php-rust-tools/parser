use crate::expect_token;
use crate::expected_token_err;
use crate::lexer::error::SyntaxError;
use crate::lexer::token::DocStringIndentationKind;
use crate::lexer::token::TokenKind;
use crate::lexer::DocStringKind;

use crate::parser::ast::identifiers::DynamicIdentifier;
use crate::parser::ast::identifiers::Identifier;
use crate::parser::ast::identifiers::SimpleIdentifier;
use crate::parser::ast::variables::Variable;
use crate::parser::ast::StringPart;
use crate::parser::ast::{Expression, IncludeKind, MagicConst};
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::internal::arrays;
use crate::parser::internal::attributes;
use crate::parser::internal::classes;
use crate::parser::internal::control_flow;
use crate::parser::internal::functions;
use crate::parser::internal::identifiers;
use crate::parser::internal::parameters;
use crate::parser::internal::precedences::Associativity;
use crate::parser::internal::precedences::Precedence;
use crate::parser::internal::utils;
use crate::parser::internal::variables;
use crate::parser::state::State;

use super::ast::operators::ArithmeticOperation;
use super::ast::operators::AssignmentOperation;
use super::ast::operators::BitwiseOperation;
use super::ast::operators::ComparisonOperation;
use super::ast::operators::LogicalOperation;

pub fn lowest_precedence(state: &mut State) -> ParseResult<Expression> {
    for_precedence(state, Precedence::Lowest)
}

pub fn null_coalesce_precedence(state: &mut State) -> ParseResult<Expression> {
    for_precedence(state, Precedence::NullCoalesce)
}

pub fn clone_or_new_precedence(state: &mut State) -> ParseResult<Expression> {
    for_precedence(state, Precedence::CloneOrNew)
}

fn for_precedence(state: &mut State, precedence: Precedence) -> ParseResult<Expression> {
    let mut left = create(state)?;

    if state.current.kind == TokenKind::SemiColon {
        return Ok(left);
    }

    state.skip_comments();

    loop {
        state.skip_comments();

        if matches!(state.current.kind, TokenKind::SemiColon | TokenKind::Eof) {
            break;
        }

        let span = state.current.span;
        let kind = state.current.kind.clone();

        if is_postfix(&kind) {
            let lpred = Precedence::postfix(&kind);

            if lpred < precedence {
                break;
            }

            left = postfix(state, left, &kind)?;
            continue;
        }

        if is_infix(&kind) {
            let rpred = Precedence::infix(&kind);

            if rpred < precedence {
                break;
            }

            if rpred == precedence && matches!(rpred.associativity(), Some(Associativity::Left)) {
                break;
            }

            if rpred == precedence && matches!(rpred.associativity(), Some(Associativity::Non)) {
                return Err(ParseError::UnexpectedToken(kind.to_string(), span));
            }

            state.next();
            state.skip_comments();

            left =
                match kind {
                    TokenKind::Question => {
                        if state.current.kind == TokenKind::Colon {
                            utils::skip_colon(state)?;
                            let r#else = lowest_precedence(state)?;

                            Expression::ShortTernary {
                                condition: Box::new(left),
                                span,
                                r#else: Box::new(r#else),
                            }
                        } else {
                            let then = lowest_precedence(state)?;
                            utils::skip_colon(state)?;
                            let r#else = lowest_precedence(state)?;

                            Expression::Ternary {
                                condition: Box::new(left),
                                then: Box::new(then),
                                r#else: Box::new(r#else),
                            }
                        }
                    }
                    TokenKind::QuestionColon => {
                        let r#else = lowest_precedence(state)?;
                        Expression::ShortTernary {
                            condition: Box::new(left),
                            span,
                            r#else: Box::new(r#else),
                        }
                    }
                    TokenKind::Equals if state.current.kind == TokenKind::Ampersand => {
                        let amper_span = state.current.span;
                        state.next();

                        // FIXME: You should only be allowed to assign a referencable variable,
                        //        here, not any old expression.
                        let right = Box::new(for_precedence(state, rpred)?);

                        Expression::AssignmentOperation(AssignmentOperation::Assign {
                            left: Box::new(left),
                            span,
                            right: Box::new(Expression::Reference {
                                span: amper_span,
                                right,
                            }),
                        })
                    }
                    TokenKind::Instanceof if state.current.kind == TokenKind::Self_ => {
                        if !state.has_class_scope {
                            return Err(ParseError::CannotFindTypeInCurrentScope(
                                state.current.kind.to_string(),
                                state.current.span,
                            ));
                        }

                        state.next();

                        Expression::Instanceof {
                            left: Box::new(left),
                            span,
                            right: Box::new(Expression::Self_),
                        }
                    }
                    TokenKind::Instanceof if state.current.kind == TokenKind::Parent => {
                        if !state.has_class_scope {
                            return Err(ParseError::CannotFindTypeInCurrentScope(
                                state.current.kind.to_string(),
                                state.current.span,
                            ));
                        }

                        state.next();

                        Expression::Instanceof {
                            left: Box::new(left),
                            span,
                            right: Box::new(Expression::Parent),
                        }
                    }
                    TokenKind::Instanceof if state.current.kind == TokenKind::Static => {
                        if !state.has_class_scope {
                            return Err(ParseError::CannotFindTypeInCurrentScope(
                                state.current.kind.to_string(),
                                state.current.span,
                            ));
                        }

                        state.next();

                        Expression::Instanceof {
                            left: Box::new(left),
                            span,
                            right: Box::new(Expression::Static),
                        }
                    }
                    TokenKind::Instanceof if state.current.kind == TokenKind::Enum => {
                        let enum_span = state.current.span;
                        state.next();

                        Expression::Instanceof {
                            left: Box::new(left),
                            span,
                            right: Box::new(Expression::Identifier(Identifier::SimpleIdentifier(
                                SimpleIdentifier {
                                    span: enum_span,
                                    name: "enum".into(),
                                },
                            ))),
                        }
                    }
                    TokenKind::Instanceof if state.current.kind == TokenKind::From => {
                        let from_span = state.current.span;
                        state.next();

                        Expression::Instanceof {
                            left: Box::new(left),
                            span,
                            right: Box::new(Expression::Identifier(Identifier::SimpleIdentifier(
                                SimpleIdentifier {
                                    span: from_span,
                                    name: "from".into(),
                                },
                            ))),
                        }
                    }
                    _ => {
                        let left = Box::new(left);
                        let right = Box::new(for_precedence(state, rpred)?);

                        match kind {
                            TokenKind::Plus => {
                                Expression::ArithmeticOperation(ArithmeticOperation::Addition {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::Minus => {
                                Expression::ArithmeticOperation(ArithmeticOperation::Subtraction {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::Asterisk => Expression::ArithmeticOperation(
                                ArithmeticOperation::Multiplication { left, span, right },
                            ),
                            TokenKind::Slash => {
                                Expression::ArithmeticOperation(ArithmeticOperation::Division {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::Percent => {
                                Expression::ArithmeticOperation(ArithmeticOperation::Modulo {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::Pow => Expression::ArithmeticOperation(
                                ArithmeticOperation::Exponentiation { left, span, right },
                            ),
                            TokenKind::Equals => {
                                Expression::AssignmentOperation(AssignmentOperation::Assign {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::PlusEquals => {
                                Expression::AssignmentOperation(AssignmentOperation::Addition {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::MinusEquals => {
                                Expression::AssignmentOperation(AssignmentOperation::Subtraction {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::AsteriskEqual => Expression::AssignmentOperation(
                                AssignmentOperation::Multiplication { left, span, right },
                            ),
                            TokenKind::SlashEquals => {
                                Expression::AssignmentOperation(AssignmentOperation::Division {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::PercentEquals => {
                                Expression::AssignmentOperation(AssignmentOperation::Modulo {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::PowEquals => Expression::AssignmentOperation(
                                AssignmentOperation::Exponentiation { left, span, right },
                            ),
                            TokenKind::AmpersandEquals => {
                                Expression::AssignmentOperation(AssignmentOperation::BitwiseAnd {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::PipeEquals => {
                                Expression::AssignmentOperation(AssignmentOperation::BitwiseOr {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::CaretEquals => {
                                Expression::AssignmentOperation(AssignmentOperation::BitwiseXor {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::LeftShiftEquals => {
                                Expression::AssignmentOperation(AssignmentOperation::LeftShift {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::RightShiftEquals => {
                                Expression::AssignmentOperation(AssignmentOperation::RightShift {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::CoalesceEqual => {
                                Expression::AssignmentOperation(AssignmentOperation::Coalesce {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::DotEquals => {
                                Expression::AssignmentOperation(AssignmentOperation::Concat {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::Ampersand => {
                                Expression::BitwiseOperation(BitwiseOperation::And {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::Pipe => Expression::BitwiseOperation(BitwiseOperation::Or {
                                left,
                                span,
                                right,
                            }),
                            TokenKind::Caret => {
                                Expression::BitwiseOperation(BitwiseOperation::Xor {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::LeftShift => {
                                Expression::BitwiseOperation(BitwiseOperation::LeftShift {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::RightShift => {
                                Expression::BitwiseOperation(BitwiseOperation::RightShift {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::DoubleEquals => {
                                Expression::ComparisonOperation(ComparisonOperation::Equal {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::TripleEquals => {
                                Expression::ComparisonOperation(ComparisonOperation::Identical {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::BangEquals => {
                                Expression::ComparisonOperation(ComparisonOperation::NotEqual {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::AngledLeftRight => Expression::ComparisonOperation(
                                ComparisonOperation::AngledNotEqual { left, span, right },
                            ),
                            TokenKind::BangDoubleEquals => {
                                Expression::ComparisonOperation(ComparisonOperation::NotIdentical {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::LessThan => {
                                Expression::ComparisonOperation(ComparisonOperation::LessThan {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::GreaterThan => {
                                Expression::ComparisonOperation(ComparisonOperation::GreaterThan {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::LessThanEquals => Expression::ComparisonOperation(
                                ComparisonOperation::LessThanOrEqual { left, span, right },
                            ),
                            TokenKind::GreaterThanEquals => Expression::ComparisonOperation(
                                ComparisonOperation::GreaterThanOrEqual { left, span, right },
                            ),
                            TokenKind::Spaceship => {
                                Expression::ComparisonOperation(ComparisonOperation::Spaceship {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::BooleanAnd => {
                                Expression::LogicalOperation(LogicalOperation::And {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::BooleanOr => {
                                Expression::LogicalOperation(LogicalOperation::Or {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::LogicalAnd => {
                                Expression::LogicalOperation(LogicalOperation::LogicalAnd {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::LogicalOr => {
                                Expression::LogicalOperation(LogicalOperation::LogicalOr {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::LogicalXor => {
                                Expression::LogicalOperation(LogicalOperation::LogicalXor {
                                    left,
                                    span,
                                    right,
                                })
                            }
                            TokenKind::Dot => Expression::Concat { left, span, right },
                            TokenKind::Instanceof => Expression::Instanceof { left, span, right },
                            _ => todo!(),
                        }
                    }
                };

            continue;
        }

        break;
    }

    state.skip_comments();

    Ok(left)
}

fn create(state: &mut State) -> ParseResult<Expression> {
    if state.is_eof() {
        return Err(ParseError::UnexpectedEndOfFile);
    }

    attributes(state)
}

macro_rules! expressions {
    ($(#[before($else:ident), current($(|)? $( $current:pat_param )|+) $(, peek($(|)? $( $peek:pat_param )|+))?] $expr:ident($out:expr))+) => {
        $(
            #[inline(never)]
            fn $expr(state: &mut State) -> ParseResult<Expression> {
                state.skip_comments();

                match &state.current.kind {
                    $( $current )|+ $( if matches!(&state.peek.kind, $( $peek )|+ ))? => $out(state),
                    _ => $else(state),
                }
            }
        )+
    };
}

expressions! {
    #[before(static_arrow_function), current(TokenKind::Attribute)]
    attributes(|state: &mut State| {
        attributes::gather_attributes(state)?;

        match &state.current.kind {
            TokenKind::Static if state.peek.kind == TokenKind::Function => {
                functions::anonymous_function(state)
            }
            TokenKind::Static if state.peek.kind == TokenKind::Fn => {
                functions::arrow_function(state)
            }
            TokenKind::Function => functions::anonymous_function(state),
            TokenKind::Fn => functions::arrow_function(state),
            _ => {
                // Note, we can get attributes and know their span, maybe use that in the
                // error in the future?
                Err(ParseError::ExpectedItemDefinitionAfterAttributes(
                    state.current.span,
                ))
            }
        }
    })

    #[before(static_anonymous_function), current(TokenKind::Static), peek(TokenKind::Fn)]
    static_arrow_function(|state: &mut State| {
        functions::arrow_function(state)
    })

    #[before(arrow_function), current(TokenKind::Static), peek(TokenKind::Function)]
    static_anonymous_function(|state: &mut State| {
        functions::anonymous_function(state)
    })

    #[before(anonymous_function), current(TokenKind::Fn)]
    arrow_function(|state: &mut State| {
        functions::arrow_function(state)
    })

    #[before(reserved_identifier_function_call), current(TokenKind::Function)]
    anonymous_function(|state: &mut State| {
        functions::anonymous_function(state)
    })

    #[before(reserved_identifier_static_call), current(
        | TokenKind::True       | TokenKind::False | TokenKind::Null
        | TokenKind::Readonly   | TokenKind::Self_ | TokenKind::Parent
        | TokenKind::Enum       | TokenKind::From
    ), peek(TokenKind::LeftParen)]
    reserved_identifier_function_call(|state: &mut State| {
        let ident = identifiers::identifier_maybe_soft_reserved(state)?;
        let lhs = Expression::Identifier(Identifier::SimpleIdentifier(ident));

        postfix(state, lhs, &TokenKind::LeftParen)
    })

    #[before(list), current(TokenKind::Enum | TokenKind::From), peek(TokenKind::DoubleColon)]
    reserved_identifier_static_call(|state: &mut State| {
        let ident = identifiers::type_identifier(state)?;
        let lhs = Expression::Identifier(Identifier::SimpleIdentifier(ident));

        postfix(state, lhs, &TokenKind::DoubleColon)
    })

    #[before(anonymous_class), current(TokenKind::List)]
    list(|state: &mut State| {
        arrays::list_expression(state)
    })

    #[before(throw), current(TokenKind::New), peek(TokenKind::Class | TokenKind::Attribute)]
    anonymous_class(|state: &mut State| {
        classes::parse_anonymous(state)
    })

    #[before(r#yield), current(TokenKind::Throw)]
    throw(|state: &mut State| {

        state.next();

        // TODO(azjezz): we start parsing from anynomous class here, because we know that
        // the right expression can't be an anonymous function, or a list.
        // however, there's many other things that it can't be.
        let value = anonymous_class(state)?;

        Ok(Expression::Throw{
            value: Box::new(value)
        })
    })

    #[before(clone), current(TokenKind::Yield)]
    r#yield(|state: &mut State| {
        state.next();

        if state.current.kind == TokenKind::SemiColon {
            Ok(Expression::Yield {
                key: None,
                value: None,
            })
        } else {
            let mut from = false;

            if state.current.kind == TokenKind::From {
                state.next();
                from = true;
            }

            let mut key = None;
            let mut value = Box::new(for_precedence(
                state,
                if from {
                    Precedence::YieldFrom
                } else {
                    Precedence::Yield
                },
            )?);

            if state.current.kind == TokenKind::DoubleArrow && !from {
                state.next();
                key = Some(value.clone());
                value = Box::new(for_precedence(state, Precedence::Yield)?);
            }

            if from {
                Ok(Expression::YieldFrom { value })
            } else {
                Ok(Expression::Yield {
                    key,
                    value: Some(value),
                })
            }
        }
    })

    #[before(r#true), current(TokenKind::Clone)]
    clone(|state: &mut State| {
        state.next();

        let target = for_precedence(state, Precedence::CloneOrNew)?;

        Ok(Expression::Clone {
            target: Box::new(target),
        })
    })

    #[before(r#false), current(TokenKind::True)]
    r#true(|state: &mut State| {
        state.next();

        Ok(Expression::Bool { value: true })
    })

    #[before(null), current(TokenKind::False)]
    r#false(|state: &mut State| {
        state.next();

        Ok(Expression::Bool { value: false })
    })

    #[before(literal_integer), current(TokenKind::Null)]
    null(|state: &mut State| {
        state.next();

        Ok(Expression::Null)
    })

    #[before(literal_float), current(TokenKind::LiteralInteger(_))]
    literal_integer(|state: &mut State| {
        if let TokenKind::LiteralInteger(i) = &state.current.kind {
            let e = Expression::LiteralInteger { i: i.clone() };
            state.next();

            Ok(e)
        } else {
            unreachable!("{}:{}", file!(), line!());
        }
    })

    #[before(literal_string), current(TokenKind::LiteralFloat(_))]
    literal_float(|state: &mut State| {
        if let TokenKind::LiteralFloat(f) = &state.current.kind {
            let e = Expression::LiteralFloat { f: f.clone() };

            state.next();

            Ok(e)
        } else {
            unreachable!("{}:{}", file!(), line!());
        }
    })

    #[before(string_part), current(TokenKind::LiteralString(_))]
    literal_string(|state: &mut State| {
        if let TokenKind::LiteralString(value) = &state.current.kind {
            let e = Expression::LiteralString { value: value.clone() };
            state.next();

            Ok(e)
        } else {
            unreachable!("{}:{}", file!(), line!());
        }
    })

    #[before(start_doc_string), current(TokenKind::StringPart(_))]
    string_part(|state: &mut State| {
        interpolated_string(state)
    })

    #[before(backtick), current(TokenKind::StartDocString(_, _))]
    start_doc_string(|state: &mut State| {
        if let TokenKind::StartDocString(_, kind) = &state.current.kind {
            let kind = *kind;

            doc_string(state, kind)
        } else {
            unreachable!("{}:{}", file!(), line!());
        }
    })

    #[before(identifier), current(TokenKind::Backtick)]
    backtick(|state: &mut State| {
        shell_exec(state)
    })

    #[before(static_postfix), current(TokenKind::Identifier(_) | TokenKind::QualifiedIdentifier(_) | TokenKind::FullyQualifiedIdentifier(_))]
    identifier(|state: &mut State| {
        Ok(Expression::Identifier(Identifier::SimpleIdentifier(identifiers::full_name(state)?)))
    })

    #[before(self_identifier), current(TokenKind::Static)]
    static_postfix(|state: &mut State| {
        state.next();

        postfix(state, Expression::Static, &TokenKind::DoubleColon)
    })

    #[before(parent_identifier), current(TokenKind::Self_)]
    self_identifier(|state: &mut State| {
        let span = state.current.span;
        state.next();

        Ok(Expression::Identifier(Identifier::SimpleIdentifier( SimpleIdentifier {
            span,
            name: "self".into()
        })))
    })

    #[before(left_parenthesis), current(TokenKind::Parent)]
    parent_identifier(|state: &mut State| {
        let span = state.current.span;
        state.next();

        Ok(Expression::Identifier(Identifier::SimpleIdentifier( SimpleIdentifier {
            span,
            name: "parent".into()
        })))
    })

    #[before(r#match), current(TokenKind::LeftParen)]
    left_parenthesis(|state: &mut State| {
        let start = state.current.span;
        state.next();

        let expr = lowest_precedence(state)?;

        let end = utils::skip_right_parenthesis(state)?;

        Ok(Expression::Parenthesized { start, expr: Box::new(expr), end })
    })

    #[before(array), current(TokenKind::Match)]
    r#match(|state: &mut State| {
        control_flow::match_expression(state)
    })

    #[before(left_bracket), current(TokenKind::Array)]
    array(|state: &mut State| {
        arrays::legacy_array_expression(state)
    })

    #[before(new), current(TokenKind::LeftBracket)]
    left_bracket(|state: &mut State| {
        arrays::array_expression(state)
    })

    #[before(directory_magic_constant), current(TokenKind::New)]
    new(|state: &mut State| {
        let span = state.current.span;

        state.next();

        let target = match state.current.kind {
            TokenKind::Self_ => {
                state.next();

                Expression::Self_
            }
            TokenKind::Static => {
                state.next();

                Expression::Static
            }
            TokenKind::Parent => {
                state.next();

                Expression::Parent
            }
            TokenKind::Enum => {
                let span = state.current.span;

                state.next();

                Expression::Identifier(Identifier::SimpleIdentifier(SimpleIdentifier { span, name: "enum".into() }))
            }
            TokenKind::From => {
                let span = state.current.span;

                state.next();

                Expression::Identifier(Identifier::SimpleIdentifier(SimpleIdentifier { span, name: "from".into() }))
            }
            _ => clone_or_new_precedence(state)?,
        };

        let mut args = vec![];
        if state.current.kind == TokenKind::LeftParen {
            args = parameters::args_list(state)?;
        }

        Ok(Expression::New {
            target: Box::new(target),
            span,
            args,
        })
    })

    #[before(file_magic_constant), current(TokenKind::DirConstant)]
    directory_magic_constant(|state: &mut State| {
        let span = state.current.span;
        state.next();

        Ok(Expression::MagicConst {
            span,
            constant: MagicConst::Directory
        })
    })

    #[before(line_magic_constant), current(TokenKind::FileConstant)]
    file_magic_constant(|state: &mut State| {
        let span = state.current.span;
        state.next();

        Ok(Expression::MagicConst {
            span,
            constant: MagicConst::File
        })
    })

    #[before(function_magic_constant), current(TokenKind::LineConstant)]
    line_magic_constant(|state: &mut State| {
        let span = state.current.span;
        state.next();

        Ok(Expression::MagicConst {
            span,
            constant: MagicConst::Line
        })
    })

    #[before(class_magic_constant), current(TokenKind::FunctionConstant)]
    function_magic_constant(|state: &mut State| {
        let span = state.current.span;
        state.next();

        Ok(Expression::MagicConst {
            span,
            constant: MagicConst::Function,
        })
    })

    #[before(method_magic_constant), current(TokenKind::ClassConstant)]
    class_magic_constant(|state: &mut State| {
        let span = state.current.span;
        state.next();

        Ok(Expression::MagicConst {
            span,
            constant: MagicConst::Class,
        })
    })

    #[before(namespace_magic_constant), current(TokenKind::MethodConstant)]
    method_magic_constant(|state: &mut State| {
        let span = state.current.span;
        state.next();

        Ok(Expression::MagicConst {
            span,
            constant: MagicConst::Method,
        })
    })

    #[before(trait_magic_constant), current(TokenKind::NamespaceConstant)]
    namespace_magic_constant(|state: &mut State| {
        let span = state.current.span;
        state.next();

        Ok(Expression::MagicConst {
            span,
            constant: MagicConst::Namespace,
        })
    })

    #[before(include), current(TokenKind::TraitConstant)]
    trait_magic_constant(|state: &mut State| {
        let span = state.current.span;
        state.next();

        Ok(Expression::MagicConst {
            span,
            constant: MagicConst::Trait
        })
    })

    #[before(cast_prefix), current(TokenKind::Include | TokenKind::IncludeOnce | TokenKind::Require | TokenKind::RequireOnce)]
    include(|state: &mut State| {
        let kind: IncludeKind = (&state.current.kind).into();
        let span = state.current.span;

        state.next();

        let path = lowest_precedence(state)?;

        Ok(Expression::Include {
            span,
            kind,
            path:Box::new(path)
        })
    })

    #[before(numeric_prefix), current(
        | TokenKind::StringCast     | TokenKind::BinaryCast     | TokenKind::ObjectCast
        | TokenKind::BoolCast       | TokenKind::BooleanCast    | TokenKind::IntCast
        | TokenKind::IntegerCast    | TokenKind::FloatCast      | TokenKind::DoubleCast
        | TokenKind::RealCast       | TokenKind::UnsetCast      | TokenKind::ArrayCast
    )]
    cast_prefix(|state: &mut State| {
        let span = state.current.span;
        let kind = state.current.kind.clone().into();

        state.next();

        let rhs = for_precedence(state, Precedence::Prefix)?;

        Ok(Expression::Cast {
            span,
            kind,
            value: Box::new(rhs),
        })
    })

    #[before(bang_prefix), current(TokenKind::Decrement | TokenKind::Increment | TokenKind::Minus | TokenKind::Plus)]
    numeric_prefix(|state: &mut State| {
        let span = state.current.span;
        let op = state.current.kind.clone();

        state.next();

        let right = Box::new(for_precedence(state, Precedence::Prefix)?);
        let expr = match op {
            TokenKind::Minus => Expression::ArithmeticOperation(ArithmeticOperation::Negation { span, right }),
            TokenKind::Plus => Expression::ArithmeticOperation(ArithmeticOperation::Identity { span, right }),
            TokenKind::Decrement => Expression::ArithmeticOperation(ArithmeticOperation::PreDecrement { span, right }),
            TokenKind::Increment => Expression::ArithmeticOperation(ArithmeticOperation::PreIncrement { span, right }),
            _ => unreachable!(),
        };

        Ok(expr)
    })

    #[before(at_prefix), current(TokenKind::Bang)]
    bang_prefix(|state: &mut State| {
        let span = state.current.span;

        state.next();

        let rhs = for_precedence(state, Precedence::Bang)?;

        Ok(Expression::LogicalOperation(LogicalOperation::Not {
            span,
            right: Box::new(rhs)
        }))
    })

    #[before(print_prefix), current(TokenKind::At)]
    at_prefix(|state: &mut State| {
        let span = state.current.span;

        state.next();

        let rhs = for_precedence(state, Precedence::Prefix)?;

        Ok(Expression::ErrorSuppress {
            span,
            expr: Box::new(rhs)
        })
    })

    #[before(bitwise_prefix), current(TokenKind::Print)]
    print_prefix(|state: &mut State| {
        let span = state.current.span;

        state.next();

        let rhs = for_precedence(state, Precedence::Prefix)?;

        Ok(Expression::Print {
            span,
            value: Box::new(rhs)
        })
    })

    #[before(variable), current(TokenKind::BitwiseNot)]
    bitwise_prefix(|state: &mut State| {
        let span = state.current.span;

        state.next();

        let right = Box::new(for_precedence(state, Precedence::Prefix)?);

        Ok(Expression::BitwiseOperation(BitwiseOperation::Not { span, right }))
    })

    #[before(unexpected_token), current(TokenKind::Dollar | TokenKind::DollarLeftBrace | TokenKind::Variable(_))]
    variable(|state: &mut State| {
        Ok(Expression::Variable(variables::dynamic_variable(state)?))
    })
}

fn unexpected_token(state: &mut State) -> ParseResult<Expression> {
    Err(ParseError::UnexpectedToken(
        state.current.kind.to_string(),
        state.current.span,
    ))
}

fn postfix(state: &mut State, lhs: Expression, op: &TokenKind) -> Result<Expression, ParseError> {
    Ok(match op {
        TokenKind::Coalesce => {
            state.next();
            state.skip_comments();

            let rhs = null_coalesce_precedence(state)?;

            Expression::Coalesce {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }
        }
        TokenKind::LeftParen => {
            state.skip_comments();
            let args = parameters::args_list(state)?;

            Expression::Call {
                target: Box::new(lhs),
                args,
            }
        }
        TokenKind::LeftBracket => {
            utils::skip_left_bracket(state)?;
            state.skip_comments();

            if state.current.kind == TokenKind::RightBracket {
                state.next();

                Expression::ArrayIndex {
                    array: Box::new(lhs),
                    index: None,
                }
            } else {
                let index = lowest_precedence(state)?;

                utils::skip_right_bracket(state)?;

                Expression::ArrayIndex {
                    array: Box::new(lhs),
                    index: Some(Box::new(index)),
                }
            }
        }
        TokenKind::DoubleColon => {
            utils::skip_double_colon(state)?;
            state.skip_comments();

            let mut must_be_method_call = false;

            let property = match state.current.kind.clone() {
                TokenKind::Variable(_) | TokenKind::Dollar | TokenKind::DollarLeftBrace => {
                    Expression::Variable(variables::dynamic_variable(state)?)
                }
                _ if identifiers::is_identifier_maybe_reserved(&state.current.kind) => {
                    Expression::Identifier(Identifier::SimpleIdentifier(
                        identifiers::identifier_maybe_reserved(state)?,
                    ))
                }
                TokenKind::LeftBrace => {
                    let start = state.current.span;
                    must_be_method_call = true;
                    state.next();

                    let name = lowest_precedence(state)?;

                    let end = utils::skip_right_brace(state)?;

                    Expression::Identifier(Identifier::DynamicIdentifier(DynamicIdentifier {
                        start,
                        expr: Box::new(name),
                        end,
                    }))
                }
                TokenKind::Class => {
                    let span = state.current.span;
                    state.next();

                    Expression::Identifier(Identifier::SimpleIdentifier(SimpleIdentifier {
                        span,
                        name: "class".into(),
                    }))
                }
                _ => {
                    return expected_token_err!(["`{`", "`$`", "an identifier"], state);
                }
            };

            let lhs = Box::new(lhs);

            match property {
                // 2. If the current token is a left paren, or if we know the property expression
                //    is only valid a method call context, we can assume we're parsing a static
                //    method call.
                _ if state.current.kind == TokenKind::LeftParen || must_be_method_call => {
                    let args = parameters::args_list(state)?;

                    Expression::StaticMethodCall {
                        target: lhs,
                        method: Box::new(property),
                        args,
                    }
                }
                // 1. If we have an identifier and the current token is not a left paren,
                //    the resulting expression must be a constant fetch.
                Expression::Identifier(Identifier::SimpleIdentifier(identifier)) => {
                    Expression::ConstFetch {
                        target: lhs,
                        constant: identifier,
                    }
                }
                // 3. If we haven't met any of the previous conditions, we can assume
                //    that we're parsing a static property fetch.
                _ => Expression::StaticPropertyFetch {
                    target: lhs,
                    property: Box::new(property),
                },
            }
        }
        TokenKind::Arrow | TokenKind::NullsafeArrow => {
            state.next();
            state.skip_comments();

            let property = match state.current.kind {
                TokenKind::Variable(_) | TokenKind::Dollar | TokenKind::DollarLeftBrace => {
                    Expression::Variable(variables::dynamic_variable(state)?)
                }
                _ if identifiers::is_identifier_maybe_reserved(&state.current.kind) => {
                    Expression::Identifier(Identifier::SimpleIdentifier(
                        identifiers::identifier_maybe_reserved(state)?,
                    ))
                }
                TokenKind::LeftBrace => {
                    let start = state.current.span;
                    state.next();

                    let name = lowest_precedence(state)?;

                    let end = utils::skip_right_brace(state)?;

                    Expression::Identifier(Identifier::DynamicIdentifier(DynamicIdentifier {
                        start,
                        expr: Box::new(name),
                        end,
                    }))
                }
                _ => {
                    return expected_token_err!(["`{`", "`$`", "an identifier"], state);
                }
            };

            if state.current.kind == TokenKind::LeftParen {
                let args = parameters::args_list(state)?;

                if op == &TokenKind::NullsafeArrow {
                    Expression::NullsafeMethodCall {
                        target: Box::new(lhs),
                        method: Box::new(property),
                        args,
                    }
                } else {
                    Expression::MethodCall {
                        target: Box::new(lhs),
                        method: Box::new(property),
                        args,
                    }
                }
            } else if op == &TokenKind::NullsafeArrow {
                Expression::NullsafePropertyFetch {
                    target: Box::new(lhs),
                    property: Box::new(property),
                }
            } else {
                Expression::PropertyFetch {
                    target: Box::new(lhs),
                    property: Box::new(property),
                }
            }
        }
        TokenKind::Increment => {
            let span = state.current.span;
            state.next();
            state.skip_comments();

            Expression::ArithmeticOperation(ArithmeticOperation::PostIncrement {
                left: Box::new(lhs),
                span,
            })
        }
        TokenKind::Decrement => {
            let span = state.current.span;
            state.next();
            state.skip_comments();

            Expression::ArithmeticOperation(ArithmeticOperation::PostDecrement {
                left: Box::new(lhs),
                span,
            })
        }
        _ => todo!("postfix: {:?}", op),
    })
}

#[inline(always)]
fn interpolated_string(state: &mut State) -> ParseResult<Expression> {
    let mut parts = Vec::new();

    while state.current.kind != TokenKind::DoubleQuote {
        if let Some(part) = interpolated_string_part(state)? {
            parts.push(part);
        }
    }

    state.next();

    Ok(Expression::InterpolatedString { parts })
}

#[inline(always)]
fn shell_exec(state: &mut State) -> ParseResult<Expression> {
    state.next();

    let mut parts = Vec::new();

    while state.current.kind != TokenKind::Backtick {
        if let Some(part) = interpolated_string_part(state)? {
            parts.push(part);
        }
    }

    state.next();

    Ok(Expression::ShellExec { parts })
}

#[inline(always)]
fn doc_string(state: &mut State, kind: DocStringKind) -> ParseResult<Expression> {
    let span = state.current.span;
    state.next();

    Ok(match kind {
        DocStringKind::Heredoc => {
            let mut parts = Vec::new();

            while !matches!(state.current.kind, TokenKind::EndDocString(_, _, _)) {
                if let Some(part) = interpolated_string_part(state)? {
                    parts.push(part);
                }
            }

            let (indentation_type, indentation_amount) = match state.current.kind {
                TokenKind::EndDocString(_, indentation_type, indentation_amount) => {
                    (indentation_type, indentation_amount)
                }
                _ => unreachable!(),
            };

            state.next();

            let mut new_line = true;
            if indentation_type != DocStringIndentationKind::None {
                let indentation_char: u8 = indentation_type.into();

                for part in parts.iter_mut() {
                    // We only need to strip and validate indentation
                    // for individual lines, so we can skip checks if
                    // we know we're not on a new line.
                    if !new_line {
                        continue;
                    }

                    match part {
                        StringPart::Const(bytes) => {
                            // 1. If this line doesn't start with any whitespace,
                            //    we can return an error early because we know
                            //    the label was indented.
                            if !bytes.starts_with(&[b' ']) && !bytes.starts_with(&[b'\t']) {
                                return Err(ParseError::SyntaxError(
                                    SyntaxError::InvalidDocBodyIndentationLevel(
                                        indentation_amount,
                                        span,
                                    ),
                                ));
                            }

                            // 2. If this line doesn't start with the correct
                            //    type of whitespace, we can also return an error.
                            if !bytes.starts_with(&[indentation_char]) {
                                return Err(ParseError::SyntaxError(
                                    SyntaxError::InvalidDocIndentation(span),
                                ));
                            }

                            // 3. We now know that the whitespace at the start of
                            //    this line is correct, so we need to check that the
                            //    amount of whitespace is correct too. In this case,
                            //    the amount of whitespace just needs to be at least
                            //    the same, so we can create a vector containing the
                            //    minimum and check using `starts_with()`.
                            let expected_whitespace_buffer =
                                vec![indentation_char; indentation_amount];
                            if !bytes.starts_with(&expected_whitespace_buffer) {
                                return Err(ParseError::SyntaxError(
                                    SyntaxError::InvalidDocBodyIndentationLevel(
                                        indentation_amount,
                                        span,
                                    ),
                                ));
                            }

                            // 4. All of the above checks have passed, so we know
                            //    there are no more possible errors. Let's now
                            //    strip the leading whitespace accordingly.
                            *bytes = bytes
                                .strip_prefix(&expected_whitespace_buffer[..])
                                .unwrap()
                                .into();
                            new_line = bytes.ends_with(&[b'\n']);
                        }
                        _ => continue,
                    }
                }
            }

            Expression::Heredoc { parts }
        }
        DocStringKind::Nowdoc => {
            let mut string_part = expect_token!([
                TokenKind::StringPart(s) => s,
            ], state, "constant string");

            let (indentation_type, indentation_amount) = match state.current.kind {
                TokenKind::EndDocString(_, indentation_type, indentation_amount) => {
                    (indentation_type, indentation_amount)
                }
                _ => unreachable!(),
            };

            state.next();

            if indentation_type != DocStringIndentationKind::None {
                let indentation_char: u8 = indentation_type.into();

                let mut lines = string_part
                    .split(|b| *b == b'\n')
                    .map(|s| s.to_vec())
                    .collect::<Vec<Vec<u8>>>();

                for line in lines.iter_mut() {
                    if line.is_empty() {
                        continue;
                    }

                    // 1. If this line doesn't start with any whitespace,
                    //    we can return an error early because we know
                    //    the label was indented.
                    if !line.starts_with(&[b' ']) && !line.starts_with(&[b'\t']) {
                        return Err(ParseError::SyntaxError(
                            SyntaxError::InvalidDocBodyIndentationLevel(indentation_amount, span),
                        ));
                    }

                    // 2. If this line doesn't start with the correct
                    //    type of whitespace, we can also return an error.
                    if !line.starts_with(&[indentation_char]) {
                        return Err(ParseError::SyntaxError(SyntaxError::InvalidDocIndentation(
                            span,
                        )));
                    }

                    // 3. We now know that the whitespace at the start of
                    //    this line is correct, so we need to check that the
                    //    amount of whitespace is correct too. In this case,
                    //    the amount of whitespace just needs to be at least
                    //    the same, so we can create a vector containing the
                    //    minimum and check using `starts_with()`.
                    let expected_whitespace_buffer = vec![indentation_char; indentation_amount];
                    if !line.starts_with(&expected_whitespace_buffer) {
                        return Err(ParseError::SyntaxError(
                            SyntaxError::InvalidDocBodyIndentationLevel(indentation_amount, span),
                        ));
                    }

                    // 4. All of the above checks have passed, so we know
                    //    there are no more possible errors. Let's now
                    //    strip the leading whitespace accordingly.
                    *line = line
                        .strip_prefix(&expected_whitespace_buffer[..])
                        .unwrap()
                        .into();
                }

                let mut bytes = Vec::new();
                for (i, line) in lines.iter().enumerate() {
                    bytes.extend(line);
                    if i < lines.len() - 1 {
                        bytes.push(b'\n');
                    }
                }
                string_part = bytes.into();
            }

            Expression::Nowdoc { value: string_part }
        }
    })
}

fn interpolated_string_part(state: &mut State) -> ParseResult<Option<StringPart>> {
    Ok(match &state.current.kind {
        TokenKind::StringPart(s) => {
            let part = if s.len() > 0 {
                Some(StringPart::Const(s.clone()))
            } else {
                None
            };

            state.next();
            part
        }
        TokenKind::DollarLeftBrace => {
            let variable = variables::dynamic_variable(state)?;

            Some(StringPart::Expr(Box::new(Expression::Variable(variable))))
        }
        TokenKind::LeftBrace => {
            // "{$expr}"
            state.next();
            let e = lowest_precedence(state)?;
            utils::skip_right_brace(state)?;
            Some(StringPart::Expr(Box::new(e)))
        }
        TokenKind::Variable(_) => {
            // "$expr", "$expr[0]", "$expr[name]", "$expr->a"
            let variable = Expression::Variable(variables::dynamic_variable(state)?);
            let e = match state.current.kind {
                TokenKind::LeftBracket => {
                    state.next();
                    // Full expression syntax is not allowed here,
                    // so we can't call expression.
                    let index = match &state.current.kind {
                        TokenKind::LiteralInteger(i) => {
                            let e = Expression::LiteralInteger { i: i.clone() };
                            state.next();
                            e
                        }
                        TokenKind::Minus => {
                            let span = state.current.span;
                            state.next();
                            if let TokenKind::LiteralInteger(i) = &state.current.kind {
                                let e = Expression::ArithmeticOperation(
                                    ArithmeticOperation::Negation {
                                        span,
                                        right: Box::new(Expression::LiteralInteger {
                                            i: i.clone(),
                                        }),
                                    },
                                );
                                state.next();
                                e
                            } else {
                                return expected_token_err!("an integer", state);
                            }
                        }
                        TokenKind::Identifier(ident) => {
                            let e = Expression::LiteralString {
                                value: ident.clone(),
                            };
                            state.next();
                            e
                        }
                        TokenKind::Variable(_) => Expression::Variable(Variable::SimpleVariable(
                            variables::simple_variable(state)?,
                        )),
                        _ => {
                            return expected_token_err!(
                                ["`-`", "an integer", "an identifier", "a variable"],
                                state
                            );
                        }
                    };

                    utils::skip_right_bracket(state)?;

                    Expression::ArrayIndex {
                        array: Box::new(variable),
                        index: Some(Box::new(index)),
                    }
                }
                TokenKind::Arrow => {
                    state.next();
                    Expression::PropertyFetch {
                        target: Box::new(variable),
                        property: Box::new(Expression::Identifier(Identifier::SimpleIdentifier(
                            identifiers::identifier_maybe_reserved(state)?,
                        ))),
                    }
                }
                TokenKind::NullsafeArrow => {
                    state.next();
                    Expression::NullsafePropertyFetch {
                        target: Box::new(variable),
                        property: Box::new(Expression::Identifier(Identifier::SimpleIdentifier(
                            identifiers::identifier_maybe_reserved(state)?,
                        ))),
                    }
                }
                _ => variable,
            };
            Some(StringPart::Expr(Box::new(e)))
        }
        _ => {
            return expected_token_err!(["`${`", "`{$", "`\"`", "a variable"], state);
        }
    })
}

fn is_infix(t: &TokenKind) -> bool {
    matches!(
        t,
        TokenKind::Pow
            | TokenKind::RightShiftEquals
            | TokenKind::LeftShiftEquals
            | TokenKind::CaretEquals
            | TokenKind::AmpersandEquals
            | TokenKind::PipeEquals
            | TokenKind::PercentEquals
            | TokenKind::PowEquals
            | TokenKind::LogicalAnd
            | TokenKind::LogicalOr
            | TokenKind::LogicalXor
            | TokenKind::Spaceship
            | TokenKind::LeftShift
            | TokenKind::RightShift
            | TokenKind::Ampersand
            | TokenKind::Pipe
            | TokenKind::Caret
            | TokenKind::Percent
            | TokenKind::Instanceof
            | TokenKind::Asterisk
            | TokenKind::Slash
            | TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Dot
            | TokenKind::LessThan
            | TokenKind::GreaterThan
            | TokenKind::LessThanEquals
            | TokenKind::GreaterThanEquals
            | TokenKind::DoubleEquals
            | TokenKind::TripleEquals
            | TokenKind::BangEquals
            | TokenKind::BangDoubleEquals
            | TokenKind::AngledLeftRight
            | TokenKind::Question
            | TokenKind::QuestionColon
            | TokenKind::BooleanAnd
            | TokenKind::BooleanOr
            | TokenKind::Equals
            | TokenKind::PlusEquals
            | TokenKind::MinusEquals
            | TokenKind::DotEquals
            | TokenKind::CoalesceEqual
            | TokenKind::AsteriskEqual
            | TokenKind::SlashEquals
    )
}

#[inline(always)]
fn is_postfix(t: &TokenKind) -> bool {
    matches!(
        t,
        TokenKind::Increment
            | TokenKind::Decrement
            | TokenKind::LeftParen
            | TokenKind::LeftBracket
            | TokenKind::Arrow
            | TokenKind::NullsafeArrow
            | TokenKind::DoubleColon
            | TokenKind::Coalesce
    )
}
