use std::fmt::Display;

use crate::lexer::token::Span;
use crate::parser::ast::Type;

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, Eq, PartialEq)]
pub enum ParseError {
    ExpectedToken(Vec<String>, Option<String>, Span),
    MultipleModifiers(String, Span),
    MultipleAccessModifiers(Span),
    UnexpectedToken(String, Span),
    UnexpectedEndOfFile,
    StandaloneTypeUsedInCombination(Type, Span),
    TryWithoutCatchOrFinally(Span),
    VariadicPromotedProperty(Span),
    MissingTypeForReadonlyProperty(String, String, Span),
    PromotedPropertyOutsideConstructor(Span),
    PromotedPropertyOnAbstractConstructor(Span),
    AbstractModifierOnNonAbstractClassMethod(Span),
    ConstructorInEnum(String, Span),
    MissingCaseValueForBackedEnum(String, String, Span),
    CaseValueForUnitEnum(String, String, Span),
    StaticModifierOnConstant(Span),
    ReadonlyModifierOnConstant(Span),
    FinalModifierOnAbstractClassMember(Span),
    FinalModifierOnPrivateConstant(Span),
    FinalModifierOnAbstractClass(Span),
    UnpredictableState(Span),
    StaticPropertyUsingReadonlyModifier(String, String, Span),
    ReadonlyPropertyHasDefaultValue(String, String, Span),
    MixingBracedAndUnBracedNamespaceDeclarations(Span),
    NestedNamespaceDeclarations(Span),
    ForbiddenTypeUsedInProperty(String, String, Type, Span),
    MatchExpressionWithMultipleDefaultArms(Span),
    CannotFindTypeInCurrentScope(String, Span),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectedToken(expected, found, span) => {
                let length  = expected.len();
                let expected = if length >= 2 {
                    let (left, right) = expected.split_at(length - 1);

                    format!("{}, or {}", left.join(", "), right[0])
                } else {
                   expected.join(",")
                };

                match found {
                    Some(token) => write!(f, "Parse Error: unexpected token `{}`, expecting {} on line {} column {}", token, expected, span.0, span.1),
                    None => write!(f, "Parse Error: unexpected end of file, expecting {} on line {} column {}", expected, span.0, span.1),
                }
            },
            Self::MissingTypeForReadonlyProperty(class, prop, span) => write!(f, "Parse Error: Readonly property {}::${} must have type on line {} column {}", class, prop, span.0, span.1),
            Self::MultipleModifiers(modifier, span) => write!(f, "Parse Error: Multiple {} modifiers are not allowed on line {} column {}", modifier, span.0, span.1),
            Self::MultipleAccessModifiers( span) => write!(f, "Parse Error: Multiple access type modifiers are not allowed on line {} column {}", span.0, span.1),
            Self::UnexpectedToken(message, span) => write!(f, "Parse Error: Unexpected token {} on line {} column {}", message, span.0, span.1),
            Self::UnexpectedEndOfFile => write!(f, "Parse Error: unexpected end of file."),
            Self::FinalModifierOnAbstractClassMember(span) => write!(f, "Parse Error: Cannot use 'final' as an abstract class member modifier on line {} column {}", span.0, span.1),
            Self::StaticModifierOnConstant(span) => write!(f, "Parse Error: Cannot use 'static' as constant modifier on line {} column {}", span.0, span.1),
            Self::ReadonlyModifierOnConstant(span) => write!(f, "Parse Error: Cannot use 'readonly' as constant modifier on line {} column {}", span.0, span.1),
            Self::FinalModifierOnPrivateConstant(span) => write!(f, "Parse Error: Private constant cannot be final as it is not visible to other classes on line {} column {}", span.0, span.1),
            Self::TryWithoutCatchOrFinally(span) => write!(f, "Parse Error: Cannot use try without catch or finally on line {} column {}", span.0, span.1),
            Self::StandaloneTypeUsedInCombination(r#type, span) => write!(f, "Parse error: '{}' can only be used as a standalone type on line {} column {}", r#type, span.0, span.1),
            Self::VariadicPromotedProperty(span) => write!(f, "Parse Error: Cannot declare variadic promoted property on line {} column {}", span.0, span.1),
            Self::PromotedPropertyOutsideConstructor(span) => write!(f, "Parse Error: Cannot declare promoted property outside a constructor on line {} column {}", span.0, span.1),
            Self::PromotedPropertyOnAbstractConstructor(span) => write!(f, "Parse Error: Cannot declare promoted property in an abstract constructor on line {} column {}", span.0, span.1),
            Self::AbstractModifierOnNonAbstractClassMethod(span) => write!(f, "Parse Error: Cannot declare abstract methods on a non-abstract class on line {} column {}", span.0, span.1),
            Self::FinalModifierOnAbstractClass(span) => write!(f, "Parse Error: Cannot use the final modifier on an abstract class on line {} column {}", span.0, span.1),
            Self::ConstructorInEnum(name, span) => write!(f, "Parse Error: Enum '{}' cannot have a constructor on line {} column {}", name, span.0, span.1),
            Self::MissingCaseValueForBackedEnum(case, name, span) => write!(f, "Parse Error: Case `{}` of backed enum `{}` must have a value on line {} column {}", case, name, span.0, span.1),
            Self::CaseValueForUnitEnum(case, name, span) => write!(f, "Parse Error: Case `{}` of unit enum `{}` must not have a value on line {} column {}", case, name, span.0, span.1),
            Self::StaticPropertyUsingReadonlyModifier(class, prop, span) => write!(f, "Parse Error: Static property {}:${} cannot be readonly on line {} column {}", class, prop, span.0, span.1),
            Self::ReadonlyPropertyHasDefaultValue(class, prop, span) => write!(f, "Parse Error: Readonly property {}:${} cannot have a default value on line {} column {}", class, prop, span.0, span.1),
            Self::MixingBracedAndUnBracedNamespaceDeclarations(span) => write!(f, "Parse Error: Cannot mix braced namespace declarations with unbraced namespace declarations on line {} column {}", span.0, span.1),
            Self::NestedNamespaceDeclarations(span) => write!(f, "Parse Error: Namespace declarations cannot be mixed on line {} column {}", span.0, span.1),
            Self::UnpredictableState(span) => write!(f, "Parse Error: Reached an unpredictable state on line {} column {}", span.0, span.1),
            Self::ForbiddenTypeUsedInProperty(class, prop, ty, span) => write!(f, "Parse Error: Property {}::${} cannot have type `{}` on line {} column {}", class, prop, ty, span.0, span.1),
            Self::MatchExpressionWithMultipleDefaultArms(span) => write!(f, "Parse Error: Match expressions may only contain one default arm on line {} column {}", span.0, span.1),
            Self::CannotFindTypeInCurrentScope(ty, span) => write!(f, "Parse Error: Cannot find type `{}` in this scope on line {} on column {}", ty, span.0, span.1),
        }
    }
}
