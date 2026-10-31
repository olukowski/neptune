//! Data state only. Normative baseline: HTML Standard source at
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, "Data state" (#data-state).
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>
//! See docs/sources.toml; destination algorithms are deliberately not implemented.

use super::{Context, Input, ParseError, Progress, State, Token};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point) => match point.value() {
            0x26 => {
                context.return_state = Some(State::Data);
                Progress::Continue(State::CharacterReference)
            }
            0x3c => Progress::Continue(State::TagOpen),
            0 => {
                // Unlike RCDATA, Data emits the current NUL, not U+FFFD.
                context.errors.push(ParseError {
                    code: "unexpected-null-character",
                    position: context.input.position(),
                });
                context.tokens.push(Token::Character(point));
                Progress::Continue(State::Data)
            }
            _ => {
                context.tokens.push(Token::Character(point));
                Progress::Continue(State::Data)
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
