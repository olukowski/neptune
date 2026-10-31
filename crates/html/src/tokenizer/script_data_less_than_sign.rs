//! Script data less-than sign only: pinned HTML Standard, lines 143654–143673.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143654-L143673>

use super::{Context, Input, Progress, State, Token};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point) if point.value() == 0x2f => {
            context.temporary_buffer.clear();
            Progress::Continue(State::ScriptDataEndTagOpen)
        }
        Input::CodePoint(point) if point.value() == 0x21 => {
            // Both character tokens precede any attempt to dispatch escape start.
            context.tokens.push(Token::Character('<'.into()));
            context.tokens.push(Token::Character('!'.into()));
            Progress::Continue(State::ScriptDataEscapeStart)
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
        // EOF is included in "anything else": emit only '<', not EOF.
        Input::CodePoint(_) | Input::Eof => {
            context.tokens.push(Token::Character('<'.into()));
            context
                .input
                .reconsume()
                .expect("just consumed a code point or EOF");
            Progress::Continue(State::ScriptData)
        }
    }
}
