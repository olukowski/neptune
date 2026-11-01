//! Comment less-than sign bang dash only: pinned HTML Standard, lines 144626–144640.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144626-L144640>
//! Branch evidence: docs/tokenizer-comment-less-than-sign-bang-dash.md.

use super::{Context, Input, Progress, State};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point) if point.value() == 0x2d => {
            Progress::Continue(State::CommentLessThanSignBangDashDash)
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
        // "Anything else" includes conceptual EOF. The destination, not this
        // state, owns comment data mutation, diagnostics and emission.
        Input::CodePoint(_) | Input::Eof => {
            context
                .input
                .reconsume()
                .expect("just consumed a code point or EOF");
            Progress::Continue(State::CommentEndDash)
        }
    }
}
