//! Script data double escaped less-than sign only: pinned HTML Standard,
//! revision 404b3eb14a7a936dddefb2d78e224dbeb01d2358, lines 144093–144108.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144093-L144108>
//! Branch/definition evidence: docs/tokenizer-script-data-double-escaped-less-than-sign.md.

use super::{Context, Input, Progress, State, Token};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point) if point.value() == 0x2f => {
            context.temporary_buffer.clear();
            context.tokens.push(Token::Character('/'.into()));
            Progress::Continue(State::ScriptDataDoubleEscapeEnd)
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
        // Anything else includes conceptual EOF. No emission or error here;
        // the preceding double-escaped state already emitted '<'.
        Input::CodePoint(_) | Input::Eof => {
            context
                .input
                .reconsume()
                .expect("just consumed a code point or EOF");
            Progress::Continue(State::ScriptDataDoubleEscaped)
        }
    }
}
