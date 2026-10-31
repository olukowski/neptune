//! RCDATA state only. Normative baseline: HTML Standard source at
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, "RCDATA state" (#rcdata-state).
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>
//! See docs/sources.toml; destination algorithms remain explicit stubs.

use super::{Context, Input, ParseError, Progress, State, Token};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point) => match point.value() {
            0x26 => {
                context.return_state = Some(State::Rcdata);
                Progress::Continue(State::CharacterReference)
            }
            0x3c => Progress::Continue(State::RcdataLessThanSign),
            0 => {
                context.errors.push(ParseError {
                    code: "unexpected-null-character",
                    position: context.input.position(),
                });
                // Unlike Data, RCDATA emits U+FFFD rather than the current NUL.
                context.tokens.push(Token::Character('\u{fffd}'.into()));
                Progress::Continue(State::Rcdata)
            }
            _ => {
                context.tokens.push(Token::Character(point));
                Progress::Continue(State::Rcdata)
            }
        },
        Input::Eof => {
            context.tokens.push(Token::Eof);
            Progress::Finished
        }
        // Open, exhausted input is suspension, not the specification's EOF.
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
