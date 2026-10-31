//! Shared selected-tag take/emission, not a tokenizer state.
//! Normative baseline: opening Tokenization paragraphs (creation, abandonment,
//! emission, and appropriate end tags) in HTML Standard source at
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>

use super::{Context, ParseError, TagKind, Token};

// Take only the selected storage; unrelated pending storage is inspectable scaffolding.
pub(super) fn take_tag(context: &mut Context) -> Token {
    let kind = context
        .current_tag
        .take()
        .expect("tag state requires a current tag");
    match kind {
        TagKind::Start => {
            let tag = context.pending_start_tag.take().expect("current start tag");
            Token::StartTag {
                name: tag.name,
                self_closing: tag.self_closing,
            }
        }
        TagKind::End => {
            let tag = context.pending_end_tag.take().expect("current end tag");
            Token::EndTag {
                name: tag.name,
                self_closing: tag.self_closing,
            }
        }
    }
}

// Shared by Tag name and Self-closing start tag: Tokenization emission rules.
// Attribute creation reaches stubs before emission; emitting an attribute-bearing
// context via manual redirection remains unsupported.
pub(super) fn emit_tag(context: &mut Context) {
    let token = take_tag(context);
    match &token {
        Token::StartTag { name, .. } => {
            // Appropriate-end-tag context tracks EMITTED start tags only.
            context.last_start_tag = Some(name.clone());
        }
        Token::EndTag {
            self_closing: true, ..
        } => {
            context.errors.push(ParseError {
                code: "end-tag-with-trailing-solidus",
                position: context.input.position(),
            });
        }
        _ => {}
    }
    context.tokens.push(token);
}
