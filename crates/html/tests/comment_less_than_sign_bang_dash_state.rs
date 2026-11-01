//! Isolated source-step evidence, not upstream conformance.
//! HTML 404b3eb14a7a936dddefb2d78e224dbeb01d2358, source 144626–144640.
//! No predecessor or destination algorithm is simulated or resumed after a stub.

use neptune_html::tokenizer::{
    Attribute, CodePoint, Context, CursorError, Execution, Input, ParseError, PendingComment,
    PendingEndTag, PendingStartTag, Position, Progress, State, TagKind, Token, UnimplementedState,
};

const SOURCE: State = State::CommentLessThanSignBangDash;

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
        name: points("name"),
        value: vec!['\0'.into(), CodePoint::new(0xdfff).unwrap()],
    };
    let mut data = points("seed\0\r<!");
    data.extend([
        CodePoint::new(0xd800).unwrap(),
        CodePoint::new(0xffff).unwrap(),
    ]);
    Context {
        tokens: vec![
            Token::Character('p'.into()),
            Token::Comment { data: vec![] },
        ],
        errors: vec![error("prior", 1, 1)],
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
        pending_comment: Some(PendingComment { data }),
        temporary_buffer: vec!['\r'.into(), CodePoint::new(0xdfff).unwrap()],
        ..Default::default()
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    state: State,
    current: Option<(Input, Position)>,
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
    input_errors: Vec<ParseError>,
    last_start_tag: Option<Vec<CodePoint>>,
    return_state: Option<State>,
    current_tag: Option<TagKind>,
    start: Option<PendingStartTag>,
    end: Option<PendingEndTag>,
    comment: Option<PendingComment>,
    scratch: Vec<CodePoint>,
}

fn snapshot(execution: &Execution) -> Snapshot {
    let context = execution.context();
    Snapshot {
        state: execution.state(),
        current: context.input.current(),
        tokens: context.tokens.clone(),
        errors: context.errors.clone(),
        input_errors: context.input.errors().to_vec(),
        last_start_tag: context.last_start_tag.clone(),
        return_state: context.return_state,
        current_tag: context.current_tag,
        start: context.pending_start_tag.clone(),
        end: context.pending_end_tag.clone(),
        comment: context.pending_comment.clone(),
        scratch: context.temporary_buffer.clone(),
    }
}

// Cursor history only, not simulated predecessor execution. The
// caller supplies initialized comment data; this state must not recreate it.
fn entry(mut context: Context, newline_prefix: bool) -> Execution {
    if newline_prefix {
        context.input.push(points("\r\n")).unwrap();
        assert_eq!(context.input.consume(), Input::CodePoint('\n'.into()));
    }
    context.input.push(points("<!-")).unwrap();
    for point in points("<!-") {
        assert_eq!(context.input.consume(), Input::CodePoint(point));
    }
    Execution::new(SOURCE, context)
}

fn assert_terminal(execution: &mut Execution, expected: &Snapshot) {
    let blocked = Err(UnimplementedState {
        state: expected.state,
    });
    for _ in 0..3 {
        assert_eq!(execution.step(), blocked);
        assert_eq!(&snapshot(execution), expected);
    }
}

