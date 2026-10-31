//! Script data double escape start only, pinned HTML source lines 143958–143991.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143958-L143991>
//! Branch and entry evidence: docs/tokenizer-script-data-double-escape-start.md.

use super::{CodePoint, Context, Input, Progress, State, Token};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point)
            if matches!(point.value(), 0x09 | 0x0a | 0x0c | 0x20 | 0x2f | 0x3e) =>
        {
            let target = if context
                .temporary_buffer
                .iter()
                .map(|point| point.value())
                .eq(b"script".iter().map(|&byte| u32::from(byte)))
            {
                State::ScriptDataDoubleEscaped
            } else {
                State::ScriptDataEscaped
            };
            context.tokens.push(Token::Character(point));
            Progress::Continue(target)
        }
        Input::CodePoint(point) if (0x41..=0x5a).contains(&point.value()) => {
            context
                .temporary_buffer
                .push(CodePoint::new(point.value() + 0x20).expect("ASCII lowercase"));
            context.tokens.push(Token::Character(point));
            Progress::Continue(State::ScriptDataDoubleEscapeStart)
        }
        Input::CodePoint(point) if (0x61..=0x7a).contains(&point.value()) => {
            context.temporary_buffer.push(point);
            context.tokens.push(Token::Character(point));
            Progress::Continue(State::ScriptDataDoubleEscapeStart)
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
        // EOF is also anything else; the escaped destination supplies its error/EOF.
        Input::CodePoint(_) | Input::Eof => {
            context
                .input
                .reconsume()
                .expect("just consumed a code point or EOF");
            Progress::Continue(State::ScriptDataEscaped)
        }
    }
}
