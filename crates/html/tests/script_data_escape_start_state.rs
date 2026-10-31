//! Local evidence for HTML 404b3eb14a7a936dddefb2d78e224dbeb01d2358,
//! #script-data-escape-start-state, source lines 143741–143755.
//! These source-step/streaming checks are not upstream conformance cases.

use neptune_html::tokenizer::{
    Attribute, CodePoint, Context, CursorError, Execution, Input, ParseError, PendingComment,
    PendingEndTag, PendingStartTag, Position, Progress, State, TagKind, Token, UnimplementedState,
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

fn seeded_context() -> Context {
    let attribute = Attribute {
        name: points("name"),
        value: vec![CodePoint::new(0xd800).unwrap(), '\0'.into()],
    };
    Context {
        tokens: vec![Token::Character('p'.into())],
        errors: vec![error("prior-diagnostic", 1, 1)],
        last_start_tag: Some(points("script")),
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
        pending_comment: Some(PendingComment {
            data: points("comment"),
        }),
        temporary_buffer: vec!['A'.into(), CodePoint::new(0xdfff).unwrap(), '\0'.into()],
        ..Default::default()
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Storage {
    last: Option<Vec<CodePoint>>,
    return_state: Option<State>,
    selected: Option<TagKind>,
    start: Option<PendingStartTag>,
    end: Option<PendingEndTag>,
    comment: Option<PendingComment>,
    buffer: Vec<CodePoint>,
}

fn storage(context: &Context) -> Storage {
    Storage {
        last: context.last_start_tag.clone(),
        return_state: context.return_state,
        selected: context.current_tag,
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
    storage: Storage,
}

fn snapshot(execution: &Execution) -> Snapshot {
    let context = execution.context();
    Snapshot {
        state: execution.state(),
        current: context.input.current(),
        tokens: context.tokens.clone(),
        errors: context.errors.clone(),
        input_errors: context.input.errors().to_vec(),
        storage: storage(context),
    }
}

// Isolated entry positions the cursor after <! without simulating sibling states.
fn entry(mut context: Context) -> Execution {
    context.input.push(points("\r\n<!")).unwrap();
    for point in points("\n<!") {
        assert_eq!(context.input.consume(), Input::CodePoint(point));
    }
    Execution::new(State::ScriptDataEscapeStart, context)
}

fn terminal(execution: &mut Execution, result: Result<Progress, UnimplementedState>) {
    let before = snapshot(execution);
    for _ in 0..3 {
        assert_eq!(execution.step(), result);
        assert_eq!(snapshot(execution), before);
    }
}

#[test]
fn every_ascii_branch_and_non_scalar_preserves_storage_and_exact_source_effects() {
    for value in (0..=0x7f).chain([
        0x80, 0x9f, 0xa0, 0xc9, 0x3a9, 0x2010, 0x2212, 0xfeff, 0x1f600, 0xd800, 0xdfff, 0xfdd0,
        0xffff, 0x10ffff,
    ]) {
        for finished in [false, true] {
            for selected in [None, Some(TagKind::Start), Some(TagKind::End)] {
                let mut context = seeded_context();
                context.current_tag = selected;
                let mut run = entry(context);
                let point = CodePoint::new(value).unwrap();
                let normalized = if value == 0x0d { '\n'.into() } else { point };
                run.context_mut().input.push([point, 'z'.into()]).unwrap();
                if finished {
                    run.context_mut().input.finish();
                }
                let mut expected = snapshot(&run);
                expected.current = Some((
                    Input::CodePoint(normalized),
                    Position { line: 2, column: 3 },
                ));
                let diagnostic = match value {
                    0xd800..=0xdfff => Some("surrogate-in-input-stream"),
                    0xfdd0 | 0xffff | 0x10ffff => Some("noncharacter-in-input-stream"),
                    1..=8 | 0x0b | 0x0e..=0x1f | 0x7f..=0x9f => {
                        Some("control-character-in-input-stream")
                    }
                    _ => None,
                };
                expected.input_errors = diagnostic
                    .into_iter()
                    .map(|code| error(code, 2, 3))
                    .collect();
                if value == 0x2d {
                    expected.state = State::ScriptDataEscapeStartDash;
                    expected.tokens.push(Token::Character('-'.into()));
                } else {
                    expected.state = State::ScriptData;
                }
                assert_eq!(run.step(), Ok(Progress::Continue(expected.state)));
                assert_eq!(snapshot(&run), expected, "U+{value:04X}");
                if value == 0x2d {
                    // Escape start dash consumes the untouched z, then Script
                    // data reconsumes it. The first hyphen was not reconsumed.
                    expected.state = State::ScriptData;
                    expected.current = Some((
                        Input::CodePoint('z'.into()),
                        Position { line: 2, column: 4 },
                    ));
                    assert_eq!(run.step(), Ok(Progress::Continue(State::ScriptData)));
                    assert_eq!(snapshot(&run), expected);
                    assert_eq!(
                        run.context_mut().input.reconsume(),
                        Err(CursorError::AlreadyReconsuming)
                    );
                    expected.tokens.push(Token::Character('z'.into()));
                    assert_eq!(run.step(), Ok(Progress::Continue(State::ScriptData)));
                    assert_eq!(snapshot(&run), expected);
                    if !finished {
                        assert_eq!(run.step(), Ok(Progress::NeedMoreInput));
                        assert_eq!(snapshot(&run), expected);
                        run.context_mut().input.finish();
                    }
                    expected.current = Some((Input::Eof, Position { line: 2, column: 5 }));
                    expected.tokens.push(Token::Eof);
                    assert_eq!(run.step(), Ok(Progress::Finished));
                    assert_eq!(snapshot(&run), expected);
                    terminal(&mut run, Ok(Progress::Finished));
                    continue;
                }
                assert_eq!(
                    run.context_mut().input.reconsume(),
                    Err(CursorError::AlreadyReconsuming)
                );
                // Replacement/diagnostics and '<' handling belong to Script data, not this source.
                if value == 0x3c {
                    expected.state = State::ScriptDataLessThanSign;
                } else if value == 0 {
                    expected.tokens.push(Token::Character('\u{fffd}'.into()));
                    expected
                        .errors
                        .push(error("unexpected-null-character", 2, 3));
                } else {
                    expected.tokens.push(Token::Character(normalized));
                }
                assert_eq!(run.step(), Ok(Progress::Continue(expected.state)));
                assert_eq!(snapshot(&run), expected);
                let tail_position = if normalized == '\n'.into() {
                    Position { line: 3, column: 1 }
                } else {
                    Position { line: 2, column: 4 }
                };
                expected.current = Some((Input::CodePoint('z'.into()), tail_position));
                if value == 0x3c {
                    expected.state = State::ScriptData;
                    expected.tokens.push(Token::Character('<'.into()));
                    assert_eq!(run.step(), Ok(Progress::Continue(State::ScriptData)));
                    assert_eq!(snapshot(&run), expected);
                }
                expected.tokens.push(Token::Character('z'.into()));
                assert_eq!(run.step(), Ok(Progress::Continue(State::ScriptData)));
                assert_eq!(snapshot(&run), expected);
                if !finished {
                    for _ in 0..3 {
                        assert_eq!(run.step(), Ok(Progress::NeedMoreInput));
                        assert_eq!(snapshot(&run), expected);
                    }
                    run.context_mut().input.finish();
                }
                expected.current = Some((
                    Input::Eof,
                    Position {
                        line: tail_position.line,
                        column: tail_position.column + 1,
                    },
                ));
                expected.tokens.push(Token::Eof);
                assert_eq!(run.step(), Ok(Progress::Finished));
                assert_eq!(snapshot(&run), expected);
                terminal(&mut run, Ok(Progress::Finished));
            }
        }
    }
}

#[test]
fn conceptual_eof_reconsumes_but_open_exhaustion_has_no_effects() {
    for seeded in [false, true] {
        let mut run = entry(if seeded {
            seeded_context()
        } else {
            Context::default()
        });
        let mut expected = snapshot(&run);
        for _ in 0..3 {
            run.context_mut().input.push([]).unwrap();
            assert_eq!(run.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&run), expected);
        }
        run.context_mut().input.finish();
        expected.state = State::ScriptData;
        expected.current = Some((Input::Eof, Position { line: 2, column: 3 }));
        assert_eq!(run.step(), Ok(Progress::Continue(State::ScriptData)));
        assert_eq!(snapshot(&run), expected);
        assert_eq!(
            run.context_mut().input.reconsume(),
            Err(CursorError::AlreadyReconsuming)
        );
        expected.tokens.push(Token::Eof);
        assert_eq!(run.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&run), expected);
        terminal(&mut run, Ok(Progress::Finished));
    }
}

#[test]
fn empty_entry_suspends_then_hyphen_emits_before_real_destination_suspension_or_eof() {
    for finished in [false, true] {
        let mut run = Execution::new(State::ScriptDataEscapeStart, Context::default());
        let before = snapshot(&run);
        assert_eq!(before.current, None);
        for _ in 0..3 {
            assert_eq!(run.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&run), before);
        }
        run.context_mut().input.push(points("-")).unwrap();
        if finished {
            run.context_mut().input.finish();
        }
        assert_eq!(
            run.step(),
            Ok(Progress::Continue(State::ScriptDataEscapeStartDash))
        );
        assert_eq!(run.context().tokens, [Token::Character('-'.into())]);
        assert_eq!(
            run.context().input.current(),
            Some((
                Input::CodePoint('-'.into()),
                Position { line: 1, column: 1 }
            ))
        );
        assert_eq!(storage(run.context()), storage(&Context::default()));
        assert!(run.context().errors.is_empty());
        assert!(run.context().input.errors().is_empty());
        let mut expected = snapshot(&run);
        if !finished {
            for _ in 0..3 {
                assert_eq!(run.step(), Ok(Progress::NeedMoreInput));
                assert_eq!(snapshot(&run), expected);
            }
            run.context_mut().input.finish();
        }
        expected.state = State::ScriptData;
        expected.current = Some((Input::Eof, Position { line: 1, column: 2 }));
        assert_eq!(run.step(), Ok(Progress::Continue(State::ScriptData)));
        assert_eq!(snapshot(&run), expected);
        assert_eq!(
            run.context_mut().input.reconsume(),
            Err(CursorError::AlreadyReconsuming)
        );
        expected.tokens.push(Token::Eof);
        assert_eq!(run.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&run), expected);
        terminal(&mut run, Ok(Progress::Finished));
    }
}

