//! RCDATA end tag name only: pinned HTML Standard source, lines 143524–143565.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>
//! Appropriate end tag definition: lines 143212–143216; reconsumption and
//! emission: opening Tokenization paragraphs. See state-local branch evidence
//! in docs/tokenizer-rcdata-end-tag-name.md.

use super::{CodePoint, Context, Input, Progress, State, Token, tag};

// Isolated entry requires current_tag = End and an initialized pending end tag,
// as created by RCDATA end tag open. Neither creation nor buffer clearing occurs here.
fn appropriate(context: &Context) -> bool {
    let candidate = &context
        .pending_end_tag
        .as_ref()
        .expect("current end tag")
        .name;
    context.last_start_tag.as_ref() == Some(candidate)
}

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::NeedMoreInput => return Progress::NeedMoreInput,
        Input::CodePoint(point) => match point.value() {
            // CR has already become LF. Delimiters consume without reconsumption
            // only when appropriate; otherwise use the literal fallback below.
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
                return Progress::Continue(State::RcdataEndTagName);
            }
            _ => {}
        },
        // EOF is "anything else", not eof-in-tag or terminal EOF here.
        Input::Eof => {}
    }
    context.tokens.push(Token::Character('<'.into()));
    context.tokens.push(Token::Character('/'.into()));
    context.tokens.extend(
        context
            .temporary_buffer
            .iter()
            .copied()
            .map(Token::Character),
    );
    context.input.reconsume().expect("current input");
    Progress::Continue(State::Rcdata)
}
