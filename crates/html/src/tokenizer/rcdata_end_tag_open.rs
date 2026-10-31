//! RCDATA end tag open only. Normative baseline: HTML Standard source at
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, lines 143505–143522.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143505-L143522>
//! Creation defaults follow the opening Tokenization paragraphs.

use super::{Context, Input, PendingEndTag, Progress, State, TagKind, Token};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        // ASCII alpha: create/select an EMPTY end tag, without appending,
        // lowercasing, emitting, or changing the temporary buffer.
        Input::CodePoint(point) if matches!(point.value(), 0x41..=0x5a | 0x61..=0x7a) => {
            context.pending_end_tag = Some(PendingEndTag::default());
            context.current_tag = Some(TagKind::End);
            context
                .input
                .reconsume()
                .expect("just consumed a code point");
            Progress::Continue(State::RcdataEndTagName)
        }
        // Anything else INCLUDES EOF. Emit only the literal prefix here;
        // RCDATA handles the reconsumed point or conceptual EOF next.
        Input::CodePoint(_) | Input::Eof => {
            context
                .tokens
                .extend([Token::Character('<'.into()), Token::Character('/'.into())]);
            context
                .input
                .reconsume()
                .expect("just consumed a code point or EOF");
            Progress::Continue(State::Rcdata)
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
