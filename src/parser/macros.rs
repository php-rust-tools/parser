#[macro_export]
macro_rules! peek_token {
    ([ $($expected:pat => $out:expr),+ $(,)? ], $state:expr, [ $($message:literal),+ $(,)? ]) => {{
        $state.skip_comments();
        match $state.current.kind.clone() {
            $(
                $expected => $out,
            )+
            _ => {
                return $crate::expected_token_err!([ $($message,)+ ], $state);
            }
        }
    }};
    ([ $($expected:pat),+ $(,)? ], $state:expr, [ $($message:literal),+ $(,)? ]) => {{
        $state.skip_comments();
        if !matches!($state.current.kind, $(| $expected )+) {
            return $crate::expected_token_err!([ $($message,)+ ], $state);
        }
    }};
    ([ $($expected:pat => $out:expr),+ $(,)? ], $state:expr, $message:literal) => {
        $crate::peek_token!([ $($expected => $out,)+ ], $state, [$message])
    };
    ([ $($expected:pat),+ $(,)? ], $state:expr, $message:literal) => {
        $crate::peek_token!([ $($expected,)+ ], $state, [$message])
    };
}

#[macro_export]
macro_rules! expect_token {
    ([ $($expected:pat => $out:expr),+ $(,)? ], $state:expr, [ $($message:literal),+ $(,)? ]) => {
        $crate::peek_token!([ $($expected => { $state.next(); $out },)+ ], $state, [$($message,)+])
    };
    ([ $($expected:pat),+ $(,)? ], $state:expr, [ $($message:literal),+ $(,)? ]) => {
        $crate::peek_token!([ $($expected => { $state.next(); },)+ ], $state, [$($message,)+])
    };
    ([ $($expected:pat => $out:expr),+ $(,)? ], $state:expr, $message:literal) => {
        $crate::peek_token!([ $($expected => { $state.next(); $out },)+ ], $state, [$message])
    };
    ([ $($expected:pat),+ $(,)? ], $state:expr, $message:literal) => {
        $crate::peek_token!([ $($expected => { $state.next(); },)+ ], $state, [$message])
    };
}

#[macro_export]
macro_rules! expect_literal {
    ($state:expr) => {{
        $state.skip_comments();
        match $state.current.kind.clone() {
            TokenKind::LiteralInteger(i) => {
                let e = Expression::LiteralInteger { i };
                $state.next();
                e
            }
            TokenKind::LiteralFloat(f) => {
                let e = Expression::LiteralFloat { f };
                $state.next();
                e
            }
            TokenKind::LiteralString(s) => {
                let e = Expression::LiteralString { value: s.clone() };
                $state.next();
                e
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
        match &$state.current.kind {
            TokenKind::Eof => {
                Err($crate::parser::error::ParseError::ExpectedToken(
                    vec![$($expected.into()),+],
                    None,
                    $state.current.span,
                ))
            },
            _ => {
                Err($crate::parser::error::ParseError::ExpectedToken(
                    vec![$($expected.into()),+],
                    Some($state.current.kind.to_string()),
                    $state.current.span,
                ))
            }
        }
    }};

    ($expected:literal, $state:expr $(,)?) => {
        $crate::expected_token_err!([$expected], $state)
    };
}
