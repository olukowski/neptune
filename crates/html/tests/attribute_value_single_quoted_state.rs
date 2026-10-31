//! Local branch evidence for the pinned Attribute value (single-quoted) state.
//! Seeded entries inspect source effects; real Data-to-value paths are tested below.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144326-L144353>

use neptune_html::tokenizer::{
    Attribute, CodePoint, Context, Execution, Input, ParseError, PendingComment, PendingEndTag,
    PendingStartTag, Position, Progress, State, TagKind, Token, UnimplementedState,
};

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

fn seeded(kind: TagKind) -> Context {
    let attributes = vec![
        attribute("prior", "retained"),
        attribute("other", "predecessor"),
        Attribute {
            name: vec![],
            value: vec![CodePoint::new(0xd800).unwrap(), 'v'.into()],
        },
    ];
    let mut context = Context {
        current_tag: Some(kind),
        pending_start_tag: Some(PendingStartTag {
            name: points("start"),
            self_closing: true,
            attributes: attributes.clone(),
            ..PendingStartTag::default()
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("end"),
            self_closing: true,
            attributes,
            ..PendingEndTag::default()
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
    // Unrelated tag storage can itself retain a detached current attribute.
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

fn storage(context: &mut Context) -> (&mut Vec<Attribute>, &mut Option<Attribute>) {
    match context.current_tag.unwrap() {
        TagKind::Start => {
            let tag = context.pending_start_tag.as_mut().unwrap();
            (&mut tag.attributes, &mut tag.discarded_current_attribute)
        }
        TagKind::End => {
            let tag = context.pending_end_tag.as_mut().unwrap();
            (&mut tag.attributes, &mut tag.discarded_current_attribute)
        }
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

impl Snapshot {
    fn storage(&mut self, kind: TagKind) -> (&mut Vec<Attribute>, &mut Option<Attribute>) {
        match kind {
            TagKind::Start => {
                let tag = self.start.as_mut().unwrap();
                (&mut tag.attributes, &mut tag.discarded_current_attribute)
            }
            TagKind::End => {
                let tag = self.end.as_mut().unwrap();
                (&mut tag.attributes, &mut tag.discarded_current_attribute)
            }
        }
    }
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
        last_start_tag: c.last_start_tag.clone(),
        return_state: c.return_state,
        buffer: c.temporary_buffer.clone(),
    }
}

fn assert_blocked(e: &mut Execution, target: State, expected: &Snapshot) {
    for _ in 0..2 {
        assert_eq!(e.step(), Err(UnimplementedState { state: target }));
        assert_eq!(&snapshot(e), expected);
    }
}

fn seeded_value(kind: TagKind, detached: bool) -> Context {
    let mut context = seeded(kind);
    let (attributes, discarded) = storage(&mut context);
    attributes.last_mut().unwrap().name = points(if detached { "prior" } else { "current" });
    if detached {
        *discarded = attributes.pop();
    }
    context
}

fn current(snapshot: &mut Snapshot, kind: TagKind) -> &mut Attribute {
    let (attributes, detached) = snapshot.storage(kind);
    detached
        .as_mut()
        .unwrap_or_else(|| attributes.last_mut().unwrap())
}

#[test]
fn every_ascii_append_branch_and_non_scalars_preserve_exact_current_identity_and_context() {
    let values = (0..=0x7f).filter(|v| !matches!(v, 0x26 | 0x27)).chain([
        0x85, 0xa0, 0x1680, 0x2003, 0x2028, 0x3000, 0xc9, 0x130, 0x212a, 0x1f600, 0xd800, 0xdfff,
        0xfdd0, 0x10ffff,
    ]);
    for value in values {
        let point = CodePoint::new(value).unwrap();
        for kind in [TagKind::Start, TagKind::End] {
            for detached in [false, true] {
                for closed in [false, true] {
                    let mut context = seeded_value(kind, detached);
                    context.input.push([point, 'Z'.into()]).unwrap();
                    if closed {
                        context.input.finish();
                    }
                    let mut e = Execution::new(State::AttributeValueSingleQuoted, context);
                    let mut expected = snapshot(&e);
                    let normalized = if value == 0x0d { '\n'.into() } else { point };
                    expected.current = Some((
                        Input::CodePoint(normalized),
                        Position { line: 1, column: 1 },
                    ));
                    current(&mut expected, kind).value.push(if value == 0 {
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
                    assert_eq!(
                        e.step(),
                        Ok(Progress::Continue(State::AttributeValueSingleQuoted))
                    );
                    assert_eq!(
                        snapshot(&e),
                        expected,
                        "{kind:?} detached={detached} U+{value:04X}"
                    );
                    // The branch consumes exactly once, even for malformed input.
                    assert_eq!(
                        e.context_mut().input.consume(),
                        Input::CodePoint('Z'.into())
                    );
                }
            }
        }
    }
}

#[test]
fn quote_and_ampersand_consume_without_appending_before_executable_or_stub_destinations() {
    for (delimiter, target) in [
        ('\'', State::AfterAttributeValueQuoted),
        ('&', State::CharacterReference),
    ] {
        for kind in [TagKind::Start, TagKind::End] {
            for detached in [false, true] {
                for closed in [false, true] {
                    let mut context = seeded_value(kind, detached);
                    let tail = if delimiter == '\'' { " >Z" } else { "Z" };
                    context
                        .input
                        .push(points(&format!("{delimiter}{tail}")))
                        .unwrap();
                    if closed {
                        context.input.finish();
                    }
                    let mut e = Execution::new(State::AttributeValueSingleQuoted, context);
                    let mut expected = snapshot(&e);
                    expected.state = target;
                    expected.current = Some((
                        Input::CodePoint(delimiter.into()),
                        Position { line: 1, column: 1 },
                    ));
                    if delimiter == '&' {
                        expected.return_state = Some(State::AttributeValueSingleQuoted);
                    }
                    assert_eq!(e.step(), Ok(Progress::Continue(target)));
                    assert_eq!(snapshot(&e), expected);
                    if delimiter == '\'' {
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
                        if !closed {
                            for _ in 0..2 {
                                assert_eq!(e.step(), Ok(Progress::NeedMoreInput));
                                assert_eq!(snapshot(&e), expected);
                            }
                            e.context_mut().input.finish();
                        }
                        expected.current = Some((Input::Eof, Position { line: 1, column: 5 }));
                        expected.tokens.push(Token::Eof);
                        for _ in 0..3 {
                            assert_eq!(e.step(), Ok(Progress::Finished));
                            assert_eq!(snapshot(&e), expected);
                        }
                    } else {
                        // Character reference remains effect-free and terminal.
                        assert_blocked(&mut e, target, &expected);
                        // Feeding/closing does not revive the stub or consume Z.
                        if !closed {
                            e.context_mut().input.push(points("more")).unwrap();
                            e.context_mut().input.finish();
                            assert_blocked(&mut e, target, &expected);
                        }
                        assert_eq!(
                            e.context_mut().input.consume(),
                            Input::CodePoint('Z'.into())
                        );
                        assert_eq!(e.step(), Err(UnimplementedState { state: target }));
                    }
                }
            }
        }
    }
}

#[test]
fn open_exhaustion_is_effect_free_and_only_explicit_eof_abandons_selected_tag() {
    for kind in [TagKind::Start, TagKind::End] {
        for detached in [false, true] {
            for text in ["", "a\r\n\0😀"] {
                let mut context = seeded_value(kind, detached);
                context.input.push(points(text)).unwrap();
                let mut e = Execution::new(State::AttributeValueSingleQuoted, context);
                let mut trace = vec![];
                assert!(!drain(&mut e, &mut trace, 20));
                let mut expected = snapshot(&e);
                // No emission (including end-tag attribute/solidus errors) while open.
                assert_eq!(expected.tokens, [Token::Character('p'.into())]);
                let (line, column) = if text.is_empty() { (1, 1) } else { (2, 3) };
                expected.current = Some((Input::Eof, Position { line, column }));
                expected.kind = None;
                match kind {
                    TagKind::Start => expected.start = None,
                    TagKind::End => expected.end = None,
                }
                expected.errors.push(error("eof-in-tag", line, column));
                expected.tokens.push(Token::Eof);
                e.context_mut().input.finish();
                assert_eq!(e.step(), Ok(Progress::Finished));
                assert_eq!(snapshot(&e), expected);
                assert_eq!(e.step(), Ok(Progress::Finished));
                assert_eq!(snapshot(&e), expected);
            }
        }
    }
}

#[test]
fn initially_empty_value_resumes_after_suspension_and_initially_absent_return_state_is_set() {
    for kind in [TagKind::Start, TagKind::End] {
        for detached in [false, true] {
            let mut context = seeded_value(kind, detached);
            let (attributes, discarded) = storage(&mut context);
            discarded
                .as_mut()
                .unwrap_or_else(|| attributes.last_mut().unwrap())
                .value
                .clear();
            context.return_state = None;
            let mut e = Execution::new(State::AttributeValueSingleQuoted, context);
            let mut trace = vec![];
            assert!(!drain(&mut e, &mut trace, 1));
            e.context_mut().input.push(points("v\0")).unwrap();
            assert!(!drain(&mut e, &mut trace, 3));
            let mut expected = snapshot(&e);
            assert_eq!(current(&mut expected, kind).value, points("v\u{fffd}"));
            assert_eq!(expected.return_state, None);
            assert_eq!(
                expected.errors.last(),
                Some(&error("unexpected-null-character", 1, 2))
            );
            e.context_mut().input.push(points("&tail")).unwrap();
            expected.state = State::CharacterReference;
            expected.return_state = Some(State::AttributeValueSingleQuoted);
            expected.current = Some((
                Input::CodePoint('&'.into()),
                Position { line: 1, column: 3 },
            ));
            assert_eq!(e.step(), Ok(Progress::Continue(State::CharacterReference)));
            assert_eq!(snapshot(&e), expected);
            assert_blocked(&mut e, State::CharacterReference, &expected);
        }
    }
}

#[test]
fn normalized_positions_nul_and_preprocessing_diagnostics_are_not_duplicated() {
    for kind in [TagKind::Start, TagKind::End] {
        for detached in [false, true] {
            let mut context = seeded_value(kind, detached);
            let mut input = points("a\r\n\0");
            input.extend([0xd800, 0xfdd0, 0x0b].map(|v| CodePoint::new(v).unwrap()));
            input.extend(points("\0'Z"));
            context.input.push(input).unwrap();
            context.input.finish();
            let mut e = Execution::new(State::AttributeValueSingleQuoted, context);
            let mut expected = snapshot(&e);
            current(&mut expected, kind)
                .value
                .extend(points("a\n\u{fffd}"));
            current(&mut expected, kind)
                .value
                .extend([0xd800, 0xfdd0, 0x0b].map(|v| CodePoint::new(v).unwrap()));
            current(&mut expected, kind).value.push('\u{fffd}'.into());
            expected.state = State::AfterAttributeValueQuoted;
            expected.current = Some((
                Input::CodePoint('\''.into()),
                Position { line: 2, column: 6 },
            ));
            expected.errors.extend([
                error("unexpected-null-character", 2, 1),
                error("unexpected-null-character", 2, 5),
            ]);
            expected.input_errors.extend([
                error("surrogate-in-input-stream", 2, 2),
                error("noncharacter-in-input-stream", 2, 3),
                error("control-character-in-input-stream", 2, 4),
            ]);
            for _ in 0..7 {
                assert_eq!(
                    e.step(),
                    Ok(Progress::Continue(State::AttributeValueSingleQuoted))
                );
            }
            assert_eq!(
                e.step(),
                Ok(Progress::Continue(State::AfterAttributeValueQuoted))
            );
            assert_eq!(snapshot(&e), expected);
            expected.state = State::BeforeAttributeName;
            expected.current = Some((
                Input::CodePoint('Z'.into()),
                Position { line: 2, column: 7 },
            ));
            expected
                .errors
                .push(error("missing-whitespace-between-attributes", 2, 7));
            assert_eq!(e.step(), Ok(Progress::Continue(State::BeforeAttributeName)));
            assert_eq!(snapshot(&e), expected);
            // The destination reconsumes Z without repeating preprocessing.
            assert_eq!(
                e.context_mut().input.consume(),
                Input::CodePoint('Z'.into())
            );
        }
    }
}

#[test]
fn an_already_preprocessed_reconsumed_point_appends_once_with_original_position() {
    // Cursor-level seeded entry, not a fabricated implementation of Before value.
    for kind in [TagKind::Start, TagKind::End] {
        for value in [0, 0xd800, 0xfdd0, 0x0b] {
            let point = CodePoint::new(value).unwrap();
            let mut context = seeded_value(kind, true);
            context
                .input
                .push(points("\r\n").into_iter().chain([point]))
                .unwrap();
            assert_eq!(context.input.consume(), Input::CodePoint('\n'.into()));
            assert_eq!(context.input.consume(), Input::CodePoint(point));
            context.input.reconsume().unwrap();
            let mut e = Execution::new(State::AttributeValueSingleQuoted, context);
            let mut expected = snapshot(&e);
            current(&mut expected, kind).value.push(if value == 0 {
                '\u{fffd}'.into()
            } else {
                point
            });
            if value == 0 {
                expected
                    .errors
                    .push(error("unexpected-null-character", 2, 1));
            }
            assert_eq!(
                e.step(),
                Ok(Progress::Continue(State::AttributeValueSingleQuoted))
            );
            assert_eq!(snapshot(&e), expected);
            assert_eq!(
                e.context().input.position(),
                Position { line: 2, column: 1 }
            );
            assert_eq!(e.context_mut().input.consume(), Input::NeedMoreInput);
        }
    }
}

#[test]
fn real_data_entry_completes_abandons_or_blocks_only_at_character_reference() {
    for (input, expected_tokens, errors, blocked) in [
        (
            "<a x='value'>",
            vec![
                Token::StartTag {
                    name: points("a"),
                    attributes: vec![attribute("x", "value")],
                    self_closing: false,
                },
                Token::Eof,
            ],
            vec![],
            false,
        ),
        (
            "</a x='value'>",
            vec![
                Token::EndTag {
                    name: points("a"),
                    attributes: vec![attribute("x", "value")],
                    self_closing: false,
                },
                Token::Eof,
            ],
            vec![error("end-tag-with-attributes", 1, 14)],
            false,
        ),
        (
            "<a x='",
            vec![Token::Eof],
            vec![error("eof-in-tag", 1, 7)],
            false,
        ),
        (
            "<a x='\0&tail",
            vec![],
            vec![error("unexpected-null-character", 1, 7)],
            true,
        ),
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
                result => {
                    assert_eq!(
                        result,
                        if blocked {
                            Err(UnimplementedState {
                                state: State::CharacterReference,
                            })
                        } else {
                            Ok(Progress::Finished)
                        }
                    );
                    break;
                }
            }
        }
        assert!(seen.contains(&State::AttributeValueSingleQuoted));
        assert_eq!(e.context().tokens, expected_tokens);
        assert_eq!(e.context().errors, errors);
        assert!(e.context().input.errors().is_empty());
        if blocked {
            assert_eq!(
                e.context().return_state,
                Some(State::AttributeValueSingleQuoted)
            );
            assert_eq!(
                e.context().pending_start_tag.as_ref().unwrap().attributes,
                [attribute("x", "\u{fffd}")]
            );
            let before = snapshot(&e);
            for _ in 0..3 {
                assert_eq!(
                    e.step(),
                    Err(UnimplementedState {
                        state: State::CharacterReference
                    })
                );
                assert_eq!(snapshot(&e), before);
            }
            assert_eq!(
                e.context_mut().input.consume(),
                Input::CodePoint('t'.into())
            );
        } else {
            assert_eq!(e.step(), Ok(Progress::Finished));
            assert_eq!(e.context().current_tag, None);
            assert!(e.context().pending_start_tag.is_none());
            assert!(e.context().pending_end_tag.is_none());
        }
    }
}

// Real execution, stopping at the FIRST terminal stub. Never reconstruct it.
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
    panic!("Single-quoted value execution did not suspend or terminate");
}

#[test]
fn all_two_chunk_splits_and_single_point_chunks_match_closed_source_step_traces() {
    let mut samples: Vec<_> = [
        "",
        "'",
        "&",
        "abc",
        "\t\n\u{c} \"<>=/\u{a0}É😀",
        "a\0\0z",
        "value'>tail",
        "a&ref;tail",
        "a\r\nb\r\r\nc\r",
        "\0\r\n'Z",
        "a\r\n&tail",
    ]
    .map(points)
    .into();
    let mut non_scalars = points("a\r\n");
    non_scalars
        .extend([0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x0b, 0].map(|v| CodePoint::new(v).unwrap()));
    samples.push(non_scalars.clone());
    non_scalars.extend(points("'tail"));
    samples.push(non_scalars);
    for input in samples {
        for kind in [TagKind::Start, TagKind::End] {
            for detached in [false, true] {
                let run = |chunks: Vec<&[CodePoint]>, closed: bool| {
                    let mut e = Execution::new(
                        State::AttributeValueSingleQuoted,
                        seeded_value(kind, detached),
                    );
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
                        "{kind:?}, detached={detached}, split {split} of {input:?}",
                    );
                }
                assert_eq!(run(input.chunks(1).collect(), false), expected);
            }
        }
    }
}
