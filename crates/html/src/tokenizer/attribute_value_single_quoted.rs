//! Attribute value (single-quoted) only. Normative baseline: HTML Standard
//! source at 404b3eb14a7a936dddefb2d78e224dbeb01d2358, lines 144326–144353.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144326-L144353>
//! EOF abandonment follows the opening Tokenization note (lines 143187–143189).

use super::{Context, Input, ParseError, Progress, State, Token, tag};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point) => match point.value() {
            // APOSTROPHE: consume the closing delimiter, without appending it.
            0x27 => Progress::Continue(State::AfterAttributeValueQuoted),
            // AMPERSAND: set the return state before attempting the reference.
            0x26 => {
                context.return_state = Some(State::AttributeValueSingleQuoted);
                Progress::Continue(State::CharacterReference)
            }
            // NULL: diagnose, then append replacement to the actual current
            // attribute (a detached duplicate is still current).
            0 => {
                context.errors.push(ParseError {
                    code: "unexpected-null-character",
                    position: context.input.position(),
                });
                tag::current_attribute_mut(context)
                    .value
                    .push('\u{fffd}'.into());
                Progress::Continue(State::AttributeValueSingleQuoted)
            }
            // Anything else, including whitespace, double quotes, angle brackets
            // and non-scalars: append literally; no branch reconsumes.
            _ => {
                tag::current_attribute_mut(context).value.push(point);
                Progress::Continue(State::AttributeValueSingleQuoted)
            }
        },
        Input::Eof => {
            context.errors.push(ParseError {
                code: "eof-in-tag",
                position: context.input.position(),
            });
            // Abandon only the selected unfinished tag, never emit it.
            tag::take_tag(context);
            context.tokens.push(Token::Eof);
            Progress::Finished
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
