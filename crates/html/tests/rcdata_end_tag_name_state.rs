//! Isolated-entry evidence for pinned source 404b3eb14a7a936dddefb2d78e224dbeb01d2358,
//! RCDATA end tag name (143524–143565), appropriate end tags (143212–143216),
//! and preprocessing/reconsumption. Local checks are not upstream conformance.

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

fn seeded(name: &[CodePoint], buffer: &[CodePoint], last: Option<Vec<CodePoint>>) -> Context {
    Context {
        tokens: vec![Token::Character('!'.into())],
        errors: vec![error("prior", 4, 5)],
        current_tag: Some(TagKind::End),
        pending_end_tag: Some(PendingEndTag {
            name: name.to_vec(),
            ..PendingEndTag::default()
        }),
        temporary_buffer: buffer.to_vec(),
        last_start_tag: last,
        pending_start_tag: Some(PendingStartTag {
            name: points("unrelated\r\0"),
            self_closing: true,
            attributes: vec![Attribute {
                name: points("kept"),
                value: points("value"),
            }],
            discarded_current_attribute: Some(Attribute::default()),
        }),
        pending_comment: Some(PendingComment {
            data: points("comment\r\0"),
        }),
        return_state: Some(State::AttributeValueDoubleQuoted),
        ..Context::default()
    }
}

fn entry(input: &[CodePoint], context: Context, finished: bool) -> Execution {
    let mut execution = Execution::new(State::RcdataEndTagName, context);
    execution
        .context_mut()
        .input
        .push(input.iter().copied())
        .unwrap();
    if finished {
        execution.context_mut().input.finish();
    }
    execution
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Snapshot {
    state: State,
    current: Option<(Input, Position)>,
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
    input_errors: Vec<ParseError>,
    current_tag: Option<TagKind>,
    end: Option<PendingEndTag>,
    start: Option<PendingStartTag>,
    comment: Option<PendingComment>,
    buffer: Vec<CodePoint>,
    last: Option<Vec<CodePoint>>,
    return_state: Option<State>,
}

fn snapshot(execution: &Execution) -> Snapshot {
    let context = execution.context();
    Snapshot {
        state: execution.state(),
        current: context.input.current(),
        tokens: context.tokens.clone(),
        errors: context.errors.clone(),
        input_errors: context.input.errors().to_vec(),
        current_tag: context.current_tag,
        end: context.pending_end_tag.clone(),
        start: context.pending_start_tag.clone(),
        comment: context.pending_comment.clone(),
        buffer: context.temporary_buffer.clone(),
        last: context.last_start_tag.clone(),
        return_state: context.return_state,
    }
}

fn literal(buffer: &[CodePoint]) -> Vec<Token> {
    points("</")
        .iter()
        .chain(buffer)
        .copied()
        .map(Token::Character)
        .collect()
}

fn stable_terminal(execution: &mut Execution, result: Result<Progress, UnimplementedState>) {
    let before = snapshot(execution);
    for _ in 0..3 {
        assert_eq!(execution.step(), result);
        assert_eq!(snapshot(execution), before);
    }
}

#[test]
fn all_ascii_letters_normalize_only_the_name_and_append_original_spelling() {
    let input = points("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz");
    for last in [None, Some(points("not-a-match"))] {
        let context = seeded(&[], &[], last);
        let tag = context.pending_end_tag.as_ref().unwrap();
        // Correct predecessor creation defaults, without implementing that predecessor.
        assert!(tag.name.is_empty());
        assert!(!tag.self_closing);
        assert!(tag.attributes.is_empty());
        assert!(tag.discarded_current_attribute.is_none());
        assert!(context.temporary_buffer.is_empty());
        let mut execution = entry(&input, context, false);
        let mut expected = snapshot(&execution);
        for (index, point) in input.iter().enumerate() {
            let normalized = CodePoint::new(if point.value() <= 0x5a {
                point.value() + 0x20
            } else {
                point.value()
            })
            .unwrap();
            expected.end.as_mut().unwrap().name.push(normalized);
            expected.buffer.push(*point);
            expected.current = Some((
                Input::CodePoint(*point),
                Position {
                    line: 1,
                    column: index + 1,
                },
            ));
            assert_eq!(
                execution.step(),
                Ok(Progress::Continue(State::RcdataEndTagName))
            );
            assert_eq!(snapshot(&execution), expected);
        }
        for _ in 0..3 {
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&execution), expected);
        }
    }
}

