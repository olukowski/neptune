//! Attribute name only. Normative baseline: HTML Standard source at
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358,
//! "Attribute name state" (#attribute-name-state), including its on-leaving rule
//! and note that a removed duplicate remains the current attribute.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>

use super::{CodePoint, Context, Input, ParseError, Progress, State, tag};

fn leave(context: &mut Context, target: State) -> Progress {
    // The branch's switch/reconsumption precedes the on-leaving comparison.
    // The completed name is compared exactly, after all normalization/appends.
    if tag::has_duplicate_attribute(context) {
        context.errors.push(ParseError {
            code: "duplicate-attribute",
            position: context.input.position(),
        });
        tag::discard_current_attribute(context);
    }
    Progress::Continue(target)
}

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::CodePoint(point) => match point.value() {
            // TAB, LF, FF, SPACE, SOLIDUS, GREATER-THAN SIGN: reconsume in
            // After attribute name (not Before attribute name or direct emission).
            // CR has already been normalized to LF by the cursor.
            0x09 | 0x0a | 0x0c | 0x20 | 0x2f | 0x3e => {
                context.input.reconsume().expect("just consumed input");
                leave(context, State::AfterAttributeName)
            }
            // EQUALS SIGN: switch WITHOUT reconsumption or appending '='.
            0x3d => leave(context, State::BeforeAttributeValue),
            // ASCII upper alpha only: add exactly 0x20, no Unicode case folding.
            0x41..=0x5a => {
                tag::current_attribute_mut(context)
                    .name
                    .push(CodePoint::new(point.value() + 0x20).unwrap());
                Progress::Continue(State::AttributeName)
            }
            // NULL: diagnose then append replacement, not the original NUL.
            0 => {
                context.errors.push(ParseError {
                    code: "unexpected-null-character",
                    position: context.input.position(),
                });
                tag::current_attribute_mut(context)
                    .name
                    .push('\u{fffd}'.into());
                Progress::Continue(State::AttributeName)
            }
            // Quotes/apostrophe/less-than: diagnose, then anything-else append.
            0x22 | 0x27 | 0x3c => {
                context.errors.push(ParseError {
                    code: "unexpected-character-in-attribute-name",
                    position: context.input.position(),
                });
                tag::current_attribute_mut(context).name.push(point);
                Progress::Continue(State::AttributeName)
            }
            // Anything else: literal code point, including non-scalars.
            _ => {
                tag::current_attribute_mut(context).name.push(point);
                Progress::Continue(State::AttributeName)
            }
        },
        Input::Eof => {
            context.input.reconsume().expect("just consumed EOF");
            leave(context, State::AfterAttributeName)
        }
        // Suspension is NOT leaving Attribute name; do not check duplicates yet.
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
