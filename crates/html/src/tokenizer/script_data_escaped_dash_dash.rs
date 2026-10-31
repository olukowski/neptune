//! Script data escaped dash dash state (#script-data-escaped-dash-dash-state).
//! Pinned WHATWG HTML 404b3eb14a7a936dddefb2d78e224dbeb01d2358,
//! source lines 143837–143869. See docs/tokenizer-script-data-escaped-dash-dash.md.

use super::{Context, Input, ParseError, Progress, State, Token};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point) if point.value() == 0x2d => {
            context.tokens.push(Token::Character('-'.into()));
            Progress::Continue(State::ScriptDataEscapedDashDash)
        }
        Input::CodePoint(point) if point.value() == 0x3c => {
            Progress::Continue(State::ScriptDataEscapedLessThanSign)
        }
        Input::CodePoint(point) if point.value() == 0x3e => {
            context.tokens.push(Token::Character('>'.into()));
            Progress::Continue(State::ScriptData)
        }
        Input::CodePoint(point) if point.value() == 0 => {
            context.errors.push(ParseError {
                code: "unexpected-null-character",
                position: context.input.position(),
            });
            context.tokens.push(Token::Character('\u{fffd}'.into()));
            Progress::Continue(State::ScriptDataEscaped)
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
            Progress::Continue(State::ScriptDataEscaped)
        }
        // Open exhaustion is not conceptual EOF.
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
