//! Comment end bang only (#comment-end-bang-state).
//! Pinned HTML 404b3eb14a7a936dddefb2d78e224dbeb01d2358, source 144708–144732.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144708-L144732>

use super::{CodePoint, Context, Input, ParseError, Progress, State, Token, comment};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        // Hyphen: append the withheld "--!", NOT this hyphen; consume it.
        Input::CodePoint(point) if point.value() == 0x2d => {
            comment::data_mut(context).extend("--!".chars().map(CodePoint::from));
            Progress::Continue(State::CommentEndDash)
        }
        // Greater-than: diagnose and emit without appending the withheld suffix.
        Input::CodePoint(point) if point.value() == 0x3e => {
            context.errors.push(ParseError {
                code: "incorrectly-closed-comment",
                position: context.input.position(),
            });
            comment::emit(context);
            Progress::Continue(State::Data)
        }
        Input::Eof => {
            context.errors.push(ParseError {
                code: "eof-in-comment",
                position: context.input.position(),
            });
            comment::emit(context);
            context.tokens.push(Token::Eof);
            Progress::Finished
        }
        // Anything else: append only "--!"; Comment handles the reconsumed point.
        Input::CodePoint(_) => {
            comment::data_mut(context).extend("--!".chars().map(CodePoint::from));
            context.input.reconsume().expect("current input exists");
            Progress::Continue(State::Comment)
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
