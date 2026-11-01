//! Local branch/streaming evidence, not upstream conformance runs.
//! Pinned WHATWG HTML Comment state, source 144561–144587:
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144561-L144587>

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

fn seeded() -> Context {
    let attribute = Attribute {
        name: points("kept"),
        value: points("value"),
    };
    let detached = Attribute {
        name: points("kept"),
        value: points("discarded"),
    };
    Context {
        tokens: vec![
            Token::Character('p'.into()),
            Token::Comment {
                data: points("prior"),
            },
        ],
        errors: vec![error("prior", 1, 1)],
        pending_comment: Some(PendingComment {
            // Already accumulated data is neither normalized nor diagnosed again.
            data: vec![
                '\0'.into(),
                '\r'.into(),
                CodePoint::new(0xd800).unwrap(),
                's'.into(),
            ],
        }),
        current_tag: Some(TagKind::End),
        pending_start_tag: Some(PendingStartTag {
            name: points("start"),
            self_closing: true,
            attributes: vec![attribute.clone()],
            discarded_current_attribute: Some(detached.clone()),
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("end"),
            self_closing: true,
            attributes: vec![attribute],
            discarded_current_attribute: Some(detached),
        }),
        last_start_tag: Some(points("title")),
        return_state: Some(State::Rcdata),
        temporary_buffer: points("scratch"),
        ..Context::default()
    }
}

fn isolated(input: &[CodePoint], closed: bool) -> Execution {
    let mut context = seeded();
    context.input.push(input.iter().copied()).unwrap();
    if closed {
        context.input.finish();
    }
    Execution::new(State::Comment, context)
}

#[derive(Debug, Eq, PartialEq)]
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

#[test]
fn every_ascii_branch_and_literal_non_scalar_preservation() {
    let values = (0..=0x7f).chain([
        0x80, 0x9f, 0xa0, 0xe9, 0x391, 0x301, 0x1f600, 0xd800, 0xdfff, 0xfdd0, 0x10ffff,
    ]);
    for value in values {
        for closed in [false, true] {
            for selector in [None, Some(TagKind::Start), Some(TagKind::End)] {
                let point = CodePoint::new(value).unwrap();
                let normalized = if value == 0x0d { '\n'.into() } else { point };
                let mut execution = isolated(&[point, 'x'.into()], closed);
                execution.context_mut().current_tag = selector;
                let mut expected = snapshot(&execution);
                let (target, appended) = match value {
                    0x3c => (State::CommentLessThanSign, Some(point)),
                    0x2d => (State::CommentEndDash, None),
                    0 => (State::Comment, Some('\u{fffd}'.into())),
                    _ => (State::Comment, Some(normalized)),
                };
                assert_eq!(
                    execution.step(),
                    Ok(Progress::Continue(target)),
                    "{value:x}"
                );
                expected.state = target;
                expected.current = Some((
                    Input::CodePoint(normalized),
                    Position { line: 1, column: 1 },
                ));
                if let Some(point) = appended {
                    expected.comment.as_mut().unwrap().data.push(point);
                }
                if value == 0 {
                    expected
                        .errors
                        .push(error("unexpected-null-character", 1, 1));
                }
                let preprocessing = match value {
                    0xd800 | 0xdfff => Some("surrogate-in-input-stream"),
                    0xfdd0 | 0x10ffff => Some("noncharacter-in-input-stream"),
                    0x01..=0x08 | 0x0b | 0x0e..=0x1f | 0x7f..=0x9f => {
                        Some("control-character-in-input-stream")
                    }
                    _ => None,
                };
                if let Some(code) = preprocessing {
                    expected.input_errors.push(error(code, 1, 1));
                }
                // Exact source effects, before any destination attempt.
                assert_eq!(snapshot(&execution), expected, "{value:x}");
                if target != State::Comment {
                    // Real destinations consume the NEXT x. End dash appends
                    // its withheld '-', less-than appends nothing; both replay x.
                    expected.state = State::Comment;
                    expected.current = Some((
                        Input::CodePoint('x'.into()),
                        Position { line: 1, column: 2 },
                    ));
                    if target == State::CommentEndDash {
                        expected.comment.as_mut().unwrap().data.push('-'.into());
                    }
                    assert_eq!(execution.step(), Ok(Progress::Continue(State::Comment)));
                    assert_eq!(snapshot(&execution), expected);
                    expected.comment.as_mut().unwrap().data.push('x'.into());
                    assert_eq!(execution.step(), Ok(Progress::Continue(State::Comment)));
                    assert_eq!(snapshot(&execution), expected);
                } else {
                    // No branch schedules outgoing reconsumption: the next point is x.
                    assert_eq!(execution.step(), Ok(Progress::Continue(State::Comment)));
                    expected.current = Some((
                        Input::CodePoint('x'.into()),
                        if normalized == '\n'.into() {
                            Position { line: 2, column: 1 }
                        } else {
                            Position { line: 1, column: 2 }
                        },
                    ));
                    expected.comment.as_mut().unwrap().data.push('x'.into());
                    assert_eq!(snapshot(&execution), expected);
                }
            }
        }
    }
}

