//! RAWTEXT less-than sign only. Normative baseline: HTML Standard source at
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, #rawtext-less-than-sign-state.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143570-L143586>

use super::{Context, Input, Progress, State, Token};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point) if point.value() == 0x2f => {
            // U+002F SOLIDUS: initialize scratch storage, not an end tag token.
            context.temporary_buffer.clear();
            Progress::Continue(State::RawtextEndTagOpen)
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
        // Anything else includes conceptual EOF. RAWTEXT handles the reconsumed
        // input on the next step; this source emits only the literal '<'.
        Input::CodePoint(_) | Input::Eof => {
            context.tokens.push(Token::Character('<'.into()));
            context
                .input
                .reconsume()
                .expect("just consumed a code point or EOF");
            Progress::Continue(State::Rawtext)
        }
    }
}
