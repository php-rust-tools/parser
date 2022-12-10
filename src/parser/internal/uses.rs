use crate::lexer::token::TokenKind;
use crate::parser::ast::Statement;
use crate::parser::ast::Use;
use crate::parser::ast::UseKind;
use crate::parser::error::ParseResult;
use crate::parser::internal::identifiers;
use crate::parser::internal::utils;
use crate::parser::state::State;

pub fn use_statement(state: &mut State) -> ParseResult<Statement> {
    state.next();

    let kind = match state.current.kind {
        TokenKind::Function => {
            state.next();
            UseKind::Function
        }
        TokenKind::Const => {
            state.next();
            UseKind::Const
        }
        _ => UseKind::Normal,
    };

    if state.peek.kind == TokenKind::LeftBrace {
        let prefix = identifiers::full_name(state)?;
        state.next();

        let mut uses = Vec::new();
        while state.current.kind != TokenKind::RightBrace {
            let name = identifiers::full_name(state)?;
            let mut alias = None;

            if state.current.kind == TokenKind::As {
                state.next();
                alias = Some(identifiers::identifier(state)?);
            }

            uses.push(Use { name, alias });

            if state.current.kind == TokenKind::Comma {
                state.next();
                continue;
            }
        }

        utils::skip_right_brace(state)?;
        utils::skip_semicolon(state)?;

        Ok(Statement::GroupUse { prefix, kind, uses })
    } else {
        let mut uses = Vec::new();
        while !state.is_eof() {
            let name = identifiers::full_name(state)?;
            let mut alias = None;

            if state.current.kind == TokenKind::As {
                state.next();
                alias = Some(identifiers::identifier(state)?);
            }

            uses.push(Use { name, alias });

            if state.current.kind == TokenKind::Comma {
                state.next();
                continue;
            }

            utils::skip_semicolon(state)?;
            break;
        }

        Ok(Statement::Use { uses, kind })
    }
}
