use super::{ParseResult, Type};
use crate::Parser;
use trunk_lexer::TokenKind;

impl Parser {
    pub(crate) fn type_string(&mut self) -> ParseResult<Type> {
        if self.current.kind == TokenKind::Question {
            self.next();
            let t = self.type_with_static()?;
            return Ok(Type::Nullable(t));
        }

        let id = self.type_with_static()?;

        if self.current.kind == TokenKind::Pipe {
            self.next();

            let mut types = vec![id];

            while !self.is_eof() {
                let id = self.type_with_static()?;
                types.push(id);

                if self.current.kind != TokenKind::Pipe {
                    break;
                } else {
                    self.next();
                }
            }

            return Ok(Type::Union(types));
        }

        if self.current.kind == TokenKind::Ampersand {
            self.next();

            let mut types = vec![id];

            while !self.is_eof() {
                let id = self.type_with_static()?;
                types.push(id);

                if self.current.kind != TokenKind::Ampersand {
                    break;
                } else {
                    self.next();
                }
            }

            return Ok(Type::Intersection(types));
        }

        Ok(match &id[..] {
            b"void" => Type::Void,
            b"null" => Type::Null,
            b"true" => Type::True,
            b"false" => Type::False,
            _ => Type::Plain(id),
        })
    }
}
