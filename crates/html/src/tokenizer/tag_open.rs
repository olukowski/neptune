//! Tag open only. Normative baseline: HTML Standard source at
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, "Tag open state" (#tag-open-state).
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>
//! Token creation defaults follow the opening Tokenization paragraphs.

use super::{Context, Input, ParseError, PendingStartTag, Progress, State, TagKind, Token};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point) => match point.value() {
            // U+0021 EXCLAMATION MARK: switch, without reconsumption.
            0x21 => Progress::Continue(State::MarkupDeclarationOpen),
            // U+002F SOLIDUS: switch, without reconsumption.
            0x2f => Progress::Continue(State::EndTagOpen),
            // ASCII alpha: create (do not emit), with an EMPTY name.
            0x41..=0x5a | 0x61..=0x7a => {
                context.pending_start_tag = Some(PendingStartTag::default());
                context.current_tag = Some(TagKind::Start);
                context
                    .input
                    .reconsume()
                    .expect("just consumed a code point");
                Progress::Continue(State::TagName)
            }
            // U+003F QUESTION MARK: pinned processing-instruction behavior.
            0x3f => {
                context.temporary_buffer.clear();
                Progress::Continue(State::ProcessingInstructionOpen)
            }
            // Anything else: emit '<', then reconsume the matched point in Data.
            _ => {
                context.errors.push(ParseError {
                    code: "invalid-first-character-of-tag-name",
                    position: context.input.position(),
                });
                context.tokens.push(Token::Character('<'.into()));
                context
                    .input
                    .reconsume()
                    .expect("just consumed a code point");
                Progress::Continue(State::Data)
            }
        },
        Input::Eof => {
            context.errors.push(ParseError {
                code: "eof-before-tag-name",
                position: context.input.position(),
            });
            context.tokens.push(Token::Character('<'.into()));
            context.tokens.push(Token::Eof);
            Progress::Finished
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