#[test]
fn real_entry_has_exact_source_steps_through_escaped_cycles_and_greater_than_return() {
    let mut context = seeded_context();
    let input = [
        points("\r\n<!---x-\0"),
        vec![
            CodePoint::new(0xd800).unwrap(),
            CodePoint::new(0xfdd0).unwrap(),
            '\u{b}'.into(),
        ],
        points("-->\0"),
        vec![
            CodePoint::new(0xd800).unwrap(),
            CodePoint::new(0xfdd0).unwrap(),
            '\u{b}'.into(),
        ],
    ]
    .concat();
    context.input.push(input).unwrap();
    context.input.finish();
    let mut run = Execution::new(State::ScriptData, context);
    let mut expected = snapshot(&run);
    // Fixed rows derived from the pinned branches, not from production dispatch.
    // Input, normalized line/column, target, emitted points, state/input diagnostic.
    type Row = (
        u32,
        usize,
        usize,
        State,
        &'static [u32],
        Option<&'static str>,
        Option<&'static str>,
    );
    let rows: &[Row] = &[
        (0x0a, 1, 1, State::ScriptData, &[0x0a], None, None),
        (0x3c, 2, 1, State::ScriptDataLessThanSign, &[], None, None),
        (
            0x21,
            2,
            2,
            State::ScriptDataEscapeStart,
            &[0x3c, 0x21],
            None,
            None,
        ),
        (
            0x2d,
            2,
            3,
            State::ScriptDataEscapeStartDash,
            &[0x2d],
            None,
            None,
        ),
        (
            0x2d,
            2,
            4,
            State::ScriptDataEscapedDashDash,
            &[0x2d],
            None,
            None,
        ),
        (
            0x2d,
            2,
            5,
            State::ScriptDataEscapedDashDash,
            &[0x2d],
            None,
            None,
        ),
        (0x78, 2, 6, State::ScriptDataEscaped, &[0x78], None, None),
        (
            0x2d,
            2,
            7,
            State::ScriptDataEscapedDash,
            &[0x2d],
            None,
            None,
        ),
        (
            0,
            2,
            8,
            State::ScriptDataEscaped,
            &[0xfffd],
            Some("unexpected-null-character"),
            None,
        ),
        (
            0xd800,
            2,
            9,
            State::ScriptDataEscaped,
            &[0xd800],
            None,
            Some("surrogate-in-input-stream"),
        ),
        (
            0xfdd0,
            2,
            10,
            State::ScriptDataEscaped,
            &[0xfdd0],
            None,
            Some("noncharacter-in-input-stream"),
        ),
        (
            0x0b,
            2,
            11,
            State::ScriptDataEscaped,
            &[0x0b],
            None,
            Some("control-character-in-input-stream"),
        ),
        (
            0x2d,
            2,
            12,
            State::ScriptDataEscapedDash,
            &[0x2d],
            None,
            None,
        ),
        (
            0x2d,
            2,
            13,
            State::ScriptDataEscapedDashDash,
            &[0x2d],
            None,
            None,
        ),
        (0x3e, 2, 14, State::ScriptData, &[0x3e], None, None),
        (
            0,
            2,
            15,
            State::ScriptData,
            &[0xfffd],
            Some("unexpected-null-character"),
            None,
        ),
        (
            0xd800,
            2,
            16,
            State::ScriptData,
            &[0xd800],
            None,
            Some("surrogate-in-input-stream"),
        ),
        (
            0xfdd0,
            2,
            17,
            State::ScriptData,
            &[0xfdd0],
            None,
            Some("noncharacter-in-input-stream"),
        ),
        (
            0x0b,
            2,
            18,
            State::ScriptData,
            &[0x0b],
            None,
            Some("control-character-in-input-stream"),
        ),
    ];
    for &(value, line, column, state, emitted, state_error, input_error) in rows {
        expected.state = state;
        expected.current = Some((
            Input::CodePoint(CodePoint::new(value).unwrap()),
            Position { line, column },
        ));
        expected.tokens.extend(
            emitted
                .iter()
                .map(|&value| Token::Character(CodePoint::new(value).unwrap())),
        );
        if let Some(code) = state_error {
            expected.errors.push(error(code, line, column));
        }
        if let Some(code) = input_error {
            expected.input_errors.push(error(code, line, column));
        }
        assert_eq!(run.step(), Ok(Progress::Continue(state)));
        assert_eq!(snapshot(&run), expected);
    }
    expected.current = Some((
        Input::Eof,
        Position {
            line: 2,
            column: 19,
        },
    ));
    expected.tokens.push(Token::Eof);
    assert_eq!(run.step(), Ok(Progress::Finished));
    assert_eq!(snapshot(&run), expected);
    terminal(&mut run, Ok(Progress::Finished));
}

