//! Comment end only (#comment-end-state). Normative baseline:
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144681-L144706>
//! Shared pending-comment operations live in comment.rs, not this state module.

use super::{CodePoint, Context, Input, ParseError, Progress, State, Token, comment};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        // '>': switch to Data and emit the current comment, excluding the delimiter.
        Input::CodePoint(point) if point.value() == 0x3e => {
            comment::emit(context);
            Progress::Continue(State::Data)
        }
        // '!': switch without appending, emitting, diagnosing or reconsuming.
        Input::CodePoint(point) if point.value() == 0x21 => {
            Progress::Continue(State::CommentEndBang)
        }
        // '-': append just this extra hyphen; remain in Comment end.
        Input::CodePoint(point) if point.value() == 0x2d => {
            comment::data_mut(context).push(point);
            Progress::Continue(State::CommentEnd)
        }
        // EOF: diagnose, emit the comment, then EOF. No state switch/reconsumption.
        Input::Eof => {
            context.errors.push(ParseError {
                code: "eof-in-comment",
                position: context.input.position(),
            });
            comment::emit(context);
            context.tokens.push(Token::Eof);
            Progress::Finished
        }
        // Anything else: restore the two withheld hyphens, NOT the current point.
        // NUL replacement and other Comment behavior belong to the destination.
        Input::CodePoint(_) => {
            comment::data_mut(context).extend([CodePoint::from('-'); 2]);
            context.input.reconsume().expect("current input exists");
            Progress::Continue(State::Comment)
        }
        // Exhausted open input is not EOF.
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
