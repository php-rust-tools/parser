use crate::expected_token;
use crate::lexer::token::TokenKind;
use crate::parser::ast::identifiers::Identifier;
use crate::parser::ast::Type;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::internal::utils;
use crate::parser::state::State;
use crate::peek_token;

pub fn data_type(state: &mut State) -> ParseResult<Type> {
    if state.current.kind == TokenKind::Question {
        return nullable(state);
    }

    // (A|B|..)&C.. or (A&B&..)|C..
    if state.current.kind == TokenKind::LeftParen {
        return dnf(state);
    }

    let ty = simple_data_type(state)?;

    if state.current.kind == TokenKind::Pipe {
        return union(state, ty, false);
    }

    if state.current.kind == TokenKind::Ampersand
        && !matches!(
            state.peek.kind,
            TokenKind::Variable(_) | TokenKind::Ellipsis | TokenKind::Ampersand
        )
    {
        return instersection(state, ty, false);
    }

    Ok(ty)
}

pub fn optional_data_type(state: &mut State) -> ParseResult<Option<Type>> {
    if state.current.kind == TokenKind::Question {
        return nullable(state).map(Some);
    }

    // (A|B|..)&C.. or (A&B&..)|C..
    if state.current.kind == TokenKind::LeftParen {
        return dnf(state).map(Some);
    }

    let ty = optional_simple_data_type(state)?;

    match ty {
        Some(ty) => {
            if state.current.kind == TokenKind::Pipe {
                return union(state, ty, false).map(Some);
            }

            if state.current.kind == TokenKind::Ampersand
                && !matches!(
                    state.peek.kind,
                    TokenKind::Variable(_) | TokenKind::Ellipsis | TokenKind::Ampersand
                )
            {
                return instersection(state, ty, false).map(Some);
            }

            Ok(Some(ty))
        }
        None => Ok(None),
    }
}

fn dnf(state: &mut State) -> ParseResult<Type> {
    // (A|B|..)&C.. or (A&B&..)|C..
    state.next();
    let ty = simple_data_type(state)?;
    peek_token!([
        TokenKind::Pipe => {
            let union = union(state, ty, true)?;

            utils::skip_right_parenthesis(state)?;

            instersection(state, union, false)
        },
        TokenKind::Ampersand => {
            let intersection = instersection(state, ty, true)?;

            utils::skip_right_parenthesis(state)?;

            union(state, intersection, false)
        },
    ], state, ["`|`", "`&`"])
}

fn optional_simple_data_type(state: &mut State) -> ParseResult<Option<Type>> {
    match state.current.kind.clone() {
        TokenKind::Array => {
            state.next();

            Ok(Some(Type::Array))
        }
        TokenKind::Callable => {
            state.next();

            Ok(Some(Type::Callable))
        }
        TokenKind::Null => {
            state.next();

            Ok(Some(Type::Null))
        }
        TokenKind::True => {
            state.next();

            Ok(Some(Type::True))
        }
        TokenKind::False => {
            state.next();

            Ok(Some(Type::False))
        }
        TokenKind::Static => {
            state.next();

            if !state.has_class_scope {
                return Err(ParseError::CannotFindTypeInCurrentScope(
                    "static".to_owned(),
                    state.current.span,
                ));
            }

            Ok(Some(Type::StaticReference))
        }
        TokenKind::Self_ => {
            state.next();

            if !state.has_class_scope {
                return Err(ParseError::CannotFindTypeInCurrentScope(
                    "self".to_owned(),
                    state.current.span,
                ));
            }

            Ok(Some(Type::SelfReference))
        }
        TokenKind::Parent => {
            state.next();

            if !state.has_class_scope {
                return Err(ParseError::CannotFindTypeInCurrentScope(
                    "parent".to_owned(),
                    state.current.span,
                ));
            }

            Ok(Some(Type::ParentReference))
        }
        TokenKind::Identifier(id) => {
            let start = state.current.span;
            state.next();
            let end = state.current.span;

            let name = &id[..];
            let lowered_name = name.to_ascii_lowercase();
            match lowered_name.as_slice() {
                b"void" => Ok(Some(Type::Void)),
                b"never" => Ok(Some(Type::Never)),
                b"float" => Ok(Some(Type::Float)),
                b"bool" => Ok(Some(Type::Boolean)),
                b"int" => Ok(Some(Type::Integer)),
                b"string" => Ok(Some(Type::String)),
                b"object" => Ok(Some(Type::Object)),
                b"mixed" => Ok(Some(Type::Mixed)),
                b"iterable" => Ok(Some(Type::Iterable)),
                b"null" => Ok(Some(Type::Null)),
                b"true" => Ok(Some(Type::True)),
                b"false" => Ok(Some(Type::False)),
                b"array" => Ok(Some(Type::Array)),
                b"callable" => Ok(Some(Type::Callable)),
                _ => Ok(Some(Type::Identifier(Identifier {
                    start,
                    name: id,
                    end,
                }))),
            }
        }
        TokenKind::QualifiedIdentifier(name) | TokenKind::FullyQualifiedIdentifier(name) => {
            let start = state.current.span;
            state.next();
            let end = state.current.span;

            Ok(Some(Type::Identifier(Identifier { start, name, end })))
        }
        _ => Ok(None),
    }
}