#[test]
fn accumulation_preserves_seeded_non_scalars_attributes_flags_and_detached_storage() {
    let seed = [
        CodePoint::new(0xd800).unwrap(),
        '\r'.into(),
        '\0'.into(),
        'É'.into(),
    ];
    let mut context = seeded(&seed, &seed, Some(seed.to_vec()));
    let tag = context.pending_end_tag.as_mut().unwrap();
    tag.self_closing = true;
    tag.attributes.push(Attribute {
        name: points("kept"),
        value: seed.to_vec(),
    });
    tag.discarded_current_attribute = Some(Attribute {
        name: points("kept"),
        value: points("discarded"),
    });
    let mut execution = entry(&points("Az"), context, true);
    let mut expected = snapshot(&execution);
    for (column, point, normalized) in [(1, 'A', 'a'), (2, 'z', 'z')] {
        expected.end.as_mut().unwrap().name.push(normalized.into());
        expected.buffer.push(point.into());
        expected.current = Some((Input::CodePoint(point.into()), Position { line: 1, column }));
        assert_eq!(
            execution.step(),
            Ok(Progress::Continue(State::RcdataEndTagName))
        );
        assert_eq!(snapshot(&execution), expected);
    }
    // Stored seed data is never preprocessed again, even when emitted literally
    // on EOF fallback. Fallback retains the selected tag and all its storage.
    expected.current = Some((Input::Eof, Position { line: 1, column: 3 }));
    expected.state = State::Rcdata;
    expected.tokens.extend(literal(&expected.buffer));
    assert_eq!(execution.step(), Ok(Progress::Continue(State::Rcdata)));
    assert_eq!(snapshot(&execution), expected);
    assert!(execution.context().input.errors().is_empty());
    expected.tokens.push(Token::Eof);
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(snapshot(&execution), expected);
    stable_terminal(&mut execution, Ok(Progress::Finished));
}

#[test]
fn every_delimiter_checks_exact_appropriateness_and_consumes_or_reconsumes() {
    let candidate = points("title");
    for last in [
        None,
        Some(points("tit")),
        Some(points("titlex")),
        Some(points("TITLE")),
        Some(candidate.clone()),
    ] {
        for (raw, normalized, target) in [
            ('\t', '\t', State::BeforeAttributeName),
            ('\n', '\n', State::BeforeAttributeName),
            ('\u{c}', '\u{c}', State::BeforeAttributeName),
            (' ', ' ', State::BeforeAttributeName),
            ('\r', '\n', State::BeforeAttributeName),
            ('/', '/', State::SelfClosingStartTag),
            ('>', '>', State::Data),
        ] {
            let appropriate = last.as_ref() == Some(&candidate);
            for finished in [false, true] {
                let mut execution = entry(
                    &points(&format!("{raw}X")),
                    seeded(&candidate, &points("TiTlE"), last.clone()),
                    finished,
                );
                let mut expected = snapshot(&execution);
                expected.current = Some((
                    Input::CodePoint(normalized.into()),
                    Position { line: 1, column: 1 },
                ));
                expected.state = if appropriate { target } else { State::Rcdata };
                if appropriate && raw == '>' {
                    expected.tokens.push(Token::EndTag {
                        name: candidate.clone(),
                        attributes: vec![],
                        self_closing: false,
                    });
                    expected.end = None;
                    expected.current_tag = None;
                } else if !appropriate {
                    expected.tokens.extend(literal(&expected.buffer));
                }
                assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                assert_eq!(snapshot(&execution), expected);
                // Cursor inspection, not execution of a destination: delimiters are
                // handed off consumed only for a matching tag; the tail stays unread.
                if !appropriate {
                    assert_eq!(
                        execution.context_mut().input.consume(),
                        Input::CodePoint(normalized.into())
                    );
                }
                assert_eq!(
                    execution.context_mut().input.consume(),
                    Input::CodePoint('X'.into())
                );
                assert_eq!(
                    execution.context().input.position(),
                    Position {
                        line: if normalized == '\n' { 2 } else { 1 },
                        column: if normalized == '\n' { 1 } else { 2 }
                    }
                );
                assert_eq!(
                    execution.context_mut().input.consume(),
                    if finished {
                        Input::Eof
                    } else {
                        Input::NeedMoreInput
                    }
                );
            }
        }
    }
    // Code-point equality, not ASCII-insensitive comparison or Unicode normalization.
    for name in [
        vec![],
        points("É"),
        points("e\u{301}"),
        vec![CodePoint::new(0xd800).unwrap()],
    ] {
        let mut execution = entry(&points(">"), seeded(&name, &[], Some(name.clone())), true);
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
        assert_eq!(
            execution.context().tokens.last(),
            Some(&Token::EndTag {
                name,
                attributes: vec![],
                self_closing: false
            })
        );
        assert!(execution.context().input.errors().is_empty());
    }
    for (name, last) in [("É", "é"), ("é", "e\u{301}"), ("title", "Title")] {
        let mut execution = entry(
            &points(">"),
            seeded(&points(name), &points(name), Some(points(last))),
            true,
        );
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Rcdata)));
        assert_eq!(&execution.context().tokens[1..], literal(&points(name)));
    }
    let mut execution = entry(&points(">"), seeded(&[], &[], None), true);
    assert_eq!(execution.step(), Ok(Progress::Continue(State::Rcdata)));
    assert_eq!(&execution.context().tokens[1..], literal(&[]));
}

