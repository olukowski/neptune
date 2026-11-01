//! Comment start only: pinned HTML Standard, source lines 144514–144532.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144514-L144532>

use super::{Context, Input, ParseError, Progress, State, comment};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point) if point.value() == 0x2d => {
            Progress::Continue(State::CommentStartDash)
        }
        Input::CodePoint(point) if point.value() == 0x3e => {
            context.errors.push(ParseError {
                code: "abrupt-closing-of-empty-comment",
                position: context.input.position(),
            });
            comment::emit(context);
            Progress::Continue(State::Data)
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
        // Includes conceptual EOF. Comment, not this state, handles EOF/NUL/data.
        Input::CodePoint(_) | Input::Eof => {
            context
                .input
                .reconsume()
                .expect("just consumed a code point or EOF");
            Progress::Continue(State::Comment)
        }
    }
}
