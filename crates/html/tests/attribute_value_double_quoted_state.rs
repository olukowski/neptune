//! Local branch evidence, not upstream conformance or reachable seeded entry.
//! Pinned HTML Standard "Attribute value (double-quoted) state":
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144296-L144324>

use neptune_html::tokenizer::{
    Attribute, CodePoint, Context, Execution, Input, ParseError, PendingComment, PendingEndTag,
    PendingStartTag, Position, Progress, State, TagKind, Token, UnimplementedState,
};

const SOURCE: State = State::AttributeValueDoubleQuoted;

fn points(text: &str) -> Vec<CodePoint> {
    text.chars().map(Into::into).collect()
}

fn error(code: &'static str, line: usize, column: usize) -> ParseError {
    ParseError {
        code,
        position: Position { line, column },
    }
}

fn attribute(name: &str, value: &str) -> Attribute {
    Attribute {
        name: points(name),
        value: points(value),
    }
}

// Valid isolated entry, NOT a path through Before attribute value (still a stub).
fn seeded(kind: TagKind, duplicate: bool) -> Context {
    let current = Attribute {
        name: points(if duplicate { "prior" } else { "current" }),
        // Stored non-scalars are not preprocessed again.
        value: vec![CodePoint::new(0xd800).unwrap(), 'v'.into()],
    };
    let mut attributes = vec![
        attribute("prior", "retained"),
        attribute("other", "predecessor"),
    ];
    let discarded = if duplicate {
        Some(current)
    } else {
        attributes.push(current);
        None
    };
    let mut context = Context {
        current_tag: Some(kind),
        pending_start_tag: Some(PendingStartTag {
            name: points("start"),
            self_closing: true,
            attributes: attributes.clone(),
            discarded_current_attribute: discarded.clone(),
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("end"),
            self_closing: true,
            attributes,
            discarded_current_attribute: discarded,
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
    };
    // Unrelated storage has an independent detached current attribute too.
    match kind {
        TagKind::Start => {
            context
                .pending_end_tag
                .as_mut()
                .unwrap()
                .discarded_current_attribute = Some(attribute("unrelated", "discarded"))
        }
        TagKind::End => {
            context
                .pending_start_tag
                .as_mut()
                .unwrap()
                .discarded_current_attribute = Some(attribute("unrelated", "discarded"))
        }
    }
    context
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

impl Snapshot {
    fn current_attribute(&mut self, kind: TagKind) -> &mut Attribute {
        let (attributes, discarded) = match kind {
            TagKind::Start => {
                let tag = self.start.as_mut().unwrap();
                (&mut tag.attributes, &mut tag.discarded_current_attribute)
            }
            TagKind::End => {
                let tag = self.end.as_mut().unwrap();
                (&mut tag.attributes, &mut tag.discarded_current_attribute)
            }
        };
        if let Some(attribute) = discarded {
            attribute
        } else {
            attributes.last_mut().unwrap()
        }
    }
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

fn assert_terminal(e: &mut Execution, result: Result<Progress, UnimplementedState>) {
    let unchanged = snapshot(e);
    for _ in 0..3 {
        assert_eq!(e.step(), result);
        assert_eq!(snapshot(e), unchanged);
    }
}

#[test]
fn nul_and_every_literal_ascii_branch_preserve_identity_and_unrelated_storage() {
    let values: Vec<_> = (0..=0x7f)
        .filter(|v| !matches!(v, 0x22 | 0x26))
        .chain([
            0x85, 0xa0, 0x1680, 0x2003, 0x2028, 0x3000, 0xc9, 0x130, 0x212a, 0x1f600, 0xd800,
            0xdfff, 0xfdd0, 0x10ffff,
        ])
        .collect();
    for value in values {
        for kind in [TagKind::Start, TagKind::End] {
            for duplicate in [false, true] {
                for closed in [false, true] {
                    let point = CodePoint::new(value).unwrap();
                    let mut c = seeded(kind, duplicate);
                    c.input.push([point, 'Z'.into()]).unwrap();
                    if closed {
                        c.input.finish();
                    }
                    let mut e = Execution::new(SOURCE, c);
                    let mut expected = snapshot(&e);
                    let normalized = if value == 0x0d { '\n'.into() } else { point };
                    expected.current = Some((
                        Input::CodePoint(normalized),
                        Position { line: 1, column: 1 },
                    ));
                    expected.current_attribute(kind).value.push(if value == 0 {
                        '\u{fffd}'.into()
                    } else {
                        normalized
                    });
                    if value == 0 {
                        expected
                            .errors
                            .push(error("unexpected-null-character", 1, 1));
                    }
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
                    assert_eq!(e.step(), Ok(Progress::Continue(SOURCE)));
                    assert_eq!(
                        snapshot(&e),
                        expected,
                        "{kind:?}, duplicate={duplicate}, {value:x}"
                    );
                    // Exactly one consumption; CR becomes LF, no branch reconsumes.
                    assert_eq!(
                        e.context_mut().input.consume(),
                        Input::CodePoint('Z'.into())
                    );
                    assert_eq!(
                        e.context().input.position(),
                        if value == 0x0a || value == 0x0d {
                            Position { line: 2, column: 1 }
                        } else {
                            Position { line: 1, column: 2 }
                        }
                    );
                }
            }
        }
    }
}

#[test]
fn quote_and_ampersand_consume_without_appending_before_executable_or_stub_destinations() {
    for kind in [TagKind::Start, TagKind::End] {
        for duplicate in [false, true] {
            for closed in [false, true] {
                for old_return in [None, Some(State::Rcdata), Some(SOURCE)] {
                    for (point, target) in [
                        ('"', State::AfterAttributeValueQuoted),
                        ('&', State::CharacterReference),
                    ] {
                        let mut c = seeded(kind, duplicate);
                        c.return_state = old_return;
                        let tail = if point == '"' { " >Z\0" } else { "Z\0" };
                        c.input.push(points(&format!("{point}{tail}"))).unwrap();
                        if closed {
                            c.input.finish();
                        }
                        let mut e = Execution::new(SOURCE, c);
                        let mut expected = snapshot(&e);
                        expected.state = target;
                        expected.current = Some((
                            Input::CodePoint(point.into()),
                            Position { line: 1, column: 1 },
                        ));
                        if point == '&' {
                            expected.return_state = Some(SOURCE);
                        }
                        assert_eq!(e.step(), Ok(Progress::Continue(target)));
                        assert_eq!(snapshot(&e), expected);
                        if point == '"' {
                            // Preserve exact source snapshots through whitespace and
                            // Before attribute name's reconsumption, then execute emission.
                            expected.state = State::BeforeAttributeName;
                            expected.current = Some((
                                Input::CodePoint(' '.into()),
                                Position { line: 1, column: 2 },
                            ));
                            assert_eq!(e.step(), Ok(Progress::Continue(expected.state)));
                            assert_eq!(snapshot(&e), expected);
                            expected.state = State::AfterAttributeName;
                            expected.current = Some((
                                Input::CodePoint('>'.into()),
                                Position { line: 1, column: 3 },
                            ));
                            assert_eq!(e.step(), Ok(Progress::Continue(expected.state)));
                            assert_eq!(snapshot(&e), expected);
                            expected.kind = None;
                            expected.state = State::Data;
                            let token = match kind {
                                TagKind::Start => {
                                    let tag = expected.start.take().unwrap();
                                    expected.last_start_tag = Some(tag.name.clone());
                                    Token::StartTag {
                                        name: tag.name,
                                        attributes: tag.attributes,
                                        self_closing: tag.self_closing,
                                    }
                                }
                                TagKind::End => {
                                    let tag = expected.end.take().unwrap();
                                    expected.errors.extend([
                                        error("end-tag-with-attributes", 1, 3),
                                        error("end-tag-with-trailing-solidus", 1, 3),
                                    ]);
                                    Token::EndTag {
                                        name: tag.name,
                                        attributes: tag.attributes,
                                        self_closing: tag.self_closing,
                                    }
                                }
                            };
                            expected.tokens.push(token);
                            assert_eq!(e.step(), Ok(Progress::Continue(State::Data)));
                            assert_eq!(snapshot(&e), expected);
                            // '>' was consumed once; only Data handles the tail.
                            expected.current = Some((
                                Input::CodePoint('Z'.into()),
                                Position { line: 1, column: 4 },
                            ));
                            expected.tokens.push(Token::Character('Z'.into()));
                            assert_eq!(e.step(), Ok(Progress::Continue(State::Data)));
                            assert_eq!(snapshot(&e), expected);
                            expected.current = Some((
                                Input::CodePoint('\0'.into()),
                                Position { line: 1, column: 5 },
                            ));
                            expected.tokens.push(Token::Character('\0'.into()));
                            expected
                                .errors
                                .push(error("unexpected-null-character", 1, 5));
                            assert_eq!(e.step(), Ok(Progress::Continue(State::Data)));
                            assert_eq!(snapshot(&e), expected);
                            if !closed {
                                for _ in 0..2 {
                                    assert_eq!(e.step(), Ok(Progress::NeedMoreInput));
                                    assert_eq!(snapshot(&e), expected);
                                }
                                e.context_mut().input.finish();
                            }
                            expected.current = Some((Input::Eof, Position { line: 1, column: 6 }));
                            expected.tokens.push(Token::Eof);
                            for _ in 0..3 {
                                assert_eq!(e.step(), Ok(Progress::Finished));
                                assert_eq!(snapshot(&e), expected);
                            }
                        } else {
                            // Character reference remains effect-free and terminal.
                            let blocked = Err(UnimplementedState { state: target });
                            assert_terminal(&mut e, blocked);
                            e.context_mut()
                                .input
                                .push([])
                                .unwrap_or_else(|_| assert!(closed));
                            assert_eq!(
                                e.context_mut().input.consume(),
                                Input::CodePoint('Z'.into())
                            );
                            assert_terminal(&mut e, blocked);
                            assert_eq!(
                                e.context_mut().input.consume(),
                                Input::CodePoint('\0'.into())
                            );
                            assert_eq!(e.context().errors, expected.errors);
                            assert_terminal(&mut e, blocked);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn eof_abandons_only_selected_tag_without_emission_errors_or_last_start_tag_update() {
    for kind in [TagKind::Start, TagKind::End] {
        for duplicate in [false, true] {
            for prefix in ["", "😀\r\n\0'<>\t"] {
                let mut c = seeded(kind, duplicate);
                c.input.push(points(prefix)).unwrap();
                c.input.finish();
                let mut e = Execution::new(SOURCE, c);
                for _ in 0..prefix.replace("\r\n", "\n").chars().count() {
                    assert_eq!(e.step(), Ok(Progress::Continue(SOURCE)));
                }
                let mut expected = snapshot(&e);
                expected.kind = None;
                match kind {
                    TagKind::Start => expected.start = None,
                    TagKind::End => expected.end = None,
                }
                let position = if prefix.is_empty() {
                    Position { line: 1, column: 1 }
                } else {
                    Position { line: 2, column: 6 }
                };
                expected.current = Some((Input::Eof, position));
                expected.errors.push(ParseError {
                    code: "eof-in-tag",
                    position,
                });
                expected.tokens.push(Token::Eof);
                assert_eq!(e.step(), Ok(Progress::Finished));
                assert_eq!(snapshot(&e), expected);
                assert_terminal(&mut e, Ok(Progress::Finished));
            }
        }
    }
}

#[test]
fn open_exhaustion_is_effect_free_before_and_after_values_and_can_finish_or_take_a_delimiter() {
    for kind in [TagKind::Start, TagKind::End] {
        for duplicate in [false, true] {
            for prefix in ["", "a\0'<>\r\n"] {
                for delimiter in [None, Some('"'), Some('&')] {
                    let mut e = Execution::new(SOURCE, seeded(kind, duplicate));
                    assert_eq!(e.step(), Ok(Progress::NeedMoreInput));
                    e.context_mut().input.push(points(prefix)).unwrap();
                    for _ in 0..prefix.replace("\r\n", "\n").chars().count() {
                        assert_eq!(e.step(), Ok(Progress::Continue(SOURCE)));
                    }
                    let unchanged = snapshot(&e);
                    for _ in 0..3 {
                        e.context_mut().input.push([]).unwrap();
                        assert_eq!(e.step(), Ok(Progress::NeedMoreInput));
                        assert_eq!(snapshot(&e), unchanged);
                    }
                    if let Some(point) = delimiter {
                        e.context_mut().input.push([point.into()]).unwrap();
                        let target = if point == '"' {
                            State::AfterAttributeValueQuoted
                        } else {
                            State::CharacterReference
                        };
                        assert_eq!(e.step(), Ok(Progress::Continue(target)));
                        if point == '"' {
                            let unchanged = snapshot(&e);
                            for _ in 0..3 {
                                assert_eq!(e.step(), Ok(Progress::NeedMoreInput));
                                assert_eq!(snapshot(&e), unchanged);
                            }
                            e.context_mut().input.finish();
                            let mut expected = unchanged;
                            expected.kind = None;
                            match kind {
                                TagKind::Start => expected.start = None,
                                TagKind::End => expected.end = None,
                            }
                            let position = if prefix.is_empty() {
                                Position { line: 1, column: 2 }
                            } else {
                                Position { line: 2, column: 2 }
                            };
                            expected.current = Some((Input::Eof, position));
                            expected.errors.push(ParseError {
                                code: "eof-in-tag",
                                position,
                            });
                            expected.tokens.push(Token::Eof);
                            assert_eq!(e.step(), Ok(Progress::Finished));
                            assert_eq!(snapshot(&e), expected);
                            assert_terminal(&mut e, Ok(Progress::Finished));
                        } else {
                            assert_terminal(&mut e, Err(UnimplementedState { state: target }));
                        }
                    } else {
                        e.context_mut().input.finish();
                        assert_eq!(e.step(), Ok(Progress::Finished));
                        assert_terminal(&mut e, Ok(Progress::Finished));
                    }
                }
            }
        }
    }
}

#[test]
fn normalized_locations_and_non_scalars_are_preserved_with_preprocessing_errors_once() {
    for kind in [TagKind::Start, TagKind::End] {
        for duplicate in [false, true] {
            let mut c = seeded(kind, duplicate);
            c.input.push(points("😀\r\n")).unwrap();
            assert_eq!(c.input.consume(), Input::CodePoint('😀'.into()));
            assert_eq!(c.input.consume(), Input::CodePoint('\n'.into()));
            let raw: Vec<_> = [0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x0b, 0, 0x22, 0x5a]
                .map(|v| CodePoint::new(v).unwrap())
                .into();
            c.input.push(raw.clone()).unwrap();
            // Valid cursor reconsumption on entry: not a simulated adjacent state.
            assert_eq!(c.input.consume(), Input::CodePoint(raw[0]));
            c.input.reconsume().unwrap();
            let mut e = Execution::new(SOURCE, c);
            let mut expected_value = vec![CodePoint::new(0xd800).unwrap(), 'v'.into()];
            for (index, point) in raw[..6].iter().enumerate() {
                assert_eq!(e.step(), Ok(Progress::Continue(SOURCE)));
                assert_eq!(
                    e.context().input.current(),
                    Some((
                        Input::CodePoint(*point),
                        Position {
                            line: 2,
                            column: index + 1
                        }
                    ))
                );
                expected_value.push(if point.value() == 0 {
                    '\u{fffd}'.into()
                } else {
                    *point
                });
                assert_eq!(snapshot(&e).current_attribute(kind).value, expected_value);
            }
            assert_eq!(
                e.context().errors,
                [
                    error("prior-diagnostic", 1, 1),
                    error("unexpected-null-character", 2, 6)
                ]
            );
            assert_eq!(
                e.context().input.errors(),
                [
                    error("surrogate-in-input-stream", 2, 1),
                    error("surrogate-in-input-stream", 2, 2),
                    error("noncharacter-in-input-stream", 2, 3),
                    error("noncharacter-in-input-stream", 2, 4),
                    error("control-character-in-input-stream", 2, 5),
                ]
            );
            assert_eq!(
                e.step(),
                Ok(Progress::Continue(State::AfterAttributeValueQuoted))
            );
            assert_eq!(
                e.context().input.position(),
                Position { line: 2, column: 7 }
            );
            let mut expected = snapshot(&e);
            expected.state = State::BeforeAttributeName;
            expected.current = Some((
                Input::CodePoint('Z'.into()),
                Position { line: 2, column: 8 },
            ));
            expected
                .errors
                .push(error("missing-whitespace-between-attributes", 2, 8));
            assert_eq!(e.step(), Ok(Progress::Continue(State::BeforeAttributeName)));
            assert_eq!(snapshot(&e), expected);
            // Recovery scheduled reconsumption; preprocessing errors remain once.
            assert_eq!(
                e.context_mut().input.consume(),
                Input::CodePoint('Z'.into())
            );
        }
    }
}

#[test]
fn real_data_entry_still_blocks_before_value_without_falsely_passing_a_tag_prefix() {
    for input in [
        "<p><a x=\"value\">",
        "<p></a x=\"value\">",
        "<a x=\"",
        "</a x=\"\0&",
    ] {
        let mut c = Context::default();
        c.input.push(points(input)).unwrap();
        c.input.finish();
        let mut e = Execution::new(State::Data, c);
        let mut seen = vec![];
        loop {
            seen.push(e.state());
            match e.step() {
                Ok(Progress::Continue(_)) => {}
                Err(block) => {
                    assert_eq!(block.state, State::BeforeAttributeValue);
                    break;
                }
                other => panic!("unexpected completion: {other:?}"),
            }
        }
        assert!(!seen.contains(&SOURCE));
        assert!(e.context().errors.is_empty());
        assert!(e.context().input.errors().is_empty());
        assert_eq!(
            e.context().tokens,
            if input.starts_with("<p>") {
                vec![Token::StartTag {
                    name: points("p"),
                    attributes: vec![],
                    self_closing: false,
                }]
            } else {
                vec![]
            }
        );
        assert_eq!(
            e.context_mut().input.consume(),
            Input::CodePoint('"'.into())
        );
        assert_terminal(
            &mut e,
            Err(UnimplementedState {
                state: State::BeforeAttributeValue,
            }),
        );
    }
}

// Stop at the first terminal result, never reconstruct or resume a blocked run.
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
            assert_terminal(e, result);
            return true;
        }
    }
    panic!("state trace did not suspend or terminate");
}

#[test]
fn all_two_chunk_splits_and_single_point_chunks_match_every_closed_source_step() {
    let mut samples: Vec<_> = [
        "",
        "\"tail",
        "&amp;",
        "\0",
        "a\0\"tail",
        "\0&tail",
        "'<>/= ABCé😀\t\u{c}",
        "\r\n\0\"tail",
        "\r\r\n'<>\r",
        "a\r",
        "<p><a x=\"value\">",
        "</a x=\"value\">",
    ]
    .map(points)
    .into();
    samples.push(
        [
            points("😀\r\n"),
            [0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x0b, 0]
                .map(|v| CodePoint::new(v).unwrap())
                .into(),
            points("&tail"),
        ]
        .concat(),
    );
    for input in samples {
        for kind in [TagKind::Start, TagKind::End] {
            for duplicate in [false, true] {
                for initial in [SOURCE, State::Data] {
                    let run = |chunks: Vec<&[CodePoint]>, closed: bool| {
                        let c = if initial == SOURCE {
                            seeded(kind, duplicate)
                        } else {
                            Context::default()
                        };
                        let mut e = Execution::new(initial, c);
                        let mut trace = vec![];
                        if closed {
                            e.context_mut().input.push(input.iter().copied()).unwrap();
                            e.context_mut().input.finish();
                        } else {
                            for chunk in chunks {
                                e.context_mut().input.push(chunk.iter().copied()).unwrap();
                                if drain(&mut e, &mut trace, input.len() * 3 + 5) {
                                    return trace;
                                }
                                e.context_mut().input.push([]).unwrap();
                                assert!(!drain(&mut e, &mut trace, 1));
                            }
                            e.context_mut().input.finish();
                        }
                        assert!(drain(&mut e, &mut trace, input.len() * 3 + 5));
                        trace
                    };
                    let expected = run(vec![], true);
                    for split in 0..=input.len() {
                        assert_eq!(
                            run(vec![&input[..split], &input[split..]], false),
                            expected,
                            "{initial:?}, {kind:?}, duplicate={duplicate}, split={split}, {input:?}"
                        );
                    }
                    assert_eq!(run(input.chunks(1).collect(), false), expected);
                }
            }
        }
    }
}
