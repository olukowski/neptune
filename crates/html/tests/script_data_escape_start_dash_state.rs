//! Local source/streaming evidence, not upstream conformance runs.
//! HTML 404b3eb14a7a936dddefb2d78e224dbeb01d2358, source 143758–143773,
//! #script-data-escape-start-dash-state. Escape start remains a predecessor stub;
//! these tests enter the assigned state directly, never resume after a stub.

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
    last_start_tag: Option<Vec<CodePoint>>,
    return_state: Option<State>,
    current_tag: Option<TagKind>,
    start: Option<PendingStartTag>,
    end: Option<PendingEndTag>,
    comment: Option<PendingComment>,
    buffer: Vec<CodePoint>,
}

fn storage(context: &Context) -> Storage {
    Storage {
        last_start_tag: context.last_start_tag.clone(),
        return_state: context.return_state,
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

// Set up cursor history without executing or simulating the stub predecessor.
// The assigned state's prior output is independent of its input/storage effects.
fn entry(mut context: Context, newline_prefix: bool) -> Execution {
    if newline_prefix {
        context.input.push(points("\r\n")).unwrap();
        assert_eq!(context.input.consume(), Input::CodePoint('\n'.into()));
    }
    context.input.push(points("<!-")).unwrap();
    for point in points("<!-") {
        assert_eq!(context.input.consume(), Input::CodePoint(point));
    }
    Execution::new(State::ScriptDataEscapeStartDash, context)
}

fn assert_terminal_stub(execution: &mut Execution, target: State) {
    let before = snapshot(execution);
    assert_eq!(before.state, target);
    for _ in 0..3 {
        assert_eq!(execution.step(), Err(UnimplementedState { state: target }));
        assert_eq!(snapshot(execution), before);
    }
    assert!(!before.tokens.contains(&Token::Eof));
}

#[test]
fn every_ascii_branch_and_non_scalar_preserves_source_effects_and_reconsumes_exactly_once() {
    for value in (0..=0x7f).chain([
        0x80, 0x9f, 0xa0, 0xc9, 0x3a9, 0xfeff, 0x1f600, 0xd800, 0xdfff, 0xfdd0, 0xffff, 0x10ffff,
    ]) {
        for finished in [false, true] {
            for selected in [None, Some(TagKind::Start), Some(TagKind::End)] {
                let point = CodePoint::new(value).unwrap();
                let normalized = if value == 0x0d { '\n'.into() } else { point };
                let mut context = seeded_context();
                context.current_tag = selected;
                let mut execution = entry(context, true);
                execution
                    .context_mut()
                    .input
                    .push([point, 'z'.into()])
                    .unwrap();
                if finished {
                    execution.context_mut().input.finish();
                }
                let mut expected = snapshot(&execution);
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
                if value == 0x2d {
                    expected.state = State::ScriptDataEscapedDashDash;
                    expected.tokens.push(Token::Character('-'.into()));
                    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                    assert_eq!(snapshot(&execution), expected);
                    // Source assertions above stay unchanged. The merged real
                    // destination consumes the untouched suffix, not this hyphen.
                    expected.state = State::ScriptDataEscaped;
                    expected.current = Some((
                        Input::CodePoint('z'.into()),
                        Position { line: 2, column: 5 },
                    ));
                    expected.tokens.push(Token::Character('z'.into()));
                    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                    assert_eq!(snapshot(&execution), expected);
                    if finished {
                        expected.current = Some((Input::Eof, Position { line: 2, column: 6 }));
                        expected.tokens.push(Token::Eof);
                        expected
                            .errors
                            .push(error("eof-in-script-html-comment-like-text", 2, 6));
                        for _ in 0..3 {
                            assert_eq!(execution.step(), Ok(Progress::Finished));
                            assert_eq!(snapshot(&execution), expected);
                        }
                    } else {
                        for _ in 0..3 {
                            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                            assert_eq!(snapshot(&execution), expected);
                        }
                    }
                    continue;
                }
                expected.state = State::ScriptData;
                assert_eq!(execution.step(), Ok(Progress::Continue(State::ScriptData)));
                assert_eq!(snapshot(&execution), expected, "U+{value:04X}");
                assert_eq!(
                    execution.context_mut().input.reconsume(),
                    Err(CursorError::AlreadyReconsuming)
                );
                // Only the destination emits/replaces/reports NUL. It sees the
                // same normalized point and position, with no duplicate input error.
                if value == 0x3c {
                    expected.state = State::ScriptDataLessThanSign;
                } else {
                    expected.tokens.push(Token::Character(if value == 0 {
                        '\u{fffd}'.into()
                    } else {
                        normalized
                    }));
                    if value == 0 {
                        expected
                            .errors
                            .push(error("unexpected-null-character", 2, 4));
                    }
                }
                assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                assert_eq!(snapshot(&execution), expected, "destination U+{value:04X}");
                // This cursor-only probe is before any terminal block. The source
                // and reconsuming destination have not eaten the following point.
                assert_eq!(
                    execution.context_mut().input.consume(),
                    Input::CodePoint('z'.into())
                );
            }
        }
    }
}

#[test]
fn hyphen_consumes_without_reconsumption_and_leaves_the_tail_for_the_destination() {
    let mut execution = entry(Context::default(), false);
    execution.context_mut().input.push(points("-tail")).unwrap();
    execution.context_mut().input.finish();
    assert_eq!(
        execution.step(),
        Ok(Progress::Continue(State::ScriptDataEscapedDashDash))
    );
    assert_eq!(execution.context().tokens, [Token::Character('-'.into())]);
    assert_eq!(
        execution.context().input.current(),
        Some((
            Input::CodePoint('-'.into()),
            Position { line: 1, column: 4 }
        ))
    );
    assert!(execution.context().errors.is_empty());
    assert!(execution.context().input.errors().is_empty());
    assert_eq!(storage(execution.context()), storage(&Context::default()));
    // Cursor-only inspection, before dispatching the destination, proves no reconsume
    // was scheduled and that only the hyphen was consumed by this source step.
    assert_eq!(
        execution.context_mut().input.consume(),
        Input::CodePoint('t'.into())
    );
    // This is only a cursor probe of the unchanged source boundary. Real
    // destination execution is checked separately without consuming its suffix.
}

#[test]
fn open_exhaustion_is_effect_free_and_only_explicit_eof_is_reconsumed() {
    for with_history in [false, true] {
        let mut execution = if with_history {
            entry(seeded_context(), true)
        } else {
            Execution::new(State::ScriptDataEscapeStartDash, Context::default())
        };
        let mut expected = snapshot(&execution);
        for _ in 0..3 {
            execution.context_mut().input.push([]).unwrap();
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&execution), expected);
        }
        execution.context_mut().input.finish();
        execution.context_mut().input.finish();
        expected.current = Some((
            Input::Eof,
            if with_history {
                Position { line: 2, column: 4 }
            } else {
                Position { line: 1, column: 1 }
            },
        ));
        expected.state = State::ScriptData;
        assert_eq!(execution.step(), Ok(Progress::Continue(State::ScriptData)));
        assert_eq!(snapshot(&execution), expected);
        assert!(!expected.tokens.contains(&Token::Eof));
        assert_eq!(
            execution.context_mut().input.reconsume(),
            Err(CursorError::AlreadyReconsuming)
        );
        assert_eq!(execution.step(), Ok(Progress::Finished));
        expected.tokens.push(Token::Eof);
        for _ in 0..3 {
            assert_eq!(snapshot(&execution), expected);
            assert_eq!(execution.step(), Ok(Progress::Finished));
        }
        assert_eq!(
            execution.context_mut().input.push([]),
            Err(CursorError::InputAlreadyFinished)
        );
    }
}

