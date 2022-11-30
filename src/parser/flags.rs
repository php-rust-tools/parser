use super::ParseResult;
use crate::ParseError;
use crate::Parser;
use crate::TokenKind;

enum FlagTarget {
    Class,
    EnumMember,
    ClassMember,
    PromotedProperty,
}

impl Parser {
    pub(crate) fn class_flags(&mut self) -> ParseResult<Vec<TokenKind>> {
        self.collect(
            vec![TokenKind::Final, TokenKind::Abstract, TokenKind::Readonly],
            FlagTarget::Class,
        )
    }

    pub(crate) fn class_members_flags(&mut self) -> ParseResult<Vec<TokenKind>> {
        self.collect(
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

    pub(crate) fn enum_members_flags(&mut self) -> ParseResult<Vec<TokenKind>> {
        self.collect(
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

    pub(crate) fn promoted_property_flags(&mut self) -> ParseResult<Vec<TokenKind>> {
        self.collect(
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
        &mut self,
        flags: Vec<TokenKind>,
        target: FlagTarget,
    ) -> ParseResult<Vec<TokenKind>> {
        let mut collected: Vec<TokenKind> = vec![];
        loop {
            if flags.contains(&self.current.kind) {
                if collected.contains(&self.current.kind) {
                    return Err(ParseError::MultipleModifiers(
                        self.current.kind.to_string(),
                        self.current.span,
                    ));
                }

                match self.current.kind {
                    TokenKind::Private
                        if collected.contains(&TokenKind::Protected)
                            || collected.contains(&TokenKind::Public) =>
                    {
                        return Err(ParseError::MultipleAccessModifiers(self.current.span));
                    }
                    TokenKind::Protected
                        if collected.contains(&TokenKind::Private)
                            || collected.contains(&TokenKind::Public) =>
                    {
                        return Err(ParseError::MultipleAccessModifiers(self.current.span));
                    }
                    TokenKind::Public
                        if collected.contains(&TokenKind::Private)
                            || collected.contains(&TokenKind::Protected) =>
                    {
                        return Err(ParseError::MultipleAccessModifiers(self.current.span));
                    }
                    TokenKind::Final if collected.contains(&TokenKind::Abstract) => match target {
                        FlagTarget::Class => {
                            return Err(ParseError::FinalModifierOnAbstractClass(
                                self.current.span,
                            ));
                        }
                        FlagTarget::ClassMember => {
                            return Err(ParseError::FinalModifierOnAbstractClassMember(
                                self.current.span,
                            ));
                        }
                        _ => {}
                    },
                    TokenKind::Abstract if collected.contains(&TokenKind::Final) => match target {
                        FlagTarget::Class => {
                            return Err(ParseError::FinalModifierOnAbstractClass(
                                self.current.span,
                            ));
                        }
                        FlagTarget::ClassMember => {
                            return Err(ParseError::FinalModifierOnAbstractClassMember(
                                self.current.span,
                            ));
                        }
                        _ => {}
                    },
                    _ => {}
                };

                collected.push(self.current.kind.clone());
                self.next();
            } else {
                break;
            }
        }

        Ok(collected)
    }
}
