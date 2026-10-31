//! Branch evidence for the pinned HTML Standard's "RCDATA state" (#rcdata-state):
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>
//! Local state/adapter tests are not upstream conformance counts.

use neptune_html::tokenizer::{
    Attribute, CodePoint, Context, CursorError, Execution, Input, ParseError, PendingComment,
    PendingEndTag, PendingStartTag, Position, Progress, State, TagKind, Token, UnimplementedState,
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

fn seeded_context() -> Context {
    Context {
        tokens: vec![Token::Character('p'.into())],
        errors: vec![error("prior-diagnostic", 4, 5)],
        return_state: Some(State::AttributeValueDoubleQuoted),
        last_start_tag: Some(points("title")),
        current_tag: Some(TagKind::End),
        pending_start_tag: Some(PendingStartTag {
            name: points("Start\r\0"),
            self_closing: true,
            attributes: vec![Attribute {
                name: points("start-attribute"),
                value: points("value"),
            }],
            discarded_current_attribute: Some(Attribute {
                name: points("start-attribute"),
                value: points("discarded-start-value"),
            }),
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("End\r\0"),
            self_closing: true,
            attributes: vec![Attribute {
                name: points("end-attribute"),
                value: points("other-value"),
            }],
            discarded_current_attribute: Some(Attribute {
                name: points("end-attribute"),
                value: points("discarded-end-value"),
            }),
        }),
        pending_comment: Some(PendingComment {
            data: points("comment\r\0"),
        }),
        temporary_buffer: points("scratch\r\0"),
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
    return_state: Option<State>,
    last_start_tag: Option<Vec<CodePoint>>,
    current_tag: Option<TagKind>,
    pending_start_tag: Option<PendingStartTag>,
    pending_end_tag: Option<PendingEndTag>,
    pending_comment: Option<PendingComment>,
    temporary_buffer: Vec<CodePoint>,
}

fn snapshot(execution: &Execution) -> Snapshot {
    let context = execution.context();
    Snapshot {
        state: execution.state(),
        current: context.input.current(),
        tokens: context.tokens.clone(),
        errors: context.errors.clone(),
        input_errors: context.input.errors().to_vec(),
        return_state: context.return_state,
        last_start_tag: context.last_start_tag.clone(),
        current_tag: context.current_tag,
        pending_start_tag: context.pending_start_tag.clone(),
        pending_end_tag: context.pending_end_tag.clone(),
        pending_comment: context.pending_comment.clone(),
        temporary_buffer: context.temporary_buffer.clone(),
    }
}

#[test]
fn rcdata_transitions_consume_once_preserve_context_and_keep_character_reference_blocked() {
    for (point, destination) in [
        ('&', State::CharacterReference),
        ('<', State::RcdataLessThanSign),
    ] {
        for prior_return in [
            None,
            Some(State::Data),
            Some(State::AttributeValueDoubleQuoted),
        ] {
            for finished in [false, true] {
                for with_tail in [false, true] {
                    let tail = CodePoint::new(0xd800).unwrap();
                    let mut context = seeded_context();
                    context.return_state = prior_return;
                    context.input.push([point.into()]).unwrap();
                    if with_tail {
                        context.input.push([tail]).unwrap();
                    }
                    if finished {
                        context.input.finish();
                    }
                    let mut execution = Execution::new(State::Rcdata, context);
                    let mut expected = snapshot(&execution);
                    expected.state = destination;
                    expected.current = Some((
                        Input::CodePoint(point.into()),
                        Position { line: 1, column: 1 },
                    ));
                    if point == '&' {
                        expected.return_state = Some(State::Rcdata);
                    }
                    assert_eq!(execution.step(), Ok(Progress::Continue(destination)));
                    assert_eq!(snapshot(&execution), expected);
                    // Less-than-sign destination effects have their own state-local suite.
                    if point == '<' {
                        continue;
                    }
                    for _ in 0..3 {
                        assert_eq!(
                            execution.step(),
                            Err(UnimplementedState { state: destination })
                        );
                        assert_eq!(snapshot(&execution), expected);
                    }
                    // Inspection only, never reconstruction/resumption after a block.
                    // Neither delimiter scheduled reconsumption or consumed the tail.
                    let next = if with_tail {
                        Input::CodePoint(tail)
                    } else if finished {
                        Input::Eof
                    } else {
                        Input::NeedMoreInput
                    };
                    assert_eq!(execution.context_mut().input.consume(), next);
                    if with_tail {
                        assert_eq!(
                            execution.context().input.errors(),
                            [error("surrogate-in-input-stream", 1, 2)]
                        );
                    }
                    if !finished {
                        execution
                            .context_mut()
                            .input
                            .push(points("new input"))
                            .unwrap();
                        execution.context_mut().input.finish();
                    }
                    let inspected = snapshot(&execution);
                    assert_eq!(
                        execution.step(),
                        Err(UnimplementedState { state: destination })
                    );
                    assert_eq!(snapshot(&execution), inspected);
                }
            }
        }
    }
}

#[test]
fn rcdata_null_replaces_only_output_and_preserves_all_unrelated_context() {
    let mut context = seeded_context();
    context.input.push(points("\0\r\n\0\r")).unwrap();
    context.input.finish();
    let mut execution = Execution::new(State::Rcdata, context);
    let mut expected = snapshot(&execution);
    for (consumed, emitted, line, column) in [
        ('\0', '\u{fffd}', 1, 1),
        ('\n', '\n', 1, 2),
        ('\0', '\u{fffd}', 2, 1),
        ('\n', '\n', 2, 2),
    ] {
        expected.current = Some((Input::CodePoint(consumed.into()), Position { line, column }));
        expected.tokens.push(Token::Character(emitted.into()));
        if consumed == '\0' {
            expected
                .errors
                .push(error("unexpected-null-character", line, column));
        }
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Rcdata)));
        assert_eq!(snapshot(&execution), expected);
    }
    expected.current = Some((Input::Eof, Position { line: 3, column: 1 }));
    expected.tokens.push(Token::Eof);
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(snapshot(&execution), expected);
    for _ in 0..3 {
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&execution), expected);
    }
}

