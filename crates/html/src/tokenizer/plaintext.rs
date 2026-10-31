//! PLAINTEXT state only. Normative baseline: HTML Standard source at
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, "PLAINTEXT state" (#plaintext-state).
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>
//! See docs/sources.toml; initial-state selection is explicit, not tree building.

use super::{Context, Input, ParseError, Progress, State, Token};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point) if point.value() == 0 => {
            context.errors.push(ParseError {
                code: "unexpected-null-character",
                position: context.input.position(),
            });
            // Unlike Data, PLAINTEXT replaces NUL with U+FFFD.
            context.tokens.push(Token::Character('\u{fffd}'.into()));
            Progress::Continue(State::Plaintext)
        }
        Input::Eof => {
            context.tokens.push(Token::Eof);
            Progress::Finished
        }
        Input::CodePoint(point) => {
            context.tokens.push(Token::Character(point));
            Progress::Continue(State::Plaintext)
        }
        // Open, exhausted input is suspension, not the specification's EOF.
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