#[test]
fn real_entry_distinguishes_ordinary_eof_from_each_escaped_eof_state() {
    for (input, state, column, escaped) in [
        ("<!", State::ScriptData, 3, false),
        ("<!-", State::ScriptData, 4, false),
        ("<!-->", State::ScriptData, 6, false),
        ("<!--", State::ScriptDataEscapedDashDash, 5, true),
        ("<!--x", State::ScriptDataEscaped, 6, true),
        ("<!--x-", State::ScriptDataEscapedDash, 7, true),
        ("<!--x--", State::ScriptDataEscapedDashDash, 8, true),
    ] {
        let mut context = seeded_context();
        context.input.push(points(input)).unwrap();
        context.input.finish();
        let mut run = Execution::new(State::ScriptData, context);
        let mut expected = snapshot(&run);
        let mut trace = vec![];
        assert!(drain(&mut run, &mut trace));
        assert_eq!(trace.last().unwrap().0, Ok(Progress::Finished));
        expected.state = state;
        expected.current = Some((Input::Eof, Position { line: 1, column }));
        expected
            .tokens
            .extend(points(input).into_iter().map(Token::Character));
        expected.tokens.push(Token::Eof);
        if escaped {
            expected
                .errors
                .push(error("eof-in-script-html-comment-like-text", 1, column));
        }
        assert_eq!(snapshot(&run), expected, "{input:?}");
        terminal(&mut run, Ok(Progress::Finished));
    }
}

