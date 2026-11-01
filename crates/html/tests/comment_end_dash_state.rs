//! Local source-step/streaming evidence, not upstream conformance runs.
//! HTML 404b3eb14a7a936dddefb2d78e224dbeb01d2358, source 144660–144678,
//! #comment-end-dash-state. Entry supplies initialized pending-comment context;
//! no test reconstructs or resumes an execution blocked at a destination stub.

use neptune_html::tokenizer::{
    Attribute, CodePoint, Context, CursorError, Execution, Input, ParseError, PendingComment,
    PendingEndTag, PendingStartTag, Position, Progress, State, TagKind, Token, UnimplementedState,
};

fn points(text: &str) -> Vec<CodePoint> {
    text.chars().map(Into::into).collect()
}

fn point(value: u32) -> CodePoint {
    CodePoint::new(value).unwrap()
}

fn error(code: &'static str, line: usize, column: usize) -> ParseError {
    ParseError {
        code,
        position: Position { line, column },
    }
}

fn seeded_context() -> Context {
    let attribute = Attribute {
        name: points("name"),
        value: vec![point(0xd800), '\0'.into()],
    };
    Context {
        tokens: vec![
            Token::Character('p'.into()),
            Token::Comment { data: vec![] },
        ],
        errors: vec![error("prior-diagnostic", 1, 1)],
        last_start_tag: Some(points("title")),
        return_state: Some(State::AttributeValueSingleQuoted),
        current_tag: Some(TagKind::End),
        pending_start_tag: Some(PendingStartTag {
            name: points("start"),
            self_closing: true,
            attributes: vec![attribute.clone()],
            discarded_current_attribute: Some(attribute.clone()),
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("end"),
            self_closing: true,
            attributes: vec![attribute.clone()],
            discarded_current_attribute: Some(attribute),
        }),
        // Existing data is literal storage, not input to preprocess again.
        pending_comment: Some(PendingComment {
            data: vec![
                's'.into(),
                '\0'.into(),
                '\r'.into(),
                point(0xdfff),
                point(0xfdd0),
            ],
        }),
        temporary_buffer: vec!['A'.into(), point(0xd800), '\0'.into()],
        ..Default::default()
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Unrelated {
    last_start_tag: Option<Vec<CodePoint>>,
    return_state: Option<State>,
    current_tag: Option<TagKind>,
    start: Option<PendingStartTag>,
    end: Option<PendingEndTag>,
    buffer: Vec<CodePoint>,
}

#[derive(Debug, Eq, PartialEq)]
struct Snapshot {
    state: State,
    current: Option<(Input, Position)>,
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
    input_errors: Vec<ParseError>,
    comment: Option<PendingComment>,
    unrelated: Unrelated,
}

fn snapshot(execution: &Execution) -> Snapshot {
    let context = execution.context();
    Snapshot {
        state: execution.state(),
        current: context.input.current(),
        tokens: context.tokens.clone(),
        errors: context.errors.clone(),
        input_errors: context.input.errors().to_vec(),
        comment: context.pending_comment.clone(),
        unrelated: Unrelated {
            last_start_tag: context.last_start_tag.clone(),
            return_state: context.return_state,
            current_tag: context.current_tag,
            start: context.pending_start_tag.clone(),
            end: context.pending_end_tag.clone(),
            buffer: context.temporary_buffer.clone(),
        },
    }
}

// Cursor history only, not simulated predecessor execution.
fn entry(mut context: Context, history: bool) -> Execution {
    if history {
        context.input.push(points("\r\nX")).unwrap();
        assert_eq!(context.input.consume(), Input::CodePoint('\n'.into()));
        assert_eq!(context.input.consume(), Input::CodePoint('X'.into()));
    }
    Execution::new(State::CommentEndDash, context)
}

fn input_error(value: u32) -> Option<&'static str> {
    match value {
        0xd800..=0xdfff => Some("surrogate-in-input-stream"),
        0xfdd0..=0xfdef | 0xfffe | 0xffff | 0x10ffff => Some("noncharacter-in-input-stream"),
        0x01..=0x08 | 0x0b | 0x0e..=0x1f | 0x7f..=0x9f => Some("control-character-in-input-stream"),
        _ => None,
    }
}

// The predecessor's independently asserted source snapshot schedules this item
// for real Comment. Check that destination separately, including EOF/take order;
// preprocessing diagnostics and unrelated context must remain unchanged.
fn comment_replay(run: &mut Execution, expected: &mut Snapshot) {
    assert_eq!(expected.state, State::Comment);
    let (input, position) = expected.current.unwrap();
    let result = match input {
        Input::Eof => {
            expected.errors.push(ParseError {
                code: "eof-in-comment",
                position,
            });
            let data = expected.comment.take().unwrap().data;
            expected
                .tokens
                .extend([Token::Comment { data }, Token::Eof]);
            Progress::Finished
        }
        Input::CodePoint(point) => {
            match point.value() {
                0x3c => {
                    expected.comment.as_mut().unwrap().data.push(point);
                    expected.state = State::CommentLessThanSign;
                }
                0x2d => expected.state = State::CommentEndDash,
                0 => {
                    expected.errors.push(ParseError {
                        code: "unexpected-null-character",
                        position,
                    });
                    expected
                        .comment
                        .as_mut()
                        .unwrap()
                        .data
                        .push('\u{fffd}'.into());
                }
                _ => expected.comment.as_mut().unwrap().data.push(point),
            }
            Progress::Continue(expected.state)
        }
        Input::NeedMoreInput => panic!("a predecessor replay requires current input"),
    };
    assert_eq!(run.step(), Ok(result));
    assert_eq!(snapshot(run), *expected);
    if result == Progress::Finished {
        assert_eq!(run.step(), Ok(result));
        assert_eq!(snapshot(run), *expected);
    }
}

#[test]
fn every_ascii_and_non_scalar_branch_preserves_source_then_executes_real_comment_or_blocks_end() {
    for value in (0..=0x7f).chain([
        0x80, 0x9f, 0xa0, 0xc9, 0x3a9, 0xfeff, 0x1f600, 0xd800, 0xdfff, 0xfdd0, 0xffff, 0x10ffff,
    ]) {
        for finished in [false, true] {
            for selected in [None, Some(TagKind::Start), Some(TagKind::End)] {
                let mut context = seeded_context();
                context.current_tag = selected;
                let mut execution = entry(context, true);
                execution
                    .context_mut()
                    .input
                    .push([point(value), 'z'.into()])
                    .unwrap();
                if finished {
                    execution.context_mut().input.finish();
                }
                let mut expected = snapshot(&execution);
                let normalized = if value == 0x0d {
                    '\n'.into()
                } else {
                    point(value)
                };
                expected.current = Some((
                    Input::CodePoint(normalized),
                    Position { line: 2, column: 2 },
                ));
                if let Some(code) = input_error(value) {
                    expected.input_errors.push(error(code, 2, 2));
                }
                expected.state = if value == 0x2d {
                    State::CommentEnd
                } else {
                    State::Comment
                };
                if value != 0x2d {
                    expected.comment.as_mut().unwrap().data.push('-'.into());
                }
                assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                assert_eq!(snapshot(&execution), expected, "U+{value:04X}");
                // In particular: '>', '<', '&', '!', NUL and non-scalars do not
                // get appended, replaced, diagnosed by this state, or emitted.
                if value == 0x2d {
                    let blocked = Err(UnimplementedState {
                        state: State::CommentEnd,
                    });
                    for _ in 0..3 {
                        assert_eq!(execution.step(), blocked);
                        assert_eq!(snapshot(&execution), expected);
                    }
                    if !finished {
                        execution.context_mut().input.push(points("more")).unwrap();
                        execution.context_mut().input.finish();
                        assert_eq!(execution.step(), blocked);
                        assert_eq!(snapshot(&execution), expected);
                    }
                } else {
                    comment_replay(&mut execution, &mut expected);
                }
            }
        }
    }
}

#[test]
fn cursor_probes_before_destination_attempt_prove_consumption_and_single_reconsumption() {
    for value in [
        0x2d, 0x3e, 0x3c, 0x26, 0x21, 0, 0x0d, 0xd800, 0xfdd0, 0x0b, 0x1f600,
    ] {
        let mut execution = entry(seeded_context(), true);
        execution
            .context_mut()
            .input
            .push([point(value), 'z'.into()])
            .unwrap();
        execution.context_mut().input.finish();
        let target = if value == 0x2d {
            State::CommentEnd
        } else {
            State::Comment
        };
        assert_eq!(execution.step(), Ok(Progress::Continue(target)));
        let before = snapshot(&execution);
        if value != 0x2d {
            assert_eq!(
                execution.context_mut().input.reconsume(),
                Err(CursorError::AlreadyReconsuming)
            );
            let normalized = if value == 0x0d {
                '\n'.into()
            } else {
                point(value)
            };
            assert_eq!(
                execution.context_mut().input.consume(),
                Input::CodePoint(normalized)
            );
            assert_eq!(snapshot(&execution), before);
        }
        // '-' was consumed without reconsumption; other points replayed exactly
        // once at their original position, without duplicate preprocessing errors.
        assert_eq!(
            execution.context_mut().input.consume(),
            Input::CodePoint('z'.into())
        );
        assert_eq!(
            execution.context().input.position(),
            if value == 0x0d {
                Position { line: 3, column: 1 }
            } else {
                Position { line: 2, column: 3 }
            }
        );
        assert_eq!(execution.context().input.errors(), before.input_errors);
        assert_eq!(execution.context_mut().input.consume(), Input::Eof);
        // This test ends at a cursor probe; it never dispatches a destination
        // with a cursor altered by the probe, nor resumes a blocked execution.
    }
}

#[test]
fn open_exhaustion_is_effect_free_and_explicit_eof_takes_comment_once_in_order() {
    for history in [false, true] {
        for seeded in [false, true] {
            let context = if seeded {
                seeded_context()
            } else {
                Context {
                    pending_comment: Some(PendingComment::default()),
                    ..Default::default()
                }
            };
            let mut execution = entry(context, history);
            let mut expected = snapshot(&execution);
            for _ in 0..3 {
                execution.context_mut().input.push([]).unwrap();
                assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                assert_eq!(snapshot(&execution), expected);
            }
            execution.context_mut().input.finish();
            execution.context_mut().input.finish();
            let position = if history {
                Position { line: 2, column: 2 }
            } else {
                Position { line: 1, column: 1 }
            };
            expected.current = Some((Input::Eof, position));
            expected.errors.push(ParseError {
                code: "eof-in-comment",
                position,
            });
            let data = expected.comment.take().unwrap().data;
            expected
                .tokens
                .extend([Token::Comment { data }, Token::Eof]);
            for _ in 0..3 {
                assert_eq!(execution.step(), Ok(Progress::Finished));
                assert_eq!(snapshot(&execution), expected);
            }
            assert_eq!(execution.state(), State::CommentEndDash);
            assert_eq!(
                execution.context_mut().input.push([]),
                Err(CursorError::InputAlreadyFinished)
            );
            // No withheld '-' is appended on EOF, no switch to Data, and no
            // second emission or access to the now-taken comment on repeat steps.
        }
    }
}

#[test]
fn caller_scheduled_reconsumption_including_eof_is_consumed_without_duplicate_input_errors() {
    for input in [
        Input::Eof,
        Input::CodePoint('-'.into()),
        Input::CodePoint(point(0xd800)),
    ] {
        let mut context = seeded_context();
        if let Input::CodePoint(p) = input {
            context.input.push([p]).unwrap();
        }
        context.input.finish();
        assert_eq!(context.input.consume(), input);
        context.input.reconsume().unwrap();
        let mut execution = Execution::new(State::CommentEndDash, context);
        let mut expected = snapshot(&execution);
        match input {
            Input::Eof => {
                expected.errors.push(error("eof-in-comment", 1, 1));
                let data = expected.comment.take().unwrap().data;
                expected
                    .tokens
                    .extend([Token::Comment { data }, Token::Eof]);
                assert_eq!(execution.step(), Ok(Progress::Finished));
            }
            Input::CodePoint(p) => {
                expected.state = if p.value() == 0x2d {
                    State::CommentEnd
                } else {
                    State::Comment
                };
                if p.value() != 0x2d {
                    expected.comment.as_mut().unwrap().data.push('-'.into());
                }
                assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
            }
            Input::NeedMoreInput => unreachable!(),
        }
        assert_eq!(snapshot(&execution), expected);
    }
}

fn drain(
    execution: &mut Execution,
    trace: &mut Vec<(Result<Progress, UnimplementedState>, Snapshot)>,
) -> bool {
    loop {
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
            return true;
        }
    }
}

