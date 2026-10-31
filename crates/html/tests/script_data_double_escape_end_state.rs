//! Local source-step evidence, not upstream conformance runs.
//! HTML 404b3eb14a7a936dddefb2d78e224dbeb01d2358, source 144110–144142.
//! Direct entry supplies scratch context; never reconstruct/resume a stub.

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
        tokens: vec![Token::Character('!'.into())],
        errors: vec![error("prior-diagnostic", 1, 1)],
        // Delimiter decisions must NOT use last-start-tag or current-tag names.
        last_start_tag: Some(points("not-script")),
        return_state: Some(State::AttributeValueSingleQuoted),
        current_tag: Some(TagKind::End),
        pending_start_tag: Some(PendingStartTag {
            name: points("script"),
            self_closing: true,
            attributes: vec![attribute.clone()],
            discarded_current_attribute: Some(attribute.clone()),
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("script"),
            self_closing: true,
            attributes: vec![attribute.clone()],
            discarded_current_attribute: Some(attribute),
        }),
        pending_comment: Some(PendingComment {
            data: points("comment"),
        }),
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
}

fn storage(context: &Context) -> Storage {
    Storage {
        last: context.last_start_tag.clone(),
        return_state: context.return_state,
        selected: context.current_tag,
        start: context.pending_start_tag.clone(),
        end: context.pending_end_tag.clone(),
        comment: context.pending_comment.clone(),
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Snapshot {
    state: State,
    current: Option<(Input, Position)>,
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
    input_errors: Vec<ParseError>,
    buffer: Vec<CodePoint>,
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
        buffer: context.temporary_buffer.clone(),
        storage: storage(context),
    }
}

fn entry(mut context: Context, buffer: Vec<CodePoint>) -> Execution {
    context.temporary_buffer = buffer;
    Execution::new(State::ScriptDataDoubleEscapeEnd, context)
}

fn assert_stub(execution: &mut Execution, state: State) {
    let before = snapshot(execution);
    assert_eq!(before.state, state);
    for _ in 0..3 {
        assert_eq!(execution.step(), Err(UnimplementedState { state }));
        assert_eq!(snapshot(execution), before);
    }
    assert!(!before.tokens.contains(&Token::Eof));
}

#[test]
fn exhaustive_source_branches_preserve_unrelated_context_and_probe_consumption() {
    for value in (0..=0x7f).chain([
        0x80, 0x9f, 0xa0, 0xc9, 0x130, 0x17f, 0x3a9, 0x2003, 0xfeff, 0x1f600, 0xd800, 0xdfff,
        0xfdd0, 0xffff, 0x10ffff,
    ]) {
        for buffer in [
            points(""),
            points("script"),
            points("SCRIPT"),
            vec!['s'.into(), '\0'.into(), CodePoint::new(0xdfff).unwrap()],
        ] {
            for selected in [None, Some(TagKind::Start), Some(TagKind::End)] {
                for finished in [false, true] {
                    let mut context = seeded_context();
                    context.current_tag = selected;
                    // Cursor history only, not a simulation of the stub predecessor.
                    context.input.push(points("\r\n/")).unwrap();
                    assert_eq!(context.input.consume(), Input::CodePoint('\n'.into()));
                    assert_eq!(context.input.consume(), Input::CodePoint('/'.into()));
                    let mut execution = entry(context, buffer.clone());
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
                    expected.current = Some((
                        Input::CodePoint(normalized),
                        Position { line: 2, column: 2 },
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
                        expected.input_errors.push(error(code, 2, 2));
                    }
                    let fallback = match value {
                        0x09 | 0x0a | 0x0c | 0x0d | 0x20 | 0x2f | 0x3e => {
                            expected.state = if buffer == points("script") {
                                State::ScriptDataEscaped
                            } else {
                                State::ScriptDataDoubleEscaped
                            };
                            expected.tokens.push(Token::Character(normalized));
                            false
                        }
                        0x41..=0x5a => {
                            expected.buffer.push(CodePoint::new(value + 0x20).unwrap());
                            expected.tokens.push(Token::Character(point));
                            false
                        }
                        0x61..=0x7a => {
                            expected.buffer.push(point);
                            expected.tokens.push(Token::Character(point));
                            false
                        }
                        _ => {
                            expected.state = State::ScriptDataDoubleEscaped;
                            true
                        }
                    };
                    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                    assert_eq!(snapshot(&execution), expected, "U+{value:04X}");
                    if fallback {
                        assert_eq!(
                            execution.context_mut().input.reconsume(),
                            Err(CursorError::AlreadyReconsuming)
                        );
                        assert_eq!(
                            execution.context_mut().input.consume(),
                            Input::CodePoint(normalized)
                        );
                        assert_eq!(snapshot(&execution), expected); // no duplicate preprocessing error
                    }
                    // Cursor-only inspection before target execution. No terminal
                    // execution is resumed; the suffix is not consumed by the source.
                    assert_eq!(
                        execution.context_mut().input.consume(),
                        Input::CodePoint('z'.into())
                    );
                }
            }
        }
    }
}

#[test]
fn all_script_case_spellings_accumulate_normalized_scratch_but_emit_original_text() {
    for mask in 0..64 {
        let spelling: String = "script"
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if mask & (1 << i) != 0 {
                    c.to_ascii_uppercase()
                } else {
                    c
                }
            })
            .collect();
        for delimiter in ['\t', '\n', '\x0c', ' ', '/', '>'] {
            let mut execution = entry(seeded_context(), Vec::new());
            let original_storage = storage(execution.context());
            execution
                .context_mut()
                .input
                .push(points(&format!("{spelling}{delimiter}x")))
                .unwrap();
            for (i, point) in spelling.chars().enumerate() {
                assert_eq!(
                    execution.step(),
                    Ok(Progress::Continue(State::ScriptDataDoubleEscapeEnd))
                );
                assert_eq!(
                    execution.context().temporary_buffer,
                    points(&"script"[..=i])
                );
                let mut expected_tokens = vec![Token::Character('!'.into())];
                expected_tokens.extend(
                    spelling
                        .chars()
                        .take(i + 1)
                        .map(|c| Token::Character(c.into())),
                );
                assert_eq!(execution.context().tokens, expected_tokens);
                assert_eq!(
                    execution.context().input.current(),
                    Some((
                        Input::CodePoint(point.into()),
                        Position {
                            line: 1,
                            column: i + 1
                        }
                    ))
                );
                assert_eq!(storage(execution.context()), original_storage);
                assert_eq!(
                    execution.context().errors,
                    [error("prior-diagnostic", 1, 1)]
                );
            }
            assert_eq!(
                execution.step(),
                Ok(Progress::Continue(State::ScriptDataEscaped))
            );
            assert_eq!(
                execution.context().tokens.last(),
                Some(&Token::Character(delimiter.into()))
            );
            assert_eq!(execution.context().temporary_buffer, points("script"));
            assert_eq!(storage(execution.context()), original_storage);
            // Actual implemented destination consumes x, NOT the delimiter.
            assert_eq!(
                execution.step(),
                Ok(Progress::Continue(State::ScriptDataEscaped))
            );
            assert_eq!(
                execution.context().tokens.last(),
                Some(&Token::Character('x'.into()))
            );
            assert!(execution.context().input.errors().is_empty());
        }
    }
}

