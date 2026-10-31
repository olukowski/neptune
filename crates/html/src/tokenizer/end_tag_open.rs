//! End tag open only. Normative baseline: HTML Standard source at
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, "End tag open state" (#end-tag-open-state).
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>
//! Token creation defaults follow the opening Tokenization paragraphs.

use super::{Context, Input, ParseError, PendingComment, PendingEndTag, Progress, State, Token};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point) => match point.value() {
            // ASCII alpha: create (do not emit), with an EMPTY name.
            0x41..=0x5a | 0x61..=0x7a => {
                context.pending_end_tag = Some(PendingEndTag::default());
                context
                    .input
                    .reconsume()
                    .expect("just consumed a code point");
                Progress::Continue(State::TagName)
            }
            // U+003E GREATER-THAN SIGN: consume, then recover in Data.
            0x3e => {
                context.errors.push(ParseError {
                    code: "missing-end-tag-name",
                    position: context.input.position(),
                });
                Progress::Continue(State::Data)
            }
            // Anything else: create an EMPTY comment, not a discarded end tag.
            // The matched point is left for Bogus comment, not appended here.
            _ => {
                context.errors.push(ParseError {
                    code: "invalid-first-character-of-tag-name",
                    position: context.input.position(),
                });
                context.pending_comment = Some(PendingComment::default());
                context
                    .input
                    .reconsume()
                    .expect("just consumed a code point");
                Progress::Continue(State::BogusComment)
            }
        },
        Input::Eof => {
            context.errors.push(ParseError {
                code: "eof-before-tag-name",
                position: context.input.position(),
            });
            context.tokens.extend([
                Token::Character('<'.into()),
                Token::Character('/'.into()),
                Token::Eof,
            ]);
            Progress::Finished
        }
        // Open, exhausted input is suspension, not the specification's EOF.
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