#[test]
fn suspended_entry_accepts_input_but_does_not_invent_predecessor_output_or_storage() {
    for (input, target, output) in [
        ("-", State::ScriptDataEscapedDashDash, "-"),
        ("x", State::ScriptData, ""),
    ] {
        let mut execution = Execution::new(State::ScriptDataEscapeStartDash, Context::default());
        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
        execution.context_mut().input.push(points(input)).unwrap();
        assert_eq!(execution.step(), Ok(Progress::Continue(target)));
        assert_eq!(storage(execution.context()), storage(&Context::default()));
        assert_eq!(
            execution.context().tokens,
            points(output)
                .into_iter()
                .map(Token::Character)
                .collect::<Vec<_>>()
        );
        assert!(execution.context().errors.is_empty());
        assert!(execution.context().input.errors().is_empty());
        if input == "-" {
            // The integrated destination now suspends on exhausted OPEN input.
            let mut expected = snapshot(&execution);
            for _ in 0..3 {
                assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                assert_eq!(snapshot(&execution), expected);
            }
            execution.context_mut().input.finish();
            expected.current = Some((Input::Eof, Position { line: 1, column: 2 }));
            expected.tokens.push(Token::Eof);
            expected
                .errors
                .push(error("eof-in-script-html-comment-like-text", 1, 2));
            for _ in 0..3 {
                assert_eq!(execution.step(), Ok(Progress::Finished));
                assert_eq!(snapshot(&execution), expected);
            }
        } else {
            assert_eq!(execution.step(), Ok(Progress::Continue(State::ScriptData)));
            assert_eq!(execution.context().tokens, [Token::Character('x'.into())]);
            let before = snapshot(&execution);
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&execution), before);
        }
    }
}