#[test]
fn delimiter_equality_is_exact_and_never_clears_or_emits_the_scratch_buffer() {
    for buffer in [
        "", "s", "scrip", "script", "scripts", "xscript", "scriptx", "SCRIPT", "Script", "ſcript",
        "scrİpt", "script\0", "script ",
    ] {
        for delimiter in ['\t', '\n', '\x0c', ' ', '/', '>'] {
            let mut execution = entry(Context::default(), points(buffer));
            execution
                .context_mut()
                .input
                .push([delimiter.into(), 'x'.into()])
                .unwrap();
            let target = if buffer == "script" {
                State::ScriptDataEscaped
            } else {
                State::ScriptDataDoubleEscaped
            };
            assert_eq!(execution.step(), Ok(Progress::Continue(target)));
            assert_eq!(execution.context().temporary_buffer, points(buffer));
            assert_eq!(
                execution.context().tokens,
                [Token::Character(delimiter.into())]
            );
            assert!(execution.context().errors.is_empty());
            if target == State::ScriptDataDoubleEscaped {
                assert_stub(&mut execution, target);
            }
        }
    }
}

#[test]
fn fallback_nul_non_scalars_and_malformed_input_stop_at_typed_destination() {
    for value in [
        0, 0x0b, 0x26, 0x27, 0x2d, 0x30, 0x3c, 0x3d, 0xa0, 0xd800, 0xfdd0,
    ] {
        for buffer in ["", "script"] {
            let mut execution = entry(Context::default(), points(buffer));
            let point = CodePoint::new(value).unwrap();
            execution
                .context_mut()
                .input
                .push([point, '\0'.into()])
                .unwrap();
            assert_eq!(
                execution.step(),
                Ok(Progress::Continue(State::ScriptDataDoubleEscaped))
            );
            assert_eq!(
                execution.context().input.current(),
                Some((Input::CodePoint(point), Position { line: 1, column: 1 }))
            );
            assert!(execution.context().tokens.is_empty());
            assert!(execution.context().errors.is_empty()); // NUL replacement belongs to destination
            assert_eq!(
                execution.context_mut().input.reconsume(),
                Err(CursorError::AlreadyReconsuming)
            );
            assert_stub(&mut execution, State::ScriptDataDoubleEscaped);
            execution.context_mut().input.finish();
            assert_stub(&mut execution, State::ScriptDataDoubleEscaped);
        }
    }
}