#[test]
fn nulls_and_eof_have_exact_normalized_locations_and_comment_take_semantics() {
    let mut execution = isolated(&points("\r\n\0😀\0>&"), true);
    for (point, line, column) in [
        ('\n', 1, 1),
        ('\0', 2, 1),
        ('😀', 2, 2),
        ('\0', 2, 3),
        ('>', 2, 4),
        ('&', 2, 5),
    ] {
        let mut expected = snapshot(&execution);
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Comment)));
        expected.current = Some((Input::CodePoint(point.into()), Position { line, column }));
        expected
            .comment
            .as_mut()
            .unwrap()
            .data
            .push(if point == '\0' {
                '\u{fffd}'.into()
            } else {
                point.into()
            });
        if point == '\0' {
            expected
                .errors
                .push(error("unexpected-null-character", line, column));
        }
        assert_eq!(snapshot(&execution), expected);
    }
    let mut expected = snapshot(&execution);
    assert_eq!(execution.step(), Ok(Progress::Finished));
    expected.current = Some((Input::Eof, Position { line: 2, column: 6 }));
    expected.errors.push(error("eof-in-comment", 2, 6));
    let data = expected.comment.take().unwrap().data;
    let mut literal = seeded().pending_comment.unwrap().data;
    literal.extend(points("\n�😀�>&"));
    assert_eq!(data, literal);
    expected
        .tokens
        .extend([Token::Comment { data }, Token::Eof]);
    assert_eq!(snapshot(&execution), expected);
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(snapshot(&execution), expected);
}

#[test]
fn open_exhaustion_is_effect_free_and_only_explicit_eof_emits_even_empty_comments() {
    for initial_data in [vec![], seeded().pending_comment.unwrap().data] {
        for (input, line, column) in [
            ("", 1, 1),
            (">&", 1, 3),
            ("\0", 1, 2),
            ("\r\n", 2, 1),
            ("\r", 2, 1),
        ] {
            let mut execution = isolated(&points(input), false);
            execution.context_mut().pending_comment = Some(PendingComment {
                data: initial_data.clone(),
            });
            assert!(!drain(&mut execution, &mut vec![], input.len() + 2));
            let mut expected = snapshot(&execution);
            for _ in 0..2 {
                execution.context_mut().input.push([]).unwrap();
                assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                assert_eq!(snapshot(&execution), expected);
            }
            execution.context_mut().input.finish();
            assert_eq!(execution.step(), Ok(Progress::Finished));
            expected.current = Some((Input::Eof, Position { line, column }));
            expected.errors.push(error("eof-in-comment", line, column));
            expected.tokens.push(Token::Comment {
                data: expected.comment.take().unwrap().data,
            });
            expected.tokens.push(Token::Eof);
            assert_eq!(snapshot(&execution), expected);
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&execution), expected);
        }
    }
    // Minimal isolated entry needs only the initialized pending comment.
    let mut context = Context {
        pending_comment: Some(PendingComment::default()),
        ..Context::default()
    };
    context.input.finish();
    let mut execution = Execution::new(State::Comment, context);
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(
        execution.context().tokens,
        [Token::Comment { data: vec![] }, Token::Eof]
    );
    assert_eq!(execution.context().errors, [error("eof-in-comment", 1, 1)]);
    assert!(execution.context().pending_comment.is_none());
}

