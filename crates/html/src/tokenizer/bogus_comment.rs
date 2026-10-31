//! Bogus comment only. Normative baseline: HTML Standard source at
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, "Bogus comment state" (#bogus-comment-state).
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>

use super::{Context, Input, ParseError, Progress, State, Token};

// End tag open establishes the pending-comment invariant before entry.
// Isolated entry must likewise supply a current comment, including its initial data.
fn emit(context: &mut Context) {
    let comment = context
        .pending_comment
        .take()
        .expect("Bogus comment requires a current comment");
    context.tokens.push(Token::Comment { data: comment.data });
}

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        // U+003E GREATER-THAN SIGN: consume, switch to Data, emit the comment.
        Input::CodePoint(point) if point.value() == 0x3e => {
            emit(context);
            Progress::Continue(State::Data)
        }
        // EOF: emit the comment, then EOF. No eof-in-comment error in this state.
        Input::Eof => {
            emit(context);
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
            context
                .pending_comment
                .as_mut()
                .expect("Bogus comment requires a current comment")
                .data
                .push(point);
            Progress::Continue(State::BogusComment)
        }
        // Open exhaustion is suspension, never the specification's EOF branch.
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
