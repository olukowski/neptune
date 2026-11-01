//! Comment end dash only: HTML 404b3eb14a7a936dddefb2d78e224dbeb01d2358,
//! source lines 144660–144678 (#comment-end-dash-state).
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144660-L144678>

use super::{Context, Input, ParseError, Progress, State, Token, comment};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        // U+002D: switch without appending or reconsuming the hyphen.
        Input::CodePoint(point) if point.value() == 0x2d => Progress::Continue(State::CommentEnd),
        Input::Eof => {
            context.errors.push(ParseError {
                code: "eof-in-comment",
                position: context.input.position(),
            });
            comment::emit(context);
            context.tokens.push(Token::Eof);
            Progress::Finished
        }
        // Append only the withheld hyphen. The destination handles this point,
        // including NUL replacement; no destination effects are performed here.
        Input::CodePoint(_) => {
            comment::data_mut(context).push('-'.into());
            context.input.reconsume().expect("current input exists");
            Progress::Continue(State::Comment)
        }
        // Open exhaustion is not EOF and must not append the withheld hyphen.
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