#[test]
fn anything_else_is_literal_even_when_candidate_matches_and_diagnostics_belong_to_destination() {
    let mut input = points("\0!\"#$%&'()*+,-.0123456789:;<=?@[\\]^_`{|}~Éé\u{a0}\u{b}\u{feff}😀");
    input.extend(
        [0xd800, 0xdfff, 0xfdd0, 0x10ffff, 1, 0x7f, 0x85].map(|v| CodePoint::new(v).unwrap()),
    );
    let buffer = [
        points("TiTlE"),
        vec!['\r'.into(), '\0'.into(), CodePoint::new(0xd800).unwrap()],
    ]
    .concat();
    for point in input {
        let mut execution = entry(
            &[point, 'X'.into()],
            seeded(&points("title"), &buffer, Some(points("title"))),
            false,
        );
        let mut expected = snapshot(&execution);
        expected.state = State::Rcdata;
        expected.current = Some((Input::CodePoint(point), Position { line: 1, column: 1 }));
        expected.tokens.extend(literal(&buffer));
        let preprocessing = match point.value() {
            0xd800..=0xdfff => Some("surrogate-in-input-stream"),
            0xfdd0 | 0x10ffff => Some("noncharacter-in-input-stream"),
            1 | 0x0b | 0x7f | 0x85 => Some("control-character-in-input-stream"),
            _ => None,
        };
        if let Some(code) = preprocessing {
            expected.input_errors.push(error(code, 1, 1));
        }
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Rcdata)));
        assert_eq!(snapshot(&execution), expected);
        // Destination processing is separate. The offending point was not appended
        // to the name/buffer or emitted by the source, and preprocessing happens once.
        let target = match point.value() {
            0x26 => State::CharacterReference,
            0x3c => State::RcdataLessThanSign,
            _ => State::Rcdata,
        };
        expected.state = target;
        match point.value() {
            0x26 => expected.return_state = Some(State::Rcdata),
            0x3c => {}
            0 => {
                expected
                    .errors
                    .push(error("unexpected-null-character", 1, 1));
                expected.tokens.push(Token::Character('\u{fffd}'.into()));
            }
            _ => expected.tokens.push(Token::Character(point)),
        }
        assert_eq!(execution.step(), Ok(Progress::Continue(target)));
        assert_eq!(snapshot(&execution), expected);
        if target != State::Rcdata {
            stable_terminal(&mut execution, Err(UnimplementedState { state: target }));
            // Feeding after a terminal error cannot clear it. Never reconstruct/resume.
            execution
                .context_mut()
                .input
                .push(points("more input"))
                .unwrap();
            execution.context_mut().input.finish();
            let before = snapshot(&execution);
            assert_eq!(execution.step(), Err(UnimplementedState { state: target }));
            assert_eq!(snapshot(&execution), before);
        } else {
            assert_eq!(execution.step(), Ok(Progress::Continue(State::Rcdata)));
            assert_eq!(
                execution.context().tokens.last(),
                Some(&Token::Character('X'.into()))
            );
        }
    }
}

