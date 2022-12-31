use std::fmt::{Display, Formatter};

use ariadne::{CharSet, Color, Config, Label, Report, ReportKind, Source};
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::error::SyntaxError;
use crate::lexer::token::{Span, Token, TokenKind};
use crate::parser::ast::attributes::AttributeGroup;
use crate::parser::ast::data_type::Type;
use crate::parser::ast::modifiers::PromotedPropertyModifier;
use crate::parser::ast::Program;

use super::ast::identifiers::SimpleIdentifier;
use super::ast::variables::SimpleVariable;
use super::state::State;

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ParseErrorAnnotationType {
    Hint,
    Error,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ParseErrorAnnotation {
    pub r#type: ParseErrorAnnotationType,
    pub message: String,
    pub position: usize,
    pub length: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ParseError {
    pub id: String,
    pub message: String,
    pub span: Span,
    pub annotations: Vec<ParseErrorAnnotation>,
    pub note: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ParseErrorStack {
    pub partial: Program,
    pub errors: Vec<ParseError>,
}

impl ParseErrorStack {
    pub fn report<'a>(
        &self,
        source: &'a str,
        origin: Option<&'a str>,
        colored: bool,
        ascii: bool,
    ) -> std::io::Result<String> {
        let mut reports = Vec::new();

        for error in &self.errors {
            reports.push(error.report(source, origin, colored, ascii)?);
        }

        Ok(reports.join("\n"))
    }
}

impl ParseError {
    pub fn new<TId: ToString, TMessage: ToString>(id: TId, message: TMessage, span: Span) -> Self {
        Self {
            id: id.to_string(),
            message: message.to_string(),
            span,
            annotations: Vec::new(),
            note: None,
        }
    }

    pub fn highlight(mut self, position: usize, length: usize) -> Self {
        self.annotations.push(ParseErrorAnnotation {
            r#type: ParseErrorAnnotationType::Hint,
            message: "".to_owned(),
            position,
            length,
        });

        self
    }

    pub fn error<T: ToString>(mut self, message: T, position: usize, length: usize) -> Self {
        self.annotations.push(ParseErrorAnnotation {
            r#type: ParseErrorAnnotationType::Error,
            message: message.to_string(),
            position,
            length,
        });

        self
    }

    pub fn note<T: ToString>(mut self, note: T) -> Self {
        self.note = Some(note.to_string());

        self
    }

    pub fn report<'a>(
        &self,
        source: &'a str,
        origin: Option<&'a str>,
        colored: bool,
        ascii: bool,
    ) -> std::io::Result<String> {
        let origin = origin.unwrap_or("input");

        let mut report = Report::build(ReportKind::Error, origin, self.span.position)
            .with_code(&self.id)
            .with_message(&self.message)
            .with_config(
                Config::default()
                    .with_color(colored)
                    .with_char_set(if ascii {
                        CharSet::Ascii
                    } else {
                        CharSet::Unicode
                    }),
            );

        for (order, annotation) in self.annotations.iter().enumerate() {
            let mut label = Label::new((
                origin,
                annotation.position..annotation.position + annotation.length,
            ))
            .with_order(order.try_into().unwrap());

            if !annotation.message.is_empty() {
                label = label.with_message(&annotation.message);
            }

            if colored {
                label = match annotation.r#type {
                    ParseErrorAnnotationType::Hint => label.with_color(Color::Cyan),
                    ParseErrorAnnotationType::Error => label.with_color(Color::Red),
                };
            }

            report = report.with_label(label);
        }

        if let Some(note) = &self.note {
            report = report.with_note(note);
        }

        let code = (origin, Source::from(source));

        let mut bytes = Vec::new();

        report.finish().write(code, &mut bytes)?;

        let string = unsafe {
            // SAFETY: We know that the bytes are valid UTF-8
            String::from_utf8_unchecked(bytes)
        };

        Ok(string)
    }
}