#[test]
fn incoming_reconsumption_including_eof_preserves_positions_and_once_only_preprocessing() {
    for value in [
        Some(0),
        Some(0x3c),
        Some(0x2d),
        Some(0x3e),
        Some(0xd800),
        Some(0xfdd0),
        Some(0x0b),
        None,
    ] {
        let input = value
            .map(|v| CodePoint::new(v).unwrap())
            .into_iter()
            .collect::<Vec<_>>();
        let mut context = seeded();
        context.input.push(points("\r\n")).unwrap();
        context.input.push(input.iter().copied()).unwrap();
        context.input.finish();
        assert_eq!(context.input.consume(), Input::CodePoint('\n'.into()));
        let current = context.input.consume();
        let position = context.input.position();
        assert_eq!(position, Position { line: 2, column: 1 });
        let preprocessing = context.input.errors().to_vec();
        context.input.reconsume().unwrap();
        let mut execution = Execution::new(State::Comment, context);
        let mut expected = snapshot(&execution);
        let target = match value {
            Some(0x3c) => State::CommentLessThanSign,
            Some(0x2d) => State::CommentEndDash,
            _ => State::Comment,
        };
        assert_eq!(
            execution.step(),
            Ok(if value.is_none() {
                Progress::Finished
            } else {
                Progress::Continue(target)
            })
        );
        expected.state = target;
        match current {
            Input::Eof => {
                expected.errors.push(error("eof-in-comment", 2, 1));
                expected.tokens.extend([
                    Token::Comment {
                        data: expected.comment.take().unwrap().data,
                    },
                    Token::Eof,
                ]);
            }
            Input::CodePoint(point) => {
                if value == Some(0) {
                    expected
                        .errors
                        .push(error("unexpected-null-character", 2, 1));
                    expected
                        .comment
                        .as_mut()
                        .unwrap()
                        .data
                        .push('\u{fffd}'.into());
                } else if value != Some(0x2d) {
                    expected.comment.as_mut().unwrap().data.push(point);
                }
            }
            Input::NeedMoreInput => unreachable!(),
        }
        assert_eq!(snapshot(&execution), expected);
        assert_eq!(execution.context().input.errors(), preprocessing);
        if target == State::Comment && value.is_some() {
            // The incoming item was reconsumed once; next is EOF, not that item again.
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(
                execution.context().errors.last(),
                Some(&error("eof-in-comment", 2, 2))
            );
            assert!(execution.context().pending_comment.is_none());
        }
    }
}

#[test]
fn real_destinations_suspend_then_handle_eof_and_data_entry_still_blocks_earlier() {
    for (input, target, suffix) in [
        ("<", State::CommentLessThanSign, "<"),
        ("-", State::CommentEndDash, ""),
    ] {
        let mut execution = isolated(&points(input), false);
        assert_eq!(execution.step(), Ok(Progress::Continue(target)));
        let mut expected_data = seeded().pending_comment.unwrap().data;
        expected_data.extend(points(suffix));
        assert_eq!(
            execution.context().pending_comment.as_ref().unwrap().data,
            expected_data
        );
        let mut expected = snapshot(&execution);
        for _ in 0..3 {
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&execution), expected);
        }
        execution.context_mut().input.finish();
        expected.current = Some((Input::Eof, Position { line: 1, column: 2 }));
        if target == State::CommentLessThanSign {
            expected.state = State::Comment;
            assert_eq!(execution.step(), Ok(Progress::Continue(State::Comment)));
            assert_eq!(snapshot(&execution), expected);
        }
        expected.errors.push(error("eof-in-comment", 1, 2));
        expected.tokens.extend([
            Token::Comment {
                data: expected.comment.take().unwrap().data,
            },
            Token::Eof,
        ]);
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&execution), expected);
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&execution), expected);
    }
    // Independent cursor probes immediately after the source step establish that
    // neither delimiter was scheduled for reconsumption and the tail was untouched.
    // These executions never attempt a destination or resume after a stub.
    for delimiter in ['<', '-'] {
        let tail = CodePoint::new(0xd800).unwrap();
        let mut execution = isolated(&[delimiter.into(), tail], true);
        let target = if delimiter == '<' {
            State::CommentLessThanSign
        } else {
            State::CommentEndDash
        };
        assert_eq!(execution.step(), Ok(Progress::Continue(target)));
        assert!(execution.context().input.errors().is_empty());
        assert_eq!(
            execution.context_mut().input.consume(),
            Input::CodePoint(tail)
        );
        assert_eq!(
            execution.context().input.position(),
            Position { line: 1, column: 2 }
        );
        assert_eq!(
            execution.context().input.errors(),
            [error("surrogate-in-input-stream", 1, 2)]
        );
        assert_eq!(execution.context_mut().input.consume(), Input::Eof);
        assert_eq!(
            execution.context().input.position(),
            Position { line: 1, column: 3 }
        );
    }
    let mut context = seeded();
    context.input.push(points("<!--comment-->")).unwrap();
    context.input.finish();
    let mut execution = Execution::new(State::Data, context);
    assert_eq!(execution.step(), Ok(Progress::Continue(State::TagOpen)));
    assert_eq!(
        execution.step(),
        Ok(Progress::Continue(State::MarkupDeclarationOpen))
    );
    let source = snapshot(&execution);
    assert_eq!(
        source.current,
        Some((
            Input::CodePoint('!'.into()),
            Position { line: 1, column: 2 }
        ))
    );
    assert_eq!(source.comment, seeded().pending_comment);
    assert_eq!(source.tokens, seeded().tokens);
    assert_eq!(
        execution.step(),
        Err(UnimplementedState {
            state: State::MarkupDeclarationOpen
        })
    );
    assert_eq!(snapshot(&execution), source);
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
        trace.push((result, snapshot(execution)));
        if !matches!(result, Ok(Progress::Continue(_))) {
            let terminal = snapshot(execution);
            assert_eq!(execution.step(), result);
            assert_eq!(snapshot(execution), terminal);
            return true;
        }
    }
    panic!("Comment did not consume, suspend, or terminate");
}