fn trace(
    chunks: &[&[CodePoint]],
    closed_up_front: bool,
    cr_history: bool,
) -> Vec<(Result<Progress, UnimplementedState>, Snapshot)> {
    let mut context = seeded_context();
    if cr_history {
        // A preceding consumed CR tests suppression of an LF supplied in the
        // next chunk, including open exhaustion after that suppression.
        context.input.push(points("\r")).unwrap();
        assert_eq!(context.input.consume(), Input::CodePoint('\n'.into()));
    }
    let mut execution = Execution::new(State::CommentEndDash, context);
    let mut trace = Vec::new();
    if closed_up_front {
        for chunk in chunks {
            execution
                .context_mut()
                .input
                .push(chunk.iter().copied())
                .unwrap();
        }
        execution.context_mut().input.finish();
        assert!(drain(&mut execution, &mut trace));
    } else {
        assert!(!drain(&mut execution, &mut trace));
        for chunk in chunks {
            execution
                .context_mut()
                .input
                .push(chunk.iter().copied())
                .unwrap();
            if drain(&mut execution, &mut trace) {
                return trace; // Never feed/reconstruct/resume past a terminal block.
            }
        }
        execution.context_mut().input.finish();
        assert!(drain(&mut execution, &mut trace));
    }
    trace
}

