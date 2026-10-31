//! Shared selected-tag storage and take/emission, not a tokenizer state.
//! Normative baseline: opening Tokenization paragraphs (creation, abandonment,
//! emission, and appropriate end tags) and Attribute name's removed-but-current
//! rule in HTML Standard source at
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>

use super::{Attribute, Context, ParseError, TagKind, Token};

// The retained list and detached duplicate belong to the selected tag, not to
// unrelated pending storage. Use these helpers in later attribute/value states.
fn attribute_storage(context: &mut Context) -> (&mut Vec<Attribute>, &mut Option<Attribute>) {
    match context
        .current_tag
        .expect("attribute state requires a current tag")
    {
        TagKind::Start => {
            let tag = context
                .pending_start_tag
                .as_mut()
                .expect("current start tag");
            (&mut tag.attributes, &mut tag.discarded_current_attribute)
        }
        TagKind::End => {
            let tag = context.pending_end_tag.as_mut().expect("current end tag");
            (&mut tag.attributes, &mut tag.discarded_current_attribute)
        }
    }
}

pub(super) fn start_attribute(context: &mut Context, attribute: Attribute) {
    let (attributes, discarded) = attribute_storage(context);
    *discarded = None;
    attributes.push(attribute);
}

// Removal from the token does not change the identity of the current attribute.
// In particular, future value appends must not mutate the prior retained entry.
pub(super) fn current_attribute_mut(context: &mut Context) -> &mut Attribute {
    let (attributes, discarded) = attribute_storage(context);
    discarded
        .as_mut()
        .unwrap_or_else(|| attributes.last_mut().expect("current attribute"))
}

// Attribute name calls this only on departure, NOT on suspension. Keeping the
// check separate lets that state diagnose BEFORE removing the duplicate.
pub(super) fn has_duplicate_attribute(context: &mut Context) -> bool {
    let (attributes, discarded) = attribute_storage(context);
    if discarded.is_some() {
        return false; // Already detached, never compare a retained predecessor.
    }
    let (current, prior) = attributes.split_last().expect("current attribute");
    prior.iter().any(|attribute| attribute.name == current.name)
}

pub(super) fn discard_current_attribute(context: &mut Context) {
    let (attributes, discarded) = attribute_storage(context);
    if discarded.is_none() {
        *discarded = Some(attributes.pop().expect("current attribute"));
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::{Execution, PendingEndTag, PendingStartTag, Progress, State};

    fn attribute(name: &str, value: &str) -> Attribute {
        Attribute {
            name: name.chars().map(Into::into).collect(),
            value: value.chars().map(Into::into).collect(),
        }
    }

    #[test]
    fn tag_creation_resets_detached_current_only_on_the_selected_new_token() {
        for (state, kind) in [
            (State::TagOpen, TagKind::Start),
            (State::EndTagOpen, TagKind::End),
        ] {
            let mut context = Context {
                pending_start_tag: Some(PendingStartTag {
                    discarded_current_attribute: Some(attribute("old-start", "value")),
                    ..PendingStartTag::default()
                }),
                pending_end_tag: Some(PendingEndTag {
                    discarded_current_attribute: Some(attribute("old-end", "value")),
                    ..PendingEndTag::default()
                }),
                ..Context::default()
            };
            let old_start = context.pending_start_tag.clone();
            let old_end = context.pending_end_tag.clone();
            context.input.push(['A'.into()]).unwrap();
            let mut execution = Execution::new(state, context);
            assert_eq!(execution.step(), Ok(Progress::Continue(State::TagName)));
            let context = execution.context();
            match kind {
                TagKind::Start => {
                    let tag = context.pending_start_tag.as_ref().unwrap();
                    assert!(tag.attributes.is_empty());
                    assert!(tag.discarded_current_attribute.is_none());
                    assert_eq!(context.pending_end_tag, old_end);
                }
                TagKind::End => {
                    let tag = context.pending_end_tag.as_ref().unwrap();
                    assert!(tag.attributes.is_empty());
                    assert!(tag.discarded_current_attribute.is_none());
                    assert_eq!(context.pending_start_tag, old_start);
                }
            }
        }
    }

    #[test]
    fn detached_duplicate_stays_current_through_mutation_and_next_attribute_creation() {
        for kind in [TagKind::Start, TagKind::End] {
            let retained = vec![attribute("same", "original"), attribute("other", "prior")];
            let duplicate = attribute("same", "new");
            let attributes = [retained.clone(), vec![duplicate.clone()]].concat();
            let mut context = Context {
                current_tag: Some(kind),
                pending_start_tag: Some(PendingStartTag {
                    attributes: attributes.clone(),
                    ..PendingStartTag::default()
                }),
                pending_end_tag: Some(PendingEndTag {
                    attributes,
                    ..PendingEndTag::default()
                }),
                ..Context::default()
            };
            let old_start = context.pending_start_tag.clone();
            let old_end = context.pending_end_tag.clone();
            assert!(attribute_storage(&mut context).1.is_none());
            assert_eq!(current_attribute_mut(&mut context), &duplicate);
            assert!(has_duplicate_attribute(&mut context));
            discard_current_attribute(&mut context);
            assert_eq!(attribute_storage(&mut context).0, &retained);
            assert_eq!(current_attribute_mut(&mut context), &duplicate);
            // Storage lifecycle evidence, NOT an implementation of a value state.
            current_attribute_mut(&mut context).value.push('!'.into());
            assert_eq!(
                current_attribute_mut(&mut context),
                &attribute("same", "new!")
            );
            assert_eq!(attribute_storage(&mut context).0, &retained);
            assert!(!has_duplicate_attribute(&mut context));
            discard_current_attribute(&mut context); // Idempotent, never detaches predecessor.
            assert_eq!(attribute_storage(&mut context).0, &retained);

            let next = attribute("next", "");
            start_attribute(&mut context, next.clone());
            assert!(attribute_storage(&mut context).1.is_none());
            assert_eq!(current_attribute_mut(&mut context), &next);
            current_attribute_mut(&mut context).value.push('v'.into());
            assert!(!has_duplicate_attribute(&mut context));
            assert_eq!(
                attribute_storage(&mut context).0,
                &[retained.clone(), vec![attribute("next", "v")]].concat()
            );
            // Another duplicate detaches afresh, not the last retained entry.
            start_attribute(&mut context, attribute("same", "again"));
            assert!(has_duplicate_attribute(&mut context));
            discard_current_attribute(&mut context);
            assert_eq!(
                current_attribute_mut(&mut context),
                &attribute("same", "again")
            );
            take_tag(&mut context); // Selected-token storage also owns the detached slot.
            assert_eq!(context.current_tag, None);
            match kind {
                TagKind::Start => {
                    assert!(context.pending_start_tag.is_none());
                    assert_eq!(context.pending_end_tag, old_end);
                }
                TagKind::End => {
                    assert!(context.pending_end_tag.is_none());
                    assert_eq!(context.pending_start_tag, old_start);
                }
            }
        }
    }
}
