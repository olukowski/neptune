//! Comment start dash only: pinned HTML Standard, lines 144535–144559.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144535-L144559>
//! Branch evidence: docs/tokenizer-comment-start-dash.md.
//! Isolated entry requires an initialized pending comment.

use super::{Context, Input, ParseError, Progress, State, Token, comment};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        // '-' is a delimiter, not comment data. No reconsumption.
        Input::CodePoint(point) if point.value() == 0x2d => Progress::Continue(State::CommentEnd),
        Input::CodePoint(point) if point.value() == 0x3e => {
            context.errors.push(ParseError {
                code: "abrupt-closing-of-empty-comment",
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
        // Append only the deferred hyphen. Comment handles the reconsumed point,
        // including NUL; no replacement or unexpected-null-character here.
        Input::CodePoint(_) => {
            comment::data_mut(context).push('-'.into());
            context
                .input
                .reconsume()
                .expect("just consumed a code point");
            Progress::Continue(State::Comment)
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
