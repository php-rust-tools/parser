use crate::lexer::byte_string::ByteString;
use crate::lexer::token::TokenKind;
use crate::parser::ast::ClassFlag;
use crate::parser::ast::MethodFlag;
use crate::parser::ast::Statement;
use crate::parser::classish_statement::ClassishDefinitionType;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::params::ParamPosition;
use crate::parser::state::State;
use crate::parser::Parser;

impl Parser {
    pub(in crate::parser) fn function(&self, state: &mut State) -> ParseResult<Statement> {
        state.next();

        let by_ref = if state.current.kind == TokenKind::Ampersand {
            state.next();
            true
        } else {
            false
        };

        let name = self.ident(state)?;

        self.lparen(state)?;

        let params = self.param_list(state, ParamPosition::Function)?;

        self.rparen(state)?;

        let mut return_type = None;

        if state.current.kind == TokenKind::Colon {
            self.colon(state)?;

            return_type = Some(self.type_string(state)?);
        }

        self.lbrace(state)?;

        let body = self.block(state, &TokenKind::RightBrace)?;

        self.rbrace(state)?;

        Ok(Statement::Function {
            name: name.into(),
            params,
            body,
            return_type,
            by_ref,
        })
    }

    pub(in crate::parser) fn method(
        &self,
        state: &mut State,
        class_type: ClassishDefinitionType,
        flags: Vec<MethodFlag>,
    ) -> ParseResult<Statement> {
        // TODO: more verification goes here, we know what type of class and what method flags there are.
        match class_type {
            ClassishDefinitionType::Class(cf)
                if !cf.contains(&ClassFlag::Abstract) && flags.contains(&MethodFlag::Abstract) =>
            {
                return Err(ParseError::AbstractModifierOnNonAbstractClassMethod(
                    state.current.span,
                ));
            }
            _ => (),
        }

        state.next();

        let has_body = match &class_type {
            ClassishDefinitionType::Class(_) | ClassishDefinitionType::Trait => {
                !flags.contains(&MethodFlag::Abstract)
            }
            ClassishDefinitionType::Interface => false,
            ClassishDefinitionType::Enum | ClassishDefinitionType::AnonymousClass => true,
        };

        let by_ref = if state.current.kind == TokenKind::Ampersand {
            state.next();
            true
        } else {
            false
        };

        let name = self.ident_maybe_reserved(state)?;

        self.lparen(state)?;

        let position = position_from_flags_and_name(class_type, flags.clone(), name.clone());

        let params = self.param_list(state, position)?;

        self.rparen(state)?;

        let mut return_type = None;

        if state.current.kind == TokenKind::Colon {
            self.colon(state)?;

            return_type = Some(self.type_string(state)?);
        }

        if !has_body {
            self.semi(state)?;

            Ok(Statement::AbstractMethod {
                name: name.into(),
                params,
                return_type,
                flags: flags.to_vec(),
                by_ref,
            })
        } else {
            self.lbrace(state)?;

            let body = self.block(state, &TokenKind::RightBrace)?;

            self.rbrace(state)?;

            Ok(Statement::Method {
                name: name.into(),
                params,
                body,
                return_type,
                by_ref,
                flags,
            })
        }
    }
}

fn position_from_flags_and_name(
    class_type: ClassishDefinitionType,
    flags: Vec<MethodFlag>,
    name: ByteString,
) -> ParamPosition {
    match class_type {
        ClassishDefinitionType::Enum
        | ClassishDefinitionType::Class(_)
        | ClassishDefinitionType::Trait
        | ClassishDefinitionType::AnonymousClass => {
            if !flags.contains(&MethodFlag::Abstract) {
                ParamPosition::Method(name.to_string())
            } else {
                ParamPosition::AbstractMethod(name.to_string())
            }
        }
        ClassishDefinitionType::Interface => ParamPosition::AbstractMethod(name.to_string()),
    }
}
