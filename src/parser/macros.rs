#[macro_export]
macro_rules! peek_token {
    ([ $($expected:pat => $out:expr),+ $(,)? ], $parser:expr, [ $($message:literal),+ $(,)? ]) => {{
        $parser.skip_comments();
        match $parser.current.kind.clone() {
            $(
                $expected => $out,
            )+
            _ => {
                return $crate::expected_token_err!([ $($message,)+ ], $parser);
            }
        }
    }};
    ([ $($expected:pat),+ $(,)? ], $parser:expr, [ $($message:literal),+ $(,)? ]) => {{
        $parser.skip_comments();
        if !matches!($parser.current.kind, $(| $expected )+) {
            return $crate::expected_token_err!([ $($message,)+ ], $parser);
        }
    }};
    ([ $($expected:pat => $out:expr),+ $(,)? ], $parser:expr, $message:literal) => {
        $crate::peek_token!([ $($expected => $out,)+ ], $parser, [$message])
    };
    ([ $($expected:pat),+ $(,)? ], $parser:expr, $message:literal) => {
        $crate::peek_token!([ $($expected,)+ ], $parser, [$message])
    };
}

#[macro_export]
macro_rules! expect_token {
    ([ $($expected:pat => $out:expr),+ $(,)? ], $parser:expr, [ $($message:literal),+ $(,)? ]) => {
        $crate::peek_token!([ $($expected => { $parser.next(); $out },)+ ], $parser, [$($message,)+])
    };
    ([ $($expected:pat),+ $(,)? ], $parser:expr, [ $($message:literal),+ $(,)? ]) => {
        $crate::peek_token!([ $($expected => { $parser.next(); },)+ ], $parser, [$($message,)+])
    };
    ([ $($expected:pat => $out:expr),+ $(,)? ], $parser:expr, $message:literal) => {
        $crate::peek_token!([ $($expected => { $parser.next(); $out },)+ ], $parser, [$message])
    };
    ([ $($expected:pat),+ $(,)? ], $parser:expr, $message:literal) => {
        $crate::peek_token!([ $($expected => { $parser.next(); },)+ ], $parser, [$message])
    };
}

#[macro_export]
macro_rules! expect_literal {
    ($parser:expr) => {{
        $parser.skip_comments();
        match $parser.current.kind.clone() {
            TokenKind::LiteralInteger(i) => {
                let e = Expression::LiteralInteger { i };
                $parser.next();
                e
            }
            TokenKind::LiteralFloat(f) => {
                let e = Expression::LiteralFloat { f };
                $parser.next();
                e
            }
            TokenKind::LiteralString(s) => {
                let e = Expression::LiteralString { value: s.clone() };
                $parser.next();
                e
            }
            _ => {
                return $crate::expected_token_err!(["a literal"], $parser);
            }
        }
    }};
}

#[macro_export]
macro_rules! expected_token_err {
    ([ $($expected:literal),+ $(,)? ], $parser:expr $(,)?) => {{
        match &$parser.current.kind {
            TokenKind::Eof => {
                Err($crate::parser::error::ParseError::ExpectedToken(
                    vec![$($expected.into()),+],
                    None,
                    $parser.current.span,
                ))
            },
            _ => {
                Err($crate::parser::error::ParseError::ExpectedToken(
                    vec![$($expected.into()),+],
                    Some($parser.current.kind.to_string()),
                    $parser.current.span,
                ))
            }
        }
    }};

    ($expected:literal, $parser:expr $(,)?) => {
        $crate::expected_token_err!([$expected], $parser)
    };
}