// Source snapshots are asserted before this helper attempts any destination.
// Only fallback reaches the newly implemented CommentEndDash; '-' still blocks
// at CommentLessThanSignBangDashDash without executing that sibling algorithm.
fn execute_destination(execution: &mut Execution, mut expected: Snapshot) -> Snapshot {
    if expected.state == State::CommentEndDash {
        assert_eq!(
            execution.context_mut().input.reconsume(),
            Err(CursorError::AlreadyReconsuming)
        );
        match expected.current.unwrap().0 {
            Input::Eof => {
                expected.errors.push(ParseError {
                    code: "eof-in-comment",
                    position: expected.current.unwrap().1,
                });
                let data = expected.comment.take().unwrap().data;
                expected
                    .tokens
                    .extend([Token::Comment { data }, Token::Eof]);
                for _ in 0..3 {
                    assert_eq!(execution.step(), Ok(Progress::Finished));
                    assert_eq!(snapshot(execution), expected);
                }
            }
            Input::CodePoint(point) => {
                assert_ne!(point, '-'.into());
                expected.state = State::Comment;
                expected.comment.as_mut().unwrap().data.push('-'.into());
                assert_eq!(execution.step(), Ok(Progress::Continue(State::Comment)));
                assert_eq!(snapshot(execution), expected);
                comment_replay(execution, &mut expected);
            }
            Input::NeedMoreInput => panic!("source must have consumed a point or EOF"),
        }
    }
    expected
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
fn all_ascii_and_representative_non_scalars_have_exact_source_effects_and_consumption() {
    for value in (0..=0x7f).chain([
        0x80, 0x9f, 0xa0, 0xc9, 0x3a9, 0x2010, 0x2212, 0xfeff, 0x1f600, 0xd800, 0xdfff, 0xfdd0,
        0xffff, 0x10ffff,
    ]) {
        for finished in [false, true] {
            for selected in [None, Some(TagKind::Start), Some(TagKind::End)] {
                for empty_comment in [false, true] {
                    let mut context = seeded();
                    context.current_tag = selected;
                    if empty_comment {
                        context.pending_comment = Some(PendingComment::default());
                    }
                    let mut execution = entry(context, true);
                    let point = CodePoint::new(value).unwrap();
                    let normalized = if value == 0x0d { '\n'.into() } else { point };
                    execution
                        .context_mut()
                        .input
                        .push([point, 'z'.into()])
                        .unwrap();
                    if finished {
                        execution.context_mut().input.finish();
                    }
                    let mut expected = snapshot(&execution);
                    expected.state = if value == 0x2d {
                        State::CommentLessThanSignBangDashDash
                    } else {
                        State::CommentEndDash
                    };
                    expected.current = Some((
                        Input::CodePoint(normalized),
                        Position { line: 2, column: 4 },
                    ));
                    let diagnostic = match value {
                        0xd800..=0xdfff => Some("surrogate-in-input-stream"),
                        0xfdd0 | 0xffff | 0x10ffff => Some("noncharacter-in-input-stream"),
                        0x01..=0x08 | 0x0b | 0x0e..=0x1f | 0x7f..=0x9f => {
                            Some("control-character-in-input-stream")
                        }
                        _ => None,
                    };
                    if let Some(code) = diagnostic {
                        expected.input_errors.push(error(code, 2, 4));
                    }
                    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                    assert_eq!(snapshot(&execution), expected);
                    // Neither branch emits, mutates, takes, normalizes or replaces
                    // existing comment data, including literal NUL/CR/non-scalars.
                    let expected = execute_destination(&mut execution, expected);
                    // Probe only after source/destination snapshots. Never execute
                    // an implemented state with this probed cursor.
                    assert_eq!(execution.context().input.current(), expected.current);
                    assert_eq!(execution.context().input.errors(), expected.input_errors);
                    assert_eq!(
                        execution.context_mut().input.consume(),
                        Input::CodePoint('z'.into())
                    );
                    let tail_position = if value == 0x0a || value == 0x0d {
                        Position { line: 3, column: 1 }
                    } else {
                        Position { line: 2, column: 5 }
                    };
                    assert_eq!(execution.context().input.position(), tail_position);
                    assert_eq!(
                        execution.context_mut().input.consume(),
                        if finished {
                            Input::Eof
                        } else {
                            Input::NeedMoreInput
                        }
                    );
                    assert_eq!(execution.context().input.errors(), expected.input_errors);
                    if value == 0x2d {
                        let probed = snapshot(&execution);
                        assert_terminal(&mut execution, &probed);
                    }
                    // Never dispatch an implemented destination after this probe.
                }
            }
        }
    }
}

#[test]
fn open_exhaustion_preserves_current_and_comment_until_explicit_eof_reconsumption() {
    for newline_prefix in [false, true] {
        let mut execution = entry(seeded(), newline_prefix);
        let mut expected = snapshot(&execution);
        for _ in 0..3 {
            execution.context_mut().input.push([]).unwrap();
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&execution), expected);
        }
        execution.context_mut().input.finish();
        execution.context_mut().input.finish();
        expected.state = State::CommentEndDash;
        expected.current = Some((
            Input::Eof,
            Position {
                line: if newline_prefix { 2 } else { 1 },
                column: 4,
            },
        ));
        assert_eq!(
            execution.step(),
            Ok(Progress::Continue(State::CommentEndDash))
        );
        assert_eq!(snapshot(&execution), expected);
        // No eof-in-comment, Comment or EOF token belongs to this source step.
        // The source's scheduled EOF is consumed by real CommentEndDash, not
        // diagnosed or emitted by CommentLessThanSignBangDash itself.
        let expected = execute_destination(&mut execution, expected);
        assert_eq!(execution.context().input.current(), expected.current);
        assert_eq!(
            execution.context_mut().input.push(points("late")),
            Err(CursorError::InputAlreadyFinished)
        );
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&execution), expected);
    }
}

