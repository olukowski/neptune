//! Attribute value (unquoted) only. Normative baseline: HTML Standard source at
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, lines 144356–144404.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144356-L144404>

use super::{Context, Input, ParseError, Progress, State, Token, tag};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point) => match point.value() {
            // TAB, LF, FF, SPACE terminate the value; CR is normalized by Cursor.
            0x09 | 0x0a | 0x0c | 0x20 => Progress::Continue(State::BeforeAttributeName),
            0x26 => {
                context.return_state = Some(State::AttributeValueUnquoted);
                Progress::Continue(State::CharacterReference)
            }
            0x3e => {
                tag::emit_tag(context);
                Progress::Continue(State::Data)
            }
            0 => {
                context.errors.push(ParseError {
                    code: "unexpected-null-character",
                    position: context.input.position(),
                });
                tag::current_attribute_mut(context)
                    .value
                    .push('\u{fffd}'.into());
                Progress::Continue(State::AttributeValueUnquoted)
            }
            // Forbidden punctuation still appends literally, as "anything else".
            0x22 | 0x27 | 0x3c | 0x3d | 0x60 => {
                context.errors.push(ParseError {
                    code: "unexpected-character-in-unquoted-attribute-value",
                    position: context.input.position(),
                });
                tag::current_attribute_mut(context).value.push(point);
                Progress::Continue(State::AttributeValueUnquoted)
            }
            // Includes literal solidus, uppercase, Unicode and non-scalars.
            _ => {
                tag::current_attribute_mut(context).value.push(point);
                Progress::Continue(State::AttributeValueUnquoted)
            }
        },
        Input::Eof => {
            context.errors.push(ParseError {
                code: "eof-in-tag",
                position: context.input.position(),
            });
            // Tokenization's EOF rule abandons the selected unfinished tag only.
            tag::take_tag(context);
            context.tokens.push(Token::Eof);
            Progress::Finished
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
