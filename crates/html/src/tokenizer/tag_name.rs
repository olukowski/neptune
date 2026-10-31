//! Tag name only. Normative baseline: HTML Standard source at
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, "Tag name state" (#tag-name-state).
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>
//! Creation, abandonment, emission, and last-start-tag semantics follow the
//! opening Tokenization paragraphs. Reachable emission paths have no attributes.

use super::{CodePoint, Context, Input, ParseError, Progress, State, TagKind, Token, tag};

// Tag open / End tag open establish the current-tag invariant before entry.
fn name(context: &mut Context) -> &mut Vec<CodePoint> {
    match context
        .current_tag
        .expect("Tag name requires a current tag")
    {
        TagKind::Start => {
            &mut context
                .pending_start_tag
                .as_mut()
                .expect("current start tag")
                .name
        }
        TagKind::End => {
            &mut context
                .pending_end_tag
                .as_mut()
                .expect("current end tag")
                .name
        }
    }
}

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point) => match point.value() {
            // TAB, LF, FF, SPACE only. CR has already been normalized to LF.
            0x09 | 0x0a | 0x0c | 0x20 => Progress::Continue(State::BeforeAttributeName),
            // U+002F SOLIDUS: consumed, not appended or reconsumed.
            0x2f => Progress::Continue(State::SelfClosingStartTag),
            // U+003E GREATER-THAN SIGN: switch to Data and emit the current tag.
            0x3e => {
                tag::emit_tag(context);
                Progress::Continue(State::Data)
            }
            // ASCII upper alpha: add exactly 0x20; never Unicode case-fold.
            0x41..=0x5a => {
                name(context).push(CodePoint::new(point.value() + 0x20).unwrap());
                Progress::Continue(State::TagName)
            }
            // NULL: diagnose at the consumed NUL, append replacement to the name.
            0 => {
                context.errors.push(ParseError {
                    code: "unexpected-null-character",
                    position: context.input.position(),
                });
                name(context).push('\u{fffd}'.into());
                Progress::Continue(State::TagName)
            }
            // Anything else: append the code point unchanged, including surrogates.
            _ => {
                name(context).push(point);
                Progress::Continue(State::TagName)
            }
        },
        Input::Eof => {
            context.errors.push(ParseError {
                code: "eof-in-tag",
                position: context.input.position(),
            });
            // The unfinished tag is abandoned, never emitted (Tokenization note).
            tag::take_tag(context);
            context.tokens.push(Token::Eof);
            Progress::Finished
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
