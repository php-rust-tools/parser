use super::classish_statement::ClassishDefinitionType;
use super::params::ParamPosition;
use super::ParseResult;
use crate::ByteString;
use crate::ClassFlag;
use crate::MethodFlag;
use crate::ParseError;
use crate::Parser;
use crate::Statement;
use crate::TokenKind;

impl Parser {
    pub(crate) fn function(&mut self) -> ParseResult<Statement> {
        self.next();

        let by_ref = if self.current.kind == TokenKind::Ampersand {
            self.next();
            true
        } else {
            false
        };

        // FIXME: We should only allow reserved words for class methods, not top-level functions.
        let name = self.ident_maybe_reserved()?;

        self.lparen()?;

        let params = self.param_list(ParamPosition::Function)?;

        self.rparen()?;

        let mut return_type = None;

        if self.current.kind == TokenKind::Colon || self.config.force_type_strings {
            self.colon()?;

            return_type = Some(self.type_string()?);
        }

        self.lbrace()?;

        let body = self.block(&TokenKind::RightBrace)?;

        self.rbrace()?;

        Ok(Statement::Function {
            name: name.into(),
            params,
            body,
            return_type,
            by_ref,
        })
    }

    pub(crate) fn method(
        &mut self,
        class_type: ClassishDefinitionType,
        flags: Vec<MethodFlag>,
    ) -> ParseResult<Statement> {
        // TODO: more verification goes here, we know what type of class and what method flags there are.
        match class_type {
            ClassishDefinitionType::Class(cf)
                if !cf.contains(&ClassFlag::Abstract) && flags.contains(&MethodFlag::Abstract) =>
            {
                return Err(ParseError::AbstractModifierOnNonAbstractClassMethod(
                    self.current.span,
                ));
            }
            _ => (),
        }

        self.next();

        let has_body = match &class_type {
            ClassishDefinitionType::Class(_) | ClassishDefinitionType::Trait => {
                !flags.contains(&MethodFlag::Abstract)
            }
            ClassishDefinitionType::Interface => false,
            ClassishDefinitionType::Enum | ClassishDefinitionType::AnonymousClass => true,
        };

        let by_ref = if self.current.kind == TokenKind::Ampersand {
            self.next();
            true
        } else {
            false
        };

        // FIXME: We should only allow reserved words for class methods, not top-level functions.
        let name = self.ident_maybe_reserved()?;

        self.lparen()?;

        let position = position_from_flags_and_name(class_type, flags.clone(), name.clone());

        let params = self.param_list(position)?;

        self.rparen()?;

        let mut return_type = None;

        if self.current.kind == TokenKind::Colon || self.config.force_type_strings {
            self.colon()?;

            return_type = Some(self.type_string()?);
        }

        if !has_body {
            self.semi()?;

            Ok(Statement::AbstractMethod {
                name: name.into(),
                params,
                return_type,
                flags: flags.to_vec(),
                by_ref,
            })
        } else {
            self.lbrace()?;

            let body = self.block(&TokenKind::RightBrace)?;

            self.rbrace()?;

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
