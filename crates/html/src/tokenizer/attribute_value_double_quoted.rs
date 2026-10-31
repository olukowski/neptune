//! Attribute value (double-quoted) only. Normative baseline: HTML Standard at
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, source lines 144296–144324.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144296-L144324>
//! EOF abandonment follows the opening Tokenization note; see tag::take_tag.

use super::{Context, Input, ParseError, Progress, State, Token, tag};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point) => match point.value() {
            // QUOTATION MARK: consumed, not appended or reconsumed.
            0x22 => Progress::Continue(State::AfterAttributeValueQuoted),
            // AMPERSAND: set the return state before attempting the stub.
            0x26 => {
                context.return_state = Some(State::AttributeValueDoubleQuoted);
                Progress::Continue(State::CharacterReference)
            }
            // NULL: diagnose and append replacement to the CURRENT attribute,
            // including when it is a detached duplicate, never its predecessor.
            0 => {
                context.errors.push(ParseError {
                    code: "unexpected-null-character",
                    position: context.input.position(),
                });
                tag::current_attribute_mut(context)
                    .value
                    .push('\u{fffd}'.into());
                Progress::Continue(State::AttributeValueDoubleQuoted)
            }
            // Anything else: literal, including whitespace, ', <, >, non-scalars.
            _ => {
                tag::current_attribute_mut(context).value.push(point);
                Progress::Continue(State::AttributeValueDoubleQuoted)
            }
        },
        Input::Eof => {
            context.errors.push(ParseError {
                code: "eof-in-tag",
                position: context.input.position(),
            });
            // Abandon only the selected tag; no tag emission diagnostics.
            tag::take_tag(context);
            context.tokens.push(Token::Eof);
            Progress::Finished
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
