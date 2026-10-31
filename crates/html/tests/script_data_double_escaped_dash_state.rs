//! Local source-step evidence, not upstream conformance runs.
//! Pinned HTML 404b3eb14a7a936dddefb2d78e224dbeb01d2358,
//! #script-data-double-escaped-dash-state, source lines 144024–144055.

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
        name: points("Name"),
        value: vec![CodePoint::new(0xd800).unwrap(), '\0'.into()],
    };
    Context {
        tokens: vec![Token::Character('p'.into())],
        errors: vec![error("prior-diagnostic", 1, 1)],
        last_start_tag: Some(points("ScRiPt")),
        return_state: Some(State::AttributeValueSingleQuoted),
        current_tag: Some(TagKind::End),
        pending_start_tag: Some(PendingStartTag {
            name: points("Start"),
            self_closing: true,
            attributes: vec![attribute.clone()],
            discarded_current_attribute: Some(attribute.clone()),
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("End"),
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

// Cursor setup only, not a reconstructed/resumed execution or a neighbor algorithm.
fn entry(mut context: Context, prefix: &str) -> Execution {
    context.input.push(points(prefix)).unwrap();
    while context.input.consume() != Input::NeedMoreInput {}
    Execution::new(State::ScriptDataDoubleEscapedDash, context)
}

fn assert_terminal(execution: &mut Execution, result: Result<Progress, UnimplementedState>) {
    let before = snapshot(execution);
    for _ in 0..3 {
        assert_eq!(execution.step(), result);
        assert_eq!(snapshot(execution), before);
    }
}

#[test]
fn every_ascii_branch_and_non_scalar_has_exact_source_effects_before_destination_stub() {
    // Every ASCII value plus non-ASCII letters, hyphen lookalikes and malformed points.
    let values = (0..=0x7f).chain([
        0x80, 0x9f, 0xa0, 0xc9, 0x3a9, 0x2010, 0x2212, 0xfeff, 0x1f600, 0xd800, 0xdfff, 0xfdd0,
        0xfdef, 0xfffe, 0xffff, 0x10ffff,
    ]);
    for value in values {
        for finished in [false, true] {
            for selected in [None, Some(TagKind::Start), Some(TagKind::End)] {
                let mut context = seeded_context();
                context.current_tag = selected;
                let mut execution = entry(context, "\r\n-");
                let point = CodePoint::new(value).unwrap();
                let normalized = if value == 0x0d { '\n'.into() } else { point };
                let tail = CodePoint::new(0xdfff).unwrap();
                execution.context_mut().input.push([point, tail]).unwrap();
                if finished {
                    execution.context_mut().input.finish();
                }
                let mut expected = snapshot(&execution);
                expected.current = Some((
                    Input::CodePoint(normalized),
                    Position { line: 2, column: 2 },
                ));
                // Normative branches: 144032–144053. '<' must emit here too.
                let target = match value {
                    0x2d => State::ScriptDataDoubleEscapedDashDash,
                    0x3c => State::ScriptDataDoubleEscapedLessThanSign,
                    _ => State::ScriptDataDoubleEscaped,
                };
                expected.state = target;
                expected.tokens.push(Token::Character(if value == 0 {
                    expected
                        .errors
                        .push(error("unexpected-null-character", 2, 2));
                    '\u{fffd}'.into()
                } else {
                    normalized
                }));
                let diagnostic = match value {
                    0xd800..=0xdfff => Some("surrogate-in-input-stream"),
                    0xfdd0..=0xfdef | 0xfffe | 0xffff | 0x10ffff => {
                        Some("noncharacter-in-input-stream")
                    }
                    1..=8 | 0x0b | 0x0e..=0x1f | 0x7f..=0x9f => {
                        Some("control-character-in-input-stream")
                    }
                    _ => None,
                };
                expected.input_errors = diagnostic
                    .into_iter()
                    .map(|code| error(code, 2, 2))
                    .collect();
                assert_eq!(execution.step(), Ok(Progress::Continue(target)));
                assert_eq!(snapshot(&execution), expected, "U+{value:04X}");
                // Destination dispatch cannot consume even on closed/exhausted input.
                assert_terminal(&mut execution, Err(UnimplementedState { state: target }));
                // Inspect the cursor only; never resume the terminal execution.
                assert_eq!(
                    execution.context_mut().input.consume(),
                    Input::CodePoint(tail)
                );
                let tail_position = if normalized == '\n'.into() {
                    Position { line: 3, column: 1 }
                } else {
                    Position { line: 2, column: 3 }
                };
                assert_eq!(execution.context().input.position(), tail_position);
                expected.input_errors.push(ParseError {
                    code: "surrogate-in-input-stream",
                    position: tail_position,
                });
                assert_eq!(execution.context().input.errors(), expected.input_errors);
            }
        }
    }
}

#[test]
fn default_direct_entry_needs_no_tag_or_scratch_setup_and_all_destinations_remain_terminal() {
    for (text, target, output) in [
        ("-", State::ScriptDataDoubleEscapedDashDash, "-"),
        ("<", State::ScriptDataDoubleEscapedLessThanSign, "<"),
        ("\0", State::ScriptDataDoubleEscaped, "\u{fffd}"),
        (">", State::ScriptDataDoubleEscaped, ">"),
        ("&", State::ScriptDataDoubleEscaped, "&"),
        ("A", State::ScriptDataDoubleEscaped, "A"),
    ] {
        for finished in [false, true] {
            let mut execution =
                Execution::new(State::ScriptDataDoubleEscapedDash, Context::default());
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(execution.context().input.current(), None);
            execution.context_mut().input.push(points(text)).unwrap();
            if finished {
                execution.context_mut().input.finish();
            }
            assert_eq!(execution.step(), Ok(Progress::Continue(target)));
            assert_eq!(storage(execution.context()), storage(&Context::default()));
            assert_eq!(
                execution.context().tokens,
                points(output)
                    .into_iter()
                    .map(Token::Character)
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                execution.context().errors,
                if text == "\0" {
                    vec![error("unexpected-null-character", 1, 1)]
                } else {
                    vec![]
                }
            );
            assert!(execution.context().input.errors().is_empty());
            assert_terminal(&mut execution, Err(UnimplementedState { state: target }));
            // In particular, neither an exhausted open destination nor closed EOF
            // substitutes NeedMoreInput or Finished for the typed development block.
        }
    }
}

#[test]
fn open_exhaustion_preserves_current_item_and_only_explicit_eof_reports_once() {
    for prefix in ["", "-", "\r", "\r\n-", "😀\r\nx"] {
        for context in [Context::default(), seeded_context()] {
            let mut execution = entry(context, prefix);
            let before = snapshot(&execution);
            for _ in 0..3 {
                execution.context_mut().input.push([]).unwrap();
                assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                assert_eq!(snapshot(&execution), before);
            }
            execution.context_mut().input.finish();
            execution.context_mut().input.finish();
            let (line, column) = match prefix {
                "" => (1, 1),
                "-" => (1, 2),
                "\r" => (2, 1),
                _ => (2, 2),
            };
            let mut expected = before;
            expected.current = Some((Input::Eof, Position { line, column }));
            expected.tokens.push(Token::Eof);
            expected
                .errors
                .push(error("eof-in-script-html-comment-like-text", line, column));
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&execution), expected);
            assert_terminal(&mut execution, Ok(Progress::Finished));
            assert_eq!(
                execution.context_mut().input.push(points("z")),
                Err(CursorError::InputAlreadyFinished)
            );
        }
    }
}

#[test]
fn incoming_reconsumption_including_eof_keeps_positions_and_once_only_preprocessing_errors() {
    for value in [0, 0xd800, 0xfdd0, 0x0b, 0x2d, 0x3c, 0x3e, 0x41] {
        let point = CodePoint::new(value).unwrap();
        let mut context = seeded_context();
        context.input.push([point, 'z'.into()]).unwrap();
        assert_eq!(context.input.consume(), Input::CodePoint(point));
        let input_errors = context.input.errors().to_vec();
        context.input.reconsume().unwrap();
        assert_eq!(
            context.input.reconsume(),
            Err(CursorError::AlreadyReconsuming)
        );
        let mut execution = Execution::new(State::ScriptDataDoubleEscapedDash, context);
        let mut expected = snapshot(&execution);
        let target = match value {
            0x2d => State::ScriptDataDoubleEscapedDashDash,
            0x3c => State::ScriptDataDoubleEscapedLessThanSign,
            _ => State::ScriptDataDoubleEscaped,
        };
        expected.state = target;
        expected.tokens.push(Token::Character(if value == 0 {
            '\u{fffd}'.into()
        } else {
            point
        }));
        if value == 0 {
            expected
                .errors
                .push(error("unexpected-null-character", 1, 1));
        }
        assert_eq!(execution.step(), Ok(Progress::Continue(target)));
        assert_eq!(snapshot(&execution), expected);
        assert_eq!(execution.context().input.errors(), input_errors);
        // No outgoing reconsumption: the next item is the untouched tail.
        assert_eq!(
            execution.context_mut().input.consume(),
            Input::CodePoint('z'.into())
        );
        assert_eq!(
            execution.context().input.position(),
            Position { line: 1, column: 2 }
        );
    }
    let mut context = seeded_context();
    context.input.finish();
    assert_eq!(context.input.consume(), Input::Eof);
    context.input.reconsume().unwrap();
    let mut execution = Execution::new(State::ScriptDataDoubleEscapedDash, context);
    let mut expected = snapshot(&execution);
    expected.tokens.push(Token::Eof);
    expected
        .errors
        .push(error("eof-in-script-html-comment-like-text", 1, 1));
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(snapshot(&execution), expected);
    assert_terminal(&mut execution, Ok(Progress::Finished));
}

#[test]
fn split_crlf_suppresses_lf_before_source_nul_or_conceptual_eof() {
    for nul in [false, true] {
        let mut execution = entry(seeded_context(), "\r");
        execution.context_mut().input.push(points("\n")).unwrap();
        let mut expected = snapshot(&execution);
        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
        assert_eq!(snapshot(&execution), expected);
        if nul {
            execution.context_mut().input.push(points("\0")).unwrap();
        }
        execution.context_mut().input.finish();
        expected.current = Some((
            if nul {
                Input::CodePoint('\0'.into())
            } else {
                Input::Eof
            },
            Position { line: 2, column: 1 },
        ));
        if nul {
            expected.state = State::ScriptDataDoubleEscaped;
            expected.tokens.push(Token::Character('\u{fffd}'.into()));
            expected
                .errors
                .push(error("unexpected-null-character", 2, 1));
            assert_eq!(
                execution.step(),
                Ok(Progress::Continue(State::ScriptDataDoubleEscaped))
            );
        } else {
            expected.tokens.push(Token::Eof);
            expected
                .errors
                .push(error("eof-in-script-html-comment-like-text", 2, 1));
            assert_eq!(execution.step(), Ok(Progress::Finished));
        }
        assert_eq!(snapshot(&execution), expected);
        assert_terminal(
            &mut execution,
            if nul {
                Err(UnimplementedState {
                    state: State::ScriptDataDoubleEscaped,
                })
            } else {
                Ok(Progress::Finished)
            },
        );
    }
}

#[test]
fn real_script_path_and_direct_predecessor_are_still_blocked_before_assigned_state() {
    let mut context = seeded_context();
    context.input.push(points("<!--<script>-\0")).unwrap();
    context.input.finish();
    let mut execution = Execution::new(State::ScriptData, context);
    let mut expected = snapshot(&execution);
    for (column, point, target, output) in [
        (1, '<', State::ScriptDataLessThanSign, ""),
        (2, '!', State::ScriptDataEscapeStart, "<!"),
        (3, '-', State::ScriptDataEscapeStartDash, "-"),
        (4, '-', State::ScriptDataEscapedDashDash, "-"),
        (5, '<', State::ScriptDataEscapedLessThanSign, ""),
    ] {
        expected.state = target;
        expected.current = Some((Input::CodePoint(point.into()), Position { line: 1, column }));
        expected
            .tokens
            .extend(points(output).into_iter().map(Token::Character));
        assert_eq!(execution.step(), Ok(Progress::Continue(target)));
        assert_eq!(snapshot(&execution), expected);
    }
    assert_terminal(
        &mut execution,
        Err(UnimplementedState {
            state: State::ScriptDataEscapedLessThanSign,
        }),
    );
    assert_eq!(
        execution.context_mut().input.consume(),
        Input::CodePoint('s'.into())
    );

    // The normative predecessor's hyphen branch (144000–144002) is unimplemented.
    // Do not fabricate its '-' emission or reconstruct after its terminal block.
    let mut context = seeded_context();
    context.input.push(points("-\0")).unwrap();
    context.input.finish();
    let mut predecessor = Execution::new(State::ScriptDataDoubleEscaped, context);
    assert_terminal(
        &mut predecessor,
        Err(UnimplementedState {
            state: State::ScriptDataDoubleEscaped,
        }),
    );
    assert_eq!(
        predecessor.context_mut().input.consume(),
        Input::CodePoint('-'.into())
    );
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
            assert_terminal(execution, result);
            return true;
        }
    }
    panic!("execution did not suspend or terminate");
}

