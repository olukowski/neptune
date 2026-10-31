//! After attribute value (quoted) only. Normative baseline: HTML Standard source
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, lines 144402–144432
//! (#after-attribute-value-quoted-state), plus Tokenization's abandonment and
//! emission rules. <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144402-L144432>

use super::{Context, Input, ParseError, Progress, State, Token, tag};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        // TAB, LF, FF, SPACE; CR has already been normalized to LF.
        Input::CodePoint(point) if matches!(point.value(), 0x09 | 0x0a | 0x0c | 0x20) => {
            Progress::Continue(State::BeforeAttributeName)
        }
        Input::CodePoint(point) if point.value() == 0x2f => {
            Progress::Continue(State::SelfClosingStartTag)
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
            // The unfinished selected token is abandoned, never emitted.
            tag::take_tag(context);
            context.tokens.push(Token::Eof);
            Progress::Finished
        }
        // No value mutation, attribute creation, NUL replacement, or attribute-
        // name diagnostic here. Recovery belongs to subsequent source steps.
        Input::CodePoint(_) => {
            context.errors.push(ParseError {
                code: "missing-whitespace-between-attributes",
                position: context.input.position(),
            });
            context.input.reconsume().expect("just consumed input");
            Progress::Continue(State::BeforeAttributeName)
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
