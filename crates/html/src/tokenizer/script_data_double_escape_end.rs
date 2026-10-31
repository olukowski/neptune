//! Script data double escape end only: pinned HTML revision
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, source lines 144110–144142.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144110-L144142>
//! Branch/entry evidence: docs/tokenizer-script-data-double-escape-end.md.

use super::{CodePoint, Context, Input, Progress, State, Token};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        // CR has already been normalized to LF by the cursor.
        Input::CodePoint(point)
            if matches!(point.value(), 0x09 | 0x0a | 0x0c | 0x20 | 0x2f | 0x3e) =>
        {
            let target = if context.temporary_buffer
                == [
                    's'.into(),
                    'c'.into(),
                    'r'.into(),
                    'i'.into(),
                    'p'.into(),
                    't'.into(),
                ] {
                State::ScriptDataEscaped
            } else {
                State::ScriptDataDoubleEscaped
            };
            context.tokens.push(Token::Character(point));
            Progress::Continue(target)
        }
        Input::CodePoint(point) if matches!(point.value(), 0x41..=0x5a) => {
            context
                .temporary_buffer
                .push(CodePoint::new(point.value() + 0x20).unwrap());
            context.tokens.push(Token::Character(point));
            Progress::Continue(State::ScriptDataDoubleEscapeEnd)
        }
        Input::CodePoint(point) if matches!(point.value(), 0x61..=0x7a) => {
            context.temporary_buffer.push(point);
            context.tokens.push(Token::Character(point));
            Progress::Continue(State::ScriptDataDoubleEscapeEnd)
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
        // Anything else includes conceptual EOF. No emission or state error here;
        // only the destination may replace NUL or diagnose/emit EOF.
        Input::CodePoint(_) | Input::Eof => {
            context
                .input
                .reconsume()
                .expect("just consumed a code point or EOF");
            Progress::Continue(State::ScriptDataDoubleEscaped)
        }
    }
}