#[test]
fn rcdata_anything_else_preserves_actual_code_points_and_exact_preprocessing_errors() {
    let mut input = points("aZ!\"'%,-./09;=>?@`{}é😀\u{0301}\u{feff}\u{fffd}\t\u{000c}");
    let prefix_len = input.len();
    input.extend(
        [0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x01, 0x0b, 0x7f, 0x85]
            .map(|value| CodePoint::new(value).unwrap()),
    );
    let mut context = seeded_context();
    context.input.push(input.iter().copied()).unwrap();
    context.input.finish();
    let mut execution = Execution::new(State::Rcdata, context);
    let mut expected = snapshot(&execution);
    for (i, &point) in input.iter().enumerate() {
        expected.current = Some((
            Input::CodePoint(point),
            Position {
                line: 1,
                column: i + 1,
            },
        ));
        expected.tokens.push(Token::Character(point));
        if i >= prefix_len {
            let code = match i - prefix_len {
                0 | 1 => "surrogate-in-input-stream",
                2 | 3 => "noncharacter-in-input-stream",
                _ => "control-character-in-input-stream",
            };
            expected.input_errors.push(error(code, 1, i + 1));
        }
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Rcdata)));
        assert_eq!(snapshot(&execution), expected);
    }
    expected.current = Some((
        Input::Eof,
        Position {
            line: 1,
            column: input.len() + 1,
        },
    ));
    expected.tokens.push(Token::Eof);
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(snapshot(&execution), expected);
}

#[test]
fn rcdata_empty_open_input_suspends_until_explicit_eof_and_finishes_once() {
    for seeded in [false, true] {
        let context = if seeded {
            seeded_context()
        } else {
            Context::default()
        };
        let mut execution = Execution::new(State::Rcdata, context);
        let mut expected = snapshot(&execution);
        for _ in 0..3 {
            execution.context_mut().input.push([]).unwrap();
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&execution), expected);
        }
        execution.context_mut().input.finish();
        execution.context_mut().input.finish();
        expected.current = Some((Input::Eof, Position { line: 1, column: 1 }));
        expected.tokens.push(Token::Eof);
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&execution), expected);
        assert_eq!(
            execution.context_mut().input.push([]),
            Err(CursorError::InputAlreadyFinished)
        );
        // Mutation of retained context cannot clear the terminal status.
        execution.context_mut().input.reconsume().unwrap();
        for _ in 0..3 {
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&execution), expected);
        }
        let mut context = if seeded {
            seeded_context()
        } else {
            Context::default()
        };
        context.input.finish();
        let mut closed = Execution::new(State::Rcdata, context);
        assert_eq!(closed.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&closed), expected);
    }
}

// Capture every source step and terminal result, not only final output.
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
            for _ in 0..2 {
                assert_eq!(execution.step(), result);
                assert_eq!(snapshot(execution), before);
            }
            return false;
        }
        trace.push((result, snapshot(execution)));
        if !matches!(result, Ok(Progress::Continue(_))) {
            let terminal = snapshot(execution);
            for _ in 0..2 {
                assert_eq!(execution.step(), result);
                assert_eq!(snapshot(execution), terminal);
            }
            return true;
        }
    }
    panic!("RCDATA failed to consume, suspend or terminate");
}

#[test]
fn rcdata_every_chunk_boundary_preserves_steps_context_output_errors_and_transitions() {
    let mut mixed = points("a!>é😀\r\n\0\r\r\nb\r");
    mixed.extend([0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x01].map(|v| CodePoint::new(v).unwrap()));
    let mut samples = vec![
        vec![],
        mixed.clone(),
        points("\r"),
        points("\r\n"),
        points("\r\r"),
        points("\0\r"),
    ];
    for delimiter in ['&', '<'] {
        for prefix in [vec![], mixed.clone(), points("\r\n\0\r")] {
            let mut sample = prefix;
            sample.push(delimiter.into());
            samples.push(sample.clone());
            // Follow implemented destinations, but never resume after a stub.
            sample.extend([
                CodePoint::new(0xd800).unwrap(),
                '\0'.into(),
                '\r'.into(),
                '\n'.into(),
            ]);
            samples.push(sample);
        }
    }
    for input in samples {
        let run = |chunks: Vec<&[CodePoint]>, finish_first: bool| {
            let mut execution = Execution::new(State::Rcdata, seeded_context());
            let mut trace = Vec::new();
            if finish_first {
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
                    if drain(&mut execution, &mut trace, input.len() * 2 + 4) {
                        return trace;
                    }
                    execution.context_mut().input.push([]).unwrap();
                    assert!(!drain(&mut execution, &mut trace, 1));
                }
                execution.context_mut().input.finish();
            }
            assert!(drain(&mut execution, &mut trace, input.len() * 2 + 4));
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
