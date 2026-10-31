//! Script data escape start only: pinned HTML Standard, lines 143741–143755.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143741-L143755>

use super::{Context, Input, Progress, State, Token};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point) if point.value() == 0x2d => {
            context.tokens.push(Token::Character('-'.into()));
            Progress::Continue(State::ScriptDataEscapeStartDash)
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
        // Conceptual EOF also takes the anything-else branch; Script data emits EOF.
        Input::CodePoint(_) | Input::Eof => {
            context
                .input
                .reconsume()
                .expect("just consumed a code point or EOF");
            Progress::Continue(State::ScriptData)
        }
    }
}
