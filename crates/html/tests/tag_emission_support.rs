//! Shared prerequisite support, NOT newly reachable attribute-state algorithms.
//! Local contexts are initialized before execution, never resumed after a stub.
use neptune_html::tokenizer::{
    Attribute, CodePoint, Context, Execution, Input, ParseError, PendingComment, PendingEndTag,
    PendingStartTag, Position, Progress, State, TagKind, Token,
};

fn points(text: &str) -> Vec<CodePoint> {
    text.chars().map(Into::into).collect()
}

fn attributes() -> Vec<Attribute> {
    vec![
        Attribute {
            name: points("zÉ"),
            value: vec![CodePoint::new(0xdfff).unwrap(), '😀'.into()],
        },
        Attribute {
            name: vec!['a'.into(), CodePoint::new(0xd800).unwrap()],
            value: vec![CodePoint::new(0xffff).unwrap(), '\0'.into(), '字'.into()],
        },
    ]
}

fn error(code: &'static str, line: usize, column: usize) -> ParseError {
    ParseError {
        code,
        position: Position { line, column },
    }
}

fn seeded(kind: TagKind, retained: Vec<Attribute>, flag: bool) -> Context {
    // Both storage slots deliberately exist; selection must not infer from presence.
    let detached = Some(Attribute {
        name: attributes()[0].name.clone(),
        value: points("discarded value must never escape"),
    });
    Context {
        current_tag: Some(kind),
        pending_start_tag: Some(PendingStartTag {
            name: points("start😀"),
            attributes: retained.clone(),
            self_closing: flag,
            discarded_current_attribute: detached.clone(),
        }),
        pending_end_tag: Some(PendingEndTag {
            name: vec!['e'.into(), CodePoint::new(0xd800).unwrap()],
            attributes: retained,
            self_closing: flag,
            discarded_current_attribute: detached,
        }),
        pending_comment: Some(PendingComment {
            data: points("comment"),
        }),
        last_start_tag: Some(points("previous")),
        return_state: Some(State::Rcdata),
        temporary_buffer: points("scratch"),
        tokens: vec![Token::Character('!'.into())],
        errors: vec![error("prior", 1, 1)],
        ..Context::default()
    }
}

// Assert selected take/abandonment while preserving ALL unrelated storage.
fn assert_storage(context: &Context, before: &Context, kind: TagKind, emitted: bool) {
    assert_eq!(context.current_tag, None);
    match kind {
        TagKind::Start => {
            assert!(context.pending_start_tag.is_none());
            assert_eq!(context.pending_end_tag, before.pending_end_tag);
        }
        TagKind::End => {
            assert!(context.pending_end_tag.is_none());
            assert_eq!(context.pending_start_tag, before.pending_start_tag);
        }
    }
    assert_eq!(context.pending_comment, before.pending_comment);
    assert_eq!(context.return_state, before.return_state);
    assert_eq!(context.temporary_buffer, before.temporary_buffer);
    assert_eq!(
        context.last_start_tag,
        if emitted && kind == TagKind::Start {
            Some(before.pending_start_tag.as_ref().unwrap().name.clone())
        } else {
            before.last_start_tag.clone()
        }
    );
}

#[test]
fn initialized_tag_emission_preserves_order_code_points_and_only_retained_attributes() {
    for state in [State::TagName, State::SelfClosingStartTag] {
        for kind in [TagKind::Start, TagKind::End] {
            for flag in [false, true] {
                // Detached-only storage must not count as attributes at emission.
                for retained in [vec![], attributes()] {
                    let before = seeded(kind, retained.clone(), flag);
                    let mut context = seeded(kind, retained.clone(), flag);
                    context.input.push(points("x\r\n>x")).unwrap();
                    assert_eq!(context.input.consume(), Input::CodePoint('x'.into()));
                    assert_eq!(context.input.consume(), Input::CodePoint('\n'.into()));
                    context.input.finish();
                    let mut execution = Execution::new(state, context);
                    assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
                    let context = execution.context();
                    assert_storage(context, &before, kind, true);
                    let self_closing = flag || state == State::SelfClosingStartTag;
                    let expected_token = match kind {
                        TagKind::Start => Token::StartTag {
                            name: before.pending_start_tag.as_ref().unwrap().name.clone(),
                            attributes: retained.clone(),
                            self_closing,
                        },
                        TagKind::End => Token::EndTag {
                            name: before.pending_end_tag.as_ref().unwrap().name.clone(),
                            attributes: retained.clone(),
                            self_closing,
                        },
                    };
                    assert_eq!(
                        context.tokens,
                        [before.tokens.clone(), vec![expected_token]].concat()
                    );
                    let mut errors = before.errors.clone();
                    if kind == TagKind::End {
                        if !retained.is_empty() {
                            errors.push(error("end-tag-with-attributes", 2, 1));
                        }
                        if self_closing {
                            errors.push(error("end-tag-with-trailing-solidus", 2, 1));
                        }
                    }
                    assert_eq!(context.errors, errors);
                    // Seeded names/values are not input: never preprocess them again.
                    assert!(context.input.errors().is_empty());
                    assert_eq!(context.input.position(), Position { line: 2, column: 1 });
                    assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
                    assert_eq!(
                        execution.context().tokens.last(),
                        Some(&Token::Character('x'.into()))
                    );
                    assert_eq!(execution.step(), Ok(Progress::Finished));
                    let tokens = execution.context().tokens.clone();
                    let errors = execution.context().errors.clone();
                    assert_eq!(tokens.last(), Some(&Token::Eof));
                    assert_eq!(execution.step(), Ok(Progress::Finished));
                    assert_eq!(execution.context().tokens, tokens);
                    assert_eq!(execution.context().errors, errors);
                }
            }
        }
    }
}

#[test]
fn initialized_unfinished_tags_suspend_then_abandon_without_emission_errors() {
    for state in [State::TagName, State::SelfClosingStartTag] {
        for kind in [TagKind::Start, TagKind::End] {
            for flag in [false, true] {
                let before = seeded(kind, attributes(), flag);
                let mut execution = Execution::new(state, seeded(kind, attributes(), flag));
                for _ in 0..2 {
                    assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                    let context = execution.context();
                    assert_eq!(context.current_tag, Some(kind));
                    assert_eq!(context.pending_start_tag, before.pending_start_tag);
                    assert_eq!(context.pending_end_tag, before.pending_end_tag);
                    assert_eq!(context.tokens, before.tokens);
                    assert_eq!(context.errors, before.errors);
                    assert_eq!(context.last_start_tag, before.last_start_tag);
                }
                execution.context_mut().input.finish();
                assert_eq!(execution.step(), Ok(Progress::Finished));
                let context = execution.context();
                assert_storage(context, &before, kind, false);
                assert_eq!(
                    context.tokens,
                    [before.tokens.clone(), vec![Token::Eof]].concat()
                );
                assert_eq!(
                    context.errors,
                    [before.errors.clone(), vec![error("eof-in-tag", 1, 1)]].concat()
                );
                assert!(context.input.errors().is_empty());
                assert_eq!(execution.step(), Ok(Progress::Finished));
                assert_eq!(execution.context().errors.len(), 2);
            }
        }
    }
}
