use crate::expected_token;
use crate::lexer::token::TokenKind;
use crate::parser::ast::TryBlockCaughtType;
use crate::parser::ast::Type;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::state::State;
use crate::parser::Parser;

impl Parser {
    pub(in crate::parser) fn try_block_caught_type_string(
        &self,
        state: &mut State,
    ) -> ParseResult<TryBlockCaughtType> {
        let id = self.full_name(state)?;

        if state.current.kind == TokenKind::Pipe {
            state.next();

            let mut types = vec![id.into()];

            while !state.is_eof() {
                let id = self.full_name(state)?;
                types.push(id.into());

                if state.current.kind != TokenKind::Pipe {
                    break;
                }

                state.next();
            }

            return Ok(TryBlockCaughtType::Union(types));
        }

        Ok(TryBlockCaughtType::Identifier(id.into()))
    }

    pub(in crate::parser) fn get_type(&self, state: &mut State) -> ParseResult<Type> {
        let ty = self.maybe_nullable(state, &|state| {
            self.maybe_static(state, &|state| self.get_simple_type(state))
        })?;

        if ty.nullable() {
            return Ok(ty);
        }

        if state.current.kind == TokenKind::Pipe {
            state.next();

            if ty.standalone() {
                return Err(ParseError::StandaloneTypeUsedInCombination(
                    ty,
                    state.current.span,
                ));
            }

            let mut types = vec![ty];
            while !state.is_eof() {
                let ty = self.maybe_static(state, &|state| self.get_simple_type(state))?;
                if ty.standalone() {
                    return Err(ParseError::StandaloneTypeUsedInCombination(
                        ty,
                        state.current.span,
                    ));
                }

                types.push(ty);

                if state.current.kind != TokenKind::Pipe {
                    break;
                } else {
                    state.next();
                }
            }

            return Ok(Type::Union(types));
        }

        if state.current.kind == TokenKind::Ampersand
            && !matches!(state.peek.kind, TokenKind::Variable(_))
        {
            state.next();

            if ty.standalone() {
                return Err(ParseError::StandaloneTypeUsedInCombination(
                    ty,
                    state.current.span,
                ));
            }

            let mut types = vec![ty];
            while !state.is_eof() {
                let ty = self.maybe_static(state, &|state| self.get_simple_type(state))?;
                if ty.standalone() {
                    return Err(ParseError::StandaloneTypeUsedInCombination(
                        ty,
                        state.current.span,
                    ));
                }

                types.push(ty);

                if state.current.kind != TokenKind::Ampersand {
                    break;
                } else {
                    state.next();
                }
            }

            return Ok(Type::Intersection(types));
        }

        Ok(ty)
    }

    pub(in crate::parser) fn get_optional_type(
        &self,
        state: &mut State,
    ) -> ParseResult<Option<Type>> {
        if state.current.kind == TokenKind::Question {
            return Ok(Some(self.get_type(state)?));
        }

        let ty = self.maybe_optional_static(state, &|state| self.get_optional_simple_type(state));

        match ty {
            Some(ty) => {
                if state.current.kind == TokenKind::Pipe {
                    state.next();

                    if ty.standalone() {
                        return Err(ParseError::StandaloneTypeUsedInCombination(
                            ty,
                            state.current.span,
                        ));
                    }

                    let mut types = vec![ty];
                    while !state.is_eof() {
                        let ty = self.maybe_static(state, &|state| self.get_simple_type(state))?;
                        if ty.standalone() {
                            return Err(ParseError::StandaloneTypeUsedInCombination(
                                ty,
                                state.current.span,
                            ));
                        }

                        types.push(ty);

                        if state.current.kind != TokenKind::Pipe {
                            break;
                        } else {
                            state.next();
                        }
                    }

                    return Ok(Some(Type::Union(types)));
                }

                if state.current.kind == TokenKind::Ampersand
                    && !matches!(state.peek.kind, TokenKind::Variable(_))
                {
                    state.next();

                    if ty.standalone() {
                        return Err(ParseError::StandaloneTypeUsedInCombination(
                            ty,
                            state.current.span,
                        ));
                    }

                    let mut types = vec![ty];
                    while !state.is_eof() {
                        let ty = self.maybe_static(state, &|state| self.get_simple_type(state))?;
                        if ty.standalone() {
                            return Err(ParseError::StandaloneTypeUsedInCombination(
                                ty,
                                state.current.span,
                            ));
                        }

                        types.push(ty);

                        if state.current.kind != TokenKind::Ampersand {
                            break;
                        } else {
                            state.next();
                        }
                    }

                    return Ok(Some(Type::Intersection(types)));
                }

                Ok(Some(ty))
            }
            None => Ok(None),
        }
    }