#[test]
fn real_entry_inspects_escaped_less_than_then_reaches_terminal_double_escape_start() {
    for (input, output, column) in [
        ("<!--<tail", "<!--", 5),
        ("<!--x<tail", "<!--x", 6),
        ("<!--x-<tail", "<!--x-", 7),
    ] {
        let mut context = seeded_context();
        context.input.push(points(input)).unwrap();
        let mut run = Execution::new(State::ScriptData, context);
        let mut expected = snapshot(&run);
        expected.state = State::ScriptDataEscapedLessThanSign;
        expected.current = Some((Input::CodePoint('<'.into()), Position { line: 1, column }));
        expected
            .tokens
            .extend(points(output).into_iter().map(Token::Character));
        // Preserve the predecessor's '<' boundary before executing the gateway.
        for _ in 0..20 {
            if run.state() == State::ScriptDataEscapedLessThanSign {
                break;
            }
            assert!(matches!(run.step(), Ok(Progress::Continue(_))));
        }
        assert_eq!(snapshot(&run), expected);
        expected.state = State::ScriptDataDoubleEscapeStart;
        expected.current = Some((
            Input::CodePoint('t'.into()),
            Position {
                line: 1,
                column: column + 1,
            },
        ));
        expected.tokens.push(Token::Character('<'.into()));
        expected.storage.buffer.clear();
        assert_eq!(run.step(), Ok(Progress::Continue(expected.state)));
        assert_eq!(snapshot(&run), expected);
        let blocked = Err(UnimplementedState {
            state: State::ScriptDataDoubleEscapeStart,
        });
        let mut trace = vec![];
        assert!(drain(&mut run, &mut trace));
        assert_eq!(trace.last().unwrap().0, blocked);
        assert_eq!(snapshot(&run), expected);
        terminal(&mut run, blocked);
        // Negative terminality check, not supported resumption or a new execution.
        run.context_mut().input.push(points("more\0")).unwrap();
        run.context_mut().input.finish();
        terminal(&mut run, blocked);
        assert_eq!(snapshot(&run), expected);
    }
}

