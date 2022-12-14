use crate::expected_token;
use crate::lexer::token::TokenKind;
use crate::parser::ast::data_type::Type;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::internal::utils;
use crate::parser::state::State;
use crate::peek_token;

pub fn data_type(state: &mut State) -> ParseResult<Type> {
    if state.stream.current().kind == TokenKind::Question {
        return nullable(state);
    }

    // (A|B|..)&C.. or (A&B&..)|C..
    if state.stream.current().kind == TokenKind::LeftParen {
        return dnf(state);
    }

    let ty = simple_data_type(state)?;

    if state.stream.current().kind == TokenKind::Pipe {
        return union(state, ty, false);
    }

    if state.stream.current().kind == TokenKind::Ampersand
        && !matches!(
            state.stream.peek().kind,
            TokenKind::Variable(_) | TokenKind::Ellipsis | TokenKind::Ampersand
        )
    {
        return instersection(state, ty, false);
    }

    Ok(ty)
}

pub fn optional_data_type(state: &mut State) -> ParseResult<Option<Type>> {
    if state.stream.current().kind == TokenKind::Question {
        return nullable(state).map(Some);
    }

    // (A|B|..)&C.. or (A&B&..)|C..
    if state.stream.current().kind == TokenKind::LeftParen {
        return dnf(state).map(Some);
    }

    let ty = optional_simple_data_type(state)?;

    match ty {
        Some(ty) => {
            if state.stream.current().kind == TokenKind::Pipe {
                return union(state, ty, false).map(Some);
            }

            if state.stream.current().kind == TokenKind::Ampersand
                && !matches!(
                    state.stream.peek().kind,
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
    state.stream.next();
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
    let current = state.stream.current();

    match &current.kind {
        TokenKind::Array => {
            let span = current.span;
            state.stream.next();

            Ok(Some(Type::Array(span)))
        }
        TokenKind::Callable => {
            let span = current.span;
            state.stream.next();

            Ok(Some(Type::Callable(span)))
        }
        TokenKind::Null => {
            let span = current.span;
            state.stream.next();

            Ok(Some(Type::Null(span)))
        }
        TokenKind::True => {
            let span = current.span;
            state.stream.next();

            Ok(Some(Type::True(span)))
        }
        TokenKind::False => {
            let span = current.span;
            state.stream.next();

            Ok(Some(Type::False(span)))
        }
        TokenKind::Static => {
            let span = current.span;
            state.stream.next();

            if !state.has_class_scope {
                return Err(ParseError::CannotFindTypeInCurrentScope(
                    "static".to_owned(),
                    span,
                ));
            }

            Ok(Some(Type::StaticReference(span)))
        }
        TokenKind::Self_ => {
            let span = current.span;
            state.stream.next();

            if !state.has_class_scope {
                return Err(ParseError::CannotFindTypeInCurrentScope(
                    "self".to_owned(),
                    span,
                ));
            }

            Ok(Some(Type::SelfReference(span)))
        }
        TokenKind::Parent => {
            let span = current.span;
            state.stream.next();

            if !state.has_class_scope {
                return Err(ParseError::CannotFindTypeInCurrentScope(
                    "parent".to_owned(),
                    span,
                ));
            }

            Ok(Some(Type::ParentReference(span)))
        }
        TokenKind::Enum | TokenKind::From => {
            let span = current.span;
            let name = current.kind.to_string().into();

            state.stream.next();

            Ok(Some(Type::Named(span, name)))
        }
        TokenKind::Identifier(id) => {
            let span = current.span;
            state.stream.next();

            let name = &id[..];
            let lowered_name = name.to_ascii_lowercase();
            match lowered_name.as_slice() {
                b"void" => Ok(Some(Type::Void(span))),
                b"never" => Ok(Some(Type::Never(span))),
                b"float" => Ok(Some(Type::Float(span))),
                b"bool" => Ok(Some(Type::Boolean(span))),
                b"int" => Ok(Some(Type::Integer(span))),
                b"string" => Ok(Some(Type::String(span))),
                b"object" => Ok(Some(Type::Object(span))),
                b"mixed" => Ok(Some(Type::Mixed(span))),
                b"iterable" => Ok(Some(Type::Iterable(span))),
                b"null" => Ok(Some(Type::Null(span))),
                b"true" => Ok(Some(Type::True(span))),
                b"false" => Ok(Some(Type::False(span))),
                b"array" => Ok(Some(Type::Array(span))),
                b"callable" => Ok(Some(Type::Callable(span))),
                _ => Ok(Some(Type::Named(span, name.into()))),
            }
        }
        TokenKind::QualifiedIdentifier(name) | TokenKind::FullyQualifiedIdentifier(name) => {
            let span = current.span;
            state.stream.next();

            Ok(Some(Type::Named(span, name.clone())))
        }
        _ => Ok(None),
    }
}

fn simple_data_type(state: &mut State) -> ParseResult<Type> {
    optional_simple_data_type(state)?.ok_or_else(|| expected_token!(["a type"], state))
}

fn nullable(state: &mut State) -> ParseResult<Type> {
    state.stream.next();

    let ty = simple_data_type(state)?;

    if ty.standalone() {
        return Err(ParseError::StandaloneTypeUsedInCombination(
            ty,
            state.stream.current().span,
        ));
    }

    Ok(Type::Nullable(Box::new(ty)))
}

fn union(state: &mut State, other: Type, within_dnf: bool) -> ParseResult<Type> {
    if other.standalone() {
        return Err(ParseError::StandaloneTypeUsedInCombination(
            other,
            state.stream.current().span,
        ));
    }

    let mut types = vec![other];

    utils::skip(state, TokenKind::Pipe)?;

    loop {
        let current = state.stream.current();
        let ty = if current.kind == TokenKind::LeftParen {
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
                return Err(ParseError::NestedDisjunctiveNormalFormTypes(current.span));
            }

            state.stream.next();

            let other = simple_data_type(state)?;
            let ty = instersection(state, other, true)?;

            utils::skip_right_parenthesis(state)?;

            ty
        } else {
            let ty = simple_data_type(state)?;
            if ty.standalone() {
                return Err(ParseError::StandaloneTypeUsedInCombination(
                    ty,
                    state.stream.current().span,
                ));
            }

            ty
        };

        types.push(ty);

        if state.stream.current().kind == TokenKind::Pipe {
            state.stream.next();
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
            state.stream.current().span,
        ));
    }

    let mut types = vec![other];

    utils::skip(state, TokenKind::Ampersand)?;

    loop {
        let current = state.stream.current();
        let ty = if current.kind == TokenKind::LeftParen {
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
                return Err(ParseError::NestedDisjunctiveNormalFormTypes(current.span));
            }

            state.stream.next();

            let other = simple_data_type(state)?;
            let ty = union(state, other, true)?;

            utils::skip_right_parenthesis(state)?;

            ty
        } else {
            let ty = simple_data_type(state)?;
            if ty.standalone() {
                return Err(ParseError::StandaloneTypeUsedInCombination(
                    ty,
                    state.stream.current().span,
                ));
            }

            ty
        };

        types.push(ty);

        if state.stream.current().kind == TokenKind::Ampersand
            && !matches!(
                state.stream.peek().kind,
                TokenKind::Variable(_) | TokenKind::Ellipsis | TokenKind::Ampersand
            )
        {
            state.stream.next();
        } else {
            break;
        }
    }

    Ok(Type::Intersection(types))
}
