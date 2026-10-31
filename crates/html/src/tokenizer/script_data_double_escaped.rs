//! Script data double escaped only (#script-data-double-escaped-state).
//! Pinned WHATWG HTML 404b3eb14a7a936dddefb2d78e224dbeb01d2358:
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143993-L144021>
//! Branch/definition evidence: docs/tokenizer-script-data-double-escaped.md.

use super::{Context, Input, ParseError, Progress, State, Token};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point) if point.value() == 0x2d => {
            context.tokens.push(Token::Character(point));
            Progress::Continue(State::ScriptDataDoubleEscapedDash)
        }
        Input::CodePoint(point) if point.value() == 0x3c => {
            // Unlike Script data escaped, this state emits the consumed '<'.
            context.tokens.push(Token::Character(point));
            Progress::Continue(State::ScriptDataDoubleEscapedLessThanSign)
        }
        Input::CodePoint(point) if point.value() == 0 => {
            context.errors.push(ParseError {
                code: "unexpected-null-character",
                position: context.input.position(),
            });
            context.tokens.push(Token::Character('\u{fffd}'.into()));
            Progress::Continue(State::ScriptDataDoubleEscaped)
        }
        Input::Eof => {
            context.errors.push(ParseError {
                code: "eof-in-script-html-comment-like-text",
                position: context.input.position(),
            });
            context.tokens.push(Token::Eof);
            Progress::Finished
        }
        Input::CodePoint(point) => {
            context.tokens.push(Token::Character(point));
            Progress::Continue(State::ScriptDataDoubleEscaped)
        }
        // Exhausted open input is suspension, not conceptual EOF.
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