type Trace = Vec<(Result<Progress, UnimplementedState>, Snapshot)>;

fn drain(run: &mut Execution, trace: &mut Trace) -> bool {
    for _ in 0..1000 {
        let before = snapshot(run);
        let result = run.step();
        if result == Ok(Progress::NeedMoreInput) {
            assert_eq!(snapshot(run), before);
            for _ in 0..3 {
                assert_eq!(run.step(), result);
                assert_eq!(snapshot(run), before);
            }
            return false;
        }
        trace.push((result, snapshot(run)));
        if !matches!(result, Ok(Progress::Continue(_))) {
            terminal(run, result);
            return true;
        }
    }
    panic!("source steps failed to progress");
}

fn trace(chunks: &[&[CodePoint]], closed: bool, real_entry: bool) -> Trace {
    let mut run = if real_entry {
        Execution::new(State::ScriptData, seeded_context())
    } else {
        entry(seeded_context())
    };
    let mut trace = vec![];
    for chunk in chunks {
        run.context_mut().input.push(chunk.iter().copied()).unwrap();
        if !closed && drain(&mut run, &mut trace) {
            return trace; // Never feed or reconstruct a blocked execution.
        }
    }
    run.context_mut().input.finish();
    assert!(drain(&mut run, &mut trace));
    trace
}

#[test]
fn every_two_chunk_split_and_single_code_point_stream_matches_closed_source_steps() {
    let mut inputs: Vec<_> = [
        "",
        "-",
        "--tail\0",
        "--",
        "--x",
        "--x-",
        "--x--",
        "--x-\0-->\0",
        "-- - -->&literal",
        "--test---></script>",
        "--test--></script>&tail",
        "--test--></script><!tail",
        "--<tail",
        "--x<tail",
        "--x-<tail",
        "--\r\n\0\u{fdd0}\u{b}-->",
        "--\r\r\n",
        "test-->",
        "!",
        "&amp;",
        "\0\0",
        "<x",
        "<!",
        "<!-tail",
        "</script>",
        "</script>&tail",
        "</script><!tail",
        "\r\nX",
        "\r\r\n",
        "É😀\r",
        "\u{b}\u{fdd0}\0",
    ]
    .into_iter()
    .map(points)
    .collect();
    inputs.push(vec![
        CodePoint::new(0xd800).unwrap(),
        '\r'.into(),
        '\n'.into(),
        CodePoint::new(0x10ffff).unwrap(),
        '\0'.into(),
    ]);
    inputs.push(
        [
            points("--\r\n\0"),
            vec![CodePoint::new(0xd800).unwrap()],
            points("-<tail"),
        ]
        .concat(),
    );
    for input in inputs {
        for real_entry in [false, true] {
            let input = if real_entry {
                [points("\r\n<!"), input.clone()].concat()
            } else {
                input.clone()
            };
            let expected = trace(&[&input], true, real_entry);
            for split in 0..=input.len() {
                assert_eq!(
                    trace(&[&input[..split], &[], &input[split..]], false, real_entry),
                    expected,
                    "split {split}: {input:?}"
                );
            }
            assert_eq!(
                trace(&input.chunks(1).collect::<Vec<_>>(), false, real_entry),
                expected,
                "single points: {input:?}"
            );
        }
    }
}
