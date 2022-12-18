#[macro_export]
macro_rules! peek_token {
    ([ $($(|)? $( $pattern:pat_param )|+ $( if $guard: expr )? => $out:expr),+ $(,)? ], $state:expr, [ $($message:literal),+ $(,)? ]) => {{
        match &$state.stream.current().kind {
            $(
                $( $pattern )|+ $( if $guard )? => $out,
            )+
            _ => {
                return $crate::expected_token_err!([ $($message,)+ ], $state);
            }
        }
    }};
    ([ $($(|)? $( $pattern:pat_param )|+ $( if $guard: expr )?),+ $(,)? ], $state:expr, [ $($message:literal),+ $(,)? ]) => {{
        if !matches!($state.stream.current().kind, $( $pattern )|+ $( if $guard )?) {
            return $crate::expected_token_err!([ $($message,)+ ], $state);
        }
    }};
    ([ $($(|)? $( $pattern:pat_param )|+ $( if $guard: expr )? => $out:expr),+ $(,)? ], $state:expr, $message:literal) => {
        $crate::peek_token!([ $($( $pattern )|+ $( if $guard )? => $out,)+ ], $state, [$message])
    };
    ([ $($(|)? $( $pattern:pat_param )|+ $( if $guard: expr )?),+ $(,)? ], $state:expr, $message:literal) => {
        $crate::peek_token!([ $($( $pattern )|+ $( if $guard )?,)+ ], $state, [$message])
    };
}

#[macro_export]
macro_rules! expect_token {
    ([ $($(|)? $( $pattern:pat_param )|+ $( if $guard: expr )? => $out:expr),+ $(,)? ], $state:expr, [ $($message:literal),+ $(,)? ]) => {{
        let token = $state.stream.current().clone();
        $state.stream.next();
        match token.kind {
            $(
                $( $pattern )|+ $( if $guard )? => {
                    $out
                },
            )+
            TokenKind::Eof => {
                return Err($crate::parser::error::ParseError::ExpectedToken(
                    vec![$($message.into(),)+],
                    None,
                    token.span,
                ))
            },
            _ => {
                return Err($crate::parser::error::ParseError::ExpectedToken(
                    vec![$($message.into(),)+],
                    Some(token.kind.to_string()),
                    token.span,
                ))
            }
        }
    }};
    ([ $($(|)? $( $pattern:pat_param )|+ $( if $guard: expr )? => $out:expr),+ $(,)? ], $state:expr, $message:literal) => {
        $crate::expect_token!([ $($( $pattern )|+ $( if $guard )? => $out,)+ ], $state, [$message])
    };
}

#[macro_export]
macro_rules! expect_literal {
    ($state:expr) => {{
        let current = $state.stream.current();

        match &current.kind {
            TokenKind::LiteralInteger(value) => {
                $state.stream.next();

                $crate::parser::ast::literals::Literal::Integer(
                    $crate::parser::ast::literals::LiteralInteger {
                        span: current.span,
                        value: value.clone(),
                    },
                )
            }
            TokenKind::LiteralFloat(value) => {
                $state.stream.next();

                $crate::parser::ast::literals::Literal::Float(
                    $crate::parser::ast::literals::LiteralFloat {
                        span: current.span,
                        value: value.clone(),
                    },
                )
            }
            TokenKind::LiteralString(value) => {
                $state.stream.next();

                $crate::parser::ast::literals::Literal::String(
                    $crate::parser::ast::literals::LiteralString {
                        span: current.span,
                        value: value.clone(),
                    },
                )
            }
            _ => {
                return $crate::expected_token_err!(["a literal"], $state);
            }
        }
    }};
}

#[macro_export]
macro_rules! expected_token_err {
    ([ $($expected:literal),+ $(,)? ], $state:expr $(,)?) => {{
        Err($crate::expected_token!([$($expected),+], $state))
    }};

    ($expected:literal, $state:expr $(,)?) => {
        $crate::expected_token_err!([$expected], $state)
    };
}

#[macro_export]
macro_rules! expected_token {
    ([ $($expected:literal),+ $(,)? ], $state:expr $(,)?) => {{
        let current = $state.stream.current();

        match &current.kind {
            TokenKind::Eof => {
                $crate::parser::error::ParseError::ExpectedToken(
                    vec![$($expected.into()),+],
                    None,
                    current.span,
                )
            },
            _ => {
                $crate::parser::error::ParseError::ExpectedToken(
                    vec![$($expected.into()),+],
                    Some(current.kind.to_string()),
                    current.span,
                )
            }
        }
    }};

    ($expected:literal, $state:expr $(,)?) => {
        $crate::expected_token!([$expected], $state)
    };
}

#[macro_export]
macro_rules! expected_scope {
    ([ $($(|)? $( $pattern:pat_param )|+ $( if $guard: expr )? => $out:expr),+ $(,)? ], $state:expr) => {{
        match $state.scope().cloned()? {
            $(
                $( $pattern )|+ $( if $guard )? => $out,
            )+
            _ => {
                return Err($crate::parser::error::ParseError::UnpredictableState($state.stream.current().span));
            }
        }
    }};
}

#[macro_export]
macro_rules! scoped {
    ($state:expr, $scope:expr, $block:block) => {{
        $state.enter($scope);

        let result = $block;

        $state.exit();

        result
    }};
}