#[test]
fn every_two_chunk_split_and_single_code_point_chunks_match_closed_source_step_traces() {
    let cases = [
        points(""),
        points("-"),
        points("-tail"),
        points("--!>"),
        points("x"),
        points("!>"),
        points("<>&"),
        points("\0tail"),
        points("\r\n"),
        points("\n"),
        points("\n-"),
        points("\r\r\n\0"),
        points("é😀"),
        vec![point(0xd800), point(0xfdd0), point(0x0b)],
    ];
    for input in cases {
        for cr_history in [false, true] {
            let expected = trace(&[&input], true, cr_history);
            for split in 0..=input.len() {
                assert_eq!(
                    trace(&[&input[..split], &[], &input[split..]], false, cr_history),
                    expected,
                    "split {split}: {input:?}"
                );
            }
            assert_eq!(
                trace(&input.chunks(1).collect::<Vec<_>>(), false, cr_history),
                expected,
                "single points: {input:?}"
            );
        }
    }
}

#[test]
fn normal_data_comment_entry_remains_terminally_blocked_without_comment_creation() {
    let mut context = Context::default();
    context.input.push(points("<!--text-->tail")).unwrap();
    context.input.finish();
    let mut execution = Execution::new(State::Data, context);
    assert_eq!(execution.step(), Ok(Progress::Continue(State::TagOpen)));
    assert_eq!(
        execution.step(),
        Ok(Progress::Continue(State::MarkupDeclarationOpen))
    );
    let expected = snapshot(&execution);
    assert_eq!(
        expected.current,
        Some((
            Input::CodePoint('!'.into()),
            Position { line: 1, column: 2 }
        ))
    );
    assert!(expected.comment.is_none());
    assert!(expected.tokens.is_empty());
    assert!(expected.errors.is_empty());
    assert!(expected.input_errors.is_empty());
    for _ in 0..3 {
        assert_eq!(
            execution.step(),
            Err(UnimplementedState {
                state: State::MarkupDeclarationOpen
            })
        );
        assert_eq!(snapshot(&execution), expected);
    }
}