#[test]
fn every_two_chunk_split_and_single_point_chunks_match_closed_source_traces() {
    let mut samples = [
        "",
        "<",
        "-",
        ">&!",
        "\0\0",
        "😀É\0>&",
        "\r\n\0😀\0>&",
        "\r\r\n",
        "\r",
        "literal<\0tail",
        "literal-\0tail",
        "<!--nested-->",
        "&amp;",
        "\r\n<tail",
        "\r\n-tail",
    ]
    .into_iter()
    .map(points)
    .collect::<Vec<_>>();
    let mut non_scalar = [0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x0b, 0x7f]
        .map(|v| CodePoint::new(v).unwrap())
        .to_vec();
    non_scalar.extend(points("\r\n\0😀>\r"));
    samples.push(non_scalar.clone());
    non_scalar.extend(points("<tail"));
    samples.push(non_scalar);
    for input in samples {
        let run = |chunks: Vec<&[CodePoint]>, closed: bool| {
            let mut execution = isolated(&[], false);
            let mut trace = vec![];
            let bound = input.len() + 3;
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
                    // Stop permanently at a stub, with no further feeding or reconstruction.
                    if drain(&mut execution, &mut trace, bound) {
                        return trace;
                    }
                    execution.context_mut().input.push([]).unwrap();
                    assert!(!drain(&mut execution, &mut trace, 1));
                }
                execution.context_mut().input.finish();
            }
            assert!(drain(&mut execution, &mut trace, bound));
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
        assert_eq!(
            run(input.chunks(1).collect(), false),
            expected,
            "single-point {input:?}"
        );
    }
}

// Integration after PRs #47–56: isolated entries only; normal Data entry is still
// blocked at MarkupDeclarationOpen. These expectations are literal pinned-spec
// outcomes, not snapshots blessed from a production run.
struct IntegratedCase {
    initial: State,
    input: Vec<CodePoint>,
    appended: Vec<CodePoint>,
    state: State,
    current: (Input, Position),
    errors: Vec<ParseError>,
    input_errors: Vec<ParseError>,
    emitted: bool,
    characters: Vec<CodePoint>,
    blocked: bool,
}

fn integrated_entry(initial: State) -> Execution {
    let mut context = seeded();
    // Context supplied by the already consumed predecessor, not an executed or
    // reconstructed blocked run. Withheld hyphens are NOT in pending data.
    let prefix = match initial {
        State::CommentLessThanSign => "<",
        State::CommentLessThanSignBang
        | State::CommentLessThanSignBangDash
        | State::CommentLessThanSignBangDashDash => "<!",
        _ => "",
    };
    context
        .pending_comment
        .as_mut()
        .unwrap()
        .data
        .extend(points(prefix));
    Execution::new(initial, context)
}

