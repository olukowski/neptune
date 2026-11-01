//! Local source-step evidence for pinned HTML Comment start dash, not conformance.
//! https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144535-L144559

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

fn seeded(kind: TagKind, data: Vec<CodePoint>) -> Context {
    let attribute = Attribute {
        name: points("a"),
        value: points("v"),
    };
    Context {
        tokens: vec![
            Token::Comment { data: vec![] },
            Token::Character('p'.into()),
        ],
        errors: vec![error("prior", 1, 1)],
        pending_comment: Some(PendingComment { data }),
        current_tag: Some(kind),
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
        last_start_tag: Some(points("title")),
        return_state: Some(State::Rcdata),
        temporary_buffer: points("scratch"),
        ..Context::default()
    }
}

fn literal_seed() -> Vec<CodePoint> {
    // Existing comment data is not input: no normalization/replacement/diagnostics.
    [
        0, 0x0d, 0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x26, 0x3c, 0x1f600,
    ]
    .map(|value| CodePoint::new(value).unwrap())
    .to_vec()
}

fn entry(kind: TagKind, data: Vec<CodePoint>, prefix: bool) -> Execution {
    let mut context = seeded(kind, data);
    if prefix {
        // Supply predecessor consumption, not an implementation of CommentStart.
        context.input.push(points("\r\n-")).unwrap();
        assert_eq!(context.input.consume(), Input::CodePoint('\n'.into()));
        assert_eq!(context.input.consume(), Input::CodePoint('-'.into()));
        assert_eq!(context.input.position(), Position { line: 2, column: 1 });
    }
    Execution::new(State::CommentStartDash, context)
}

fn isolated(input: &[CodePoint], closed: bool) -> Execution {
    let mut execution = entry(TagKind::End, literal_seed(), true);
    execution
        .context_mut()
        .input
        .push(input.iter().copied())
        .unwrap();
    if closed {
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
    comment: Option<PendingComment>,
    current_tag: Option<TagKind>,
    start: Option<PendingStartTag>,
    end: Option<PendingEndTag>,
    last_start_tag: Option<Vec<CodePoint>>,
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
        comment: c.pending_comment.clone(),
        current_tag: c.current_tag,
        start: c.pending_start_tag.clone(),
        end: c.pending_end_tag.clone(),
        last_start_tag: c.last_start_tag.clone(),
        return_state: c.return_state,
        buffer: c.temporary_buffer.clone(),
    }
}

fn terminal(execution: &mut Execution, result: Result<Progress, UnimplementedState>) {
    let before = snapshot(execution);
    for _ in 0..3 {
        assert_eq!(execution.step(), result);
        assert_eq!(snapshot(execution), before);
    }
}

#[test]
fn hyphen_consumes_without_appending_emitting_or_reconsuming_then_blocks() {
    for kind in [TagKind::Start, TagKind::End] {
        for data in [vec![], literal_seed()] {
            let mut execution = entry(kind, data, true);
            execution.context_mut().input.push(points("-x")).unwrap();
            execution.context_mut().input.finish();
            let mut expected = snapshot(&execution);
            expected.state = State::CommentEnd;
            expected.current = Some((
                Input::CodePoint('-'.into()),
                Position { line: 2, column: 2 },
            ));
            assert_eq!(execution.step(), Ok(Progress::Continue(State::CommentEnd)));
            assert_eq!(snapshot(&execution), expected);
            terminal(
                &mut execution,
                Err(UnimplementedState {
                    state: State::CommentEnd,
                }),
            );
            // Cursor inspection only, never reconstruct/resume terminal execution.
            assert_eq!(
                execution.context_mut().input.consume(),
                Input::CodePoint('x'.into())
            );
            assert_eq!(execution.context_mut().input.consume(), Input::Eof);
        }
    }
}

