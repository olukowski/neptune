//! Before attribute value only. Normative baseline: HTML Standard source at
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, "Before attribute value state".
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144266-L144293>

use super::{Context, Input, ParseError, Progress, State, tag};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        // TAB, LF, FF, SPACE: ignore. CR is normalized to LF by the cursor.
        Input::CodePoint(point) if matches!(point.value(), 0x09 | 0x0a | 0x0c | 0x20) => {
            Progress::Continue(State::BeforeAttributeValue)
        }
        // Quote delimiters are consumed, not appended or reconsumed. Neither
        // branch initializes/resets a value: the current attribute stays current,
        // including a detached duplicate. Value operations belong to destinations.
        Input::CodePoint(point) if point.value() == 0x22 => {
            Progress::Continue(State::AttributeValueDoubleQuoted)
        }
        Input::CodePoint(point) if point.value() == 0x27 => {
            Progress::Continue(State::AttributeValueSingleQuoted)
        }
        Input::CodePoint(point) if point.value() == 0x3e => {
            context.errors.push(ParseError {
                code: "missing-attribute-value",
                position: context.input.position(),
            });
            tag::emit_tag(context);
            Progress::Continue(State::Data)
        }
        // Anything else INCLUDES EOF: this state has no separate EOF branch.
        // Do not abandon/emit EOF, replace NUL, or append here. The unquoted
        // destination owns value mutation and EOF abandonment on its next step.
        Input::CodePoint(_) | Input::Eof => {
            context.input.reconsume().expect("just consumed input");
            Progress::Continue(State::AttributeValueUnquoted)
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
