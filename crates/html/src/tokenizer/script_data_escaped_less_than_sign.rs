//! Script data escaped less-than sign only, pinned HTML Standard lines 143872–143892.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143872-L143892>
//! Branch/definition evidence: docs/tokenizer-script-data-escaped-less-than-sign.md.

use super::{Context, Input, Progress, State, Token};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point) if point.value() == 0x2f => {
            context.temporary_buffer.clear();
            Progress::Continue(State::ScriptDataEscapedEndTagOpen)
        }
        Input::CodePoint(point) if matches!(point.value(), 0x41..=0x5a | 0x61..=0x7a) => {
            context.temporary_buffer.clear();
            context.tokens.push(Token::Character('<'.into()));
            context
                .input
                .reconsume()
                .expect("just consumed an ASCII alpha");
            Progress::Continue(State::ScriptDataDoubleEscapeStart)
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
        // Conceptual EOF also takes the literal fallback, not an EOF-emission branch.
        Input::CodePoint(_) | Input::Eof => {
            context.tokens.push(Token::Character('<'.into()));
            context
                .input
                .reconsume()
                .expect("just consumed a code point or EOF");
            Progress::Continue(State::ScriptDataEscaped)
        }
    }
}
