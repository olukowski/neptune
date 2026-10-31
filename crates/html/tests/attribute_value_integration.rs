//! Combined Data-entry evidence, not additional upstream conformance runs.
//! Pinned branches: source#L144145-L144432 at HTML revision
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358 (see docs/sources.toml).
use neptune_html::tokenizer::{
    Attribute, CodePoint, Context, Execution, Input, ParseError, Position, Progress, State,
    TagKind, Token, UnimplementedState,
};

fn points(s: &str) -> Vec<CodePoint> {
    s.chars().map(Into::into).collect()
}
fn attribute(name: &str, value: &str) -> Attribute {
    Attribute {
        name: points(name),
        value: points(value),
    }
}
fn error(code: &'static str, column: usize) -> ParseError {
    ParseError {
        code,
        position: Position { line: 1, column },
    }
}
fn selected(c: &Context, kind: TagKind) -> (&[Attribute], &Option<Attribute>) {
    match kind {
        TagKind::Start => {
            let t = c.pending_start_tag.as_ref().unwrap();
            (&t.attributes, &t.discarded_current_attribute)
        }
        TagKind::End => {
            let t = c.pending_end_tag.as_ref().unwrap();
            (&t.attributes, &t.discarded_current_attribute)
        }
    }
}
fn entry(input: &str) -> Execution {
    let mut c = Context {
        last_start_tag: Some(points("prior")),
        ..Context::default()
    };
    c.input.push(points(input)).unwrap();
    c.input.finish();
    Execution::new(State::Data, c)
}
fn finish(e: &mut Execution) {
    for _ in 0..300 {
        match e.step() {
            Ok(Progress::Continue(_)) => {}
            Ok(Progress::Finished) => return,
            other => panic!("unexpected {other:?}"),
        }
    }
    panic!("did not terminate");
}

#[test]
fn data_to_attributes_compares_values_recovery_duplicates_and_end_tag_diagnostics() {
    // Source-derived columns for start-tag spellings. Inserting '/' shifts
    // diagnostics one normalized point in the corresponding end-tag spelling.
    for (start, attrs, flag, diagnostics) in [
        (
            "<p a='one' b=\"two\" c=THREE d>",
            vec![
                attribute("a", "one"),
                attribute("b", "two"),
                attribute("c", "THREE"),
                attribute("d", ""),
            ],
            false,
            vec![],
        ),
        (
            "<p a=first a='lost' b=next>",
            vec![attribute("a", "first"), attribute("b", "next")],
            false,
            vec![error("duplicate-attribute", 13)],
        ),
        (
            "<p a='x'b=two>",
            vec![attribute("a", "x"), attribute("b", "two")],
            false,
            vec![error("missing-whitespace-between-attributes", 9)],
        ),
        (
            "<p a='x'a=two c>",
            vec![attribute("a", "x"), attribute("c", "")],
            false,
            vec![
                error("missing-whitespace-between-attributes", 9),
                error("duplicate-attribute", 10),
            ],
        ),
        (
            "<p a=first a='lost' b=>",
            vec![attribute("a", "first"), attribute("b", "")],
            false,
            vec![
                error("duplicate-attribute", 13),
                error("missing-attribute-value", 23),
            ],
        ),
        (
            "<p a==`>",
            vec![attribute("a", "=`")],
            false,
            vec![
                error("unexpected-character-in-unquoted-attribute-value", 6),
                error("unexpected-character-in-unquoted-attribute-value", 7),
            ],
        ),
        ("<p a='x'/>", vec![attribute("a", "x")], true, vec![]),
        ("<p a=x/>", vec![attribute("a", "x/")], false, vec![]),
        ("<p a=x />", vec![attribute("a", "x")], true, vec![]),
    ] {
        for kind in [TagKind::Start, TagKind::End] {
            let input = if kind == TagKind::End {
                start.replacen('<', "</", 1)
            } else {
                start.to_owned()
            };
            let mut e = entry(&input);
            finish(&mut e);
            let mut errors = diagnostics.clone();
            let token = match kind {
                TagKind::Start => Token::StartTag {
                    name: points("p"),
                    attributes: attrs.clone(),
                    self_closing: flag,
                },
                TagKind::End => {
                    for err in &mut errors {
                        err.position.column += 1;
                    }
                    errors.push(error("end-tag-with-attributes", input.chars().count()));
                    if flag {
                        errors.push(error(
                            "end-tag-with-trailing-solidus",
                            input.chars().count(),
                        ));
                    }
                    Token::EndTag {
                        name: points("p"),
                        attributes: attrs.clone(),
                        self_closing: flag,
                    }
                }
            };
            assert_eq!(e.context().tokens, [token.clone(), Token::Eof], "{input:?}");
            assert_eq!(e.context().errors, errors, "{input:?}");
            assert!(e.context().input.errors().is_empty());
            assert_eq!(
                e.context().last_start_tag,
                Some(points(if kind == TagKind::Start { "p" } else { "prior" }))
            );
            assert_eq!(e.context().current_tag, None);
            assert!(e.context().pending_start_tag.is_none());
            assert!(e.context().pending_end_tag.is_none());
            for _ in 0..3 {
                assert_eq!(e.step(), Ok(Progress::Finished));
            }
            assert_eq!(e.context().tokens, [token, Token::Eof]);
        }
    }
}

