//! Branch evidence for pinned HTML "Before attribute name state":
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>
//! Local source-step tests, not additional upstream conformance runs.

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

fn seeded(kind: TagKind) -> Context {
    let attribute = Attribute {
        name: points("prior"),
        value: vec![CodePoint::new(0xd800).unwrap(), 'v'.into()],
    };
    Context {
        current_tag: Some(kind),
        pending_start_tag: Some(PendingStartTag {
            name: points("start"),
            attributes: vec![attribute.clone()],
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("end"),
            attributes: vec![attribute],
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

impl Snapshot {
    fn attributes(&mut self, kind: TagKind) -> &mut Vec<Attribute> {
        match kind {
            TagKind::Start => &mut self.start.as_mut().unwrap().attributes,
            TagKind::End => &mut self.end.as_mut().unwrap().attributes,
        }
    }
}

fn snapshot(execution: &Execution) -> Snapshot {
    let context = execution.context();
    Snapshot {
        state: execution.state(),
        current: context.input.current(),
        tokens: context.tokens.clone(),
        errors: context.errors.clone(),
        input_errors: context.input.errors().to_vec(),
        kind: context.current_tag,
        start: context.pending_start_tag.clone(),
        end: context.pending_end_tag.clone(),
        comment: context.pending_comment.clone(),
        last_start_tag: context.last_start_tag.clone(),
        return_state: context.return_state,
        buffer: context.temporary_buffer.clone(),
    }
}

fn assert_blocked(execution: &mut Execution, target: State, expected: &Snapshot) {
    for _ in 0..2 {
        assert_eq!(execution.step(), Err(UnimplementedState { state: target }));
        assert_eq!(&snapshot(execution), expected);
    }
}

#[test]
fn before_attribute_name_ignores_only_exact_whitespace_without_attribute_creation_or_reconsumption()
{
    for kind in [TagKind::Start, TagKind::End] {
        for (text, consumed) in [
            ("\t", '\t'),
            ("\n", '\n'),
            ("\u{c}", '\u{c}'),
            (" ", ' '),
            ("\r", '\n'),
            ("\r\n", '\n'),
        ] {
            for closed in [false, true] {
                let mut context = seeded(kind);
                context.input.push(points(&format!("{text}Z"))).unwrap();
                if closed {
                    context.input.finish();
                }
                let mut execution = Execution::new(State::BeforeAttributeName, context);
                let mut expected = snapshot(&execution);
                expected.current = Some((
                    Input::CodePoint(consumed.into()),
                    Position { line: 1, column: 1 },
                ));
                assert_eq!(
                    execution.step(),
                    Ok(Progress::Continue(State::BeforeAttributeName))
                );
                assert_eq!(snapshot(&execution), expected);
                // Cursor inspection proves consumption, not a downstream algorithm.
                assert_eq!(
                    execution.context_mut().input.consume(),
                    Input::CodePoint('Z'.into())
                );
                assert_eq!(
                    execution.context().input.position(),
                    if consumed == '\n' {
                        Position { line: 2, column: 1 }
                    } else {
                        Position { line: 1, column: 2 }
                    }
                );
            }
        }
    }
}

#[test]
fn before_attribute_name_equals_diagnoses_and_appends_named_empty_attribute_then_switches() {
    for kind in [TagKind::Start, TagKind::End] {
        for closed in [false, true] {
            let mut context = seeded(kind);
            context.input.push(points("\r\n \t=Z")).unwrap();
            if closed {
                context.input.finish();
            }
            let mut execution = Execution::new(State::BeforeAttributeName, context);
            let original = snapshot(&execution);
            for _ in 0..3 {
                assert_eq!(
                    execution.step(),
                    Ok(Progress::Continue(State::BeforeAttributeName))
                );
            }
            let mut expected = original;
            expected.state = State::AttributeName;
            expected.current = Some((
                Input::CodePoint('='.into()),
                Position { line: 2, column: 3 },
            ));
            expected
                .errors
                .push(error("unexpected-equals-sign-before-attribute-name", 2, 3));
            expected.attributes(kind).push(Attribute {
                name: points("="),
                value: vec![],
            });
            assert_eq!(
                execution.step(),
                Ok(Progress::Continue(State::AttributeName))
            );
            assert_eq!(snapshot(&execution), expected);
            assert_blocked(&mut execution, State::AttributeName, &expected);
            // Inspection only; execution cannot resume after the stub. '=' was
            // NOT scheduled for reconsumption; the next point is untouched Z.
            assert_eq!(
                execution.context_mut().input.consume(),
                Input::CodePoint('Z'.into())
            );
            assert_eq!(
                execution.context().input.position(),
                Position { line: 2, column: 4 }
            );
            let inspected = snapshot(&execution);
            assert_blocked(&mut execution, State::AttributeName, &inspected);
        }
    }
}

#[test]
fn before_attribute_name_anything_else_creates_empty_attribute_and_reconsumes_unchanged_point() {
    // Exhaust the ASCII branches, including NUL, VT, quotes and uppercase.
    // Non-ASCII whitespace/letters and actual non-scalars must not be classified
    // with Rust char whitespace, replaced, folded or appended in this state.
    let values = (0..=0x7f)
        .filter(|value| !matches!(value, 0x09 | 0x0a | 0x0c | 0x0d | 0x20 | 0x2f | 0x3d | 0x3e))
        .chain([
            0x85, 0xa0, 0x1680, 0x2003, 0x2028, 0x3000, 0xc9, 0x130, 0x1f600, 0xd800, 0xdfff,
            0xfdd0, 0x10ffff,
        ]);
    for value in values {
        let point = CodePoint::new(value).unwrap();
        for kind in [TagKind::Start, TagKind::End] {
            for closed in [false, true] {
                let mut context = seeded(kind);
                context.input.push([point, 'Z'.into()]).unwrap();
                if closed {
                    context.input.finish();
                }
                let mut execution = Execution::new(State::BeforeAttributeName, context);
                let mut expected = snapshot(&execution);
                expected.state = State::AttributeName;
                expected.current = Some((Input::CodePoint(point), Position { line: 1, column: 1 }));
                expected.attributes(kind).push(Attribute::default());
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
                    execution.step(),
                    Ok(Progress::Continue(State::AttributeName)),
                    "{value:x}"
                );
                assert_eq!(snapshot(&execution), expected, "{value:x}");
                assert_blocked(&mut execution, State::AttributeName, &expected);
                assert_eq!(
                    execution.context_mut().input.consume(),
                    Input::CodePoint(point)
                );
                assert_eq!(snapshot(&execution), expected); // same position; no duplicate diagnostics
                assert_eq!(
                    execution.context_mut().input.consume(),
                    Input::CodePoint('Z'.into())
                );
                assert_eq!(
                    execution.context().input.position(),
                    Position { line: 1, column: 2 }
                );
                let inspected = snapshot(&execution);
                assert_blocked(&mut execution, State::AttributeName, &inspected);
            }
        }
    }
}

#[test]
fn before_attribute_name_solidus_greater_than_and_eof_reconsume_in_after_attribute_name() {
    for kind in [TagKind::Start, TagKind::End] {
        for text in ["/Z", ">Z", ""] {
            for closed in [false, true] {
                if text.is_empty() && !closed {
                    continue;
                }
                let mut context = seeded(kind);
                context.input.push(points(text)).unwrap();
                if closed {
                    context.input.finish();
                }
                let mut execution = Execution::new(State::BeforeAttributeName, context);
                let mut expected = snapshot(&execution);
                let consumed = text
                    .chars()
                    .next()
                    .map_or(Input::Eof, |c| Input::CodePoint(c.into()));
                expected.state = State::AfterAttributeName;
                expected.current = Some((consumed, Position { line: 1, column: 1 }));
                assert_eq!(
                    execution.step(),
                    Ok(Progress::Continue(State::AfterAttributeName))
                );
                assert_eq!(snapshot(&execution), expected); // no creation, emission or EOF error
                assert_blocked(&mut execution, State::AfterAttributeName, &expected);
                assert_eq!(execution.context_mut().input.consume(), consumed);
                assert_eq!(snapshot(&execution), expected);
                if consumed != Input::Eof {
                    assert_eq!(
                        execution.context_mut().input.consume(),
                        Input::CodePoint('Z'.into())
                    );
                }
            }
        }
    }
}

#[test]
fn before_attribute_name_open_exhaustion_suspends_until_explicit_eof_without_source_eof_effects() {
    for kind in [TagKind::Start, TagKind::End] {
        let mut execution = Execution::new(State::BeforeAttributeName, seeded(kind));
        for text in ["", " \t\r\n\u{c} ", ""] {
            execution.context_mut().input.push(points(text)).unwrap();
            while execution.step() == Ok(Progress::Continue(State::BeforeAttributeName)) {}
            let before = snapshot(&execution);
            for _ in 0..2 {
                execution.context_mut().input.push([]).unwrap();
                assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                assert_eq!(snapshot(&execution), before);
            }
        }
        execution.context_mut().input.finish();
        assert_eq!(
            execution.step(),
            Ok(Progress::Continue(State::AfterAttributeName))
        );
        assert_eq!(
            execution.context().input.current(),
            Some((Input::Eof, Position { line: 2, column: 3 }))
        );
        let expected = snapshot(&execution);
        assert_blocked(&mut execution, State::AfterAttributeName, &expected);
        assert_eq!(execution.context_mut().input.consume(), Input::Eof);
        assert_eq!(snapshot(&execution), expected);
    }
}

#[test]
fn before_attribute_name_real_tag_name_entry_preserves_tag_and_initializes_only_on_next_step() {
    for (text, kind, column) in [
        ("<DIv \t=Z", TagKind::Start, 7),
        ("</DIv \t\0Z", TagKind::End, 8),
    ] {
        let mut context = Context::default();
        context.input.push(points(text)).unwrap();
        context.input.finish();
        let mut execution = Execution::new(State::Data, context);
        while execution.state() != State::BeforeAttributeName {
            assert!(matches!(execution.step(), Ok(Progress::Continue(_))));
        }
        let mut expected = snapshot(&execution);
        assert_eq!(expected.kind, Some(kind));
        assert!(expected.attributes(kind).is_empty());
        match kind {
            TagKind::Start => assert_eq!(expected.start.as_ref().unwrap().name, points("div")),
            TagKind::End => assert_eq!(expected.end.as_ref().unwrap().name, points("div")),
        }
        assert_eq!(
            execution.step(),
            Ok(Progress::Continue(State::BeforeAttributeName))
        ); // TAB
        assert_eq!(
            execution.step(),
            Ok(Progress::Continue(State::AttributeName))
        );
        let attribute = if kind == TagKind::Start {
            expected.errors.push(error(
                "unexpected-equals-sign-before-attribute-name",
                1,
                column,
            ));
            Attribute {
                name: points("="),
                value: vec![],
            }
        } else {
            Attribute::default()
        };
        expected.attributes(kind).push(attribute);
        expected.state = State::AttributeName;
        expected.current = Some((
            Input::CodePoint(if kind == TagKind::Start { '=' } else { '\0' }.into()),
            Position { line: 1, column },
        ));
        assert_eq!(snapshot(&execution), expected);
        assert_blocked(&mut execution, State::AttributeName, &expected);
    }
}

#[test]
fn before_attribute_name_pending_attributes_start_empty_on_creation_and_survive_unrelated_branches()
{
    for initial in [
        State::Data,
        State::Plaintext,
        State::TagOpen,
        State::EndTagOpen,
    ] {
        for text in ["", "A", "z", "!", "?", "/", ">", "0", "<", "&", "\0"] {
            let mut context = seeded(TagKind::End);
            let start = context.pending_start_tag.clone().unwrap();
            let end = context.pending_end_tag.clone().unwrap();
            context.input.push(points(text)).unwrap();
            context.input.finish();
            let mut execution = Execution::new(initial, context);
            assert!(execution.step().is_ok());
            let creates = matches!(text, "A" | "z");
            assert_eq!(
                execution
                    .context()
                    .pending_start_tag
                    .as_ref()
                    .unwrap()
                    .attributes,
                if initial == State::TagOpen && creates {
                    vec![]
                } else {
                    start.attributes
                }
            );
            assert_eq!(
                execution
                    .context()
                    .pending_end_tag
                    .as_ref()
                    .unwrap()
                    .attributes,
                if initial == State::EndTagOpen && creates {
                    vec![]
                } else {
                    end.attributes
                }
            );
        }
    }
}

// Record every real source step through the terminal stub. Suspension is not a
// completed source step; verify it and terminal calls are effect-free.
fn drain(
    execution: &mut Execution,
    trace: &mut Vec<(Result<Progress, UnimplementedState>, Snapshot)>,
    bound: usize,
) -> bool {
    for _ in 0..bound {
        let before = snapshot(execution);
        let result = execution.step();
        if result == Ok(Progress::NeedMoreInput) {
            assert_eq!(snapshot(execution), before);
            assert_eq!(execution.step(), result);
            assert_eq!(snapshot(execution), before);
            return false;
        }
        trace.push((result, snapshot(execution)));
        if !matches!(result, Ok(Progress::Continue(_))) {
            let terminal = snapshot(execution);
            assert_eq!(execution.step(), result);
            assert_eq!(snapshot(execution), terminal);
            return true;
        }
    }
    panic!("Before attribute name integration failed to consume or terminate");
}

#[test]
fn before_attribute_name_all_chunk_splits_match_closed_input_steps_and_terminal_targets() {
    let mut samples: Vec<_> = [
        "",
        " \t\n\u{c} ",
        "\r\n \t=Z",
        "\r\r\n\t\0Z",
        "/Z",
        ">Z",
        "=Z",
        "A",
        "\r",
        "<A ",
        "</A\t\r\n ",
        "<A \t=Z",
        "</A\r\n\0Z",
        "<A \"x",
        "<A />",
        "</A >",
        "<A \r\r\n",
    ]
    .map(points)
    .into();
    for value in [0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x0b, 0xa0, 0xc9] {
        let mut input = points("<A \r\n\t");
        input.push(CodePoint::new(value).unwrap());
        input.extend(points("Z"));
        samples.push(input);
    }
    for input in samples {
        for initial in [State::Data, State::BeforeAttributeName] {
            let run = |chunks: Vec<&[CodePoint]>, closed: bool| {
                let context = if initial == State::Data {
                    Context::default()
                } else {
                    seeded(TagKind::End)
                };
                let mut execution = Execution::new(initial, context);
                let mut trace = vec![];
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
                        if drain(&mut execution, &mut trace, input.len() * 3 + 5) {
                            return trace;
                        }
                        execution.context_mut().input.push([]).unwrap();
                        assert!(!drain(&mut execution, &mut trace, 1));
                    }
                    execution.context_mut().input.finish();
                }
                assert!(drain(&mut execution, &mut trace, input.len() * 3 + 5));
                trace
            };
            let expected = run(vec![], true);
            for split in 0..=input.len() {
                assert_eq!(
                    run(vec![&input[..split], &input[split..]], false),
                    expected,
                    "{initial:?}, split {split} of {input:?}"
                );
            }
            assert_eq!(run(input.chunks(1).collect(), false), expected);
        }
    }
}