fn assert_integrated_outcome(execution: &Execution, case: &IntegratedCase) {
    let mut expected = snapshot(&integrated_entry(case.initial));
    expected.state = case.state;
    expected.current = Some(case.current);
    expected.errors.extend(case.errors.clone());
    expected.input_errors = case.input_errors.clone();
    if case.state == State::CharacterReference {
        expected.return_state = Some(State::Data);
    }
    if case.state == State::ProcessingInstructionOpen {
        expected.buffer.clear();
    }
    expected
        .comment
        .as_mut()
        .unwrap()
        .data
        .extend(case.appended.clone());
    if case.emitted {
        expected.tokens.push(Token::Comment {
            data: expected.comment.take().unwrap().data,
        });
        expected
            .tokens
            .extend(case.characters.iter().copied().map(Token::Character));
    }
    if !case.blocked {
        expected.tokens.push(Token::Eof);
    }
    assert_eq!(snapshot(execution), expected);
}

fn integrated_run(case: &IntegratedCase, chunks: Vec<&[CodePoint]>, closed: bool) -> Trace {
    let mut execution = integrated_entry(case.initial);
    let mut trace = vec![];
    if closed {
        execution
            .context_mut()
            .input
            .push(case.input.iter().copied())
            .unwrap();
        execution.context_mut().input.finish();
    } else {
        for chunk in chunks {
            execution
                .context_mut()
                .input
                .push(chunk.iter().copied())
                .unwrap();
            if drain(&mut execution, &mut trace, case.input.len() * 5 + 10) {
                assert_integrated_outcome(&execution, case);
                assert_eq!(
                    trace.last().unwrap().0,
                    Err(UnimplementedState { state: case.state })
                );
                assert!(case.blocked);
                return trace; // No feeding, reconstruction or resumption at a stub.
            }
        }
        execution.context_mut().input.finish();
    }
    assert!(drain(&mut execution, &mut trace, case.input.len() * 5 + 10));
    assert_integrated_outcome(&execution, case);
    assert_eq!(
        trace.last().unwrap().0,
        if case.blocked {
            Err(UnimplementedState { state: case.state })
        } else {
            Ok(Progress::Finished)
        }
    );
    trace
}