pub fn unexpected_token(expected: Vec<String>, found: &Token) -> ParseError {
    let (found_name, eof) = match &found.kind {
        TokenKind::Eof => ("end of file".to_string(), true),
        kind => match kind {
            TokenKind::Identifier
            | TokenKind::QualifiedIdentifier
            | TokenKind::FullyQualifiedIdentifier => ("identifier".to_string(), false),
            TokenKind::Variable => ("variable".to_string(), false),
            TokenKind::LiteralInteger | TokenKind::LiteralFloat | TokenKind::LiteralString => {
                ("literal".to_string(), false)
            }
            _ => (format!("token `{}`", found.value), false),
        },
    };

    if expected.is_empty() {
        return if eof {
            ParseError::new("E002", format!("unexpected {}", found_name), found.span)
        } else {
            ParseError::new("E003", format!("unexpected {}", found_name), found.span).error(
                "try removing this".to_string(),
                found.span.position,
                found.value.len(),
            )
        };
    }

    let expected: Vec<String> = expected
        .iter()
        .map(|s| {
            if s.starts_with("a ") || s.starts_with("an ") {
                s.to_string()
            } else {
                format!("`{}`", s)
            }
        })
        .collect();

    let length = expected.len();
    let expected = if length > 2 {
        let (left, right) = expected.split_at(length - 1);

        format!("{}, or {}", left.join(", "), right[0])
    } else {
        expected.join(", or ")
    };

    ParseError::new(
        "E005",
        format!("unexpected {}, expecting {}", found_name, expected),
        found.span,
    )
    .error(
        format!("expected {}", expected),
        found.span.position,
        found.value.len(),
    )
}

pub fn unexpected_identifier(expected: Vec<String>, found: String, span: Span) -> ParseError {
    let length = expected.len();
    let expected = if length >= 2 {
        let (left, right) = expected.split_at(length - 1);

        format!("{}`, or `{}", left.join("`, `"), right[0])
    } else {
        expected.join("")
    };

    ParseError::new(
        "E006",
        format!(
            "unexpected identifier `{}`, expecting `{}`",
            found, expected
        ),
        span,
    )
    .error(
        format!("try replacing this with `{}`", expected),
        span.position,
        found.len(),
    )
}

pub fn multiple_modifiers(modifier: String, first: Span, second: Span) -> ParseError {
    ParseError::new(
        "E007",
        format!("multiple `{}` modifiers are not allowed", modifier),
        second,
    )
    .highlight(first.position, modifier.len())
    .error("try removing this", second.position, modifier.len())
}

pub fn multiple_visibility_modifiers(first: (String, Span), second: (String, Span)) -> ParseError {
    ParseError::new(
        "E008",
        "multiple visibility modifiers are not allowed",
        second.1,
    )
    .highlight(first.1.position, first.0.len())
    .error("try removing this", second.1.position, second.0.len())
}

pub fn standalone_type_used_as_nullable(ty: &Type, span: Span) -> ParseError {
    let type_span = ty.first_span();
    let type_string = ty.to_string();

    ParseError::new(
        "E009",
        format!("standalone type `{}` cannot be nullable", type_string),
        type_span,
    )
    .error("try removing this", span.position, 1)
    .highlight(type_span.position, type_string.len())
    .note("`never`, `void`, and `mixed` cannot be nullable")
}

pub fn standalone_type_used_in_union(ty: &Type, span: Span) -> ParseError {
    let type_span = ty.first_span();
    let type_string = ty.to_string();

    ParseError::new(
        "E010",
        format!(
            "standalone type `{}` cannot be used in a union",
            type_string
        ),
        type_span,
    )
    .error(
        format!("try using a type other than `{}`", type_string),
        type_span.position,
        type_string.len(),
    )
    .highlight(span.position, 1)
    .note("`never`, `void`, `mixed`, and nullable types cannot be used in a union")
}

pub fn standalone_type_used_in_intersection(ty: &Type, span: Span) -> ParseError {
    let type_span = ty.first_span();
    let type_string = ty.to_string();

    ParseError::new(
        "E011",
        format!(
            "standalone type `{}` cannot be used in an intersection",
            type_string
        ),
        type_span,
    )
    .error(
        format!("try using a type other than `{}`", type_string),
        type_span.position,
        type_string.len(),
    )
    .highlight(span.position, 1)
    .note("`never`, `void`, `mixed`, and nullable types cannot be used in an intersection")
}

pub fn try_without_catch_or_finally(try_span: Span, last_right_brace: Span) -> ParseError {
    ParseError::new(
        "E012",
        "cannot use `try` without `catch` or `finally`",
        try_span,
    )
    .highlight(
        try_span.position,
        last_right_brace.position - try_span.position + 1,
    )
}