#[test]
fn empty_open_input_and_partial_name_suspend_but_eof_falls_back_then_is_reconsumed() {
    for prefix in ["", "Ti", "TiTlE"] {
        let mut execution = entry(
            &points(prefix),
            seeded(&[], &[], Some(points("title"))),
            false,
        );
        for _ in prefix.chars() {
            assert_eq!(
                execution.step(),
                Ok(Progress::Continue(State::RcdataEndTagName))
            );
        }
        let before = snapshot(&execution);
        for _ in 0..3 {
            execution.context_mut().input.push([]).unwrap();
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&execution), before);
        }
        execution.context_mut().input.finish();
        let mut expected = before;
        expected.current = Some((
            Input::Eof,
            Position {
                line: 1,
                column: prefix.len() + 1,
            },
        ));
        expected.state = State::Rcdata;
        expected.tokens.extend(literal(&points(prefix)));
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Rcdata)));
        assert_eq!(snapshot(&execution), expected);
        // The source emits no EOF and reports no eof-in-tag, even for a match.
        assert_eq!(execution.step(), Ok(Progress::Finished));
        expected.tokens.push(Token::Eof);
        assert_eq!(snapshot(&execution), expected);
        stable_terminal(&mut execution, Ok(Progress::Finished));
    }
}

#[test]
fn matching_delimiters_hand_off_to_existing_tag_support_not_rcdata() {
    for (suffix, target, self_closing, attributes, diagnostics) in [
        (">", State::Data, false, vec![], vec![]),
        (
            "/>",
            State::SelfClosingStartTag,
            true,
            vec![],
            vec![error("end-tag-with-trailing-solidus", 1, 2)],
        ),
        (
            " a='v'>",
            State::BeforeAttributeName,
            false,
            vec![Attribute {
                name: points("a"),
                value: points("v"),
            }],
            vec![error("end-tag-with-attributes", 1, 7)],
        ),
    ] {
        let context = seeded(&points("title"), &points("TiTlE"), Some(points("title")));
        let start = context.pending_start_tag.clone();
        let mut execution = entry(&points(suffix), context, true);
        assert_eq!(execution.step(), Ok(Progress::Continue(target)));
        if target != State::Data {
            assert_eq!(execution.context().tokens.len(), 1);
            assert!(
                !execution
                    .context()
                    .pending_end_tag
                    .as_ref()
                    .unwrap()
                    .self_closing
            );
            assert!(
                execution
                    .context()
                    .pending_end_tag
                    .as_ref()
                    .unwrap()
                    .attributes
                    .is_empty()
            );
        }
        for _ in 0..30 {
            if execution.step() == Ok(Progress::Finished) {
                break;
            }
        }
        assert_eq!(
            execution.context().tokens,
            vec![
                Token::Character('!'.into()),
                Token::EndTag {
                    name: points("title"),
                    attributes,
                    self_closing
                },
                Token::Eof
            ]
        );
        assert_eq!(
            execution.context().errors,
            [vec![error("prior", 4, 5)], diagnostics].concat()
        );
        assert_eq!(execution.context().pending_start_tag, start);
        assert!(execution.context().pending_end_tag.is_none());
        assert_eq!(execution.context().current_tag, None);
        assert_eq!(execution.context().last_start_tag, Some(points("title")));
        assert_eq!(execution.context().temporary_buffer, points("TiTlE"));
        stable_terminal(&mut execution, Ok(Progress::Finished));
    }
    // Emission uses existing shared support, retaining attributes/flag and dropping
    // only the selected tag (including its detached duplicate).
    let mut context = seeded(&points("title"), &points("TiTlE"), Some(points("title")));
    let tag = context.pending_end_tag.as_mut().unwrap();
    tag.self_closing = true;
    tag.attributes = vec![Attribute {
        name: points("x"),
        value: vec![CodePoint::new(0xd800).unwrap()],
    }];
    tag.discarded_current_attribute = Some(Attribute::default());
    let attributes = tag.attributes.clone();
    let mut execution = entry(&points(">"), context, true);
    assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
    assert_eq!(
        execution.context().tokens.last(),
        Some(&Token::EndTag {
            name: points("title"),
            attributes,
            self_closing: true
        })
    );
    assert_eq!(
        execution.context().errors,
        [
            error("prior", 4, 5),
            error("end-tag-with-attributes", 1, 1),
            error("end-tag-with-trailing-solidus", 1, 1)
        ]
    );
    assert!(execution.context().input.errors().is_empty());
}