#[test]
fn integrated_literal_loops_fallbacks_closures_and_genuine_blocks_match_independent_outcomes_at_every_split()
 {
    use State::*;
    let mut cases = vec![];
    // (entry, raw input, appended data, EOF state, EOF line/column, earlier errors)
    for (initial, input, appended, state, line, column, mut errors) in [
        (Comment, "", "", Comment, 1, 1, vec![]),
        (Comment, "a>&!", "a>&!", Comment, 1, 5, vec![]),
        (Comment, "<x", "<x", Comment, 1, 3, vec![]),
        (Comment, "<<!x", "<<!x", Comment, 1, 5, vec![]),
        (Comment, "<!-x", "<!-x", Comment, 1, 5, vec![]),
        (Comment, "<!-x-", "<!-x", CommentEndDash, 1, 6, vec![]),
        (Comment, "a-b", "a-b", Comment, 1, 4, vec![]),
        (Comment, "a-", "a", CommentEndDash, 1, 3, vec![]),
        (Comment, "--", "", CommentEnd, 1, 3, vec![]),
        (Comment, "---", "-", CommentEnd, 1, 4, vec![]),
        (Comment, "--!", "", CommentEndBang, 1, 4, vec![]),
        (Comment, "--!-", "--!", CommentEndDash, 1, 5, vec![]),
        (Comment, "--!--", "--!", CommentEnd, 1, 6, vec![]),
        (Comment, "--!--x", "--!--x", Comment, 1, 7, vec![]),
        (Comment, "--!!", "--!!", Comment, 1, 5, vec![]),
        (
            Comment,
            "--!-x--!-",
            "--!-x--!",
            CommentEndDash,
            1,
            10,
            vec![],
        ),
        (
            Comment,
            "a--\0",
            "a--�",
            Comment,
            1,
            5,
            vec![error("unexpected-null-character", 1, 4)],
        ),
        (
            Comment,
            "--!\0",
            "--!�",
            Comment,
            1,
            5,
            vec![error("unexpected-null-character", 1, 4)],
        ),
        (
            Comment,
            "--\r\n\0",
            "--\n�",
            Comment,
            2,
            2,
            vec![error("unexpected-null-character", 2, 1)],
        ),
        (CommentStartDash, "-x", "--x", Comment, 1, 3, vec![]),
        (CommentEnd, "!->", "--!->", Comment, 1, 4, vec![]),
        (Comment, "<!-", "<!", CommentEndDash, 1, 4, vec![]),
        (
            Comment,
            "x<\0-\0",
            "x<�-�",
            Comment,
            1,
            6,
            vec![
                error("unexpected-null-character", 1, 3),
                error("unexpected-null-character", 1, 5),
            ],
        ),
        (Comment, "\r\n<\r\n-\r", "\n<\n-\n", Comment, 4, 1, vec![]),
        (CommentStart, "abc", "abc", Comment, 1, 4, vec![]),
        (CommentStart, "-x", "-x", Comment, 1, 3, vec![]),
        (CommentStart, "", "", Comment, 1, 1, vec![]),
        (
            CommentStartDash,
            "\0",
            "-�",
            Comment,
            1,
            2,
            vec![error("unexpected-null-character", 1, 1)],
        ),
        (CommentLessThanSign, "", "", Comment, 1, 1, vec![]),
        (CommentLessThanSign, "x", "x", Comment, 1, 2, vec![]),
        (CommentLessThanSignBang, "", "", Comment, 1, 1, vec![]),
        (CommentLessThanSignBang, "x", "x", Comment, 1, 2, vec![]),
        (
            CommentLessThanSignBangDash,
            "",
            "",
            CommentEndDash,
            1,
            1,
            vec![],
        ),
        (
            CommentLessThanSignBangDash,
            "x",
            "-x",
            Comment,
            1,
            2,
            vec![],
        ),
        (CommentEndDash, ">", "->", Comment, 1, 2, vec![]),
        (CommentEndBang, "-x", "--!-x", Comment, 1, 3, vec![]),
        (CommentEndBang, "-", "--!", CommentEndDash, 1, 2, vec![]),
        (
            CommentEndBang,
            "\0",
            "--!�",
            Comment,
            1,
            2,
            vec![error("unexpected-null-character", 1, 1)],
        ),
    ] {
        errors.push(error("eof-in-comment", line, column));
        cases.push(IntegratedCase {
            initial,
            input: points(input),
            appended: points(appended),
            state,
            current: (Input::Eof, Position { line, column }),
            errors,
            input_errors: vec![],
            emitted: true,
            characters: vec![],
            blocked: false,
        });
    }
    // Start/StartDash abrupt close and EndBang incorrect close retain their
    // original diagnostics. Core closure via End is independently covered below.
    for (initial, input, characters, line, column, errors) in [
        (
            CommentStart,
            ">",
            "",
            1,
            2,
            vec![error("abrupt-closing-of-empty-comment", 1, 1)],
        ),
        (
            CommentStart,
            "->z\0",
            "z\0",
            1,
            5,
            vec![
                error("abrupt-closing-of-empty-comment", 1, 2),
                error("unexpected-null-character", 1, 4),
            ],
        ),
        (
            CommentStartDash,
            ">z\0",
            "z\0",
            1,
            4,
            vec![
                error("abrupt-closing-of-empty-comment", 1, 1),
                error("unexpected-null-character", 1, 3),
            ],
        ),
        (
            CommentEndBang,
            ">z\0",
            "z\0",
            1,
            4,
            vec![
                error("incorrectly-closed-comment", 1, 1),
                error("unexpected-null-character", 1, 3),
            ],
        ),
        (
            CommentEndBang,
            ">\r\n",
            "\n",
            2,
            1,
            vec![error("incorrectly-closed-comment", 1, 1)],
        ),
    ] {
        cases.push(IntegratedCase {
            initial,
            input: points(input),
            appended: vec![],
            state: Data,
            current: (Input::Eof, Position { line, column }),
            errors,
            input_errors: vec![],
            emitted: true,
            characters: points(characters),
            blocked: false,
        });
    }
    // Former nested-opener blocks now execute BangDashDash and real End.
    // The second nested '-' is withheld; only extra '-' points append in End.
    for (initial, input, appended, state, column, errors) in [
        (Comment, "<!-->", "<!", Data, 6, vec![]),
        (
            Comment,
            "<<!---->",
            "<<!--",
            Data,
            9,
            vec![error("nested-comment", 1, 6)],
        ),
        (
            CommentLessThanSignBangDash,
            "-",
            "",
            CommentEnd,
            2,
            vec![error("eof-in-comment", 1, 2)],
        ),
        (
            CommentEndBang,
            "<!--",
            "--!<!",
            CommentEnd,
            5,
            vec![error("eof-in-comment", 1, 5)],
        ),
        (CommentLessThanSignBangDashDash, ">", "", Data, 2, vec![]),
    ] {
        cases.push(IntegratedCase {
            initial,
            input: points(input),
            appended: points(appended),
            state,
            current: (Input::Eof, Position { line: 1, column }),
            errors,
            input_errors: vec![],
            emitted: true,
            characters: vec![],
            blocked: false,
        });
    }
    // Independently expected closing outcomes through the newly implemented End:
    // withheld delimiters are excluded unless restored by fallback/bang loops.
    for (initial, input, appended, characters, column, errors) in [
        (Comment, "-->", "", "", 4, vec![]),
        (CommentStart, "-->", "", "", 4, vec![]),
        (CommentStartDash, "->", "", "", 3, vec![]),
        (CommentEndDash, "->", "", "", 3, vec![]),
        (CommentEnd, ">", "", "", 2, vec![]),
        (CommentEndBang, "-->", "--!", "", 4, vec![]),
        (CommentLessThanSign, "x-->", "x", "", 5, vec![]),
        (CommentLessThanSignBang, "x-->", "x", "", 5, vec![]),
        (CommentLessThanSignBangDash, "x-->", "-x", "", 5, vec![]),
        (Comment, "--->", "-", "", 5, vec![]),
        (Comment, "--!-->", "--!", "", 7, vec![]),
        (Comment, "--!-x-->", "--!-x", "", 9, vec![]),
        (Comment, "--!x-->", "--!x", "", 8, vec![]),
        (CommentEnd, "!->-->", "--!->", "", 7, vec![]),
        (
            Comment,
            "--!>",
            "",
            "",
            5,
            vec![error("incorrectly-closed-comment", 1, 4)],
        ),
        (
            CommentStart,
            "--!>",
            "",
            "",
            5,
            vec![error("incorrectly-closed-comment", 1, 4)],
        ),
        (
            Comment,
            "--!--!>",
            "--!",
            "",
            8,
            vec![error("incorrectly-closed-comment", 1, 7)],
        ),
        (
            Comment,
            "text-->z\0",
            "text",
            "z\0",
            10,
            vec![error("unexpected-null-character", 1, 9)],
        ),
    ] {
        cases.push(IntegratedCase {
            initial,
            input: points(input),
            appended: points(appended),
            state: Data,
            current: (Input::Eof, Position { line: 1, column }),
            errors,
            input_errors: vec![],
            emitted: true,
            characters: points(characters),
            blocked: false,
        });
    }
    for (input, state, current, column, appended, characters, errors) in [
        ("x-->&", CharacterReference, '&', 5, "x", "", vec![]),
        ("x--><!", MarkupDeclarationOpen, '!', 6, "x", "", vec![]),
        ("x--><?", ProcessingInstructionOpen, '?', 6, "x", "", vec![]),
        (
            "--!> \0<!",
            MarkupDeclarationOpen,
            '!',
            8,
            "",
            " \0",
            vec![
                error("incorrectly-closed-comment", 1, 4),
                error("unexpected-null-character", 1, 6),
            ],
        ),
    ] {
        cases.push(IntegratedCase {
            initial: Comment,
            input: points(input),
            appended: points(appended),
            state,
            current: (
                Input::CodePoint(current.into()),
                Position { line: 1, column },
            ),
            errors,
            input_errors: vec![],
            emitted: true,
            characters: points(characters),
            blocked: true,
        });
    }
    // A matching emitted comment prefix never hides a later genuine block.
    for (initial, input, state, current, code, characters) in [
        (
            CommentEndBang,
            ">&",
            CharacterReference,
            '&',
            "incorrectly-closed-comment",
            "",
        ),
        (
            CommentStart,
            "> <!",
            MarkupDeclarationOpen,
            '!',
            "abrupt-closing-of-empty-comment",
            " ",
        ),
    ] {
        cases.push(IntegratedCase {
            initial,
            input: points(input),
            appended: vec![],
            state,
            current: (
                Input::CodePoint(current.into()),
                Position {
                    line: 1,
                    column: input.len(),
                },
            ),
            errors: vec![error(code, 1, 1)],
            input_errors: vec![],
            emitted: true,
            characters: points(characters),
            blocked: true,
        });
    }
    let surrogate = CodePoint::new(0xd800).unwrap();
    let noncharacter = CodePoint::new(0xfdd0).unwrap();
    let mut end_bang_input = points("--\0--!");
    end_bang_input.push(surrogate);
    end_bang_input.extend(points("\r\n--!>\u{b}"));
    let mut end_bang_data = points("--�--!");
    end_bang_data.extend([surrogate, '\n'.into()]);
    cases.push(IntegratedCase {
        initial: Comment,
        input: end_bang_input,
        appended: end_bang_data,
        state: Data,
        current: (Input::Eof, Position { line: 2, column: 6 }),
        errors: vec![
            error("unexpected-null-character", 1, 3),
            error("incorrectly-closed-comment", 2, 4),
        ],
        input_errors: vec![
            error("surrogate-in-input-stream", 1, 7),
            error("control-character-in-input-stream", 2, 5),
        ],
        emitted: true,
        characters: points("\u{b}"),
        blocked: false,
    });
    cases.push(IntegratedCase {
        initial: Comment,
        input: vec![
            'a'.into(),
            '<'.into(),
            surrogate,
            '-'.into(),
            noncharacter,
            '\u{b}'.into(),
            '\0'.into(),
            '>'.into(),
            '\r'.into(),
            '\n'.into(),
        ],
        appended: vec![
            'a'.into(),
            '<'.into(),
            surrogate,
            '-'.into(),
            noncharacter,
            '\u{b}'.into(),
            '�'.into(),
            '>'.into(),
            '\n'.into(),
        ],
        state: Comment,
        current: (Input::Eof, Position { line: 2, column: 1 }),
        errors: vec![
            error("unexpected-null-character", 1, 7),
            error("eof-in-comment", 2, 1),
        ],
        input_errors: vec![
            error("surrogate-in-input-stream", 1, 3),
            error("noncharacter-in-input-stream", 1, 5),
            error("control-character-in-input-stream", 1, 6),
        ],
        emitted: true,
        characters: vec![],
        blocked: false,
    });
    for case in cases {
        let expected = integrated_run(&case, vec![], true);
        for split in 0..=case.input.len() {
            assert_eq!(
                integrated_run(
                    &case,
                    vec![&case.input[..split], &[], &case.input[split..]],
                    false
                ),
                expected,
                "{:?} {:?} split {split}",
                case.initial,
                case.input
            );
        }
        assert_eq!(
            integrated_run(&case, case.input.chunks(1).collect(), false),
            expected
        );
    }
}

