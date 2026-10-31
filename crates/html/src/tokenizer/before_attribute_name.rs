//! Before attribute name only. Normative baseline: HTML Standard source at
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358,
//! "Before attribute name state" (#before-attribute-name-state).
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>

use super::{Attribute, Context, Input, ParseError, Progress, State, tag};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        // TAB, LF, FF, SPACE only; CR is normalized to LF by the cursor.
        Input::CodePoint(point) if matches!(point.value(), 0x09 | 0x0a | 0x0c | 0x20) => {
            Progress::Continue(State::BeforeAttributeName)
        }
        // SOLIDUS, GREATER-THAN SIGN: reconsume, NOT a direct switch.
        Input::CodePoint(point) if matches!(point.value(), 0x2f | 0x3e) => {
            context.input.reconsume().expect("just consumed input");
            Progress::Continue(State::AfterAttributeName)
        }
        // EQUALS SIGN: diagnose, initialize name to '=', value empty; switch.
        Input::CodePoint(point) if point.value() == 0x3d => {
            context.errors.push(ParseError {
                code: "unexpected-equals-sign-before-attribute-name",
                position: context.input.position(),
            });
            tag::start_attribute(
                context,
                Attribute {
                    name: vec![point],
                    value: Vec::new(),
                },
            );
            Progress::Continue(State::AttributeName)
        }
        // Anything else: initialize EMPTY name/value, then reconsume.
        Input::CodePoint(_) => {
            tag::start_attribute(context, Attribute::default());
            context
                .input
                .reconsume()
                .expect("just consumed a code point");
            Progress::Continue(State::AttributeName)
        }
        // EOF follows the same reconsumption branch; no eof-in-tag here.
        Input::Eof => {
            context.input.reconsume().expect("just consumed EOF");
            Progress::Continue(State::AfterAttributeName)
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
