//! RAWTEXT state only. Normative baseline: HTML Standard source at
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, "RAWTEXT state" (#rawtext-state).
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>
//! See docs/sources.toml; less-than sign and end tag open execute;
//! ASCII alpha in end tag open reaches the terminal end tag name stub.

use super::{Context, Input, ParseError, Progress, State, Token};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point) if point.value() == 0x3c => {
            Progress::Continue(State::RawtextLessThanSign)
        }
        Input::CodePoint(point) if point.value() == 0 => {
            context.errors.push(ParseError {
                code: "unexpected-null-character",
                position: context.input.position(),
            });
            context.tokens.push(Token::Character('\u{fffd}'.into()));
            Progress::Continue(State::Rawtext)
        }
        Input::Eof => {
            context.tokens.push(Token::Eof);
            Progress::Finished
        }
        Input::CodePoint(point) => {
            // In particular, '&' is ordinary input, not a character reference.
            context.tokens.push(Token::Character(point));
            Progress::Continue(State::Rawtext)
        }
        // Open, exhausted input is suspension, not the specification's EOF.
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
