//! After attribute name only. Normative baseline: HTML Standard source at
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, "After attribute name state".
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144232-L144270>

use super::{Attribute, Context, Input, ParseError, Progress, State, Token, tag};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        // TAB, LF, FF, SPACE only; the cursor normalizes CR to LF.
        Input::CodePoint(point) if matches!(point.value(), 0x09 | 0x0a | 0x0c | 0x20) => {
            Progress::Continue(State::AfterAttributeName)
        }
        Input::CodePoint(point) if point.value() == 0x2f => {
            Progress::Continue(State::SelfClosingStartTag)
        }
        Input::CodePoint(point) if point.value() == 0x3d => {
            Progress::Continue(State::BeforeAttributeValue)
        }
        Input::CodePoint(point) if point.value() == 0x3e => {
            tag::emit_tag(context);
            Progress::Continue(State::Data)
        }
        Input::Eof => {
            context.errors.push(ParseError {
                code: "eof-in-tag",
                position: context.input.position(),
            });
            // The Tokenization preamble abandons unfinished tokens at EOF.
            tag::take_tag(context);
            context.tokens.push(Token::Eof);
            Progress::Finished
        }
        Input::CodePoint(_) => {
            // Entry from Before attribute name need not have a current attribute.
            // Starting a new one also clears any detached duplicate-current slot.
            tag::start_attribute(context, Attribute::default());
            context
                .input
                .reconsume()
                .expect("just consumed a code point");
            Progress::Continue(State::AttributeName)
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