#[test]
fn open_exhaustion_preserves_current_input_and_explicit_eof_is_reconsumed_not_emitted() {
    for seeded in [false, true] {
        for buffer in ["", "scrip", "script"] {
            let context = if seeded {
                seeded_context()
            } else {
                Context::default()
            };
            let mut execution = entry(context, points(buffer));
            if seeded {
                execution.context_mut().input.push(points("\r\n/")).unwrap();
                assert_eq!(
                    execution.context_mut().input.consume(),
                    Input::CodePoint('\n'.into())
                );
                assert_eq!(
                    execution.context_mut().input.consume(),
                    Input::CodePoint('/'.into())
                );
            }
            let mut expected = snapshot(&execution);
            for _ in 0..3 {
                execution.context_mut().input.push([]).unwrap();
                assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                assert_eq!(snapshot(&execution), expected);
            }
            execution.context_mut().input.finish();
            execution.context_mut().input.finish();
            expected.state = State::ScriptDataDoubleEscaped;
            expected.current = Some((
                Input::Eof,
                if seeded {
                    Position { line: 2, column: 2 }
                } else {
                    Position { line: 1, column: 1 }
                },
            ));
            assert_eq!(
                execution.step(),
                Ok(Progress::Continue(State::ScriptDataDoubleEscaped))
            );
            assert_eq!(snapshot(&execution), expected);
            assert_eq!(
                execution.context_mut().input.reconsume(),
                Err(CursorError::AlreadyReconsuming)
            );
            assert_stub(&mut execution, State::ScriptDataDoubleEscaped);
            assert_eq!(
                execution.context_mut().input.push([]),
                Err(CursorError::InputAlreadyFinished)
            );
        }
    }
}

