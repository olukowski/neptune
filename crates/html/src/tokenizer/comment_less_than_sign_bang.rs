//! Comment less-than sign bang only: pinned HTML Standard, lines 144610–144624.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144610-L144624>
//! Isolated entry supplies the current pending comment; this state never mutates
//! or emits it. The predecessor has already appended '<!'.

use super::{Context, Input, Progress, State};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point) if point.value() == 0x2d => {
            Progress::Continue(State::CommentLessThanSignBangDash)
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
        // EOF also takes anything else. Comment, not this state, handles it.
        Input::CodePoint(_) | Input::Eof => {
            context
                .input
                .reconsume()
                .expect("just consumed a code point or EOF");
            Progress::Continue(State::Comment)
        }
    }
}