#[test]
fn anything_else_appends_only_one_hyphen_and_reconsumes_literal_input() {
    // All ASCII fallback points, whitespace, malformed punctuation, and non-scalars.
    let values = (0..=0x7f).filter(|&v| v != 0x2d && v != 0x3e).chain([
        0x85, 0xa0, 0xc9, 0x391, 0x301, 0x1f600, 0xd800, 0xdfff, 0xfdd0, 0x10ffff,
    ]);
    for value in values {
        let point = CodePoint::new(value).unwrap();
        let normalized = if value == 0x0d { '\n'.into() } else { point };
        let mut execution = isolated(&[point, 'x'.into()], true);
        let mut expected = snapshot(&execution);
        expected.state = State::Comment;
        expected.current = Some((
            Input::CodePoint(normalized),
            Position { line: 2, column: 2 },
        ));
        expected.comment.as_mut().unwrap().data.push('-'.into());
        let preprocessing = match value {
            0xd800 | 0xdfff => Some("surrogate-in-input-stream"),
            0xfdd0 | 0x10ffff => Some("noncharacter-in-input-stream"),
            0x01..=0x08 | 0x0b | 0x0e..=0x1f | 0x7f | 0x85 => {
                Some("control-character-in-input-stream")
            }
            _ => None,
        };
        if let Some(code) = preprocessing {
            expected.input_errors.push(error(code, 2, 2));
        }
        assert_eq!(
            execution.step(),
            Ok(Progress::Continue(State::Comment)),
            "{value:x}"
        );
        assert_eq!(snapshot(&execution), expected, "{value:x}");
        terminal(
            &mut execution,
            Err(UnimplementedState {
                state: State::Comment,
            }),
        );
        // Inspect cursor only: no destination algorithm or blocked resumption.
        let cursor = &mut execution.context_mut().input;
        assert_eq!(cursor.consume(), Input::CodePoint(normalized));
        assert_eq!(cursor.current(), expected.current);
        assert_eq!(cursor.errors(), expected.input_errors);
        assert_eq!(cursor.consume(), Input::CodePoint('x'.into()));
        assert_eq!(
            cursor.position(),
            if normalized == '\n'.into() {
                Position { line: 3, column: 1 }
            } else {
                Position { line: 2, column: 3 }
            }
        );
        assert_eq!(cursor.consume(), Input::Eof);
    }
}

#[test]
fn greater_than_emits_and_takes_exact_comment_then_real_data_continues() {
    for kind in [TagKind::Start, TagKind::End] {
        for data in [vec![], literal_seed()] {
            let mut execution = entry(kind, data.clone(), true);
            execution
                .context_mut()
                .input
                .push(points(">x\r\n\0"))
                .unwrap();
            execution.context_mut().input.finish();
            let mut expected = snapshot(&execution);
            expected.state = State::Data;
            expected.current = Some((
                Input::CodePoint('>'.into()),
                Position { line: 2, column: 2 },
            ));
            expected
                .errors
                .push(error("abrupt-closing-of-empty-comment", 2, 2));
            expected.comment = None;
            expected.tokens.push(Token::Comment { data });
            assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
            assert_eq!(snapshot(&execution), expected);
            for (point, line, column) in [('x', 2, 3), ('\n', 2, 4), ('\0', 3, 1)] {
                expected.current =
                    Some((Input::CodePoint(point.into()), Position { line, column }));
                expected.tokens.push(Token::Character(point.into()));
                if point == '\0' {
                    expected
                        .errors
                        .push(error("unexpected-null-character", line, column));
                }
                assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
                assert_eq!(snapshot(&execution), expected);
            }
            expected.current = Some((Input::Eof, Position { line: 3, column: 2 }));
            expected.tokens.push(Token::Eof);
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&execution), expected);
            terminal(&mut execution, Ok(Progress::Finished));
        }
    }
}

