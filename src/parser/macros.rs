#[macro_export]
macro_rules! peek_token {
    ([ $($(|)? $( $pattern:pat_param )|+ $( if $guard: expr )? => $out:expr),+ $(,)? ], $state:expr, [ $($message:literal),+ $(,)? ]) => {{
        $state.skip_comments();
        match $state.current.kind.clone() {
            $(
                $( $pattern )|+ $( if $guard )? => $out,
            )+
            _ => {
                return $crate::expected_token_err!([ $($message,)+ ], $state);
            }
        }
    }};
    ([ $($(|)? $( $pattern:pat_param )|+ $( if $guard: expr )?),+ $(,)? ], $state:expr, [ $($message:literal),+ $(,)? ]) => {{
        $state.skip_comments();
        if !matches!($state.current.kind, $( $pattern )|+ $( if $guard )?) {
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
    ([ $($(|)? $( $pattern:pat_param )|+ $( if $guard: expr )? => $out:expr),+ $(,)? ], $state:expr, [ $($message:literal),+ $(,)? ]) => {
        $crate::peek_token!([ $($( $pattern )|+ $( if $guard )? => { $state.next(); $out },)+ ], $state, [$($message,)+])
    };
    ([ $($(|)? $( $pattern:pat_param )|+ $( if $guard: expr )?),+ $(,)? ], $state:expr, [ $($message:literal),+ $(,)? ]) => {
        $crate::peek_token!([ $($( $pattern )|+ $( if $guard )? => { $state.next(); },)+ ], $state, [$($message,)+])
    };
    ([ $($(|)? $( $pattern:pat_param )|+ $( if $guard: expr )? => $out:expr),+ $(,)? ], $state:expr, $message:literal) => {
        $crate::peek_token!([ $($( $pattern )|+ $( if $guard )? => { $state.next(); $out },)+ ], $state, [$message])
    };
    ([ $($(|)? $( $pattern:pat_param )|+ $( if $guard: expr )?),+ $(,)? ], $state:expr, $message:literal) => {
        $crate::peek_token!([ $($( $pattern )|+ $( if $guard )? => { $state.next(); },)+ ], $state, [$message])
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
        Err($crate::expected_token!([$($expected),+], $state))
    }};

    ($expected:literal, $state:expr $(,)?) => {
        $crate::expected_token_err!([$expected], $state)
    };
}

#[macro_export]
macro_rules! expected_token {
    ([ $($expected:literal),+ $(,)? ], $state:expr $(,)?) => {{
        match &$state.current.kind {
            TokenKind::Eof => {
                $crate::parser::error::ParseError::ExpectedToken(
                    vec![$($expected.into()),+],
                    None,
                    $state.current.span,
                )
            },
            _ => {
                $crate::parser::error::ParseError::ExpectedToken(
                    vec![$($expected.into()),+],
                    Some($state.current.kind.to_string()),
                    $state.current.span,
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
                return Err($crate::parser::error::ParseError::UnpredictableState($state.current.span));
            }
        }
    }};
}

#[macro_export]
macro_rules! scoped {
    ($state:expr, $scope:expr, $block:block) => {{
        let scope = $scope;
        $state.enter(scope.clone());

        let result = $block?;

        $state.exit();

        Ok(result)
    }};
}