#[test]
fn suspended_entry_can_receive_a_hyphen_or_reconsumed_character_without_finishing() {
    for point in ['-', '\0', '>', '\r', 'é'] {
        let mut execution = entry(seeded(), false);
        let before = snapshot(&execution);
        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
        assert_eq!(snapshot(&execution), before);
        execution.context_mut().input.push([point.into()]).unwrap();
        let target = if point == '-' {
            State::CommentLessThanSignBangDashDash
        } else {
            State::CommentEndDash
        };
        assert_eq!(execution.step(), Ok(Progress::Continue(target)));
        let expected = snapshot(&execution);
        let mut expected = execute_destination(&mut execution, expected);
        if point == '-' {
            assert_terminal(&mut execution, &expected);
        } else {
            for _ in 0..3 {
                assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                assert_eq!(snapshot(&execution), expected);
            }
            execution.context_mut().input.finish();
            expected.current = Some((
                Input::Eof,
                if point == '\r' {
                    Position { line: 2, column: 1 }
                } else {
                    Position { line: 1, column: 5 }
                },
            ));
            comment_replay(&mut execution, &mut expected);
        }
    }
}

type Trace = Vec<(Result<Progress, UnimplementedState>, Snapshot)>;

fn drain(execution: &mut Execution, trace: &mut Trace) -> bool {
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
            let terminal = snapshot(execution);
            assert_eq!(execution.step(), result);
            assert_eq!(snapshot(execution), terminal);
            return true;
        }
    }
}

fn trace(chunks: &[&[CodePoint]], closed: bool, newline_prefix: bool) -> Trace {
    let mut execution = entry(seeded(), newline_prefix);
    let mut trace = vec![];
    if closed {
        for chunk in chunks {
            execution
                .context_mut()
                .input
                .push(chunk.iter().copied())
                .unwrap();
        }
        execution.context_mut().input.finish();
    } else {
        for chunk in chunks {
            execution
                .context_mut()
                .input
                .push(chunk.iter().copied())
                .unwrap();
            // Stop permanently at a genuine stub, with no feeding or replay.
            if drain(&mut execution, &mut trace) {
                return trace;
            }
        }
        execution.context_mut().input.finish();
    }
    assert!(drain(&mut execution, &mut trace));
    trace
}

#[test]
fn every_two_chunk_split_and_single_point_chunks_preserve_source_and_unconsumed_tail() {
    let mut inputs: Vec<_> = [
        "",
        "-",
        "-->",
        "-tail\r\n\0",
        ">tail",
        "\0<!-",
        "\r\n\r\r\ntail",
        "<!---->",
        "é&",
    ]
    .into_iter()
    .map(points)
    .collect();
    inputs.push(
        [0xd800, 0xfdd0, 0xb, 0, 0xd, 0xa, 0x2d]
            .map(|value| CodePoint::new(value).unwrap())
            .to_vec(),
    );
    inputs.push(vec![CodePoint::new(0x10ffff).unwrap()]);
    for input in &inputs {
        for newline_prefix in [false, true] {
            let expected = trace(&[input], true, newline_prefix);
            for split in 0..=input.len() {
                assert_eq!(
                    trace(
                        &[&input[..split], &[], &input[split..]],
                        false,
                        newline_prefix
                    ),
                    expected,
                    "split {split}: {input:?}"
                );
            }
            assert_eq!(
                trace(&input.chunks(1).collect::<Vec<_>>(), false, newline_prefix),
                expected,
                "single points: {input:?}"
            );
        }
    }
}

#[test]
fn normal_data_comment_entry_still_stops_at_markup_declaration_open() {
    let mut context = Context::default();
    context.input.push(points("<!--x-->tail")).unwrap();
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
    assert!(expected.tokens.is_empty());
    assert!(expected.errors.is_empty());
    assert!(expected.input_errors.is_empty());
    assert!(expected.comment.is_none());
    assert_terminal(&mut execution, &expected);
    assert_eq!(
        execution.context_mut().input.consume(),
        Input::CodePoint('-'.into())
    );
}
