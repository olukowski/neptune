//! Script data escaped end tag open only (#script-data-escaped-end-tag-open-state).
//! HTML source revision 404b3eb14a7a936dddefb2d78e224dbeb01d2358:
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143895-L143911>
//! Creation defaults/reconsumption: lines 143160–143189. Branch evidence and
//! isolated-entry requirements: docs/tokenizer-script-data-escaped-end-tag-open.md.

use super::{Context, Input, PendingEndTag, Progress, State, TagKind, Token};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point) if matches!(point.value(), 0x41..=0x5a | 0x61..=0x7a) => {
            // Create only; the destination owns normalization and accumulation.
            context.pending_end_tag = Some(PendingEndTag::default());
            context.current_tag = Some(TagKind::End);
            context
                .input
                .reconsume()
                .expect("just consumed a code point");
            Progress::Continue(State::ScriptDataEscapedEndTagName)
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
        // Anything else includes conceptual EOF, reconsumed by Escaped.
        Input::CodePoint(_) | Input::Eof => {
            context
                .tokens
                .extend([Token::Character('<'.into()), Token::Character('/'.into())]);
            context
                .input
                .reconsume()
                .expect("just consumed a code point or EOF");
            Progress::Continue(State::ScriptDataEscaped)
        }
    }
}
