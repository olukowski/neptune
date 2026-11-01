//! Bogus comment only. Normative baseline: HTML Standard source at
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, "Bogus comment state" (#bogus-comment-state).
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>

use super::{Context, Input, ParseError, Progress, State, Token, comment};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        // U+003E GREATER-THAN SIGN: consume, switch to Data, emit the comment.
        Input::CodePoint(point) if point.value() == 0x3e => {
            comment::emit(context);
            Progress::Continue(State::Data)
        }
        // EOF: emit the comment, then EOF. No eof-in-comment error in this state.
        Input::Eof => {
            comment::emit(context);
            context.tokens.push(Token::Eof);
            Progress::Finished
        }
        Input::CodePoint(point) => {
            let point = if point.value() == 0 {
                // NULL: diagnose at the consumed point; append U+FFFD, not NUL.
                context.errors.push(ParseError {
                    code: "unexpected-null-character",
                    position: context.input.position(),
                });
                '\u{fffd}'.into()
            } else {
                // Anything else: unchanged, including '<', '&', and surrogates.
                point
            };
            comment::data_mut(context).push(point);
            Progress::Continue(State::BogusComment)
        }
        // Open exhaustion is suspension, never the specification's EOF branch.
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
