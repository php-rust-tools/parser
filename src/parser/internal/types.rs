use crate::expect_token;
use crate::expected_token;
use crate::lexer::token::TokenKind;
use crate::parser::ast::identifier::Identifier;
use crate::parser::ast::TryBlockCaughtType;
use crate::parser::ast::Type;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::state::State;
use crate::parser::Parser;
use crate::peek_token;

impl Parser {
    pub(in crate::parser) fn try_block_caught_type_string(
        &self,
        state: &mut State,
    ) -> ParseResult<TryBlockCaughtType> {
        let id = self.full_name(state)?;

        if state.current.kind == TokenKind::Pipe {
            state.next();

            let mut types = vec![id];

            while !state.is_eof() {
                let id = self.full_name(state)?;
                types.push(id);

                if state.current.kind != TokenKind::Pipe {
                    break;
                }

                state.next();
            }

            return Ok(TryBlockCaughtType::Union(types));
        }

        Ok(TryBlockCaughtType::Identifier(id))
    }

    pub(in crate::parser) fn get_type(&self, state: &mut State) -> ParseResult<Type> {
        if state.current.kind == TokenKind::Question {
            return self.get_nullable_type(state);
        }

        // (A|B|..)&C.. or (A&B&..)|C..
        if state.current.kind == TokenKind::LeftParen {
            state.next();
            let ty = self.get_simple_type(state)?;
            return peek_token!([
                TokenKind::Pipe => {
                    let union = self.get_union_type(state, ty, true)?;

                    self.rparen(state)?;

                    self.get_intersection_type(state, union, false)
                },
                TokenKind::Ampersand => {
                    let intersection = self.get_intersection_type(state, ty, true)?;

                    self.rparen(state)?;

                    self.get_union_type(state, intersection, false)
                },
            ], state, ["`|`", "`&`"]);
        }

        let ty = self.get_simple_type(state)?;

        if state.current.kind == TokenKind::Pipe {
            return self.get_union_type(state, ty, false);
        }

        if state.current.kind == TokenKind::Ampersand
            && !matches!(state.peek.kind, TokenKind::Variable(_))
        {
            return self.get_intersection_type(state, ty, false);
        }

        Ok(ty)
    }

    pub(in crate::parser) fn get_optional_type(
        &self,
        state: &mut State,
    ) -> ParseResult<Option<Type>> {
        if state.current.kind == TokenKind::Question {
            return self.get_nullable_type(state).map(Some);
        }

        // (A|B|..)&C.. or (A&B&..)|C..
        if state.current.kind == TokenKind::LeftParen {
            state.next();
            let ty = self.get_simple_type(state)?;
            return peek_token!([
                TokenKind::Pipe => {
                    let union = self.get_union_type(state, ty, true)?;

                    self.rparen(state)?;

                    self.get_intersection_type(state, union, false).map(Some)
                },
                TokenKind::Ampersand => {
                    let intersection = self.get_intersection_type(state, ty, true)?;

                    self.rparen(state)?;

                    self.get_union_type(state, intersection, false).map(Some)
                },
            ], state, ["`|`", "`&`"]);
        }

        let ty = self.get_optional_simple_type(state)?;

        match ty {
            Some(ty) => {
                if state.current.kind == TokenKind::Pipe {
                    return Ok(Some(self.get_union_type(state, ty, false)?));
                }

                if state.current.kind == TokenKind::Ampersand
                    && !matches!(state.peek.kind, TokenKind::Variable(_))
                {
                    return Ok(Some(self.get_intersection_type(state, ty, false)?));
                }

                Ok(Some(ty))
            }
            None => Ok(None),
        }
    }

    fn get_nullable_type(&self, state: &mut State) -> ParseResult<Type> {
        state.next();

        let ty = self.get_simple_type(state)?;

        if ty.standalone() {
            return Err(ParseError::StandaloneTypeUsedInCombination(
                ty,
                state.current.span,
            ));
        }

        Ok(Type::Nullable(Box::new(ty)))
    }

    fn get_optional_simple_type(&self, state: &mut State) -> ParseResult<Option<Type>> {
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
                    b"self" => {
                        if !state.has_class_scope {
                            return Err(ParseError::CannotFindTypeInCurrentScope(
                                "self".to_owned(),
                                state.current.span,
                            ));
                        }

                        Ok(Some(Type::SelfReference))
                    }
                    b"parent" => {
                        if !state.has_class_parent_scope {
                            return Err(ParseError::CannotFindTypeInCurrentScope(
                                "parent".to_owned(),
                                state.current.span,
                            ));
                        }

                        Ok(Some(Type::ParentReference))
                    }
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

    fn get_simple_type(&self, state: &mut State) -> ParseResult<Type> {
        self.get_optional_simple_type(state)?
            .ok_or_else(|| expected_token!(["a type"], state))
    }

    fn get_union_type(
        &self,
        state: &mut State,
        other: Type,
        within_dnf: bool,
    ) -> ParseResult<Type> {
        if other.standalone() {
            return Err(ParseError::StandaloneTypeUsedInCombination(
                other,
                state.current.span,
            ));
        }

        let mut types = vec![other];

        expect_token!([TokenKind::Pipe], state, ["|"]);
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

                let other = self.get_simple_type(state)?;
                let ty = self.get_intersection_type(state, other, true)?;

                self.rparen(state)?;

                ty
            } else {
                let ty = self.get_simple_type(state)?;
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

    fn get_intersection_type(
        &self,
        state: &mut State,
        other: Type,
        within_dnf: bool,
    ) -> ParseResult<Type> {
        if other.standalone() {
            return Err(ParseError::StandaloneTypeUsedInCombination(
                other,
                state.current.span,
            ));
        }

        let mut types = vec![other];

        expect_token!([TokenKind::Ampersand], state, ["&"]);
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

                let other = self.get_simple_type(state)?;
                let ty = self.get_union_type(state, other, true)?;

                self.rparen(state)?;

                ty
            } else {
                let ty = self.get_simple_type(state)?;
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
                && !matches!(state.peek.kind, TokenKind::Variable(_))
            {
                state.next();
            } else {
                break;
            }
        }

        Ok(Type::Intersection(types))
    }
}