pub fn variadic_promoted_property(
    state: &mut State,
    class: Option<&SimpleIdentifier>,
    property: &SimpleVariable,
    span: Span,
    modifier: &PromotedPropertyModifier,
) -> ParseError {
    let error = ParseError::new(
        "E013",
        format!(
            "promoted property `{}::{}` cannot declare variadic",
            class
                .map(|c| state.named(c))
                .unwrap_or_else(|| "anonymous@class".to_string()),
            property.name
        ),
        span,
    )
    .highlight(modifier.span().position, modifier.to_string().len())
    .highlight(property.span.position, property.name.len())
    .error("try removing this variadic declaration", span.position, 3);

    if let Some(class) = class {
        error.highlight(class.span.position, class.value.len())
    } else {
        error
    }
}

pub fn missing_type_for_readonly_property(
    state: &mut State,
    class: Option<&SimpleIdentifier>,
    property: &SimpleVariable,
    readonly_span: Span,
) -> ParseError {
    let error = ParseError::new(
        "E014",
        format!(
            "missing type for readonly property `{}::{}`",
            class
                .map(|c| state.named(c))
                .unwrap_or_else(|| "anonymous@class".to_string()),
            property.name
        ),
        property.span,
    )
    .error(
        format!("try adding a type before `{}`", property.name),
        property.span.position,
        property.name.len(),
    )
    .highlight(readonly_span.position, 8);

    if let Some(class) = class {
        error.highlight(class.span.position, class.value.len())
    } else {
        error
    }
}

pub fn abstract_method_on_a_non_abstract_class(
    state: &mut State,
    class: &SimpleIdentifier,
    method: &SimpleIdentifier,
    abstract_span: Span,
    semicolon_span: Span,
) -> ParseError {
    ParseError::new(
        "E015",
        format!(
            "cannot declare method `{}::{}` abstract, as `{}` class is not abstract",
            state.named(&class),
            method.value,
            class,
        ),
        semicolon_span,
    )
    .error(
        "try removing this `abstract` modifier",
        abstract_span.position,
        "abstract".len(),
    )
    .highlight(class.span.position, class.value.len())
    .highlight(method.span.position, method.value.len())
}

