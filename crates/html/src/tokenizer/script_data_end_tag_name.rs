//! Script data end tag name only: WHATWG HTML source, revision
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, lines 143695–143738.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143695-L143738>

use super::{CodePoint, Context, Input, Progress, State, Token, tag};

// Isolated entry requires selected End storage and original-spelling scratch
// context, as established by the preceding states. Appropriate means exact name
// equality with the last emitted start tag, if any (source lines 143213–143215).
fn appropriate(context: &Context) -> bool {
    context.last_start_tag.as_ref().is_some_and(|last| {
        last == &context
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
            // Preprocessing has already normalized CR to LF.
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
            0x41..=0x5a => {
                context
                    .pending_end_tag
                    .as_mut()
                    .expect("current end tag")
                    .name
                    .push(CodePoint::new(point.value() + 0x20).unwrap());
                context.temporary_buffer.push(point);
                return Progress::Continue(State::ScriptDataEndTagName);
            }
            0x61..=0x7a => {
                context
                    .pending_end_tag
                    .as_mut()
                    .expect("current end tag")
                    .name
                    .push(point);
                context.temporary_buffer.push(point);
                return Progress::Continue(State::ScriptDataEndTagName);
            }
            _ => {}
        },
        Input::Eof => {}
    }

    // Anything else includes inappropriate delimiters and conceptual EOF.
    // Emit scratch literally; do not clear it, discard storage, replace NUL,
    // diagnose EOF-in-tag, or execute the destination as part of this step.
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
    Progress::Continue(State::ScriptData)
}
