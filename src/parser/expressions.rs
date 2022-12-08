use crate::expect_token;
use crate::expected_token_err;
use crate::lexer::token::TokenKind;
use crate::lexer::DocStringKind;
use crate::parser::ast;
use crate::parser::ast::identifiers::Identifier;
use crate::parser::ast::variables::Variable;
use crate::parser::ast::StringPart;
use crate::parser::ast::{Expression, IncludeKind, MagicConst};
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::internal::arrays;
use crate::parser::internal::attributes;
use crate::parser::internal::classish;
use crate::parser::internal::control_flow;
use crate::parser::internal::functions;
use crate::parser::internal::identifiers;
use crate::parser::internal::parameters;
use crate::parser::internal::precedences::Associativity;
use crate::parser::internal::precedences::Precedence;
use crate::parser::internal::utils;
use crate::parser::internal::variables;
use crate::parser::state::State;

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

            match kind {
                TokenKind::Question => {
                    let then = lowest_precedence(state)?;
                    utils::skip_colon(state)?;
                    let otherwise = for_precedence(state, rpred)?;
                    left = Expression::Ternary {
                        condition: Box::new(left),
                        then: Some(Box::new(then)),
                        r#else: Box::new(otherwise),
                    }
                }
                TokenKind::QuestionColon => {
                    let r#else = lowest_precedence(state)?;
                    left = Expression::Ternary {
                        condition: Box::new(left),
                        then: None,
                        r#else: Box::new(r#else),
                    }
                }
                _ => {
                    // FIXME: Hacky, should probably be refactored.
                    left = match kind {
                        TokenKind::Equals if state.current.kind == TokenKind::Ampersand => {
                            state.next();
                            Expression::Infix {
                                lhs: Box::new(left),
                                op: ast::InfixOp::AssignRef,
                                rhs: Box::new(for_precedence(state, rpred)?),
                            }
                        }
                        TokenKind::Instanceof if state.current.kind == TokenKind::Self_ => {
                            if !state.has_class_scope {
                                return Err(ParseError::CannotFindTypeInCurrentScope(
                                    state.current.kind.to_string(),
                                    state.current.span,
                                ));
                            }

                            state.next();

                            Expression::Infix {
                                lhs: Box::new(left),
                                op: ast::InfixOp::Instanceof,
                                rhs: Box::new(Expression::Self_),
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

                            Expression::Infix {
                                lhs: Box::new(left),
                                op: ast::InfixOp::Instanceof,
                                rhs: Box::new(Expression::Parent),
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

                            Expression::Infix {
                                lhs: Box::new(left),
                                op: ast::InfixOp::Instanceof,
                                rhs: Box::new(Expression::Static),
                            }
                        }
                        _ => Expression::Infix {
                            lhs: Box::new(left),
                            op: kind.into(),
                            rhs: Box::new(for_precedence(state, rpred)?),
                        },
                    };
                }
            }

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

    #[before(list), current(TokenKind::Function)]
    anonymous_function(|state: &mut State| {
        functions::anonymous_function(state)
    })

    #[before(anonymous_class), current(TokenKind::List)]
    list(|state: &mut State| {
        arrays::list_expression(state)
    })

    #[before(throw), current(TokenKind::New), peek(TokenKind::Class | TokenKind::Attribute)]
    anonymous_class(|state: &mut State| {
        classish::anonymous_class_definition(state)
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

    #[before(variable), current(TokenKind::Clone)]
    clone(|state: &mut State| {
        state.next();

        let target = for_precedence(state, Precedence::CloneOrNew)?;

        Ok(Expression::Clone {
            target: Box::new(target),
        })
    })

    #[before(r#true), current(TokenKind::Variable(_))]
    variable(|state: &mut State| {
        Ok(Expression::Variable(
            identifiers::var(state)?
        ))
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

    #[before(self_postfix), current(TokenKind::Identifier(_) | TokenKind::QualifiedIdentifier(_) | TokenKind::FullyQualifiedIdentifier(_))]
    identifier(|state: &mut State| {
        Ok(Expression::Identifier(identifiers::full_name(state)?))
    })

    #[before(static_postfix), current(TokenKind::Self_)]
    self_postfix(|state: &mut State| {
        if !state.has_class_scope {
            return Err(ParseError::CannotFindTypeInCurrentScope(
                state.current.kind.to_string(),
                state.current.span,
            ));
        }

        state.next();

        postfix(state, Expression::Self_, &TokenKind::DoubleColon)
    })

    #[before(parent_postfix), current(TokenKind::Static)]
    static_postfix(|state: &mut State| {
        if !state.has_class_scope {
            return Err(ParseError::CannotFindTypeInCurrentScope(
                state.current.kind.to_string(),
                state.current.span,
            ));
        }

        state.next();

        postfix(state, Expression::Static, &TokenKind::DoubleColon)
    })

    #[before(left_parenthesis), current(TokenKind::Parent)]
    parent_postfix(|state: &mut State| {
        if !state.has_class_scope {
            return Err(ParseError::CannotFindTypeInCurrentScope(
                state.current.kind.to_string(),
                state.current.span,
            ));
        }

        state.next();

        postfix(state, Expression::Parent, &TokenKind::DoubleColon)
    })

    #[before(r#match), current(TokenKind::LeftParen)]
    left_parenthesis(|state: &mut State| {
        state.next();

        let e = lowest_precedence(state)?;

        utils::skip_right_parenthesis(state)?;

        Ok(e)
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
        state.next();
        let target = match state.current.kind {
            TokenKind::Self_ => {
                if !state.has_class_scope {
                    return Err(ParseError::CannotFindTypeInCurrentScope(
                        state.current.kind.to_string(),
                        state.current.span,
                    ));
                }

                state.next();

                Expression::Self_
            }
            TokenKind::Static => {
                if !state.has_class_scope {
                    return Err(ParseError::CannotFindTypeInCurrentScope(
                        state.current.kind.to_string(),
                        state.current.span,
                    ));
                }

                state.next();

                Expression::Static
            }
            TokenKind::Parent => {
                if !state.has_class_scope {
                    return Err(ParseError::CannotFindTypeInCurrentScope(
                        state.current.kind.to_string(),
                        state.current.span,
                    ));
                }

                state.next();

                Expression::Parent
            }
            _ => clone_or_new_precedence(state)?,
        };

        let mut args = vec![];
        if state.current.kind == TokenKind::LeftParen {
            args = parameters::args_list(state)?;
        }

        Ok(Expression::New{target:Box::new(target),args,})
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

        let rhs = for_precedence(state, Precedence::Prefix)?;

        let expr = match op {
            TokenKind::Minus => Expression::Negate {
                span,
                value: Box::new(rhs),
            },
            TokenKind::Plus => Expression::UnaryPlus {
                span,
                value: Box::new(rhs),
            },
            TokenKind::Decrement => Expression::PreDecrement {
                span,
                value: Box::new(rhs),
            },
            TokenKind::Increment => Expression::PreIncrement {
                span,
                value: Box::new(rhs),
            },
            _ => unreachable!(),
        };

        Ok(expr)
    })

    #[before(at_prefix), current(TokenKind::Bang)]
    bang_prefix(|state: &mut State| {
        let span = state.current.span;

        state.next();

        let rhs = for_precedence(state, Precedence::Bang)?;

        Ok(Expression::BooleanNot {
            span,
            value: Box::new(rhs)
        })
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

    #[before(dynamic_variable), current(TokenKind::BitwiseNot)]
    bitwise_prefix(|state: &mut State| {
        let span = state.current.span;

        state.next();

        let rhs = for_precedence(state, Precedence::Prefix)?;

        Ok(Expression::BitwiseNot {
            span,
            value: Box::new(rhs)
        })
    })

    #[before(unexpected_token), current(TokenKind::Dollar)]
    dynamic_variable(|state: &mut State| {
        variables::dynamic_variable(state)
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

            let rhs = null_coalesce_precedence(state)?;

            Expression::Coalesce {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }
        }
        TokenKind::LeftParen => {
            let args = parameters::args_list(state)?;

            Expression::Call {
                target: Box::new(lhs),
                args,
            }
        }
        TokenKind::LeftBracket => {
            utils::skip_left_bracket(state)?;

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

            let mut must_be_method_call = false;

            let property = match state.current.kind.clone() {
                TokenKind::Dollar => variables::dynamic_variable(state)?,
                TokenKind::Variable(_) => Expression::Variable(identifiers::var(state)?),
                TokenKind::Identifier(_) => Expression::Identifier(identifiers::ident(state)?),
                TokenKind::LeftBrace => {
                    must_be_method_call = true;
                    state.next();

                    let name = lowest_precedence(state)?;

                    utils::skip_right_brace(state)?;

                    Expression::DynamicVariable {
                        name: Box::new(name),
                    }
                }
                TokenKind::Class => {
                    let start = state.current.span;
                    state.next();
                    let end = state.current.span;

                    Expression::Identifier(Identifier {
                        start,
                        name: "class".into(),
                        end,
                    })
                }
                _ if identifiers::is_reserved_ident(&state.current.kind) => {
                    Expression::Identifier(identifiers::ident_maybe_reserved(state)?)
                }
                _ => {
                    return expected_token_err!(["`{`", "`$`", "an identifier"], state);
                }
            };

            let lhs = Box::new(lhs);

            match property {
                // 1. If we have an identifier and the current token is not a left paren,
                //    the resulting expression must be a constant fetch.
                Expression::Identifier(identifier)
                    if state.current.kind != TokenKind::LeftParen =>
                {
                    Expression::ConstFetch {
                        target: lhs,
                        constant: identifier,
                    }
                }
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

            let property = match state.current.kind {
                TokenKind::LeftBrace => {
                    utils::skip_left_brace(state)?;
                    let expr = lowest_precedence(state)?;
                    utils::skip_right_brace(state)?;
                    expr
                }
                TokenKind::Variable(_) => Expression::Variable(identifiers::var(state)?),
                TokenKind::Dollar => variables::dynamic_variable(state)?,
                _ => Expression::Identifier(identifiers::ident_maybe_reserved(state)?),
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
            state.next();
            Expression::Increment {
                value: Box::new(lhs),
            }
        }
        TokenKind::Decrement => {
            state.next();

            Expression::Decrement {
                value: Box::new(lhs),
            }
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

            // FIXME: Can we move this logic above into the loop, by peeking ahead in
            //        the token stream for the EndHeredoc? Might be more performant.
            if let Some(indentation_type) = indentation_type {
                let search_char: u8 = indentation_type.into();

                for part in parts.iter_mut() {
                    match part {
                        StringPart::Const(bytes) => {
                            for _ in 0..indentation_amount {
                                if bytes.starts_with(&[search_char]) {
                                    bytes.remove(0);
                                }
                            }
                        }
                        _ => continue,
                    }
                }
            }

            Expression::Heredoc { parts }
        }
        DocStringKind::Nowdoc => {
            // FIXME: This feels hacky. We should probably produce different tokens from the lexer
            //        but since I already had the logic in place for parsing heredocs, this was
            //        the fastest way to get nowdocs working too.
            let mut s = expect_token!([
                    TokenKind::StringPart(s) => s
                ], state, "constant string");

            let (indentation_type, indentation_amount) = expect_token!([
                    TokenKind::EndDocString(_, indentation_type, indentation_amount) => (indentation_type, indentation_amount)
                ], state, "label");

            // FIXME: Hacky code, but it's late and I want to get this done.
            if let Some(indentation_type) = indentation_type {
                let search_char: u8 = indentation_type.into();
                let mut lines = s
                    .split(|b| *b == b'\n')
                    .map(|s| s.to_vec())
                    .collect::<Vec<Vec<u8>>>();
                for line in lines.iter_mut() {
                    for _ in 0..indentation_amount {
                        if line.starts_with(&[search_char]) {
                            line.remove(0);
                        }
                    }
                }
                let mut bytes = Vec::new();
                for (i, line) in lines.iter().enumerate() {
                    bytes.extend(line);
                    if i < lines.len() - 1 {
                        bytes.push(b'\n');
                    }
                }
                s = bytes.into();
            }

            Expression::Nowdoc { value: s }
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
            state.next();
            let e = match (state.current.kind.clone(), state.peek.kind.clone()) {
                (TokenKind::Identifier(name), TokenKind::RightBrace) => {
                    let start = state.current.span;
                    let end = state.peek.span;

                    state.next();
                    state.next();
                    // "${var}"
                    // TODO: we should use a different node for this.
                    Expression::Variable(Variable { start, name, end })
                }
                (TokenKind::Identifier(name), TokenKind::LeftBracket) => {
                    let start = state.current.span;
                    let end = state.peek.span;
                    state.next();
                    state.next();
                    let var = Expression::Variable(Variable { start, name, end });

                    let e = lowest_precedence(state)?;
                    utils::skip_right_bracket(state)?;
                    utils::skip_right_brace(state)?;

                    // TODO: we should use a different node for this.
                    Expression::ArrayIndex {
                        array: Box::new(var),
                        index: Some(Box::new(e)),
                    }
                }
                _ => {
                    // Arbitrary expressions are allowed, but are treated as variable variables.
                    let e = lowest_precedence(state)?;
                    utils::skip_right_brace(state)?;

                    Expression::DynamicVariable { name: Box::new(e) }
                }
            };
            Some(StringPart::Expr(Box::new(e)))
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
            let var = Expression::Variable(identifiers::var(state)?);
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
                                let e = Expression::Negate {
                                    span,
                                    value: Box::new(Expression::LiteralInteger { i: i.clone() }),
                                };
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
                        TokenKind::Variable(_) => {
                            let v = identifiers::var(state)?;
                            Expression::Variable(v)
                        }
                        _ => {
                            return expected_token_err!(
                                ["`-`", "an integer", "an identifier", "a variable"],
                                state
                            );
                        }
                    };

                    utils::skip_right_bracket(state)?;

                    Expression::ArrayIndex {
                        array: Box::new(var),
                        index: Some(Box::new(index)),
                    }
                }
                TokenKind::Arrow => {
                    state.next();
                    Expression::PropertyFetch {
                        target: Box::new(var),
                        property: Box::new(Expression::Identifier(
                            identifiers::ident_maybe_reserved(state)?,
                        )),
                    }
                }
                TokenKind::NullsafeArrow => {
                    state.next();
                    Expression::NullsafePropertyFetch {
                        target: Box::new(var),
                        property: Box::new(Expression::Identifier(
                            identifiers::ident_maybe_reserved(state)?,
                        )),
                    }
                }
                _ => var,
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