#[test]
fn open_exhaustion_has_no_effects_but_explicit_eof_diagnoses_emits_and_takes() {
    for kind in [TagKind::Start, TagKind::End] {
        for data in [vec![], literal_seed()] {
            for prefix in [false, true] {
                let mut execution = entry(kind, data.clone(), prefix);
                terminal(&mut execution, Ok(Progress::NeedMoreInput));
                execution.context_mut().input.push([]).unwrap();
                terminal(&mut execution, Ok(Progress::NeedMoreInput));
                execution.context_mut().input.finish();
                let mut expected = snapshot(&execution);
                let position = if prefix {
                    Position { line: 2, column: 2 }
                } else {
                    Position { line: 1, column: 1 }
                };
                expected.current = Some((Input::Eof, position));
                expected.errors.push(ParseError {
                    code: "eof-in-comment",
                    position,
                });
                expected.comment = None;
                expected
                    .tokens
                    .extend([Token::Comment { data: data.clone() }, Token::Eof]);
                assert_eq!(execution.step(), Ok(Progress::Finished));
                assert_eq!(snapshot(&execution), expected);
                terminal(&mut execution, Ok(Progress::Finished));
                // EOF was consumed, not scheduled for reconsumption by this state.
                let cursor = &mut execution.context_mut().input;
                cursor.reconsume().unwrap();
                assert_eq!(cursor.consume(), Input::Eof);
                assert_eq!(cursor.current(), expected.current);
                assert_eq!(cursor.errors(), expected.input_errors);
            }
        }
    }
}

#[test]
fn incoming_reconsumption_of_code_points_and_eof_is_consumed_once() {
    for first in [
        Input::CodePoint('-'.into()),
        Input::CodePoint('>'.into()),
        Input::CodePoint(CodePoint::new(0xd800).unwrap()),
        Input::Eof,
    ] {
        let mut execution = entry(TagKind::Start, literal_seed(), false);
        if let Input::CodePoint(point) = first {
            execution.context_mut().input.push([point]).unwrap();
        }
        execution.context_mut().input.finish();
        assert_eq!(execution.context_mut().input.consume(), first);
        execution.context_mut().input.reconsume().unwrap();
        let before = snapshot(&execution);
        let result = execution.step().unwrap();
        assert_eq!(execution.context().input.current(), before.current);
        assert_eq!(execution.context().input.errors(), before.input_errors);
        match first {
            Input::CodePoint(point) if point.value() == 0x2d => {
                assert_eq!(result, Progress::Continue(State::CommentEnd));
                // No outgoing reconsumption: scheduling a new one must succeed.
                execution.context_mut().input.reconsume().unwrap();
            }
            Input::CodePoint(point) if point.value() == 0x3e => {
                assert_eq!(result, Progress::Continue(State::Data));
                assert_eq!(
                    execution.context().errors,
                    [
                        error("prior", 1, 1),
                        error("abrupt-closing-of-empty-comment", 1, 1)
                    ]
                );
                assert!(execution.context().pending_comment.is_none());
            }
            Input::CodePoint(point) => {
                assert_eq!(result, Progress::Continue(State::Comment));
                let mut data = literal_seed();
                data.push('-'.into());
                assert_eq!(
                    execution.context().pending_comment,
                    Some(PendingComment { data })
                );
                assert_eq!(
                    execution.context_mut().input.consume(),
                    Input::CodePoint(point)
                );
                assert_eq!(execution.context().input.errors(), before.input_errors);
                assert_eq!(execution.context_mut().input.consume(), Input::Eof);
            }
            Input::Eof => {
                assert_eq!(result, Progress::Finished);
                assert_eq!(
                    execution.context().errors,
                    [error("prior", 1, 1), error("eof-in-comment", 1, 1)]
                );
                assert!(execution.context().pending_comment.is_none());
                assert_eq!(execution.context().tokens.last(), Some(&Token::Eof));
            }
            Input::NeedMoreInput => unreachable!(),
        }
        // No dispatch after cursor-only inspection of source effects.
    }
}