fn simple_data_type(state: &mut State) -> ParseResult<Type> {
    optional_simple_data_type(state)?.ok_or_else(|| expected_token!(["a type"], state))
}

fn nullable(state: &mut State) -> ParseResult<Type> {
    state.next();

    let ty = simple_data_type(state)?;

    if ty.standalone() {
        return Err(ParseError::StandaloneTypeUsedInCombination(
            ty,
            state.current.span,
        ));
    }

    Ok(Type::Nullable(Box::new(ty)))
}

fn union(state: &mut State, other: Type, within_dnf: bool) -> ParseResult<Type> {
    if other.standalone() {
        return Err(ParseError::StandaloneTypeUsedInCombination(
            other,
            state.current.span,
        ));
    }

    let mut types = vec![other];

    utils::skip(state, TokenKind::Pipe)?;

    loop {
        let ty = if state.current.kind == TokenKind::LeftParen {
            if within_dnf {
                // don't allow nesting.
                //
                // examples on how we got here:
                //
                // v-- get_intersection_type: within_dnf = fasle
                //     v-- get_union_type: within_dnf = true
                //      v-- error
                // F&(A|(D&S))
                //
                // v-- get_intersection_type: within_dnf = fasle
                //     v-- get_union_type: within_dnf = true
                //        v-- error
                // F&(A|B|(D&S))
                return Err(ParseError::NestedDisjunctiveNormalFormTypes(
                    state.current.span,
                ));
            }

            state.next();

            let other = simple_data_type(state)?;
            let ty = instersection(state, other, true)?;

            utils::skip_right_parenthesis(state)?;

            ty
        } else {
            let ty = simple_data_type(state)?;
            if ty.standalone() {
                return Err(ParseError::StandaloneTypeUsedInCombination(
                    ty,
                    state.current.span,
                ));
            }

            ty
        };

        types.push(ty);

        if state.current.kind == TokenKind::Pipe {
            state.next();
        } else {
            break;
        }
    }

    Ok(Type::Union(types))
}

fn instersection(state: &mut State, other: Type, within_dnf: bool) -> ParseResult<Type> {
    if other.standalone() {
        return Err(ParseError::StandaloneTypeUsedInCombination(
            other,
            state.current.span,
        ));
    }

    let mut types = vec![other];

    utils::skip(state, TokenKind::Ampersand)?;

    loop {
        let ty = if state.current.kind == TokenKind::LeftParen {
            if within_dnf {
                // don't allow nesting.
                //
                // examples on how we got here:
                //
                //  v-- get_union_type: within_dnf = fasle
                //     v-- get_intersection_type: within_dnf = true
                //      v-- error
                // F|(A&(D|S))
                //
                //  v-- get_union_type: within_dnf = fasle
                //     v-- get_intersection_type: within_dnf = true
                //        v-- error
                // F|(A&B&(D|S))
                return Err(ParseError::NestedDisjunctiveNormalFormTypes(
                    state.current.span,
                ));
            }

            state.next();

            let other = simple_data_type(state)?;
            let ty = union(state, other, true)?;

            utils::skip_right_parenthesis(state)?;

            ty
        } else {
            let ty = simple_data_type(state)?;
            if ty.standalone() {
                return Err(ParseError::StandaloneTypeUsedInCombination(
                    ty,
                    state.current.span,
                ));
            }

            ty
        };

        types.push(ty);

        if state.current.kind == TokenKind::Ampersand
            && !matches!(
                state.peek.kind,
                TokenKind::Variable(_) | TokenKind::Ellipsis | TokenKind::Ampersand
            )
        {
            state.next();
        } else {
            break;
        }
    }

    Ok(Type::Intersection(types))
}
