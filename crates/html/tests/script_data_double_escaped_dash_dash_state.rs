//! Pinned #script-data-double-escaped-dash-dash-state, source 144057–144090.
//! See docs/tokenizer-script-data-double-escaped-dash-dash.md. Local, not upstream runs.

use neptune_html::tokenizer::{
    Attribute, CodePoint, Context, Execution, Input, ParseError, PendingComment, PendingEndTag,
    PendingStartTag, Position, Progress, State, TagKind, Token, UnimplementedState,
};

const SOURCE: State = State::ScriptDataDoubleEscapedDashDash;

fn points(text: &str) -> Vec<CodePoint> {
    text.chars().map(Into::into).collect()
}

fn error(code: &'static str, line: usize, column: usize) -> ParseError {
    ParseError {
        code,
        position: Position { line, column },
    }
}

fn seeded() -> Context {
    let attribute = Attribute {
        name: points("A"),
        value: vec![CodePoint::new(0xd800).unwrap()],
    };
    let discarded = Attribute {
        name: points("A"),
        value: points("discarded"),
    };
    Context {
        tokens: vec![Token::Character('p'.into())],
        errors: vec![error("prior-diagnostic", 1, 1)],
        return_state: Some(State::Rcdata),
        last_start_tag: Some(points("script")),
        current_tag: Some(TagKind::End),
        pending_start_tag: Some(PendingStartTag {
            name: points("StArT"),
            self_closing: true,
            attributes: vec![attribute.clone()],
            discarded_current_attribute: Some(discarded.clone()),
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("EnD"),
            self_closing: true,
            attributes: vec![attribute],
            discarded_current_attribute: Some(discarded),
        }),
        pending_comment: Some(PendingComment {
            data: points("comment"),
        }),
        temporary_buffer: points("ScRiPt\0\u{fffd}"),
        ..Default::default()
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Retained {
    return_state: Option<State>,
    last_start_tag: Option<Vec<CodePoint>>,
    current_tag: Option<TagKind>,
    start: Option<PendingStartTag>,
    end: Option<PendingEndTag>,
    comment: Option<PendingComment>,
    buffer: Vec<CodePoint>,
}

fn retained(context: &Context) -> Retained {
    Retained {
        return_state: context.return_state,
        last_start_tag: context.last_start_tag.clone(),
        current_tag: context.current_tag,
        start: context.pending_start_tag.clone(),
        end: context.pending_end_tag.clone(),
        comment: context.pending_comment.clone(),
        buffer: context.temporary_buffer.clone(),
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Snapshot {
    state: State,
    current: Option<(Input, Position)>,
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
    input_errors: Vec<ParseError>,
    retained: Retained,
}

fn snapshot(execution: &Execution) -> Snapshot {
    let context = execution.context();
    Snapshot {
        state: execution.state(),
        current: context.input.current(),
        tokens: context.tokens.clone(),
        errors: context.errors.clone(),
        input_errors: context.input.errors().to_vec(),
        retained: retained(context),
    }
}

#[test]
fn every_ascii_branch_consumes_once_preserving_context_and_untouched_tail() {
    for value in 0..=0x7f {
        for finished in [false, true] {
            let point = CodePoint::new(value).unwrap();
            let normalized = if value == 0x0d { '\n'.into() } else { point };
            let mut context = seeded();
            context
                .input
                .push([point, 'z'.into(), '\0'.into()])
                .unwrap();
            if finished {
                context.input.finish();
            }
            let mut execution = Execution::new(SOURCE, context);
            let mut expected = snapshot(&execution);
            let (target, emitted) = match value {
                0x2d => (SOURCE, point),
                0x3c => (State::ScriptDataDoubleEscapedLessThanSign, point),
                0x3e => (State::ScriptData, point),
                0 => (State::ScriptDataDoubleEscaped, '\u{fffd}'.into()),
                _ => (State::ScriptDataDoubleEscaped, normalized),
            };
            expected.state = target;
            expected.current = Some((
                Input::CodePoint(normalized),
                Position { line: 1, column: 1 },
            ));
            expected.tokens.push(Token::Character(emitted));
            if value == 0 {
                expected
                    .errors
                    .push(error("unexpected-null-character", 1, 1));
            }
            if matches!(value, 1..=8 | 0x0b | 0x0e..=0x1f | 0x7f) {
                expected
                    .input_errors
                    .push(error("control-character-in-input-stream", 1, 1));
            }
            assert_eq!(
                execution.step(),
                Ok(Progress::Continue(target)),
                "ASCII {value:x}"
            );
            assert_eq!(snapshot(&execution), expected, "ASCII {value:x}");
            // Source-only probe, before any destination execution or terminal block.
            // No branch schedules reconsumption or eagerly processes the tail.
            assert_eq!(
                execution.context_mut().input.consume(),
                Input::CodePoint('z'.into())
            );
            assert_eq!(execution.context().errors, expected.errors);
            assert_eq!(execution.context().input.errors(), expected.input_errors);
        }
    }
}

#[test]
fn unicode_non_scalars_and_controls_are_literal_with_exact_preprocessing_errors() {
    for (value, diagnostic) in [
        (0x85, Some("control-character-in-input-stream")),
        (0x9f, Some("control-character-in-input-stream")),
        (0xa0, None),
        (0xc9, None),
        (0x2014, None),
        (0xd800, Some("surrogate-in-input-stream")),
        (0xdfff, Some("surrogate-in-input-stream")),
        (0xfdd0, Some("noncharacter-in-input-stream")),
        (0xffff, Some("noncharacter-in-input-stream")),
        (0x1fffe, Some("noncharacter-in-input-stream")),
        (0x10ffff, Some("noncharacter-in-input-stream")),
        (0xfeff, None),
        (0xfffd, None),
        (0x1f600, None),
    ] {
        let point = CodePoint::new(value).unwrap();
        let mut context = seeded();
        context.input.push(points("--")).unwrap();
        context.input.push([point, '\0'.into()]).unwrap();
        context.input.finish();
        let preserved = retained(&context);
        let mut execution = Execution::new(SOURCE, context);
        for _ in 0..2 {
            assert_eq!(execution.step(), Ok(Progress::Continue(SOURCE)));
        }
        assert_eq!(
            execution.step(),
            Ok(Progress::Continue(State::ScriptDataDoubleEscaped))
        );
        assert_eq!(
            execution.context().tokens,
            [
                Token::Character('p'.into()),
                Token::Character('-'.into()),
                Token::Character('-'.into()),
                Token::Character(point),
            ]
        );
        assert_eq!(
            execution.context().errors,
            [error("prior-diagnostic", 1, 1)]
        );
        let expected: Vec<_> = diagnostic
            .into_iter()
            .map(|code| error(code, 1, 3))
            .collect();
        assert_eq!(execution.context().input.errors(), expected);
        assert_eq!(
            execution.context().input.current(),
            Some((Input::CodePoint(point), Position { line: 1, column: 3 }))
        );
        assert_eq!(retained(execution.context()), preserved);
        let boundary = snapshot(&execution);
        for _ in 0..3 {
            assert_eq!(
                execution.step(),
                Err(UnimplementedState {
                    state: State::ScriptDataDoubleEscaped
                })
            );
            assert_eq!(snapshot(&execution), boundary);
        }
    }
}

#[test]
fn less_than_null_and_literal_effects_precede_terminal_destination_errors() {
    for (text, target, emitted, diagnostic) in [
        (
            "--<\0tail",
            State::ScriptDataDoubleEscapedLessThanSign,
            '<',
            None,
        ),
        (
            "--\0\0tail",
            State::ScriptDataDoubleEscaped,
            '\u{fffd}',
            Some("unexpected-null-character"),
        ),
        ("--&amp;", State::ScriptDataDoubleEscaped, '&', None),
        ("--A\0tail", State::ScriptDataDoubleEscaped, 'A', None),
    ] {
        for populated in [false, true] {
            let mut context = if populated {
                seeded()
            } else {
                Context::default()
            };
            context.input.push(points(text)).unwrap();
            context.input.finish();
            let mut execution = Execution::new(SOURCE, context);
            let mut expected = snapshot(&execution);
            for column in 1..=2 {
                expected.current =
                    Some((Input::CodePoint('-'.into()), Position { line: 1, column }));
                expected.tokens.push(Token::Character('-'.into()));
                assert_eq!(execution.step(), Ok(Progress::Continue(SOURCE)));
                assert_eq!(snapshot(&execution), expected);
            }
            expected.state = target;
            expected.current = Some((
                Input::CodePoint(text.chars().nth(2).unwrap().into()),
                Position { line: 1, column: 3 },
            ));
            expected.tokens.push(Token::Character(emitted.into()));
            if let Some(code) = diagnostic {
                expected.errors.push(error(code, 1, 3));
            }
            assert_eq!(execution.step(), Ok(Progress::Continue(target)));
            assert_eq!(snapshot(&execution), expected);
            for _ in 0..3 {
                assert_eq!(execution.step(), Err(UnimplementedState { state: target }));
                assert_eq!(snapshot(&execution), expected);
            }
        }
    }
}

#[test]
fn open_exhaustion_hyphen_runs_and_explicit_eof_are_distinct_and_idempotent() {
    for text in ["", "-", "---"] {
        for populated in [false, true] {
            let mut context = if populated {
                seeded()
            } else {
                Context::default()
            };
            context.input.push(points(text)).unwrap();
            let mut execution = Execution::new(SOURCE, context);
            let mut expected = snapshot(&execution);
            for column in 1..=text.len() {
                expected.current =
                    Some((Input::CodePoint('-'.into()), Position { line: 1, column }));
                expected.tokens.push(Token::Character('-'.into()));
                assert_eq!(execution.step(), Ok(Progress::Continue(SOURCE)));
                assert_eq!(snapshot(&execution), expected);
            }
            for _ in 0..3 {
                execution.context_mut().input.push([]).unwrap();
                assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                assert_eq!(snapshot(&execution), expected);
            }
            execution.context_mut().input.finish();
            expected.current = Some((
                Input::Eof,
                Position {
                    line: 1,
                    column: text.len() + 1,
                },
            ));
            expected.errors.push(error(
                "eof-in-script-html-comment-like-text",
                1,
                text.len() + 1,
            ));
            expected.tokens.push(Token::Eof);
            for _ in 0..3 {
                assert_eq!(execution.step(), Ok(Progress::Finished));
                assert_eq!(snapshot(&execution), expected);
            }
        }
    }
}

#[test]
fn explicitly_reconsumed_entry_and_eof_keep_normalized_positions_without_duplicate_errors() {
    for value in [0, 0xd800, 0x3c, 0x2d] {
        let point = CodePoint::new(value).unwrap();
        let mut context = seeded();
        context.input.push(points("é😀\r\n")).unwrap();
        context.input.push([point, 'z'.into()]).unwrap();
        context.input.finish();
        for prefix in points("é😀\n") {
            assert_eq!(context.input.consume(), Input::CodePoint(prefix));
        }
        assert_eq!(context.input.consume(), Input::CodePoint(point));
        context.input.reconsume().unwrap();
        let mut execution = Execution::new(SOURCE, context);
        let target = match value {
            0x2d => SOURCE,
            0x3c => State::ScriptDataDoubleEscapedLessThanSign,
            _ => State::ScriptDataDoubleEscaped,
        };
        let mut expected = snapshot(&execution);
        expected.state = target;
        expected.tokens.push(Token::Character(if value == 0 {
            '\u{fffd}'.into()
        } else {
            point
        }));
        if value == 0 {
            expected
                .errors
                .push(error("unexpected-null-character", 2, 1));
        }
        assert_eq!(execution.step(), Ok(Progress::Continue(target)));
        assert_eq!(snapshot(&execution), expected);
        assert_eq!(
            execution.context().input.position(),
            Position { line: 2, column: 1 }
        );
        let input_errors = if value == 0xd800 {
            vec![error("surrogate-in-input-stream", 2, 1)]
        } else {
            vec![]
        };
        assert_eq!(execution.context().input.errors(), input_errors);
        assert_eq!(
            execution.context_mut().input.consume(),
            Input::CodePoint('z'.into())
        );
    }

    let mut context = seeded();
    context.input.push(points("é😀\r\n\r")).unwrap();
    context.input.finish();
    for point in points("é😀\n\n") {
        assert_eq!(context.input.consume(), Input::CodePoint(point));
    }
    assert_eq!(context.input.consume(), Input::Eof);
    context.input.reconsume().unwrap();
    let mut execution = Execution::new(SOURCE, context);
    let mut expected = snapshot(&execution);
    expected
        .errors
        .push(error("eof-in-script-html-comment-like-text", 3, 1));
    expected.tokens.push(Token::Eof);
    for _ in 0..3 {
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&execution), expected);
    }
}

#[test]
fn greater_than_returns_to_real_script_data_and_predecessor_entry_is_still_blocked() {
    let mut context = seeded();
    context.input.push(points("-->\r\n\0\0")).unwrap();
    context
        .input
        .push([CodePoint::new(0xd800).unwrap()])
        .unwrap();
    context.input.finish();
    let preserved = retained(&context);
    let mut execution = Execution::new(SOURCE, context);
    for _ in 0..2 {
        assert_eq!(execution.step(), Ok(Progress::Continue(SOURCE)));
    }
    for _ in 0..5 {
        assert_eq!(execution.step(), Ok(Progress::Continue(State::ScriptData)));
    }
    assert_eq!(execution.step(), Ok(Progress::Finished));
    let expected: Vec<_> = points("p-->\n\u{fffd}\u{fffd}")
        .into_iter()
        .chain([CodePoint::new(0xd800).unwrap()])
        .map(Token::Character)
        .chain([Token::Eof])
        .collect();
    assert_eq!(execution.context().tokens, expected);
    assert_eq!(
        execution.context().errors,
        [
            error("prior-diagnostic", 1, 1),
            error("unexpected-null-character", 2, 1),
            error("unexpected-null-character", 2, 2)
        ]
    );
    assert_eq!(
        execution.context().input.errors(),
        [error("surrogate-in-input-stream", 2, 3)]
    );
    assert_eq!(
        execution.context().input.current(),
        Some((Input::Eof, Position { line: 2, column: 4 }))
    );
    assert_eq!(retained(execution.context()), preserved);

    // No implemented predecessor can enter the assigned state on this base.
    // Exercise the real available chain, without reconstructing after its stub.
    let mut context = Context::default();
    context.input.push(points("<!--<script>-->")).unwrap();
    context.input.finish();
    let mut execution = Execution::new(State::ScriptData, context);
    for target in [
        State::ScriptDataLessThanSign,
        State::ScriptDataEscapeStart,
        State::ScriptDataEscapeStartDash,
        State::ScriptDataEscapedDashDash,
        State::ScriptDataEscapedLessThanSign,
    ] {
        assert_eq!(execution.step(), Ok(Progress::Continue(target)));
    }
    assert_eq!(
        execution.context().tokens,
        points("<!--")
            .into_iter()
            .map(Token::Character)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        execution.context().input.current(),
        Some((
            Input::CodePoint('<'.into()),
            Position { line: 1, column: 5 }
        ))
    );
    assert!(execution.context().errors.is_empty());
    let boundary = snapshot(&execution);
    for _ in 0..3 {
        assert_eq!(
            execution.step(),
            Err(UnimplementedState {
                state: State::ScriptDataEscapedLessThanSign
            })
        );
        assert_eq!(snapshot(&execution), boundary);
    }
}

type Trace = Vec<(Result<Progress, UnimplementedState>, Snapshot)>;

fn drain(execution: &mut Execution, trace: &mut Trace, bound: usize) -> bool {
    for _ in 0..bound {
        let before = snapshot(execution);
        let result = execution.step();
        if result == Ok(Progress::NeedMoreInput) {
            assert_eq!(snapshot(execution), before);
            assert_eq!(execution.step(), result);
            assert_eq!(snapshot(execution), before);
            return false;
        }
        let terminal = !matches!(result, Ok(Progress::Continue(_)));
        trace.push((result, snapshot(execution)));
        if terminal {
            let done = snapshot(execution);
            assert_eq!(execution.step(), result);
            assert_eq!(snapshot(execution), done);
            return true;
        }
    }
    panic!("state failed to consume or terminate");
}

#[test]
fn every_two_chunk_split_and_single_point_stream_matches_closed_source_steps() {
    let mut inputs = vec![
        vec![],
        points("-"),
        points("---"),
        points("--<\0tail"),
        points("--\0\0tail"),
        points("--&amp;"),
        points("--A"),
        points("--\r\n"),
        points("\r"),
        points("\r\r"),
        points("--\t"),
        points("-->"),
        points("-->\r\n\0\0é😀\r\r\nb\r"),
        points("--><!tail"),
        points("--><!--<tail"),
        points("--></script>&"),
        points("--></SCRIPT>\r\n\0"),
        points("-->\r\n\r\r\n\0\r"),
    ];
    for value in [0xd800, 0xdfff, 0xfdd0, 0x1fffe, 0x10ffff, 0x85, 0xfffd] {
        let point = CodePoint::new(value).unwrap();
        inputs.push(vec!['-'.into(), point, '\0'.into()]);
        inputs.push(vec![
            '>'.into(),
            '\r'.into(),
            '\n'.into(),
            point,
            '\0'.into(),
        ]);
    }
    for input in inputs {
        let run = |chunks: Vec<&[CodePoint]>, closed: bool| {
            let mut execution = Execution::new(SOURCE, seeded());
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
                    if drain(&mut execution, &mut trace, input.len() * 3 + 8) {
                        // No feeding, reconstruction or resumption after a terminal stub.
                        return trace;
                    }
                    execution.context_mut().input.push([]).unwrap();
                    assert!(!drain(&mut execution, &mut trace, 1));
                }
                execution.context_mut().input.finish();
            }
            assert!(drain(&mut execution, &mut trace, input.len() * 3 + 8));
            trace
        };
        let expected = run(vec![], true);
        for split in 0..=input.len() {
            assert_eq!(
                run(vec![&input[..split], &input[split..]], false),
                expected,
                "split {split} of {input:?}"
            );
        }
        assert_eq!(run(input.chunks(1).collect(), false), expected);
    }
}
