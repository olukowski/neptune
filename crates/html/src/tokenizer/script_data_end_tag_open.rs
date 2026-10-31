//! Script data end tag open only. Normative baseline: HTML Standard source at
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, lines 143676–143692.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143676-L143692>
//! Creation defaults and reconsumption: lines 143160–143189.

use super::{Context, Input, PendingEndTag, Progress, State, TagKind, Token};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point) if matches!(point.value(), 0x41..=0x5a | 0x61..=0x7a) => {
            // Creation only: the name state handles the original letter next.
            context.pending_end_tag = Some(PendingEndTag::default());
            context.current_tag = Some(TagKind::End);
            context
                .input
                .reconsume()
                .expect("just consumed a code point");
            Progress::Continue(State::ScriptDataEndTagName)
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
        // Anything else includes conceptual EOF, reconsumed by Script data.
        _ => {
            context
                .tokens
                .extend([Token::Character('<'.into()), Token::Character('/'.into())]);
            context
                .input
                .reconsume()
                .expect("just consumed a code point or EOF");
            Progress::Continue(State::ScriptData)
        }
    }
}
