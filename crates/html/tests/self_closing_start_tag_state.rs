//! Local branch evidence, not upstream conformance counts. Pinned algorithm:
//! HTML 404b3eb14a7a936dddefb2d78e224dbeb01d2358, Self-closing start tag state
//! (#self-closing-start-tag-state) and opening Tokenization emission rules.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>

use neptune_html::tokenizer::{
    Attribute, CodePoint, Context, Execution, Input, ParseError, PendingComment, PendingEndTag,
    PendingStartTag, Position, Progress, State, TagKind, Token, UnimplementedState,
};

fn points(text: &str) -> Vec<CodePoint> {
    text.chars().map(CodePoint::from).collect()
}

fn error(code: &'static str, line: usize, column: usize) -> ParseError {
    ParseError {
        code,
        position: Position { line, column },
    }
}

fn seeded(kind: TagKind, flag: bool) -> Context {
    Context {
        current_tag: Some(kind),
        pending_start_tag: Some(PendingStartTag {
            name: points("start"),
            self_closing: flag,
            ..PendingStartTag::default()
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("end"),
            self_closing: flag,
            ..PendingEndTag::default()
        }),
        pending_comment: Some(PendingComment {
            data: points("comment"),
        }),
        tokens: vec![Token::Character('p'.into())],
        errors: vec![error("prior", 1, 1)],
        last_start_tag: Some(points("title")),
        return_state: Some(State::Rcdata),
        temporary_buffer: vec![CodePoint::new(0xd800).unwrap()],
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
    last: Option<Vec<CodePoint>>,
    return_state: Option<State>,
    buffer: Vec<CodePoint>,
}

fn snapshot(execution: &Execution) -> Snapshot {
    let c = execution.context();
    Snapshot {
        state: execution.state(),
        current: c.input.current(),
        tokens: c.tokens.clone(),
        errors: c.errors.clone(),
        input_errors: c.input.errors().to_vec(),
        kind: c.current_tag,
        start: c.pending_start_tag.clone(),
        end: c.pending_end_tag.clone(),
        comment: c.pending_comment.clone(),
        last: c.last_start_tag.clone(),
        return_state: c.return_state,
        buffer: c.temporary_buffer.clone(),
    }
}

#[test]
fn greater_than_sets_only_current_flag_and_emits_before_following_data() {
    for kind in [TagKind::Start, TagKind::End] {
        for flag in [false, true] {
            let mut c = seeded(kind, flag);
            c.input.push(points(">x")).unwrap();
            c.input.finish();
            let mut e = Execution::new(State::SelfClosingStartTag, c);
            let mut expected = snapshot(&e);
            assert_eq!(e.step(), Ok(Progress::Continue(State::Data)));
            expected.state = State::Data;
            expected.current = Some((
                Input::CodePoint('>'.into()),
                Position { line: 1, column: 1 },
            ));
            expected.kind = None;
            match kind {
                TagKind::Start => {
                    expected.start = None;
                    expected.last = Some(points("start"));
                    expected.tokens.push(Token::StartTag {
                        name: points("start"),
                        self_closing: true,
                    });
                }
                TagKind::End => {
                    expected.end = None;
                    expected
                        .errors
                        .push(error("end-tag-with-trailing-solidus", 1, 1));
                    expected.tokens.push(Token::EndTag {
                        name: points("end"),
                        self_closing: true,
                    });
                }
            }
            assert_eq!(snapshot(&e), expected);
            assert_eq!(e.step(), Ok(Progress::Continue(State::Data)));
            assert_eq!(
                e.context().tokens.last(),
                Some(&Token::Character('x'.into()))
            );
            assert_eq!(
                e.context().input.position(),
                Position { line: 1, column: 2 }
            );
            assert_eq!(e.step(), Ok(Progress::Finished));
            assert_eq!(e.context().tokens.last(), Some(&Token::Eof));
        }
    }
}

#[test]
fn anything_else_preserves_flags_and_reconsumes_exact_point_before_destination_entry() {
    // Every ASCII point except '>', plus Unicode scalars, non-scalars, and
    // preprocessing diagnostics. NUL gets no replacement/null error in THIS state.
    let values: Vec<_> = (0..=0x7f)
        .filter(|v| *v != 0x3e)
        .chain([0xa0, 0xc9, 0x1f600, 0xd800, 0xdfff, 0xfdd0, 0x10ffff])
        .collect();
    for kind in [TagKind::Start, TagKind::End] {
        for flag in [false, true] {
            for value in &values {
                let point = CodePoint::new(*value).unwrap();
                let normalized = if *value == 0x0d { '\n'.into() } else { point };
                let mut c = seeded(kind, flag);
                c.input.push([point, 'Z'.into()]).unwrap();
                c.input.finish();
                let mut e = Execution::new(State::SelfClosingStartTag, c);
                let mut expected = snapshot(&e);
                assert_eq!(e.step(), Ok(Progress::Continue(State::BeforeAttributeName)));
                expected.state = State::BeforeAttributeName;
                expected.current = Some((
                    Input::CodePoint(normalized),
                    Position { line: 1, column: 1 },
                ));
                expected
                    .errors
                    .push(error("unexpected-solidus-in-tag", 1, 1));
                let preprocessing = match *value {
                    0xd800 | 0xdfff => Some("surrogate-in-input-stream"),
                    0xfdd0 | 0x10ffff => Some("noncharacter-in-input-stream"),
                    0x01..=0x08 | 0x0b | 0x0e..=0x1f | 0x7f => {
                        Some("control-character-in-input-stream")
                    }
                    _ => None,
                };
                if let Some(code) = preprocessing {
                    expected.input_errors.push(error(code, 1, 1));
                }
                assert_eq!(snapshot(&e), expected);
                // Source-local cursor inspection only. Integration below executes
                // Before attribute name and checks its remaining destination stubs.
                assert_eq!(
                    e.context_mut().input.consume(),
                    Input::CodePoint(normalized)
                );
                assert_eq!(snapshot(&e), expected);
                assert_eq!(
                    e.context_mut().input.consume(),
                    Input::CodePoint('Z'.into())
                );
                assert_eq!(e.context().input.errors(), expected.input_errors);
            }
        }
    }
}

#[test]
fn exhausted_open_input_suspends_but_eof_abandons_only_selected_tag_without_setting_flag() {
    for kind in [TagKind::Start, TagKind::End] {
        for flag in [false, true] {
            let mut c = seeded(kind, flag);
            c.input.push(points("😀\r\n")).unwrap();
            assert_eq!(c.input.consume(), Input::CodePoint('😀'.into()));
            assert_eq!(c.input.consume(), Input::CodePoint('\n'.into()));
            let mut e = Execution::new(State::SelfClosingStartTag, c);
            let mut expected = snapshot(&e);
            for _ in 0..2 {
                e.context_mut().input.push([]).unwrap();
                assert_eq!(e.step(), Ok(Progress::NeedMoreInput));
                assert_eq!(snapshot(&e), expected);
            }
            e.context_mut().input.finish();
            assert_eq!(e.step(), Ok(Progress::Finished));
            expected.current = Some((Input::Eof, Position { line: 2, column: 1 }));
            expected.errors.push(error("eof-in-tag", 2, 1));
            expected.tokens.push(Token::Eof);
            expected.kind = None;
            match kind {
                TagKind::Start => expected.start = None,
                TagKind::End => expected.end = None,
            }
            assert_eq!(snapshot(&e), expected);
            for _ in 0..2 {
                assert_eq!(e.step(), Ok(Progress::Finished));
                assert_eq!(snapshot(&e), expected);
            }
        }
    }
}

fn drain(
    e: &mut Execution,
    trace: &mut Vec<(Result<Progress, UnimplementedState>, Snapshot)>,
) -> bool {
    for _ in 0..200 {
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
    panic!("execution did not consume or terminate");
}

fn complete(text: &str) -> Execution {
    let mut e = Execution::new(State::Data, Context::default());
    e.context_mut().input.push(points(text)).unwrap();
    e.context_mut().input.finish();
    assert!(drain(&mut e, &mut vec![]));
    assert_eq!(e.step(), Ok(Progress::Finished));
    e
}

#[test]
fn real_data_path_emits_self_closing_tags_and_end_tag_diagnostic_in_order() {
    let e = complete("<br/><DIV/>x</DIV/>");
    assert_eq!(
        e.context().tokens,
        [
            Token::StartTag {
                name: points("br"),
                self_closing: true
            },
            Token::StartTag {
                name: points("div"),
                self_closing: true
            },
            Token::Character('x'.into()),
            Token::EndTag {
                name: points("div"),
                self_closing: true
            },
            Token::Eof,
        ]
    );
    assert_eq!(
        e.context().errors,
        [error("end-tag-with-trailing-solidus", 1, 19)]
    );
    assert_eq!(e.context().last_start_tag, Some(points("div")));
    // No tree builder: a set flag does not make arbitrary HTML elements void,
    // and acknowledgement-dependent diagnostics do not belong to this shell.
    for (text, column) in [("<br/", 5), ("</br/", 6)] {
        let e = complete(text);
        assert_eq!(e.context().tokens, [Token::Eof]);
        assert_eq!(e.context().errors, [error("eof-in-tag", 1, column)]);
        assert_eq!(e.context().current_tag, None);
        assert_eq!(e.context().pending_start_tag, None);
        assert_eq!(e.context().pending_end_tag, None);
        assert_eq!(e.context().last_start_tag, None);
    }
    let e = complete("\r\n</br/>");
    assert_eq!(
        e.context().errors,
        [error("end-tag-with-trailing-solidus", 2, 6)]
    );
    assert_eq!(e.context().last_start_tag, None);
}

#[test]
fn slash_is_consumed_by_tag_name_and_need_more_input_does_not_set_flag() {
    let mut e = Execution::new(State::Data, Context::default());
    e.context_mut().input.push(points("<br/")).unwrap();
    assert!(!drain(&mut e, &mut vec![]));
    assert_eq!(e.state(), State::SelfClosingStartTag);
    assert_eq!(
        e.context().input.current(),
        Some((
            Input::CodePoint('/'.into()),
            Position { line: 1, column: 4 }
        ))
    );
    assert_eq!(
        e.context().pending_start_tag,
        Some(PendingStartTag {
            name: points("br"),
            self_closing: false,
            attributes: vec![]
        })
    );
    assert!(e.context().tokens.is_empty());
    assert!(e.context().errors.is_empty());
    e.context_mut().input.push(points(">")).unwrap();
    assert!(!drain(&mut e, &mut vec![]));
    assert_eq!(
        e.context().tokens,
        [Token::StartTag {
            name: points("br"),
            self_closing: true
        }]
    );
    e.context_mut().input.finish();
    assert_eq!(e.step(), Ok(Progress::Finished));
}

#[test]
fn creation_resets_selected_tag_defaults_without_touching_unrelated_storage() {
    for (initial, kind) in [
        (State::TagOpen, TagKind::Start),
        (State::EndTagOpen, TagKind::End),
    ] {
        let mut c = seeded(TagKind::End, true);
        c.pending_start_tag
            .as_mut()
            .unwrap()
            .attributes
            .push(Attribute {
                name: points("old-start"),
                value: points("value"),
            });
        c.pending_end_tag
            .as_mut()
            .unwrap()
            .attributes
            .push(Attribute {
                name: points("old-end"),
                value: points("value"),
            });
        let old_start = c.pending_start_tag.clone();
        let old_end = c.pending_end_tag.clone();
        c.input.push(points("A")).unwrap();
        let mut e = Execution::new(initial, c);
        assert_eq!(e.step(), Ok(Progress::Continue(State::TagName)));
        assert_eq!(e.context().current_tag, Some(kind));
        match kind {
            TagKind::Start => {
                let tag = e.context().pending_start_tag.as_ref().unwrap();
                assert!(tag.name.is_empty());
                assert!(!tag.self_closing);
                assert!(tag.attributes.is_empty());
                assert_eq!(e.context().pending_end_tag, old_end);
            }
            TagKind::End => {
                let tag = e.context().pending_end_tag.as_ref().unwrap();
                assert!(tag.name.is_empty());
                assert!(!tag.self_closing);
                assert!(tag.attributes.is_empty());
                assert_eq!(e.context().pending_start_tag, old_start);
            }
        }
    }
}

#[test]
fn recovery_error_uses_normalized_code_point_location_before_attribute_name_stub() {
    let mut e = Execution::new(State::Data, Context::default());
    e.context_mut().input.push(points("😀\r\n<a/\0>")).unwrap();
    e.context_mut().input.finish();
    assert!(drain(&mut e, &mut vec![]));
    assert_eq!(
        e.step(),
        Err(UnimplementedState {
            state: State::AttributeName
        })
    );
    assert_eq!(
        e.context().errors,
        [error("unexpected-solidus-in-tag", 2, 4)]
    );
    assert_eq!(
        e.context().tokens,
        [Token::Character('😀'.into()), Token::Character('\n'.into())]
    );
    assert_eq!(
        e.context().pending_start_tag,
        Some(PendingStartTag {
            name: points("a"),
            self_closing: false,
            attributes: vec![Attribute::default()]
        })
    );
    assert_eq!(
        e.context_mut().input.consume(),
        Input::CodePoint('\0'.into())
    );
    assert_eq!(
        e.context().input.position(),
        Position { line: 2, column: 4 }
    );
    assert_eq!(
        e.context_mut().input.consume(),
        Input::CodePoint('>'.into())
    );
}

#[test]
fn recovery_executes_before_attribute_name_then_blocks_at_its_unimplemented_destinations() {
    for (text, destination, attribute_count) in [
        ("<a//>", State::AfterAttributeName, 0),
        ("<a/ >", State::AfterAttributeName, 0),
        ("<a/x>", State::AttributeName, 1),
        ("</a/x>", State::AttributeName, 1),
    ] {
        let mut e = Execution::new(State::Data, Context::default());
        e.context_mut().input.push(points(text)).unwrap();
        e.context_mut().input.finish();
        assert!(drain(&mut e, &mut vec![]));
        let blocked = Err(UnimplementedState { state: destination });
        assert_eq!(e.step(), blocked);
        let effects = snapshot(&e);
        assert!(e.context().tokens.is_empty());
        let column = if text.starts_with("</") { 5 } else { 4 };
        assert_eq!(
            e.context().errors,
            [error("unexpected-solidus-in-tag", 1, column)]
        );
        let (flag, attributes) = match e.context().current_tag.unwrap() {
            TagKind::Start => {
                let tag = e.context().pending_start_tag.as_ref().unwrap();
                (tag.self_closing, &tag.attributes)
            }
            TagKind::End => {
                let tag = e.context().pending_end_tag.as_ref().unwrap();
                (tag.self_closing, &tag.attributes)
            }
        };
        assert!(!flag);
        assert_eq!(attributes.len(), attribute_count);
        assert_eq!(e.step(), blocked);
        assert_eq!(snapshot(&e), effects);
    }
}

#[test]
fn shared_tag_name_emission_preserves_flags_and_applies_same_end_tag_rule() {
    for kind in [TagKind::Start, TagKind::End] {
        let mut c = seeded(kind, true);
        c.input.push(points(">")).unwrap();
        let mut e = Execution::new(State::TagName, c);
        assert_eq!(e.step(), Ok(Progress::Continue(State::Data)));
        let expected = match kind {
            TagKind::Start => Token::StartTag {
                name: points("start"),
                self_closing: true,
            },
            TagKind::End => Token::EndTag {
                name: points("end"),
                self_closing: true,
            },
        };
        assert_eq!(e.context().tokens.last(), Some(&expected));
        assert_eq!(
            e.context().errors.len(),
            if kind == TagKind::End { 2 } else { 1 }
        );
        if kind == TagKind::End {
            assert_eq!(
                e.context().errors[1],
                error("end-tag-with-trailing-solidus", 1, 1)
            );
            assert_eq!(e.context().last_start_tag, Some(points("title")));
        } else {
            assert_eq!(e.context().last_start_tag, Some(points("start")));
        }
    }
}

#[test]
fn every_chunk_split_matches_closed_source_step_trace_including_blocks_and_preprocessing() {
    let mut samples = [
        "<br/>",
        "</br/>",
        "<DIV/>x</DIV/>",
        "<br/",
        "</br/",
        "<a//>",
        "<a/ >",
        "</a/x>",
        "<a/\0>",
        "<a/😀>",
        "<a/\r\n>",
        "<a/\r\r\n>",
        "<a/\r",
        "\r\n<a\0/>\r\n</A/>\r",
        "<p/><a/x",
    ]
    .map(points)
    .to_vec();
    for value in [0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x01] {
        let mut input = points("<a/");
        input.push(CodePoint::new(value).unwrap());
        input.extend(points(">"));
        samples.push(input);
    }
    for input in samples {
        let run = |chunks: Vec<&[CodePoint]>, closed: bool| {
            let mut e = Execution::new(State::Data, Context::default());
            let mut trace = vec![];
            if closed {
                e.context_mut().input.push(input.iter().copied()).unwrap();
                e.context_mut().input.finish();
            } else {
                for chunk in chunks {
                    e.context_mut().input.push(chunk.iter().copied()).unwrap();
                    if drain(&mut e, &mut trace) {
                        return trace;
                    }
                    e.context_mut().input.push([]).unwrap();
                    assert!(!drain(&mut e, &mut trace));
                }
                e.context_mut().input.finish();
            }
            assert!(drain(&mut e, &mut trace));
            trace
        };
        let expected = run(vec![], true);
        for split in 0..=input.len() {
            assert_eq!(
                run(vec![&input[..split], &input[split..]], false),
                expected,
                "split {split}: {input:?}"
            );
        }
        assert_eq!(run(input.chunks(1).collect(), false), expected);
    }
}
