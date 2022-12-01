use crate::lexer::token::TokenKind;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::parser::state::State;
use crate::parser::Parser;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum FlagTarget {
    Class,
    EnumMember,
    ClassMember,
    InterfaceMember,
    PromotedProperty,
}

impl Parser {
    pub(in crate::parser) fn class_flags(&self, state: &mut State) -> ParseResult<Vec<TokenKind>> {
        self.collect(
            state,
            vec![TokenKind::Final, TokenKind::Abstract, TokenKind::Readonly],
            FlagTarget::Class,
        )
    }

    pub(in crate::parser) fn interface_members_flags(
        &self,
        state: &mut State,
    ) -> ParseResult<Vec<TokenKind>> {
        self.collect(
            state,
            vec![TokenKind::Public, TokenKind::Static],
            FlagTarget::InterfaceMember,
        )
    }

    pub(in crate::parser) fn class_members_flags(
        &self,
        state: &mut State,
    ) -> ParseResult<Vec<TokenKind>> {
        self.collect(
            state,
            vec![
                TokenKind::Final,
                TokenKind::Abstract,
                TokenKind::Private,
                TokenKind::Protected,
                TokenKind::Public,
                TokenKind::Static,
                TokenKind::Readonly,
            ],
            FlagTarget::ClassMember,
        )
    }

    pub(in crate::parser) fn enum_members_flags(
        &self,
        state: &mut State,
    ) -> ParseResult<Vec<TokenKind>> {
        self.collect(
            state,
            vec![
                TokenKind::Final,
                TokenKind::Private,
                TokenKind::Protected,
                TokenKind::Public,
                TokenKind::Static,
            ],
            FlagTarget::EnumMember,
        )
    }

    pub(in crate::parser) fn promoted_property_flags(
        &self,
        state: &mut State,
    ) -> ParseResult<Vec<TokenKind>> {
        self.collect(
            state,
            vec![
                TokenKind::Private,
                TokenKind::Protected,
                TokenKind::Public,
                TokenKind::Readonly,
            ],
            FlagTarget::PromotedProperty,
        )
    }

    fn collect(
        &self,
        state: &mut State,
        flags: Vec<TokenKind>,
        target: FlagTarget,
    ) -> ParseResult<Vec<TokenKind>> {
        let mut collected: Vec<TokenKind> = vec![];
        loop {
            if flags.contains(&state.current.kind) {
                if collected.contains(&state.current.kind) {
                    return Err(ParseError::MultipleModifiers(
                        state.current.kind.to_string(),
                        state.current.span,
                    ));
                }

                match state.current.kind {
                    TokenKind::Private
                        if collected.contains(&TokenKind::Protected)
                            || collected.contains(&TokenKind::Public) =>
                    {
                        return Err(ParseError::MultipleAccessModifiers(state.current.span));
                    }
                    TokenKind::Protected
                        if collected.contains(&TokenKind::Private)
                            || collected.contains(&TokenKind::Public) =>
                    {
                        return Err(ParseError::MultipleAccessModifiers(state.current.span));
                    }
                    TokenKind::Public
                        if collected.contains(&TokenKind::Private)
                            || collected.contains(&TokenKind::Protected) =>
                    {
                        return Err(ParseError::MultipleAccessModifiers(state.current.span));
                    }
                    _ => {}
                };

                if matches!(target, FlagTarget::ClassMember | FlagTarget::Class) {
                    match state.current.kind {
                        TokenKind::Final if collected.contains(&TokenKind::Abstract) => {
                            if target == FlagTarget::Class {
                                return Err(ParseError::FinalModifierOnAbstractClass(
                                    state.current.span,
                                ));
                            } else {
                                return Err(ParseError::FinalModifierOnAbstractClassMember(
                                    state.current.span,
                                ));
                            }
                        }
                        TokenKind::Abstract if collected.contains(&TokenKind::Final) => {
                            if target == FlagTarget::Class {
                                return Err(ParseError::FinalModifierOnAbstractClass(
                                    state.current.span,
                                ));
                            } else {
                                return Err(ParseError::FinalModifierOnAbstractClassMember(
                                    state.current.span,
                                ));
                            }
                        }
                        _ => {}
                    };
                }

                collected.push(state.current.kind.clone());
                state.next();
            } else {
                break;
            }
        }

        Ok(collected)
    }
}
