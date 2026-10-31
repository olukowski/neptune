//! Local branch/streaming evidence, not upstream conformance runs.
//! Pinned algorithm (notably, EOF takes anything else):
//! https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144266-L144293

use neptune_html::tokenizer::{
    Attribute, CodePoint, Context, Execution, Input, ParseError, PendingComment, PendingEndTag,
    PendingStartTag, Position, Progress, State, TagKind, Token, UnimplementedState,
};

fn points(text: &str) -> Vec<CodePoint> {
    text.chars().map(Into::into).collect()
}

fn attribute(name: &str, value: &str) -> Attribute {
    Attribute {
        name: points(name),
        value: points(value),
    }
}

fn error(code: &'static str, line: usize, column: usize) -> ParseError {
    ParseError {
        code,
        position: Position { line, column },
    }
}

// Isolated valid storage is not a claim of reachable seeded entry or resumption.
fn seeded(kind: TagKind, detached: bool) -> Context {
    let mut attributes = vec![
        attribute("a", "retained"),
        attribute("other", "predecessor"),
    ];
    let current = Attribute {
        name: points(if detached { "a" } else { "current" }),
        value: vec![CodePoint::new(0xd800).unwrap(), 'v'.into()],
    };
    if !detached {
        attributes.push(current.clone());
    }
    Context {
        current_tag: Some(kind),
        pending_start_tag: Some(PendingStartTag {
            name: points("start"),
            self_closing: true,
            attributes: attributes.clone(),
            discarded_current_attribute: if kind == TagKind::Start && detached {
                Some(current.clone())
            } else if kind == TagKind::End {
                Some(attribute("unrelated", "start"))
            } else {
                None
            },
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("end"),
            self_closing: true,
            attributes,
            discarded_current_attribute: if kind == TagKind::End && detached {
                Some(current)
            } else if kind == TagKind::Start {
                Some(attribute("unrelated", "end"))
            } else {
                None
            },
        }),
        pending_comment: Some(PendingComment {
            data: points("comment"),
        }),
        tokens: vec![Token::Character('p'.into())],
        errors: vec![error("prior-diagnostic", 1, 1)],
        last_start_tag: Some(points("title")),
        return_state: Some(State::Rcdata),
        temporary_buffer: points("scratch"),
        ..Context::default()
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Snapshot {
    state: State,
    current: Option<(Input, Position)>,
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
    input_errors: Vec<ParseError>,
    kind: Option<TagKind>,
    start: Option<PendingStartTag>,
    end: Option<PendingEndTag>,
    comment: Option<PendingComment>,
    last_start_tag: Option<Vec<CodePoint>>,
    return_state: Option<State>,
    buffer: Vec<CodePoint>,
}

fn snapshot(e: &Execution) -> Snapshot {
    let c = e.context();
    Snapshot {
        state: e.state(),
        current: c.input.current(),
        tokens: c.tokens.clone(),
        errors: c.errors.clone(),
        input_errors: c.input.errors().to_vec(),
        kind: c.current_tag,
        start: c.pending_start_tag.clone(),
        end: c.pending_end_tag.clone(),
        comment: c.pending_comment.clone(),
        last_start_tag: c.last_start_tag.clone(),
        return_state: c.return_state,
        buffer: c.temporary_buffer.clone(),
    }
}

fn assert_blocked(e: &mut Execution, state: State) {
    let before = snapshot(e);
    for _ in 0..3 {
        assert_eq!(e.step(), Err(UnimplementedState { state }));
        assert_eq!(snapshot(e), before);
    }
}

#[test]
fn whitespace_quotes_and_anything_else_preserve_values_identity_and_exact_consumption() {
    let values = (0..=0x7f).filter(|v| *v != 0x3e).chain([
        0x85, 0xa0, 0x1680, 0x2003, 0x2028, 0x3000, 0xc9, 0x1f600, 0xd800, 0xdfff, 0xfdd0, 0x10ffff,
    ]);
    for value in values {
        for kind in [TagKind::Start, TagKind::End] {
            for detached in [false, true] {
                for closed in [false, true] {
                    let point = CodePoint::new(value).unwrap();
                    let normalized = if value == 0xd { '\n'.into() } else { point };
                    let target = match value {
                        0x09 | 0x0a | 0x0c | 0x0d | 0x20 => State::BeforeAttributeValue,
                        0x22 => State::AttributeValueDoubleQuoted,
                        0x27 => State::AttributeValueSingleQuoted,
                        _ => State::AttributeValueUnquoted,
                    };
                    let mut c = seeded(kind, detached);
                    c.input.push([point, 'Z'.into()]).unwrap();
                    if closed {
                        c.input.finish();
                    }
                    let mut e = Execution::new(State::BeforeAttributeValue, c);
                    let mut expected = snapshot(&e);
                    expected.state = target;
                    expected.current = Some((
                        Input::CodePoint(normalized),
                        Position { line: 1, column: 1 },
                    ));
                    let preprocessing = match value {
                        0xd800 | 0xdfff => Some("surrogate-in-input-stream"),
                        0xfdd0 | 0x10ffff => Some("noncharacter-in-input-stream"),
                        0x01..=0x08 | 0x0b | 0x0e..=0x1f | 0x7f | 0x85 => {
                            Some("control-character-in-input-stream")
                        }
                        _ => None,
                    };
                    if let Some(code) = preprocessing {
                        expected.input_errors.push(error(code, 1, 1));
                    }
                    assert_eq!(e.step(), Ok(Progress::Continue(target)));
                    assert_eq!(snapshot(&e), expected, "{value:x}, {kind:?}, {detached}");
                    // No NUL replacement, value clearing, appends, or state error.
                    assert_eq!(
                        e.context_mut().input.consume(),
                        Input::CodePoint(if target == State::AttributeValueUnquoted {
                            normalized
                        } else {
                            'Z'.into()
                        })
                    );
                    if target == State::AttributeValueUnquoted {
                        assert_eq!(
                            e.context_mut().input.consume(),
                            Input::CodePoint('Z'.into())
                        );
                    }
                    assert_eq!(e.context().input.errors(), expected.input_errors);
                }
            }
        }
    }
}

#[test]
fn open_exhaustion_is_effect_free_and_eof_is_reconsumed_not_abandoned_here() {
    for kind in [TagKind::Start, TagKind::End] {
        for detached in [false, true] {
            let mut e = Execution::new(State::BeforeAttributeValue, seeded(kind, detached));
            for text in ["", "\t", "\r", "\n", "\u{c} "] {
                e.context_mut().input.push(points(text)).unwrap();
                assert!(!drain(&mut e, &mut vec![], text.chars().count() + 1));
                let before = snapshot(&e);
                for _ in 0..3 {
                    e.context_mut().input.push([]).unwrap();
                    assert_eq!(e.step(), Ok(Progress::NeedMoreInput));
                    assert_eq!(snapshot(&e), before);
                }
            }
            let mut expected = snapshot(&e);
            expected.state = State::AttributeValueUnquoted;
            expected.current = Some((Input::Eof, Position { line: 2, column: 3 }));
            e.context_mut().input.finish();
            assert_eq!(
                e.step(),
                Ok(Progress::Continue(State::AttributeValueUnquoted))
            );
            assert_eq!(snapshot(&e), expected);
            // Abandonment belongs to the destination's next source step.
            expected.kind = None;
            match kind {
                TagKind::Start => expected.start = None,
                TagKind::End => expected.end = None,
            }
            expected.errors.push(error("eof-in-tag", 2, 3));
            expected.tokens.push(Token::Eof);
            for _ in 0..3 {
                assert_eq!(e.step(), Ok(Progress::Finished));
                assert_eq!(snapshot(&e), expected);
            }
        }
    }
}

#[test]
fn greater_than_reports_missing_value_and_reuses_selected_tag_emission() {
    for kind in [TagKind::Start, TagKind::End] {
        for detached in [false, true] {
            for detached_only in [false, true] {
                if detached_only && !detached {
                    continue;
                }
                for flag in [false, true] {
                    let mut c = seeded(kind, detached);
                    let (name, attributes) = match kind {
                        TagKind::Start => {
                            let tag = c.pending_start_tag.as_mut().unwrap();
                            tag.self_closing = flag;
                            if detached_only {
                                tag.attributes.clear();
                            }
                            (tag.name.clone(), tag.attributes.clone())
                        }
                        TagKind::End => {
                            let tag = c.pending_end_tag.as_mut().unwrap();
                            tag.self_closing = flag;
                            if detached_only {
                                tag.attributes.clear();
                            }
                            (tag.name.clone(), tag.attributes.clone())
                        }
                    };
                    // Establish normalized position; do not reprocess seeded values.
                    c.input.push(points("😀\r\n")).unwrap();
                    assert_eq!(c.input.consume(), Input::CodePoint('😀'.into()));
                    assert_eq!(c.input.consume(), Input::CodePoint('\n'.into()));
                    c.input.push(points(">Z")).unwrap();
                    c.input.finish();
                    let mut e = Execution::new(State::BeforeAttributeValue, c);
                    let mut expected = snapshot(&e);
                    expected.state = State::Data;
                    expected.current = Some((
                        Input::CodePoint('>'.into()),
                        Position { line: 2, column: 1 },
                    ));
                    expected.kind = None;
                    expected.errors.push(error("missing-attribute-value", 2, 1));
                    let token = match kind {
                        TagKind::Start => {
                            expected.start = None;
                            expected.last_start_tag = Some(name.clone());
                            Token::StartTag {
                                name,
                                attributes,
                                self_closing: flag,
                            }
                        }
                        TagKind::End => {
                            expected.end = None;
                            if !attributes.is_empty() {
                                expected.errors.push(error("end-tag-with-attributes", 2, 1));
                            }
                            if flag {
                                expected
                                    .errors
                                    .push(error("end-tag-with-trailing-solidus", 2, 1));
                            }
                            Token::EndTag {
                                name,
                                attributes,
                                self_closing: flag,
                            }
                        }
                    };
                    expected.tokens.push(token);
                    assert_eq!(e.step(), Ok(Progress::Continue(State::Data)));
                    assert_eq!(snapshot(&e), expected);
                    // '>' consumed exactly once; real Data continues, then one EOF.
                    assert_eq!(e.step(), Ok(Progress::Continue(State::Data)));
                    expected.current = Some((
                        Input::CodePoint('Z'.into()),
                        Position { line: 2, column: 2 },
                    ));
                    expected.tokens.push(Token::Character('Z'.into()));
                    assert_eq!(snapshot(&e), expected);
                    assert_eq!(e.step(), Ok(Progress::Finished));
                    expected.current = Some((Input::Eof, Position { line: 2, column: 3 }));
                    expected.tokens.push(Token::Eof);
                    assert_eq!(snapshot(&e), expected);
                    assert_eq!(e.step(), Ok(Progress::Finished));
                    assert_eq!(snapshot(&e), expected);
                }
            }
        }
    }
}

#[test]
fn attribute_name_detachment_stays_current_through_before_value_without_mutating_predecessor() {
    for kind in [TagKind::Start, TagKind::End] {
        for (suffix, target) in [
            ("\"tail", State::AttributeValueDoubleQuoted),
            ("'tail", State::AttributeValueSingleQuoted),
            ("\0tail", State::AttributeValueUnquoted),
            ("", State::AttributeValueUnquoted),
        ] {
            let mut c = seeded(kind, false);
            // Attribute name will perform the real duplicate check on departure.
            match kind {
                TagKind::Start => {
                    c.pending_start_tag
                        .as_mut()
                        .unwrap()
                        .attributes
                        .last_mut()
                        .unwrap()
                        .name = points("a")
                }
                TagKind::End => {
                    c.pending_end_tag
                        .as_mut()
                        .unwrap()
                        .attributes
                        .last_mut()
                        .unwrap()
                        .name = points("a")
                }
            }
            c.input.push(points(&format!("= {suffix}"))).unwrap();
            c.input.finish();
            let mut e = Execution::new(State::AttributeName, c);
            assert_eq!(
                e.step(),
                Ok(Progress::Continue(State::BeforeAttributeValue))
            );
            let c = e.context();
            let (attributes, detached) = match kind {
                TagKind::Start => {
                    let t = c.pending_start_tag.as_ref().unwrap();
                    (&t.attributes, &t.discarded_current_attribute)
                }
                TagKind::End => {
                    let t = c.pending_end_tag.as_ref().unwrap();
                    (&t.attributes, &t.discarded_current_attribute)
                }
            };
            assert_eq!(
                attributes,
                &[
                    attribute("a", "retained"),
                    attribute("other", "predecessor")
                ]
            );
            assert_eq!(detached.as_ref().unwrap().name, points("a"));
            assert_eq!(
                detached.as_ref().unwrap().value,
                [CodePoint::new(0xd800).unwrap(), 'v'.into()]
            );
            assert_eq!(
                c.errors,
                [
                    error("prior-diagnostic", 1, 1),
                    error("duplicate-attribute", 1, 1)
                ]
            );
            let mut expected = snapshot(&e);
            assert_eq!(
                e.step(),
                Ok(Progress::Continue(State::BeforeAttributeValue))
            );
            assert_eq!(e.step(), Ok(Progress::Continue(target)));
            expected.state = target;
            expected.current = Some((
                suffix
                    .chars()
                    .next()
                    .map_or(Input::Eof, |ch| Input::CodePoint(ch.into())),
                Position { line: 1, column: 3 },
            ));
            assert_eq!(snapshot(&e), expected);
            if suffix.is_empty() {
                assert_eq!(e.step(), Ok(Progress::Finished));
                assert_eq!(e.context().errors.last(), Some(&error("eof-in-tag", 1, 3)));
                assert_eq!(e.context().current_tag, None);
            } else {
                // Actual destination mutates the detached duplicate, never predecessor.
                assert_eq!(e.step(), Ok(Progress::Continue(target)));
                let (attrs, detached) = match kind {
                    TagKind::Start => {
                        let t = e.context().pending_start_tag.as_ref().unwrap();
                        (&t.attributes, &t.discarded_current_attribute)
                    }
                    TagKind::End => {
                        let t = e.context().pending_end_tag.as_ref().unwrap();
                        (&t.attributes, &t.discarded_current_attribute)
                    }
                };
                assert_eq!(
                    attrs,
                    &[
                        attribute("a", "retained"),
                        attribute("other", "predecessor")
                    ]
                );
                let mut value = vec![CodePoint::new(0xd800).unwrap(), 'v'.into()];
                value.push(if suffix.starts_with('\0') {
                    '\u{fffd}'.into()
                } else {
                    't'.into()
                });
                assert_eq!(detached.as_ref().unwrap().value, value);
                assert_eq!(
                    e.context().errors.last(),
                    Some(&if suffix.starts_with('\0') {
                        error("unexpected-null-character", 1, 3)
                    } else {
                        error("duplicate-attribute", 1, 1)
                    })
                );
            }
        }
    }
}

#[test]
fn real_entry_completes_values_abandons_at_destination_eof_and_keeps_later_stubs() {
    for (input, target, errors, tokens) in [
        (
            "<P a=>",
            None,
            vec![error("missing-attribute-value", 1, 6)],
            vec![
                Token::StartTag {
                    name: points("p"),
                    attributes: vec![attribute("a", "")],
                    self_closing: false,
                },
                Token::Eof,
            ],
        ),
        (
            "</P a=\r\n>",
            None,
            vec![
                error("missing-attribute-value", 2, 1),
                error("end-tag-with-attributes", 2, 1),
            ],
            vec![
                Token::EndTag {
                    name: points("p"),
                    attributes: vec![attribute("a", "")],
                    self_closing: false,
                },
                Token::Eof,
            ],
        ),
        (
            "<p a=\"x\">",
            None,
            vec![],
            vec![
                Token::StartTag {
                    name: points("p"),
                    attributes: vec![attribute("a", "x")],
                    self_closing: false,
                },
                Token::Eof,
            ],
        ),
        (
            "</p a='x'>",
            None,
            vec![error("end-tag-with-attributes", 1, 10)],
            vec![
                Token::EndTag {
                    name: points("p"),
                    attributes: vec![attribute("a", "x")],
                    self_closing: false,
                },
                Token::Eof,
            ],
        ),
        (
            "<p a=\0>",
            None,
            vec![error("unexpected-null-character", 1, 6)],
            vec![
                Token::StartTag {
                    name: points("p"),
                    attributes: vec![attribute("a", "\u{fffd}")],
                    self_closing: false,
                },
                Token::Eof,
            ],
        ),
        (
            "<p a=",
            None,
            vec![error("eof-in-tag", 1, 6)],
            vec![Token::Eof],
        ),
        (
            "<p a= \r\n",
            None,
            vec![error("eof-in-tag", 2, 1)],
            vec![Token::Eof],
        ),
        (
            "<p a=>ok&",
            Some(State::CharacterReference),
            vec![error("missing-attribute-value", 1, 6)],
            vec![
                Token::StartTag {
                    name: points("p"),
                    attributes: vec![attribute("a", "")],
                    self_closing: false,
                },
                Token::Character('o'.into()),
                Token::Character('k'.into()),
            ],
        ),
    ] {
        let mut c = Context::default();
        c.input.push(points(input)).unwrap();
        c.input.finish();
        let mut e = Execution::new(State::Data, c);
        let mut trace = vec![];
        assert!(drain(&mut e, &mut trace, input.len() * 3 + 10));
        assert!(
            trace
                .iter()
                .any(|(result, _)| *result == Ok(Progress::Continue(State::BeforeAttributeValue)))
        );
        assert_eq!(e.context().tokens, tokens, "{input:?}");
        assert_eq!(e.context().errors, errors, "{input:?}");
        assert!(e.context().input.errors().is_empty());
        if let Some(target) = target {
            assert_blocked(&mut e, target);
        } else {
            assert_eq!(e.step(), Ok(Progress::Finished));
        }
    }
}

#[test]
fn real_non_scalar_and_control_input_diagnostics_have_normalized_positions_once() {
    for kind in [TagKind::Start, TagKind::End] {
        for (value, code) in [
            (0xd800, "surrogate-in-input-stream"),
            (0xdfff, "surrogate-in-input-stream"),
            (0xfdd0, "noncharacter-in-input-stream"),
            (0x10ffff, "noncharacter-in-input-stream"),
            (0x0b, "control-character-in-input-stream"),
        ] {
            let point = CodePoint::new(value).unwrap();
            let prefix = if kind == TagKind::Start {
                "😀\r\n<A x=\r\n"
            } else {
                "😀\r\n</A x=\r\n"
            };
            let mut c = Context::default();
            c.input
                .push(points(prefix).into_iter().chain([point, 'Z'.into()]))
                .unwrap();
            c.input.finish();
            let mut e = Execution::new(State::Data, c);
            while e.state() != State::AttributeValueUnquoted {
                assert!(matches!(e.step(), Ok(Progress::Continue(_))));
            }
            assert_eq!(e.context().current_tag, Some(kind));
            assert_eq!(
                e.context().input.current(),
                Some((Input::CodePoint(point), Position { line: 3, column: 1 }))
            );
            assert_eq!(e.context().input.errors(), [error(code, 3, 1)]);
            assert!(e.context().errors.is_empty());
            assert_eq!(
                e.context().tokens,
                [Token::Character('😀'.into()), Token::Character('\n'.into())]
            );
            let attributes = match kind {
                TagKind::Start => &e.context().pending_start_tag.as_ref().unwrap().attributes,
                TagKind::End => &e.context().pending_end_tag.as_ref().unwrap().attributes,
            };
            assert_eq!(attributes, &[attribute("x", "")]);
            // Source-local inspection confirms reconsumption, not an append, and
            // preprocessing diagnostics are not duplicated on that reconsumption.
            assert_eq!(e.context_mut().input.consume(), Input::CodePoint(point));
            assert_eq!(e.context().input.errors(), [error(code, 3, 1)]);
            assert_eq!(
                e.context_mut().input.consume(),
                Input::CodePoint('Z'.into())
            );
            assert_eq!(
                e.context().input.position(),
                Position { line: 3, column: 2 }
            );
            // Direct cursor inspection above consumed the two points solely to
            // prove source reconsumption; destination behavior is tested separately.
        }
    }
}

type Trace = Vec<(Result<Progress, UnimplementedState>, Snapshot)>;
fn drain(e: &mut Execution, trace: &mut Trace, bound: usize) -> bool {
    for _ in 0..bound {
        let before = snapshot(e);
        let result = e.step();
        if result == Ok(Progress::NeedMoreInput) {
            assert_eq!(snapshot(e), before);
            assert_eq!(e.step(), result);
            assert_eq!(snapshot(e), before);
            return false;
        }
        trace.push((result, snapshot(e)));
        if !matches!(result, Ok(Progress::Continue(_))) {
            let terminal = snapshot(e);
            assert_eq!(e.step(), result);
            assert_eq!(snapshot(e), terminal);
            return true;
        }
    }
    panic!("Before attribute value trace neither suspended nor terminated");
}

#[test]
fn all_two_chunk_splits_and_single_points_match_closed_traces_including_first_terminal_block() {
    let mut samples: Vec<_> = [
        "",
        "\t\n\u{c} ",
        "\r\n\r\r\n",
        "\"tail",
        "'tail",
        "x>",
        "\0>",
        "/>",
        "&amp;",
        "=>",
        "<x",
        "é😀",
        ">tail\0",
        ">ok&",
        ">x<a y>",
        "<A x=>text</A>",
        "</A x=\r\n>",
        "<A x=",
        "<A x=\r",
        "<A x=\r\n\r\r\n>",
        "<A x='tail'>",
        "</A x=\"tail\">",
        "<A x=\0>",
        "<A x=unquoted>",
        "<p a='one' b=\"two\" c=THREE d>",
        "<p a=first a='lost' b=next>",
        "</p a=first a='lost' b=next>",
        "<p a='x'b=two>",
        "<p a='x'a=two c>",
        "<p a=first a='lost' b=>",
        "</p a='x'/>",
        "<p a=x/>",
        "</p a=x />",
        "<p a=first a='lost&tail",
        "</p a=first a=\"lost&tail",
        "<p a=first a=lost&tail",
    ]
    .map(points)
    .into();
    for value in [0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x0b] {
        for prefix in ["\r\n", "😀\r\n<A x=\r\n"] {
            let mut input = points(prefix);
            input.push(CodePoint::new(value).unwrap());
            input.extend(points("Z>"));
            samples.push(input);
        }
    }
    for input in samples {
        for initial in [State::Data, State::BeforeAttributeValue] {
            for kind in [TagKind::Start, TagKind::End] {
                for detached in [false, true] {
                    let run = |chunks: Vec<&[CodePoint]>, closed: bool| {
                        let c = if initial == State::Data {
                            Context::default()
                        } else {
                            seeded(kind, detached)
                        };
                        let mut e = Execution::new(initial, c);
                        let mut trace = vec![];
                        if closed {
                            e.context_mut().input.push(input.iter().copied()).unwrap();
                            e.context_mut().input.finish();
                        } else {
                            for chunk in chunks {
                                e.context_mut().input.push(chunk.iter().copied()).unwrap();
                                if drain(&mut e, &mut trace, input.len() * 3 + 10) {
                                    return trace;
                                }
                                e.context_mut().input.push([]).unwrap();
                                assert!(!drain(&mut e, &mut trace, 1));
                            }
                            e.context_mut().input.finish();
                        }
                        assert!(drain(&mut e, &mut trace, input.len() * 3 + 10));
                        trace
                    };
                    let expected = run(vec![], true);
                    for split in 0..=input.len() {
                        assert_eq!(
                            run(vec![&input[..split], &input[split..]], false),
                            expected,
                            "{initial:?} {kind:?} {detached} split {split} {input:?}"
                        );
                    }
                    assert_eq!(run(input.chunks(1).collect(), false), expected);
                }
            }
        }
    }
}