#[test]
fn integrated_bang_dash_fallback_and_end_dash_eof_have_independent_source_step_snapshots() {
    let mut execution = integrated_entry(State::Comment);
    execution.context_mut().input.push(points("<!-x-")).unwrap();
    execution.context_mut().input.finish();
    let mut expected = snapshot(&execution);
    for (state, point, column, appended) in [
        (State::CommentLessThanSign, '<', 1, "<"),
        (State::CommentLessThanSignBang, '!', 2, "!"),
        (State::CommentLessThanSignBangDash, '-', 3, ""),
        (State::CommentEndDash, 'x', 4, ""),
        (State::Comment, 'x', 4, "-"),
        (State::Comment, 'x', 4, "x"),
        (State::CommentEndDash, '-', 5, ""),
    ] {
        expected.state = state;
        expected.current = Some((Input::CodePoint(point.into()), Position { line: 1, column }));
        expected
            .comment
            .as_mut()
            .unwrap()
            .data
            .extend(points(appended));
        assert_eq!(execution.step(), Ok(Progress::Continue(state)));
        assert_eq!(snapshot(&execution), expected);
    }
    expected.current = Some((Input::Eof, Position { line: 1, column: 6 }));
    expected.errors.push(error("eof-in-comment", 1, 6));
    let mut data = seeded().pending_comment.unwrap().data;
    data.extend(points("<!-x")); // The trailing withheld '-' is not appended at EOF.
    assert_eq!(expected.comment.take().unwrap().data, data);
    expected
        .tokens
        .extend([Token::Comment { data }, Token::Eof]);
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(snapshot(&execution), expected);
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(snapshot(&execution), expected);
}