#[test]
fn normalized_positions_and_diagnostic_multiplicities_survive_fallback_and_continuation() {
    let mut execution = entry(Context::default(), true);
    let mut input = points("\r\n\0\0");
    input.extend([
        CodePoint::new(0xd800).unwrap(),
        CodePoint::new(0xfdd0).unwrap(),
        '\u{b}'.into(),
        '&'.into(),
    ]);
    execution.context_mut().input.push(input).unwrap();
    execution.context_mut().input.finish();
    assert_eq!(execution.step(), Ok(Progress::Continue(State::ScriptData)));
    assert!(execution.context().tokens.is_empty());
    assert!(execution.context().errors.is_empty());
    assert_eq!(
        execution.context().input.current(),
        Some((
            Input::CodePoint('\n'.into()),
            Position { line: 2, column: 4 }
        ))
    );
    for _ in 0..7 {
        assert_eq!(execution.step(), Ok(Progress::Continue(State::ScriptData)));
    }
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(
        execution.context().tokens,
        vec![
            Token::Character('\n'.into()),
            Token::Character('\u{fffd}'.into()),
            Token::Character('\u{fffd}'.into()),
            Token::Character(CodePoint::new(0xd800).unwrap()),
            Token::Character(CodePoint::new(0xfdd0).unwrap()),
            Token::Character('\u{b}'.into()),
            Token::Character('&'.into()),
            Token::Eof,
        ]
    );
    assert_eq!(
        execution.context().errors,
        [
            error("unexpected-null-character", 3, 1),
            error("unexpected-null-character", 3, 2)
        ]
    );
    assert_eq!(
        execution.context().input.errors(),
        [
            error("surrogate-in-input-stream", 3, 3),
            error("noncharacter-in-input-stream", 3, 4),
            error("control-character-in-input-stream", 3, 5),
        ]
    );
    assert_eq!(
        execution.context().input.current(),
        Some((Input::Eof, Position { line: 3, column: 7 }))
    );
}