#[test]
fn matched_delimiter_executes_real_escaped_continuation_and_exact_diagnostics() {
    let mut execution = entry(seeded_context(), Vec::new());
    let preserved = storage(execution.context());
    execution
        .context_mut()
        .input
        .push(points("ScRiPt>\r\n\0\0"))
        .unwrap();
    execution
        .context_mut()
        .input
        .push([
            CodePoint::new(0xd800).unwrap(),
            CodePoint::new(0xffff).unwrap(),
            '\x0b'.into(),
        ])
        .unwrap();
    execution.context_mut().input.finish();
    for _ in 0..6 {
        assert_eq!(
            execution.step(),
            Ok(Progress::Continue(State::ScriptDataDoubleEscapeEnd))
        );
    }
    assert_eq!(
        execution.step(),
        Ok(Progress::Continue(State::ScriptDataEscaped))
    );
    assert_eq!(
        execution.context().tokens,
        points("!ScRiPt>")
            .into_iter()
            .map(Token::Character)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        execution.context().errors,
        [error("prior-diagnostic", 1, 1)]
    );
    for _ in 0..6 {
        assert_eq!(
            execution.step(),
            Ok(Progress::Continue(State::ScriptDataEscaped))
        );
    }
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
        [
            error("surrogate-in-input-stream", 2, 3),
            error("noncharacter-in-input-stream", 2, 4),
            error("control-character-in-input-stream", 2, 5)
        ]
    );
    assert_eq!(execution.step(), Ok(Progress::Finished));
    let mut expected_tokens: Vec<_> = points("!ScRiPt>\n\u{fffd}\u{fffd}")
        .into_iter()
        .map(Token::Character)
        .collect();
    expected_tokens.extend([
        Token::Character(CodePoint::new(0xd800).unwrap()),
        Token::Character(CodePoint::new(0xffff).unwrap()),
        Token::Character('\x0b'.into()),
        Token::Eof,
    ]);
    assert_eq!(execution.context().tokens, expected_tokens);
    assert_eq!(
        execution.context().errors,
        [
            error("prior-diagnostic", 1, 1),
            error("unexpected-null-character", 2, 1),
            error("unexpected-null-character", 2, 2),
            error("eof-in-script-html-comment-like-text", 2, 6)
        ]
    );
    assert_eq!(execution.context().temporary_buffer, points("script"));
    assert_eq!(storage(execution.context()), preserved);
    let before = snapshot(&execution);
    for _ in 0..3 {
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&execution), before);
    }
}

#[test]
fn real_predecessor_entry_clears_scratch_while_ordinary_script_entry_still_blocks() {
    let mut context = Context::default();
    context
        .input
        .push(points("<!--<script></script> -->"))
        .unwrap();
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
    assert_stub(&mut execution, State::ScriptDataEscapedLessThanSign);

    // Retain immediate-predecessor assertions and also execute the new Dash
    // source before LessThanSign, without reconstructing any terminal run.
    for initial in [
        State::ScriptDataDoubleEscapedLessThanSign,
        State::ScriptDataDoubleEscapedDash,
    ] {
        let mut context = seeded_context();
        context.temporary_buffer = points("preserved");
        context.input.push(points("<")).unwrap();
        if initial == State::ScriptDataDoubleEscapedLessThanSign {
            assert_eq!(context.input.consume(), Input::CodePoint('<'.into()));
            context.tokens.push(Token::Character('<'.into()));
        }
        context.input.push(points("/script>")).unwrap();
        context.input.finish();
        let mut execution = Execution::new(initial, context);
        let mut expected = snapshot(&execution);
        if initial == State::ScriptDataDoubleEscapedDash {
            expected.state = State::ScriptDataDoubleEscapedLessThanSign;
            expected.current = Some((
                Input::CodePoint('<'.into()),
                Position { line: 1, column: 1 },
            ));
            expected.tokens.push(Token::Character('<'.into()));
            assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&execution), expected);
        }
        expected.state = State::ScriptDataDoubleEscapeEnd;
        expected.current = Some((
            Input::CodePoint('/'.into()),
            Position { line: 1, column: 2 },
        ));
        expected.tokens.push(Token::Character('/'.into()));
        expected.buffer.clear();
        assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
        assert_eq!(snapshot(&execution), expected);
        for (index, point) in points("script").into_iter().enumerate() {
            expected.current = Some((
                Input::CodePoint(point),
                Position {
                    line: 1,
                    column: index + 3,
                },
            ));
            expected.tokens.push(Token::Character(point));
            expected.buffer.push(point);
            assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&execution), expected);
        }
        expected.state = State::ScriptDataEscaped;
        expected.current = Some((
            Input::CodePoint('>'.into()),
            Position { line: 1, column: 9 },
        ));
        expected.tokens.push(Token::Character('>'.into()));
        assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
        assert_eq!(snapshot(&execution), expected);
        expected.current = Some((
            Input::Eof,
            Position {
                line: 1,
                column: 10,
            },
        ));
        expected.tokens.push(Token::Eof);
        expected
            .errors
            .push(error("eof-in-script-html-comment-like-text", 1, 10));
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&execution), expected);
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&execution), expected);
    }
}

