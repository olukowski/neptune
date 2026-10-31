//! RAWTEXT end tag name only. HTML Standard source at
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, lines 143608–143651.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143608-L143651>

use super::{CodePoint, Context, Input, Progress, State, Token, tag};

// Entry requires the current end tag and original-spelling temporary buffer
// initialized by RAWTEXT end tag open. Compare names, not the temporary buffer;
// no prior emitted start tag means no appropriate end tag (Tokenization definition).
fn appropriate(context: &Context) -> bool {
    context.last_start_tag.as_ref().is_some_and(|name| {
        name == &context
            .pending_end_tag
            .as_ref()
            .expect("current end tag")
            .name
    })
}

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::NeedMoreInput => return Progress::NeedMoreInput,
        Input::CodePoint(point) => match point.value() {
            // ASCII alpha only: normalize the name, preserve spelling in scratch.
            0x41..=0x5a | 0x61..=0x7a => {
                let normalized = if point.value() <= 0x5a {
                    CodePoint::new(point.value() + 0x20).unwrap()
                } else {
                    point
                };
                context
                    .pending_end_tag
                    .as_mut()
                    .expect("current end tag")
                    .name
                    .push(normalized);
                context.temporary_buffer.push(point);
                return Progress::Continue(State::RawtextEndTagName);
            }
            // CR has already become LF. Appropriate delimiters are consumed,
            // not appended or reconsumed; destination effects occur next step.
            0x09 | 0x0a | 0x0c | 0x20 if appropriate(context) => {
                return Progress::Continue(State::BeforeAttributeName);
            }
            0x2f if appropriate(context) => {
                return Progress::Continue(State::SelfClosingStartTag);
            }
            0x3e if appropriate(context) => {
                tag::emit_tag(context);
                return Progress::Continue(State::Data);
            }
            _ => {}
        },
        Input::Eof => {}
    }

    // Anything else, including inappropriate delimiters and conceptual EOF.
    // Neither discard the candidate nor clear scratch: the spec only emits it.
    context.tokens.push(Token::Character('<'.into()));
    context.tokens.push(Token::Character('/'.into()));
    context.tokens.extend(
        context
            .temporary_buffer
            .iter()
            .copied()
            .map(Token::Character),
    );
    context.input.reconsume().expect("current input exists");
    Progress::Continue(State::Rawtext)
}
