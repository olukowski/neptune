//! Comment less-than sign only. Normative baseline: HTML Standard source at
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, lines 144590–144607
//! (#comment-less-than-sign-state).
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144590-L144607>

use super::{Context, Input, Progress, State, comment};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        // U+0021 EXCLAMATION MARK: append, then switch without reconsumption.
        Input::CodePoint(point) if point.value() == 0x21 => {
            comment::data_mut(context).push(point);
            Progress::Continue(State::CommentLessThanSignBang)
        }
        // U+003C LESS-THAN SIGN: append and stay in this state.
        Input::CodePoint(point) if point.value() == 0x3c => {
            comment::data_mut(context).push(point);
            Progress::Continue(State::CommentLessThanSign)
        }
        // Anything else INCLUDES EOF. No append, emission or state error here.
        Input::CodePoint(_) | Input::Eof => {
            context.input.reconsume().expect("current input exists");
            Progress::Continue(State::Comment)
        }
        // Open exhaustion is not EOF and must not schedule reconsumption.
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