    fn get_optional_simple_type(&self, state: &mut State) -> Option<Type> {
        match state.current.kind.clone() {
            TokenKind::Array => {
                state.next();

                Some(Type::Array)
            }
            TokenKind::Callable => {
                state.next();

                Some(Type::Callable)
            }
            TokenKind::Null => {
                state.next();

                Some(Type::Null)
            }
            TokenKind::True => {
                state.next();

                Some(Type::True)
            }
            TokenKind::False => {
                state.next();

                Some(Type::False)
            }
            TokenKind::Identifier(id) => {
                state.next();

                let name = &id[..];
                let lowered_name = name.to_ascii_lowercase();
                match lowered_name.as_slice() {
                    b"void" => Some(Type::Void),
                    b"never" => Some(Type::Never),
                    b"float" => Some(Type::Float),
                    b"bool" => Some(Type::Boolean),
                    b"int" => Some(Type::Integer),
                    b"string" => Some(Type::String),
                    b"object" => Some(Type::Object),
                    b"mixed" => Some(Type::Mixed),
                    b"iterable" => Some(Type::Iterable),
                    b"null" => Some(Type::Null),
                    b"true" => Some(Type::True),
                    b"false" => Some(Type::False),
                    b"array" => Some(Type::Array),
                    b"callable" => Some(Type::Callable),
                    _ => Some(Type::Identifier(id.into())),
                }
            }
            TokenKind::QualifiedIdentifier(id) | TokenKind::FullyQualifiedIdentifier(id) => {
                state.next();

                Some(Type::Identifier(id.into()))
            }
            _ => None,
        }
    }

    fn get_simple_type(&self, state: &mut State) -> ParseResult<Type> {
        self.get_optional_simple_type(state)
            .ok_or_else(|| expected_token!(["a type"], state))
    }

    fn maybe_nullable(
        &self,
        state: &mut State,
        otherwise: &(dyn Fn(&mut State) -> ParseResult<Type>),
    ) -> ParseResult<Type> {
        if state.current.kind == TokenKind::Question {
            state.next();
            let inner = otherwise(state)?;
            if inner.standalone() {
                return Err(ParseError::StandaloneTypeUsedInCombination(
                    inner,
                    state.current.span,
                ));
            }

            Ok(Type::Nullable(Box::new(inner)))
        } else {
            otherwise(state)
        }
    }

    fn maybe_static(
        &self,
        state: &mut State,
        otherwise: &(dyn Fn(&mut State) -> ParseResult<Type>),
    ) -> ParseResult<Type> {
        if TokenKind::Static == state.current.kind {
            state.next();

            return Ok(Type::StaticReference);
        }

        if let TokenKind::Identifier(id) = &state.current.kind {
            let name = &id[..];
            let lowered_name = name.to_ascii_lowercase();
            match lowered_name.as_slice() {
                b"self" => {
                    state.next();

                    return Ok(Type::SelfReference);
                }
                b"parent" => {
                    state.next();

                    return Ok(Type::ParentReference);
                }
                _ => {}
            };
        }

        otherwise(state)
    }

    fn maybe_optional_static(
        &self,
        state: &mut State,
        otherwise: &(dyn Fn(&mut State) -> Option<Type>),
    ) -> Option<Type> {
        if TokenKind::Static == state.current.kind {
            state.next();

            return Some(Type::StaticReference);
        }

        if let TokenKind::Identifier(id) = &state.current.kind {
            let name = &id[..];
            let lowered_name = name.to_ascii_lowercase();
            match lowered_name.as_slice() {
                b"self" => {
                    state.next();

                    return Some(Type::SelfReference);
                }
                b"parent" => {
                    state.next();

                    return Some(Type::ParentReference);
                }
                _ => {}
            };
        }

        otherwise(state)
    }
}