// Collect every real source step, never output-prefix comparisons or dynamic
// support detection. Stop immediately at a terminal result, without feeding again.
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
        match result {
            Ok(Progress::Continue(_)) => {}
            Ok(Progress::Finished) | Err(_) => {
                if let Err(block) = result {
                    assert!(matches!(
                        block.state,
                        State::ScriptDataDoubleEscaped | State::ScriptDataEscapedLessThanSign
                    ));
                }
                let before = snapshot(execution);
                assert_eq!(execution.step(), result);
                assert_eq!(snapshot(execution), before);
                return true;
            }
            Ok(Progress::NeedMoreInput) => unreachable!(),
        }
    }
}

fn trace(
    chunks: &[&[CodePoint]],
    closed_first: bool,
    seeded: bool,
    buffer: &str,
) -> Vec<(Result<Progress, UnimplementedState>, Snapshot)> {
    let context = if seeded {
        seeded_context()
    } else {
        Context::default()
    };
    let mut execution = entry(context, points(buffer));
    let mut trace = Vec::new();
    if closed_first {
        assert_eq!(chunks.len(), 1);
        execution
            .context_mut()
            .input
            .push(chunks[0].iter().copied())
            .unwrap();
        execution.context_mut().input.finish();
        assert!(drain(&mut execution, &mut trace));
        return trace;
    }
    for chunk in chunks {
        execution
            .context_mut()
            .input
            .push(chunk.iter().copied())
            .unwrap();
        if drain(&mut execution, &mut trace) {
            return trace;
        }
        execution.context_mut().input.push([]).unwrap();
        assert!(!drain(&mut execution, &mut trace));
    }
    execution.context_mut().input.finish();
    assert!(drain(&mut execution, &mut trace));
    trace
}

#[test]
fn all_two_chunk_splits_and_single_code_point_chunks_match_closed_source_traces() {
    let mut inputs: Vec<Vec<CodePoint>> = [
        "",
        "scrip",
        "script",
        "SCRIPT",
        "sCrIpT>",
        "script/-->",
        "script>text-->",
        "script\t\0\0",
        "script\r\n\0-->",
        "script\r\r\n\r",
        "script><",
        "scripts>tail",
        "ScriptX/",
        "script-",
        "script<",
        "script=",
        "script&",
        "script\0",
        "script\x0b",
        "ſcript>",
        "scrİpt>",
    ]
    .into_iter()
    .map(points)
    .collect();
    inputs.push(
        [
            points("sCrIpT>\r\n"),
            vec![
                CodePoint::new(0xd800).unwrap(),
                CodePoint::new(0xfdd0).unwrap(),
                '\0'.into(),
                '\x0b'.into(),
            ],
            points("\r\n-->"),
        ]
        .concat(),
    );
    inputs.push(vec![CodePoint::new(0xdfff).unwrap(), 'x'.into()]);
    for seeded in [false, true] {
        for buffer in ["", "scrip", "script", "SCRIPT"] {
            for input in &inputs {
                let expected = trace(&[input], true, seeded, buffer);
                for split in 0..=input.len() {
                    assert_eq!(
                        trace(&[&input[..split], &input[split..]], false, seeded, buffer),
                        expected,
                        "split {split}, {input:?}, prefix {buffer}"
                    );
                }
                assert_eq!(
                    trace(&input.chunks(1).collect::<Vec<_>>(), false, seeded, buffer),
                    expected,
                    "point chunks {input:?}, prefix {buffer}"
                );
            }
        }
    }
}
