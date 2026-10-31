//! Script data escape start dash only: pinned HTML Standard, lines 143758–143773.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143758-L143773>
//! Branch and definition evidence: docs/tokenizer-script-data-escape-start-dash.md.

use super::{Context, Input, Progress, State, Token};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point) if point.value() == 0x2d => {
            context.tokens.push(Token::Character('-'.into()));
            Progress::Continue(State::ScriptDataEscapedDashDash)
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
        // Conceptual EOF also takes "anything else"; only Script data emits EOF.
        Input::CodePoint(_) | Input::Eof => {
            context
                .input
                .reconsume()
                .expect("just consumed a code point or EOF");
            Progress::Continue(State::ScriptData)
        }
    }
}