fn trace(chunks: &[&[CodePoint]], closed: bool, prefix: &str, initial: State) -> Trace {
    let mut context = seeded_context();
    context.input.push(points(prefix)).unwrap();
    while context.input.consume() != Input::NeedMoreInput {}
    let mut execution = Execution::new(initial, context);
    let mut trace = vec![];
    let bound = chunks.iter().map(|chunk| chunk.len()).sum::<usize>() + 5;
    for chunk in chunks {
        execution
            .context_mut()
            .input
            .push(chunk.iter().copied())
            .unwrap();
        if !closed && drain(&mut execution, &mut trace, bound) {
            // Stop immediately: never feed, reconstruct or resume terminal runs.
            return trace;
        }
    }
    execution.context_mut().input.finish();
    assert!(drain(&mut execution, &mut trace, bound));
    trace
}

#[test]
fn every_two_chunk_split_and_single_point_chunking_match_closed_source_step_traces() {
    let mut inputs: Vec<_> = [
        "",
        "-",
        "-->",
        "<",
        "</script>",
        ">",
        "&amp;",
        "A",
        "a",
        "\0",
        "\0\0tail",
        "\r",
        "\r\n",
        "\r\n\0",
        "\n\0",
        "\r\r\n",
        "É😀",
        "\u{b}x",
        "\u{fdd0}x",
        "<!--<script>-\0",
        "<!--<script>--",
        "<!--<script>-<",
        "<!--<script>-\r\n",
    ]
    .into_iter()
    .map(points)
    .collect();
    inputs.push(vec![
        CodePoint::new(0xd800).unwrap(),
        '\r'.into(),
        '\n'.into(),
    ]);
    inputs.push(vec![CodePoint::new(0x10ffff).unwrap(), '\0'.into()]);
    for initial in [State::ScriptDataDoubleEscapedDash, State::ScriptData] {
        for input in &inputs {
            for prefix in ["", "\r", "\r\n-"] {
                let expected = trace(&[input], true, prefix, initial);
                for split in 0..=input.len() {
                    assert_eq!(
                        trace(
                            &[&input[..split], &[], &input[split..]],
                            false,
                            prefix,
                            initial
                        ),
                        expected,
                        "split {split} of {input:?}, prefix {prefix:?}, {initial:?}"
                    );
                }
                assert_eq!(
                    trace(&input.chunks(1).collect::<Vec<_>>(), false, prefix, initial),
                    expected,
                    "{input:?}, prefix {prefix:?}, {initial:?}"
                );
            }
        }
    }
}
