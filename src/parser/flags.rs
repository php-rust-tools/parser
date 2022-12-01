use crate::lexer::token::TokenKind;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
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
    pub(in crate::parser) fn class_flags(&mut self) -> ParseResult<Vec<TokenKind>> {
        self.collect(
            vec![TokenKind::Final, TokenKind::Abstract, TokenKind::Readonly],
            FlagTarget::Class,
        )
    }

    pub(in crate::parser) fn interface_members_flags(&mut self) -> ParseResult<Vec<TokenKind>> {
        self.collect(
            vec![TokenKind::Public, TokenKind::Static],
            FlagTarget::InterfaceMember,
        )
    }

    pub(in crate::parser) fn class_members_flags(&mut self) -> ParseResult<Vec<TokenKind>> {
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

    pub(in crate::parser) fn enum_members_flags(&mut self) -> ParseResult<Vec<TokenKind>> {
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

    pub(in crate::parser) fn promoted_property_flags(&mut self) -> ParseResult<Vec<TokenKind>> {
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
                    _ => {}
                };

                if matches!(target, FlagTarget::ClassMember | FlagTarget::Class) {
                    match self.current.kind {
                        TokenKind::Final if collected.contains(&TokenKind::Abstract) => {
                            if target == FlagTarget::Class {
                                return Err(ParseError::FinalModifierOnAbstractClass(
                                    self.current.span,
                                ));
                            } else {
                                return Err(ParseError::FinalModifierOnAbstractClassMember(
                                    self.current.span,
                                ));
                            }
                        }
                        TokenKind::Abstract if collected.contains(&TokenKind::Final) => {
                            if target == FlagTarget::Class {
                                return Err(ParseError::FinalModifierOnAbstractClass(
                                    self.current.span,
                                ));
                            } else {
                                return Err(ParseError::FinalModifierOnAbstractClassMember(
                                    self.current.span,
                                ));
                            }
                        }
                        _ => {}
                    };
                }

                collected.push(self.current.kind.clone());
                self.next();
            } else {
                break;
            }
        }

        Ok(collected)
    }
}