pub fn constructor_in_enum(
    state: &mut State,
    r#enum: &SimpleIdentifier,
    constructor: &SimpleIdentifier,
) -> ParseError {
    ParseError::new(
        "E016",
        format!(
            "cannot declare a constructor on enum `{}`",
            state.named(&r#enum)
        ),
        constructor.span,
    )
    .error(
        "try removing this constructor",
        constructor.span.position,
        constructor.value.len(),
    )
    .highlight(r#enum.span.position, r#enum.value.len())
}

pub fn magic_method_in_enum(
    state: &mut State,
    r#enum: &SimpleIdentifier,
    method: &SimpleIdentifier,
) -> ParseError {
    ParseError::new(
        "E017",
        format!(
            "cannot declare magic method `{}::{}` in an enum",
            state.named(&r#enum),
            method.value
        ),
        method.span,
    )
    .error(
        "try removing this magic method",
        method.span.position,
        method.value.len(),
    )
    .highlight(r#enum.span.position, r#enum.value.len())
}

pub fn missing_case_value_for_backed_enum(
    state: &mut State,
    r#enum: &SimpleIdentifier,
    case: &SimpleIdentifier,
    semicolon_span: Span,
) -> ParseError {
    ParseError::new(
        "E018",
        format!(
            "case `{}::{}` of backed enum `{}` must have a value",
            state.named(&r#enum),
            case,
            r#enum
        ),
        semicolon_span,
    )
    .error("try adding a value", semicolon_span.position, 1)
    .highlight(case.span.position, case.value.len())
    .highlight(r#enum.span.position, r#enum.value.len())
}

pub fn case_value_for_unit_enum(
    state: &mut State,
    r#enum: &SimpleIdentifier,
    case: &SimpleIdentifier,
    equals_span: Span,
) -> ParseError {
    ParseError::new(
        "E019",
        format!(
            "case `{}::{}` of unit enum `{}` cannot have a value",
            state.named(&r#enum),
            case,
            r#enum
        ),
        equals_span,
    )
    .error("try replacing this with `;`", equals_span.position, 1)
    .highlight(case.span.position, case.value.len())
    .highlight(r#enum.span.position, r#enum.value.len())
}

pub fn modifier_cannot_be_used_for_constant(modifier: String, modifier_span: Span) -> ParseError {
    ParseError::new(
        "E020",
        format!("cannot use '{}' as constant modifier", modifier),
        modifier_span,
    )
    .error("try removing this", modifier_span.position, modifier.len())
    .note("only `public`, `protected`, `private`, and `final` modifiers can be used on constants")
}

pub fn modifier_cannot_be_used_for_interface_constant(
    modifier: String,
    modifier_span: Span,
) -> ParseError {
    ParseError::new(
        "E021",
        format!(
            "cannot use '{}' as an interface constant modifier",
            modifier
        ),
        modifier_span,
    )
    .error("try removing this", modifier_span.position, modifier.len())
    .note("only `public`, and `final` modifiers can be used on interface constants")
}

pub fn modifier_cannot_be_used_for_promoted_property(
    modifier: String,
    modifier_span: Span,
) -> ParseError {
    ParseError::new(
        "E022",
        format!("cannot use '{}' as a promoted property modifier", modifier),
        modifier_span,
    )
    .error(
        "try removing this",
        modifier_span.position,
        modifier.len(),
    )
    .note("only `public`, `protected`, `private`, and `readonly` modifiers can be used on promoted properties")
}

pub fn modifier_cannot_be_used_for_property(modifier: String, modifier_span: Span) -> ParseError {
    ParseError::new(
        "E023",
        format!("cannot use '{}' as a property modifier", modifier),
        modifier_span,
    )
    .error(
        "try removing this",
        modifier_span.position,
        modifier.len(),
    )
    .note("only `public`, `protected`, `private`, `static`, and `readonly` modifiers can be used on properties")
}

pub fn modifier_cannot_be_used_for_class(modifier: String, modifier_span: Span) -> ParseError {
    ParseError::new(
        "E024",
        format!("cannot use '{}' as a class modifier", modifier),
        modifier_span,
    )
    .error("try removing this", modifier_span.position, modifier.len())
    .note("only `final`, `abstract`, and `readonly` modifiers can be used on classes")
}

pub fn modifier_cannot_be_used_for_class_method(
    modifier: String,
    modifier_span: Span,
) -> ParseError {
    ParseError::new(
        "E025",
        format!("cannot use '{}' as a class method modifier", modifier),
        modifier_span,
    )
    .error(
        "try removing this",
        modifier_span.position,
        modifier.len(),
    )
    .note("only `public`, `protected`, `private`, `final`, `static`, and `abstract` modifiers can be used on class methods")
}

pub fn modifier_cannot_be_used_for_enum_method(
    modifier: String,
    modifier_span: Span,
) -> ParseError {
    ParseError::new(
        "E026",
        format!("cannot use '{}' as an enum method modifier", modifier),
        modifier_span,
    )
    .error(
        "try removing this",
        modifier_span.position,
        modifier.len(),
    )
    .note("only `public`, `protected`, `private`, `final`, and `static` modifiers can be used on enum methods")
}

pub fn modifier_cannot_be_used_for_interface_method(
    modifier: String,
    modifier_span: Span,
) -> ParseError {
    ParseError::new(
        "E027",
        format!("cannot use '{}' as an interface method modifier", modifier),
        modifier_span,
    )
    .error("try removing this", modifier_span.position, modifier.len())
    .note("only `public`, and `static` modifiers can be used on interface methods")
}

pub fn final_and_abstract_modifiers_combined_for_class(
    final_span: Span,
    abstract_span: Span,
) -> ParseError {
    ParseError::new(
        "E028",
        "cannot declare a `final` class as `abstract`",
        abstract_span,
    )
    .highlight(final_span.position, "final".len())
    .error(
        "try removing this",
        abstract_span.position,
        "abstract".len(),
    )
}

pub fn final_and_abstract_modifiers_combined_for_class_member(
    final_span: Span,
    abstract_span: Span,
) -> ParseError {
    ParseError::new(
        "E029",
        "cannot declare a `final` class member as `abstract`",
        abstract_span,
    )
    .highlight(final_span.position, "final".len())
    .error(
        "try removing this",
        abstract_span.position,
        "abstract".len(),
    )
}

pub fn final_and_private_modifiers_combined_for_constant(
    final_span: Span,
    private_span: Span,
) -> ParseError {
    ParseError::new(
        "E030",
        "cannot declare a `private` constant as `final`",
        final_span,
    )
    .highlight(private_span.position, "private".len())
    .error("try removing this", final_span.position, "final".len())
    .note("private constants cannot be final as they are not visible to other classes")
}

pub fn reached_unpredictable_state(span: Span) -> ParseError {
    ParseError::new("E031", "reached unpredictable state", span).error(
        "please report this as a bug",
        span.position,
        1,
    )
}

pub fn static_property_cannot_be_readonly(
    state: &mut State,
    class: Option<&SimpleIdentifier>,
    property: &SimpleVariable,
    static_span: Span,
    readonly_span: Span,
) -> ParseError {
    let error = ParseError::new(
        "E032",
        format!(
            "cannot declare `readonly` property `{}::{}` as 'static'",
            class
                .map(|c| state.named(c))
                .unwrap_or_else(|| "anonymous@class".to_string()),
            property.name,
        ),
        static_span,
    )
    .highlight(property.span.position, property.name.len())
    .highlight(readonly_span.position, "readonly".len())
    .error("try removing this", static_span.position, "static".len());

    // If the class is anonymous, we don't have a span to highlight
    if let Some(class) = class {
        error.highlight(class.span.position, class.value.len())
    } else {
        error
    }
}

pub fn readonly_property_has_default_value(
    state: &mut State,
    class: Option<&SimpleIdentifier>,
    property: &SimpleVariable,
    readonly_span: Span,
    equals_span: Span,
) -> ParseError {
    let error = ParseError::new(
        "E033",
        format!(
            "readonly property `{}::{}` cannot have a default value",
            class
                .map(|c| state.named(c))
                .unwrap_or_else(|| "anonymous@class".to_string()),
            property.name,
        ),
        equals_span,
    )
    .highlight(property.span.position, property.name.len())
    .highlight(readonly_span.position, "readonly".len())
    .error("try removing this `=`", equals_span.position, 1);

    // If the class is anonymous, we don't have a span to highlight
    if let Some(class) = class {
        error.highlight(class.span.position, class.value.len())
    } else {
        error
    }
}

pub fn unbraced_namespace_declarations_in_braced_context(span: Span) -> ParseError {
    ParseError::new(
        "E034",
        "cannot mix braced and unbraced namespace declarations",
        span,
    )
    .error("try replacing this `;` with `{`", span.position, 1)
}

pub fn braced_namespace_declarations_in_unbraced_context(span: Span) -> ParseError {
    ParseError::new(
        "E035",
        "cannot mix braced and unbraced namespace declarations",
        span,
    )
    .error("try replacing this `{` with `;`", span.position, 1)
}

pub fn nested_namespace_declarations(span: Span) -> ParseError {
    ParseError::new("E035", "cannot nest namespace declarations", span).error(
        "try closing previous namespace with `}` before declaring a new one",
        span.position,
        1,
    )
}

pub fn forbidden_type_used_in_property(
    state: &mut State,
    class: Option<&SimpleIdentifier>,
    property: &SimpleVariable,
    ty: Type,
) -> ParseError {
    let type_string = ty.to_string();
    let type_span = ty.first_span();

    let error = ParseError::new(
        "E037".to_string(),
        format!(
            "property `{}::{}` cannot have type `{}`",
            class
                .map(|c| state.named(c))
                .unwrap_or_else(|| "anonymous@class".to_string()),
            property.name,
            type_string
        ),
        type_span,
    )
    .highlight(property.span.position, property.name.len())
    .error(
        "try using a different type",
        type_span.position,
        type_string.len(),
    )
    .note("`void`, `never`, and `callable` types are not allowed in properties");

    // If the class is anonymous, we don't have a span to highlight
    if let Some(class) = class {
        error.highlight(class.span.position, class.value.len())
    } else {
        error
    }
}

pub fn match_expression_has_multiple_default_arms(first: Span, second: Span) -> ParseError {
    ParseError::new(
        "E038".to_string(),
        "match expression cannot have more than one default arm",
        second,
    )
    .highlight(first.position, "default".len())
    .error("try removing this arm", second.position, "default".len())
}

pub fn missing_item_definition_after_attributes(
    attributes: &Vec<AttributeGroup>,
    current: &Token,
) -> ParseError {
    let mut annotations = vec![];

    for attribute in attributes {
        annotations.push(ParseErrorAnnotation {
            r#type: ParseErrorAnnotationType::Hint,
            message: "".to_string(),
            position: attribute.start.position,
            length: attribute.end.position - attribute.start.position,
        });
    }

    annotations.push(match current.kind {
        TokenKind::Eof => ParseErrorAnnotation {
            r#type: ParseErrorAnnotationType::Error,
            message: "reached end of file before an item definition".to_string(),
            position: current.span.position,
            length: current.value.len(),
        },
        _ => ParseErrorAnnotation {
            r#type: ParseErrorAnnotationType::Error,
            message: format!("expected an item definition, found `{}`", current.value),
            position: current.span.position,
            length: current.value.len(),
        },
    });

    ParseError {
        id: "E039".to_string(),
        message: "missing item definition after attribute(s)".to_string(),
        span: current.span,
        annotations,
        note: None,
    }
}

pub fn nested_disjunctive_normal_form_types(span: Span) -> ParseError {
    ParseError::new(
        "E040".to_string(),
        "cannot nest disjunctive normal form types",
        span,
    )
    .error("try removing this", span.position, 1)
}

pub fn illegal_spread_operator_usage(span: Span) -> ParseError {
    ParseError::new("E041".to_string(), "illegal spread operator usage", span).error(
        "try removing this",
        span.position,
        3,
    )
}

pub fn cannot_assign_reference_to_non_referencable_value(span: Span) -> ParseError {
    ParseError::new(
        "E042".to_string(),
        "cannot assign reference to non-referencable value",
        span,
    )
    .error("try removing this", span.position, 1)
}

pub fn mixing_keyed_and_unkeyed_list_entries(span: Span) -> ParseError {
    ParseError::new(
        "E043".to_string(),
        "cannot mix keyed and un-keyed list entries",
        span,
    )
    .error("", span.position, 1)
}

pub fn cannot_use_positional_argument_after_named_argument(
    span: Span,
    current_span: Span,
) -> ParseError {
    ParseError::new(
        "E044".to_string(),
        "cannot use positional argument after named argument",
        span,
    )
    .error(
        "try adding a name for this argument",
        span.position,
        current_span.position - span.position,
    )
}

pub fn cannot_use_reserved_keyword_as_a_type_name(span: Span, keyword: String) -> ParseError {
    ParseError::new(
        "E045".to_string(),
        format!("cannot use reserved keyword `{}` as a type name", keyword),
        span,
    )
    .error("try using a different name", span.position, keyword.len())
}

pub fn cannot_use_reserved_keyword_as_a_goto_label(span: Span, keyword: String) -> ParseError {
    ParseError::new(
        "E046".to_string(),
        format!("cannot use reserved keyword `{}` as a goto label", keyword),
        span,
    )
    .error("try using a different name", span.position, keyword.len())
}

pub fn cannot_use_reserved_keyword_as_a_constant_name(span: Span, keyword: String) -> ParseError {
    ParseError::new(
        "E047".to_string(),
        format!(
            "cannot use reserved keyword `{}` as a constant name",
            keyword
        ),
        span,
    )
    .error("try using a different name", span.position, keyword.len())
}

pub fn cannot_use_type_in_context(span: Span, ty: String) -> ParseError {
    ParseError::new(
        "E048".to_string(),
        format!("cannot use type `{}` in current context", ty),
        span,
    )
    .error("try using a different type", span.position, ty.len())
}

pub fn only_positional_arguments_are_accepted(span: Span, current_span: Span) -> ParseError {
    ParseError::new(
        "E049".to_string(),
        "cannot use named argument, only positional arguments are accepted",
        span,
    )
    .error(
        "try changing this to a positional argument",
        span.position,
        current_span.position - span.position,
    )
}

pub fn only_one_argument_is_accepted(span: Span, current_span: Span) -> ParseError {
    ParseError::new("E050".to_string(), "only one argument are accepted", span).error(
        "try removing this argument",
        span.position,
        current_span.position - span.position,
    )
}

impl From<SyntaxError> for ParseError {
    fn from(e: SyntaxError) -> Self {
        Self {
            id: "E001".to_string(),
            message: format!("syntax error, {}", e),
            annotations: vec![],
            span: e.span(),
            note: None,
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] Error: {} on line {} column {}",
            self.id, self.message, self.span.line, self.span.column
        )?;

        if let Some(note) = &self.note {
            write!(f, ", Note: {}", note)?;
        }

        Ok(())
    }
}

impl Display for ParseErrorStack {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for error in &self.errors {
            writeln!(f, "{}", error)?;
        }

        Ok(())
    }
}
