//! RAWTEXT end tag open only. Normative baseline: HTML Standard source at
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, lines 143589–143607.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143589-L143607>
//! Creation defaults and reconsumption follow the opening Tokenization paragraphs.

use super::{Context, Input, PendingEndTag, Progress, State, TagKind, Token};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point) if matches!(point.value(), 0x41..=0x5a | 0x61..=0x7a) => {
            // Create, do not emit or append the matched letter (even uppercase).
            context.pending_end_tag = Some(PendingEndTag::default());
            context.current_tag = Some(TagKind::End);
            context
                .input
                .reconsume()
                .expect("just consumed a code point");
            Progress::Continue(State::RawtextEndTagName)
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
        // Anything else includes conceptual EOF; RAWTEXT handles it next.
        _ => {
            context
                .tokens
                .extend([Token::Character('<'.into()), Token::Character('/'.into())]);
            context
                .input
                .reconsume()
                .expect("just consumed a code point or EOF");
            Progress::Continue(State::Rawtext)
        }
    }
}