#[test]
fn delimiter_handoff_eof_is_destination_abandonment_not_source_fallback() {
    for (delimiter, target) in [
        (' ', State::BeforeAttributeName),
        ('/', State::SelfClosingStartTag),
    ] {
        let mut execution = entry(
            &points(&delimiter.to_string()),
            seeded(&points("title"), &points("TiTlE"), Some(points("title"))),
            true,
        );
        let mut expected = snapshot(&execution);
        expected.state = target;
        expected.current = Some((
            Input::CodePoint(delimiter.into()),
            Position { line: 1, column: 1 },
        ));
        assert_eq!(execution.step(), Ok(Progress::Continue(target)));
        assert_eq!(snapshot(&execution), expected);
        // No source EOF, emission, fallback characters, flag mutation, or errors.
        if delimiter == ' ' {
            assert_eq!(
                execution.step(),
                Ok(Progress::Continue(State::AfterAttributeName))
            );
            assert_eq!(execution.context().errors, expected.errors);
            assert_eq!(execution.context().pending_end_tag, expected.end);
        }
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(
            execution.context().errors,
            [error("prior", 4, 5), error("eof-in-tag", 1, 2)]
        );
        assert_eq!(
            execution.context().tokens,
            [Token::Character('!'.into()), Token::Eof]
        );
        assert!(execution.context().pending_end_tag.is_none());
        assert!(execution.context().current_tag.is_none());
        assert_eq!(execution.context().temporary_buffer, expected.buffer);
        assert_eq!(execution.context().pending_start_tag, expected.start);
        assert_eq!(execution.context().last_start_tag, expected.last);
        stable_terminal(&mut execution, Ok(Progress::Finished));
    }
}

#[test]
fn matching_tag_emission_errors_use_normalized_destination_position() {
    let mut execution = entry(
        &points("TiTlE\r\n a='v'/>"),
        seeded(&[], &[], Some(points("title"))),
        true,
    );
    for _ in 0..5 {
        assert_eq!(
            execution.step(),
            Ok(Progress::Continue(State::RcdataEndTagName))
        );
    }
    assert_eq!(
        execution.step(),
        Ok(Progress::Continue(State::BeforeAttributeName))
    );
    assert_eq!(
        execution.context().input.current(),
        Some((
            Input::CodePoint('\n'.into()),
            Position { line: 1, column: 6 }
        ))
    );
    assert_eq!(execution.context().errors, [error("prior", 4, 5)]);
    assert_eq!(execution.context().tokens, [Token::Character('!'.into())]);
    for _ in 0..30 {
        if execution.step() == Ok(Progress::Finished) {
            break;
        }
    }
    assert_eq!(
        execution.context().tokens,
        [
            Token::Character('!'.into()),
            Token::EndTag {
                name: points("title"),
                attributes: vec![Attribute {
                    name: points("a"),
                    value: points("v")
                }],
                self_closing: true,
            },
            Token::Eof
        ]
    );
    assert_eq!(
        execution.context().errors,
        [
            error("prior", 4, 5),
            error("end-tag-with-attributes", 2, 8),
            error("end-tag-with-trailing-solidus", 2, 8)
        ]
    );
    assert_eq!(execution.context().temporary_buffer, points("TiTlE"));
    assert!(execution.context().input.errors().is_empty());
    assert_eq!(
        execution.context().input.current(),
        Some((Input::Eof, Position { line: 2, column: 9 }))
    );
    stable_terminal(&mut execution, Ok(Progress::Finished));
}

#[test]
fn normalized_positions_and_no_duplicate_preprocessing_on_reconsumption() {
    let mut input = points("\r\n");
    input.extend([
        CodePoint::new(0xd800).unwrap(),
        '\0'.into(),
        '\r'.into(),
        '\n'.into(),
    ]);
    let mut execution = entry(&input, seeded(&points("x"), &points("X"), None), true);
    assert_eq!(execution.step(), Ok(Progress::Continue(State::Rcdata)));
    assert_eq!(
        execution.context().input.current(),
        Some((
            Input::CodePoint('\n'.into()),
            Position { line: 1, column: 1 }
        ))
    );
    for _ in 0..5 {
        assert!(execution.step().is_ok());
    }
    assert_eq!(
        execution.context().tokens,
        [
            vec![Token::Character('!'.into())],
            literal(&points("X")),
            vec![
                Token::Character('\n'.into()),
                Token::Character(CodePoint::new(0xd800).unwrap()),
                Token::Character('\u{fffd}'.into()),
                Token::Character('\n'.into()),
                Token::Eof
            ]
        ]
        .concat()
    );
    assert_eq!(
        execution.context().input.errors(),
        [error("surrogate-in-input-stream", 2, 1)]
    );
    assert_eq!(
        execution.context().errors,
        [
            error("prior", 4, 5),
            error("unexpected-null-character", 2, 2)
        ]
    );
    assert_eq!(
        execution.context().input.current(),
        Some((Input::Eof, Position { line: 3, column: 1 }))
    );
    stable_terminal(&mut execution, Ok(Progress::Finished));
}