#[test]
fn fallback_runs_real_script_data_and_keeps_later_stub_blocks_explicit() {
    for (input, output, blocked) in [
        // Suffix after the first hyphen of pinned test1.test tests[35]. This
        // isolated local run is separate from the now-comparable upstream run.
        ("test-->", "ptest-->", None),
        ("<!tail", "p<!tail", None),
        (
            "<!--<tail",
            "p<!--",
            Some(State::ScriptDataEscapedLessThanSign),
        ),
        ("</script a='x&y'>", "p", Some(State::CharacterReference)),
    ] {
        let mut execution = entry(seeded_context(), false);
        execution.context_mut().input.push(points(input)).unwrap();
        execution.context_mut().input.finish();
        let before = snapshot(&execution);
        assert_eq!(execution.step(), Ok(Progress::Continue(State::ScriptData)));
        assert_eq!(execution.context().tokens, before.tokens);
        assert_eq!(execution.context().errors, before.errors);
        assert_eq!(storage(execution.context()), before.storage);
        let expected_terminal = blocked.map_or(Ok(Progress::Finished), |state| {
            Err(UnimplementedState { state })
        });
        let mut terminal = None;
        for _ in 0..input.len() * 4 + 10 {
            let result = execution.step();
            if !matches!(result, Ok(Progress::Continue(_))) {
                terminal = Some(result);
                break;
            }
        }
        assert_eq!(terminal, Some(expected_terminal));
        let mut tokens: Vec<_> = points(output).into_iter().map(Token::Character).collect();
        if blocked.is_none() {
            tokens.push(Token::Eof);
        }
        assert_eq!(execution.context().tokens, tokens);
        assert_eq!(
            execution.context().errors,
            [error("prior-diagnostic", 1, 1)]
        );
        assert!(execution.context().input.errors().is_empty());
        let after = snapshot(&execution);
        for _ in 0..3 {
            assert_eq!(execution.step(), expected_terminal);
            assert_eq!(snapshot(&execution), after);
        }
    }
}

