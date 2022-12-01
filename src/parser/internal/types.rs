use crate::lexer::byte_string::ByteString;
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

    pub(in crate::parser) fn type_string(&self, state: &mut State) -> ParseResult<Type> {
        if state.current.kind == TokenKind::Question {
            state.next();
            let t = self.type_with_static(state)?;
            return Ok(Type::Nullable(Box::new(parse_simple_type(t))));
        }

        let id = self.type_with_static(state)?;

        if state.current.kind == TokenKind::Pipe {
            state.next();

            let r#type = parse_simple_type(id);
            if r#type.standalone() {
                return Err(ParseError::StandaloneTypeUsedInCombination(
                    r#type,
                    state.current.span,
                ));
            }

            let mut types = vec![r#type];

            while !state.is_eof() {
                let id = self.type_with_static(state)?;
                let r#type = parse_simple_type(id);
                if r#type.standalone() {
                    return Err(ParseError::StandaloneTypeUsedInCombination(
                        r#type,
                        state.current.span,
                    ));
                }

                types.push(r#type);

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

            let r#type = parse_simple_type(id);
            if r#type.standalone() {
                return Err(ParseError::StandaloneTypeUsedInCombination(
                    r#type,
                    state.current.span,
                ));
            }

            let mut types = vec![r#type];

            while !state.is_eof() {
                let id = self.type_with_static(state)?;
                let r#type = parse_simple_type(id);
                if r#type.standalone() {
                    return Err(ParseError::StandaloneTypeUsedInCombination(
                        r#type,
                        state.current.span,
                    ));
                }

                types.push(r#type);

                if state.current.kind != TokenKind::Ampersand {
                    break;
                } else {
                    state.next();
                }
            }

            return Ok(Type::Intersection(types));
        }

        Ok(parse_simple_type(id))
    }
}

fn parse_simple_type(id: ByteString) -> Type {
    let name = &id[..];
    let lowered_name = name.to_ascii_lowercase();
    match lowered_name.as_slice() {
        b"void" => Type::Void,
        b"never" => Type::Never,
        b"null" => Type::Null,
        b"true" => Type::True,
        b"false" => Type::False,
        b"float" => Type::Float,
        b"bool" => Type::Boolean,
        b"int" => Type::Integer,
        b"string" => Type::String,
        b"array" => Type::Array,
        b"object" => Type::Object,
        b"mixed" => Type::Mixed,
        b"iterable" => Type::Iterable,
        b"callable" => Type::Callable,
        _ => Type::Identifier(id.into()),
    }
}
