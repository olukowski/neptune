//! Script data state only (#script-data-state), pinned WHATWG HTML source
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, lines 143338–143359:
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143338-L143359>
//! Branch/definition evidence: docs/tokenizer-script-data.md.
//! Script data less-than sign remains a terminal unimplemented destination.

use super::{Context, Input, ParseError, Progress, State, Token};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point) if point.value() == 0x3c => {
            Progress::Continue(State::ScriptDataLessThanSign)
        }
        Input::CodePoint(point) if point.value() == 0 => {
            context.errors.push(ParseError {
                code: "unexpected-null-character",
                position: context.input.position(),
            });
            context.tokens.push(Token::Character('\u{fffd}'.into()));
            Progress::Continue(State::ScriptData)
        }
        Input::Eof => {
            context.tokens.push(Token::Eof);
            Progress::Finished
        }
        Input::CodePoint(point) => {
            // Ampersands are ordinary script data, not character references.
            context.tokens.push(Token::Character(point));
            Progress::Continue(State::ScriptData)
        }
        // Exhausting open input is not the specification's conceptual EOF.
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
