//! Self-closing start tag only. Normative baseline: HTML Standard source at
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, "Self-closing start tag state"
//! (#self-closing-start-tag-state), and opening Tokenization emission rules.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>

use super::{Context, Input, ParseError, Progress, State, TagKind, Token, tag};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        // U+003E GREATER-THAN SIGN: set the CURRENT tag's flag, switch to Data,
        // and emit it (including the preamble's end-tag diagnostic, if applicable).
        Input::CodePoint(point) if point.value() == 0x3e => {
            match context
                .current_tag
                .expect("tag state requires a current tag")
            {
                TagKind::Start => {
                    context
                        .pending_start_tag
                        .as_mut()
                        .expect("current start tag")
                        .self_closing = true;
                }
                TagKind::End => {
                    context
                        .pending_end_tag
                        .as_mut()
                        .expect("current end tag")
                        .self_closing = true;
                }
            }
            tag::emit_tag(context);
            Progress::Continue(State::Data)
        }
        Input::Eof => {
            context.errors.push(ParseError {
                code: "eof-in-tag",
                position: context.input.position(),
            });
            // Unfinished tokens are abandoned, not emitted (Tokenization note).
            tag::take_tag(context);
            context.tokens.push(Token::Eof);
            Progress::Finished
        }
        // Anything else: do not set the flag, append, replace NUL, or emit.
        Input::CodePoint(_) => {
            context.errors.push(ParseError {
                code: "unexpected-solidus-in-tag",
                position: context.input.position(),
            });
            context
                .input
                .reconsume()
                .expect("just consumed a code point");
            Progress::Continue(State::BeforeAttributeName)
        }
        Input::NeedMoreInput => Progress::NeedMoreInput,
    }
}
