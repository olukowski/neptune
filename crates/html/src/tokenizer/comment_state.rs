//! Comment only (not the shared `comment` helpers).
//! Pinned WHATWG HTML 404b3eb14a7a936dddefb2d78e224dbeb01d2358,
//! "Comment state", source lines 144561–144587:
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144561-L144587>

use super::{Context, Input, ParseError, Progress, State, Token, comment};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        // '<': append the consumed character before switching.
        Input::CodePoint(point) if point.value() == 0x3c => {
            comment::data_mut(context).push(point);
            Progress::Continue(State::CommentLessThanSign)
        }
        // '-': switch without appending the delimiter.
        Input::CodePoint(point) if point.value() == 0x2d => {
            Progress::Continue(State::CommentEndDash)
        }
        Input::CodePoint(point) if point.value() == 0 => {
            context.errors.push(ParseError {
                code: "unexpected-null-character",
                position: context.input.position(),
            });
            comment::data_mut(context).push('\u{fffd}'.into());
            Progress::Continue(State::Comment)
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
        // Anything else: literal code points, including '>', '&', and surrogates.
        Input::CodePoint(point) => {
            comment::data_mut(context).push(point);
            Progress::Continue(State::Comment)
        }
        // Open exhaustion is not conceptual EOF.
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