#[test]
fn hyphen_executes_real_escaped_cycles_with_exact_source_steps_and_later_boundaries() {
    for exit in ["", ">\r\n\0", "<tail"] {
        for finished in [false, true] {
            let mut execution = entry(seeded_context(), true);
            let mut input = points("-a--\0-\0--\r\n");
            input.extend([
                CodePoint::new(0xd800).unwrap(),
                CodePoint::new(0xfdd0).unwrap(),
                '\u{b}'.into(),
            ]);
            input.extend(points("---"));
            input.extend(points(exit));
            execution.context_mut().input.push(input).unwrap();
            if finished {
                execution.context_mut().input.finish();
            }
            let mut expected = snapshot(&execution);
            // Fixed specification-derived targets/positions, not dispatch-derived
            // expectations. The first row is this PR's unchanged source branch.
            for (value, target, output, line, column) in [
                (0x2d, State::ScriptDataEscapedDashDash, 0x2d, 2, 4),
                (0x61, State::ScriptDataEscaped, 0x61, 2, 5),
                (0x2d, State::ScriptDataEscapedDash, 0x2d, 2, 6),
                (0x2d, State::ScriptDataEscapedDashDash, 0x2d, 2, 7),
                (0, State::ScriptDataEscaped, 0xfffd, 2, 8),
                (0x2d, State::ScriptDataEscapedDash, 0x2d, 2, 9),
                (0, State::ScriptDataEscaped, 0xfffd, 2, 10),
                (0x2d, State::ScriptDataEscapedDash, 0x2d, 2, 11),
                (0x2d, State::ScriptDataEscapedDashDash, 0x2d, 2, 12),
                (0x0a, State::ScriptDataEscaped, 0x0a, 2, 13),
                (0xd800, State::ScriptDataEscaped, 0xd800, 3, 1),
                (0xfdd0, State::ScriptDataEscaped, 0xfdd0, 3, 2),
                (0x0b, State::ScriptDataEscaped, 0x0b, 3, 3),
                (0x2d, State::ScriptDataEscapedDash, 0x2d, 3, 4),
                (0x2d, State::ScriptDataEscapedDashDash, 0x2d, 3, 5),
                (0x2d, State::ScriptDataEscapedDashDash, 0x2d, 3, 6),
            ] {
                expected.state = target;
                expected.current = Some((
                    Input::CodePoint(CodePoint::new(value).unwrap()),
                    Position { line, column },
                ));
                expected
                    .tokens
                    .push(Token::Character(CodePoint::new(output).unwrap()));
                if value == 0 {
                    expected
                        .errors
                        .push(error("unexpected-null-character", line, column));
                }
                let input_error = match value {
                    0xd800 => Some("surrogate-in-input-stream"),
                    0xfdd0 => Some("noncharacter-in-input-stream"),
                    0x0b => Some("control-character-in-input-stream"),
                    _ => None,
                };
                if let Some(code) = input_error {
                    expected.input_errors.push(error(code, line, column));
                }
                assert_eq!(execution.step(), Ok(Progress::Continue(target)));
                assert_eq!(snapshot(&execution), expected);
            }
            if exit == "<tail" {
                expected.state = State::ScriptDataEscapedLessThanSign;
                expected.current = Some((
                    Input::CodePoint('<'.into()),
                    Position { line: 3, column: 7 },
                ));
                assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                assert_eq!(snapshot(&execution), expected);
                assert_terminal_stub(&mut execution, State::ScriptDataEscapedLessThanSign);
                continue;
            }
            let eof_position = if exit.is_empty() {
                Position { line: 3, column: 7 }
            } else {
                // DashDash's '>' returns to real Script data; its NUL and EOF
                // handling is distinct from escaped EOF diagnostics.
                for (point, output, line, column) in [
                    ('>', '>', 3, 7),
                    ('\n', '\n', 3, 8),
                    ('\0', '\u{fffd}', 4, 1),
                ] {
                    expected.state = State::ScriptData;
                    expected.current =
                        Some((Input::CodePoint(point.into()), Position { line, column }));
                    expected.tokens.push(Token::Character(output.into()));
                    if point == '\0' {
                        expected
                            .errors
                            .push(error("unexpected-null-character", line, column));
                    }
                    assert_eq!(execution.step(), Ok(Progress::Continue(State::ScriptData)));
                    assert_eq!(snapshot(&execution), expected);
                }
                Position { line: 4, column: 2 }
            };
            if !finished {
                for _ in 0..3 {
                    assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                    assert_eq!(snapshot(&execution), expected);
                }
                execution.context_mut().input.finish();
            }
            expected.current = Some((Input::Eof, eof_position));
            expected.tokens.push(Token::Eof);
            if exit.is_empty() {
                expected.errors.push(ParseError {
                    code: "eof-in-script-html-comment-like-text",
                    position: eof_position,
                });
            }
            for _ in 0..3 {
                assert_eq!(execution.step(), Ok(Progress::Finished));
                assert_eq!(snapshot(&execution), expected);
            }
        }
    }
}

type Trace = Vec<(Result<Progress, UnimplementedState>, Snapshot)>;

fn drain(execution: &mut Execution, trace: &mut Trace, bound: usize) -> bool {
    for _ in 0..bound {
        let before = snapshot(execution);
        let result = execution.step();
        if result == Ok(Progress::NeedMoreInput) {
            for _ in 0..3 {
                assert_eq!(snapshot(execution), before);
                assert_eq!(execution.step(), result);
            }
            return false;
        }
        trace.push((result, snapshot(execution)));
        if !matches!(result, Ok(Progress::Continue(_))) {
            let terminal = snapshot(execution);
            for _ in 0..3 {
                assert_eq!(execution.step(), result);
                assert_eq!(snapshot(execution), terminal);
            }
            return true;
        }
    }
    panic!("source steps failed to consume, suspend, or terminate");
}

fn trace(chunks: &[&[CodePoint]], closed: bool, newline_prefix: bool) -> Trace {
    let mut execution = entry(seeded_context(), newline_prefix);
    let mut snapshots = Vec::new();
    let bound = chunks.iter().map(|chunk| chunk.len()).sum::<usize>() * 4 + 10;
    for chunk in chunks {
        execution
            .context_mut()
            .input
            .push(chunk.iter().copied())
            .unwrap();
        if !closed && drain(&mut execution, &mut snapshots, bound) {
            // Stop feeding immediately at a real terminal block, never reconstruct.
            return snapshots;
        }
    }
    execution.context_mut().input.finish();
    assert!(drain(&mut execution, &mut snapshots, bound));
    snapshots
}