// Compare every non-suspension source step, not merely final output. Stop at the
// first stub and never reconstruct/resume a blocked execution.
type Trace = Vec<(Result<Progress, UnimplementedState>, Snapshot)>;
fn drain(execution: &mut Execution, trace: &mut Trace, bound: usize) -> bool {
    for _ in 0..bound {
        let before = snapshot(execution);
        let result = execution.step();
        if result == Ok(Progress::NeedMoreInput) {
            assert_eq!(snapshot(execution), before);
            for _ in 0..3 {
                assert_eq!(execution.step(), result);
                assert_eq!(snapshot(execution), before);
            }
            return false;
        }
        trace.push((result, snapshot(execution)));
        if !matches!(result, Ok(Progress::Continue(_))) {
            stable_terminal(execution, result);
            return true;
        }
    }
    panic!("did not consume, suspend or terminate");
}

#[test]
fn every_two_chunk_split_and_single_point_chunking_preserve_source_step_traces() {
    let mut samples = vec![
        vec![],
        points("TiTlE"),
        points("TiTlE>tail"),
        points("TiTlEx>"),
        points("TiTlE a='v'>tail"),
        points("TiTlE/>tail"),
        points("TiTlE/"),
        points("TiTlE/x>"),
        points("TiTlE / >"),
        points("TiTlE "),
        points("TiTlE\r\n A=\"v\">"),
        points("TiTlE\r\r\n>"),
        points("Other\r\n\0\r\ré\r"),
        points("TiTlE\0>"),
        points("TiTlE&tail"),
        points("TiTlE<tail"),
        points("TiTlE a='&tail"),
        points("TiTlEÉ>"),
    ];
    samples.push(
        [
            points("Ti"),
            vec![
                CodePoint::new(0xd800).unwrap(),
                CodePoint::new(0xfdd0).unwrap(),
                '\u{b}'.into(),
                '\r'.into(),
            ],
        ]
        .concat(),
    );
    for last in [None, Some(points("title"))] {
        for input in &samples {
            let run = |chunks: Vec<&[CodePoint]>, closed: bool| {
                let mut execution = entry(&[], seeded(&[], &[], last.clone()), false);
                let mut trace = Vec::new();
                if closed {
                    execution
                        .context_mut()
                        .input
                        .push(input.iter().copied())
                        .unwrap();
                    execution.context_mut().input.finish();
                } else {
                    for chunk in chunks {
                        execution
                            .context_mut()
                            .input
                            .push(chunk.iter().copied())
                            .unwrap();
                        if drain(&mut execution, &mut trace, input.len() * 4 + 20) {
                            return trace;
                        }
                        execution.context_mut().input.push([]).unwrap();
                        assert!(!drain(&mut execution, &mut trace, 1));
                    }
                    execution.context_mut().input.finish();
                }
                assert!(drain(&mut execution, &mut trace, input.len() * 4 + 20));
                trace
            };
            let expected = run(vec![], true);
            for split in 0..=input.len() {
                assert_eq!(
                    run(vec![&input[..split], &input[split..]], false),
                    expected,
                    "split {split} of {input:?}, last {last:?}"
                );
            }
            assert_eq!(run(input.chunks(1).collect(), false), expected);
        }
    }
}

#[test]
fn adjacent_entry_states_remain_effect_free_terminal_stubs() {
    for state in [
        State::RcdataLessThanSign,
        State::RcdataEndTagOpen,
        State::RawtextLessThanSign,
        State::RawtextEndTagOpen,
        State::RawtextEndTagName,
    ] {
        for finished in [false, true] {
            let mut context = seeded(&[], &[], Some(points("title")));
            context.input.push(points("TiTlE>")).unwrap();
            if finished {
                context.input.finish();
            }
            let mut execution = Execution::new(state, context);
            stable_terminal(&mut execution, Err(UnimplementedState { state }));
            assert_eq!(execution.context().input.current(), None);
        }
    }
}