type Trace = Vec<(Result<Progress, UnimplementedState>, Snapshot)>;

fn drain(execution: &mut Execution, trace: &mut Trace, bound: usize) -> bool {
    for _ in 0..bound {
        let before = snapshot(execution);
        let result = execution.step();
        if result == Ok(Progress::NeedMoreInput) {
            assert_eq!(snapshot(execution), before);
            terminal(execution, result);
            return false;
        }
        trace.push((result, snapshot(execution)));
        if !matches!(result, Ok(Progress::Continue(_))) {
            terminal(execution, result);
            return true;
        }
    }
    panic!("execution did not consume, suspend, or terminate");
}

fn trace(chunks: &[&[CodePoint]], closed: bool, prefix: bool) -> Trace {
    let mut execution = entry(TagKind::End, literal_seed(), prefix);
    let bound = chunks.iter().map(|chunk| chunk.len()).sum::<usize>() * 4 + 10;
    let mut trace = vec![];
    for chunk in chunks {
        execution
            .context_mut()
            .input
            .push(chunk.iter().copied())
            .unwrap();
        if !closed && drain(&mut execution, &mut trace, bound) {
            return trace; // Never feed/reconstruct/resume after a terminal stub.
        }
    }
    execution.context_mut().input.finish();
    assert!(drain(&mut execution, &mut trace, bound));
    trace
}

#[test]
fn every_two_chunk_split_and_single_point_chunks_match_closed_source_traces() {
    let mut inputs: Vec<_> = [
        "",
        "-",
        "-->",
        "-tail",
        ">",
        ">x\r\n\0",
        ">\r\r\n",
        ">\r",
        ">\0\0",
        ">&amp;",
        "><!---->",
        "\0tail",
        "<x",
        "&x",
        "\r\nX",
        "\r\r\n",
        "\r",
        "É😀",
        "\u{b}x",
        "\u{fdd0}x",
    ]
    .into_iter()
    .map(points)
    .collect();
    inputs.push(vec![
        CodePoint::new(0xd800).unwrap(),
        '\r'.into(),
        '\n'.into(),
    ]);
    let mut mixed = points(">😀\r\n");
    mixed.extend([
        CodePoint::new(0xdfff).unwrap(),
        CodePoint::new(0x10ffff).unwrap(),
        '\u{b}'.into(),
        '\0'.into(),
    ]);
    inputs.push(mixed);
    for input in &inputs {
        for prefix in [false, true] {
            let expected = trace(&[input], true, prefix);
            for split in 0..=input.len() {
                assert_eq!(
                    trace(&[&input[..split], &[], &input[split..]], false, prefix),
                    expected,
                    "split {split}, prefix {prefix}: {input:?}"
                );
            }
            assert_eq!(
                trace(&input.chunks(1).collect::<Vec<_>>(), false, prefix),
                expected,
                "single points, prefix {prefix}: {input:?}"
            );
        }
    }
}

#[test]
fn normal_data_comment_entry_is_still_terminally_blocked_before_comment_creation() {
    let mut context = Context::default();
    context.input.push(points("<!--->")).unwrap();
    context.input.finish();
    let mut execution = Execution::new(State::Data, context);
    assert_eq!(execution.step(), Ok(Progress::Continue(State::TagOpen)));
    assert_eq!(
        execution.step(),
        Ok(Progress::Continue(State::MarkupDeclarationOpen))
    );
    assert_eq!(
        execution.context().input.current(),
        Some((
            Input::CodePoint('!'.into()),
            Position { line: 1, column: 2 }
        ))
    );
    assert!(execution.context().pending_comment.is_none());
    assert!(execution.context().tokens.is_empty());
    assert!(execution.context().errors.is_empty());
    assert!(execution.context().input.errors().is_empty());
    terminal(
        &mut execution,
        Err(UnimplementedState {
            state: State::MarkupDeclarationOpen,
        }),
    );
}