#[test]
fn real_duplicate_value_stays_detached_until_start_attribute_clears_it() {
    for kind in [TagKind::Start, TagKind::End] {
        let input = if kind == TagKind::Start {
            "<p a=first a='lost' b=next>"
        } else {
            "</p a=first a='lost' b=next>"
        };
        let shift = usize::from(kind == TagKind::End);
        let mut e = entry(input);
        let mut checkpoints = vec![];
        for _ in 0..100 {
            match e.step() {
                Ok(Progress::Continue(_)) => {}
                Ok(Progress::Finished) => break,
                other => panic!("unexpected {other:?}"),
            }
            let column = e.context().input.position().column;
            if e.state() == State::BeforeAttributeValue && column == 13 + shift {
                assert_eq!(
                    selected(e.context(), kind),
                    (&[attribute("a", "first")][..], &Some(attribute("a", "")))
                );
                checkpoints.push("detached");
            }
            if e.state() == State::AfterAttributeValueQuoted && column == 19 + shift {
                assert_eq!(
                    selected(e.context(), kind),
                    (
                        &[attribute("a", "first")][..],
                        &Some(attribute("a", "lost"))
                    )
                );
                checkpoints.push("valued");
            }
            if e.state() == State::AttributeName && column == 21 + shift {
                let (attrs, detached) = selected(e.context(), kind);
                assert!(detached.is_none());
                assert_eq!(attrs[0], attribute("a", "first"));
                assert_eq!(attrs[1].value, []);
                if attrs[1].name.is_empty() {
                    checkpoints.push("new");
                }
            }
        }
        assert_eq!(checkpoints, ["detached", "valued", "new"]);
        assert_eq!(e.step(), Ok(Progress::Finished));
        assert_eq!(e.context().tokens.len(), 2);
        assert_eq!(
            e.context().errors[0],
            error("duplicate-attribute", 13 + shift)
        );
    }
}

#[test]
fn real_ampersands_remain_terminal_even_in_discarded_duplicate_values() {
    for kind in [TagKind::Start, TagKind::End] {
        for (open, state) in [
            ("\"", State::AttributeValueDoubleQuoted),
            ("'", State::AttributeValueSingleQuoted),
            ("", State::AttributeValueUnquoted),
        ] {
            let prefix = if kind == TagKind::Start { "<p" } else { "</p" };
            let input = format!("{prefix} a=first a={open}lost&tail");
            let mut e = entry(&input);
            for _ in 0..100 {
                if e.state() == State::CharacterReference {
                    break;
                }
                assert!(matches!(e.step(), Ok(Progress::Continue(_))));
            }
            assert_eq!(e.state(), State::CharacterReference);
            assert_eq!(e.context().return_state, Some(state));
            assert_eq!(
                selected(e.context(), kind),
                (
                    &[attribute("a", "first")][..],
                    &Some(attribute("a", "lost"))
                )
            );
            let errors = vec![error(
                "duplicate-attribute",
                if kind == TagKind::Start { 13 } else { 14 },
            )];
            let current = e.context().input.current();
            for _ in 0..3 {
                assert_eq!(
                    e.step(),
                    Err(UnimplementedState {
                        state: State::CharacterReference
                    })
                );
                assert_eq!(
                    selected(e.context(), kind),
                    (
                        &[attribute("a", "first")][..],
                        &Some(attribute("a", "lost"))
                    )
                );
                assert_eq!(e.context().input.current(), current);
                assert_eq!(e.context().errors, errors);
                assert!(e.context().tokens.is_empty());
                assert!(e.context().input.errors().is_empty());
                assert_eq!(e.context().last_start_tag, Some(points("prior")));
            }
            assert_eq!(
                e.context_mut().input.consume(),
                Input::CodePoint('t'.into())
            );
            assert_eq!(
                e.step(),
                Err(UnimplementedState {
                    state: State::CharacterReference
                })
            );
        }
    }
}