#[test]
fn every_two_chunk_split_and_single_code_point_stream_matches_closed_source_traces() {
    let mut inputs: Vec<Vec<CodePoint>> = [
        "",
        "-",
        "-tail",
        "-test-->",
        "-->",
        "-<tail",
        "-a<tail",
        "-a-<tail",
        "->\r\n\0\0",
        "-a--\0-\0--\r\n---",
        "-a--\0-\0--\r\n--->\0",
        "-a--\0-\0--\r\n---<tail",
        "-></script>&tail",
        "x",
        "test-->",
        "!",
        "&amp;",
        "\0tail",
        "\0\0",
        "<",
        "<!tail",
        "</script>",
        "</script a='x&y'>",
        "<x",
        "\r\nX",
        "\r\r\n",
        "\r",
        "\n",
        "É😀\r",
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
        CodePoint::new(0xdfff).unwrap(),
        '\0'.into(),
    ]);
    inputs.push(vec![CodePoint::new(0x10ffff).unwrap(), '\0'.into()]);
    let mut mixed = points("-a--\0-\0--\r\n");
    mixed.extend([
        CodePoint::new(0xd800).unwrap(),
        CodePoint::new(0xfdd0).unwrap(),
        '\u{b}'.into(),
    ]);
    mixed.extend(points("--->\r\n\0"));
    inputs.push(mixed);
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
                    "split {split} of {input:?}"
                );
            }
            let chunks: Vec<_> = input.chunks(1).collect();
            assert_eq!(
                trace(&chunks, false, newline_prefix),
                expected,
                "single points: {input:?}"
            );
        }
    }
}

#[test]
fn real_script_entry_preserves_predecessor_effects_then_executes_the_escape_chain() {
    let mut context = Context::default();
    context.input.push(points("<!--tail")).unwrap();
    context.input.finish();
    let mut execution = Execution::new(State::ScriptData, context);
    assert_eq!(
        execution.step(),
        Ok(Progress::Continue(State::ScriptDataLessThanSign))
    );
    assert_eq!(
        execution.step(),
        Ok(Progress::Continue(State::ScriptDataEscapeStart))
    );
    assert_eq!(
        execution.context().tokens,
        [Token::Character('<'.into()), Token::Character('!'.into())]
    );
    assert_eq!(
        execution.context().input.current(),
        Some((
            Input::CodePoint('!'.into()),
            Position { line: 1, column: 2 }
        ))
    );
    assert!(execution.context().errors.is_empty());
    assert!(execution.context().input.errors().is_empty());
    let mut expected = snapshot(&execution);
    for (point, column, state) in [
        ('-', 3, State::ScriptDataEscapeStartDash),
        ('-', 4, State::ScriptDataEscapedDashDash),
        ('t', 5, State::ScriptDataEscaped),
        ('a', 6, State::ScriptDataEscaped),
        ('i', 7, State::ScriptDataEscaped),
        ('l', 8, State::ScriptDataEscaped),
    ] {
        expected.state = state;
        expected.current = Some((Input::CodePoint(point.into()), Position { line: 1, column }));
        expected.tokens.push(Token::Character(point.into()));
        assert_eq!(execution.step(), Ok(Progress::Continue(state)));
        assert_eq!(snapshot(&execution), expected);
    }
    expected.current = Some((Input::Eof, Position { line: 1, column: 9 }));
    expected
        .errors
        .push(error("eof-in-script-html-comment-like-text", 1, 9));
    expected.tokens.push(Token::Eof);
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(snapshot(&execution), expected);
    for _ in 0..3 {
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&execution), expected);
    }
}
