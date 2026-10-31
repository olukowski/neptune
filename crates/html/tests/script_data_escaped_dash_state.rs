//! State-local source-step checks, not upstream conformance runs.
//! Pinned HTML 404b3eb14a7a936dddefb2d78e224dbeb01d2358,
//! #script-data-escaped-dash-state, source lines 143805–143835.

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

// Cursor setup only: never execute/reconstruct an earlier terminal stub.
fn entry(mut context: Context, prefix: &str) -> Execution {
    context.input.push(points(prefix)).unwrap();
    while context.input.consume() != Input::NeedMoreInput {}
    Execution::new(State::ScriptDataEscapedDash, context)
}

fn assert_terminal(execution: &mut Execution, result: Result<Progress, UnimplementedState>) {
    let before = snapshot(execution);
    for _ in 0..3 {
        assert_eq!(execution.step(), result);
        assert_eq!(snapshot(execution), before);
    }
}

#[test]
fn every_ascii_branch_and_non_scalar_preserves_storage_and_consumes_exactly_once() {
    let values = (0..=0x7f).chain([
        0x80, 0x9f, 0xa0, 0xc9, 0x3a9, 0xfeff, 0x1f600, 0xd800, 0xdfff, 0xfdd0, 0xfdef, 0xfffe,
        0xffff, 0x10ffff,
    ]);
    for value in values {
        for finished in [false, true] {
            for selected in [None, Some(TagKind::Start), Some(TagKind::End)] {
                let mut context = seeded_context();
                context.current_tag = selected;
                let mut execution = entry(context, "\r\n-");
                let point = CodePoint::new(value).unwrap();
                let normalized = if value == 0x0d { '\n'.into() } else { point };
                // A malformed tail would add a diagnostic if consumed too soon.
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
                let target = match value {
                    0x2d => {
                        expected.tokens.push(Token::Character('-'.into()));
                        State::ScriptDataEscapedDashDash
                    }
                    0x3c => State::ScriptDataEscapedLessThanSign,
                    0 => {
                        expected
                            .errors
                            .push(error("unexpected-null-character", 2, 2));
                        expected.tokens.push(Token::Character('\u{fffd}'.into()));
                        State::ScriptDataEscaped
                    }
                    _ => {
                        expected.tokens.push(Token::Character(normalized));
                        State::ScriptDataEscaped
                    }
                };
                expected.state = target;
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
                assert!(!execution.context().tokens.contains(&Token::Eof));
                let tail_position = if normalized == '\n'.into() {
                    Position { line: 3, column: 1 }
                } else {
                    Position { line: 2, column: 3 }
                };
                expected.input_errors.push(ParseError {
                    code: "surrogate-in-input-stream",
                    position: tail_position,
                });
                if value == 0x3c {
                    // Gateway fallback consumes the surrogate, emits '<', and
                    // reconsumes in Escaped without replacing/diagnosing it again.
                    expected.state = State::ScriptDataEscaped;
                    expected.current = Some((Input::CodePoint(tail), tail_position));
                    expected.tokens.push(Token::Character('<'.into()));
                    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                    assert_eq!(snapshot(&execution), expected);
                }
                // Escaped or the newly integrated DashDash consumes the untouched
                // surrogate once, emits it literally and returns/stays in Escaped.
                expected.state = State::ScriptDataEscaped;
                expected.current = Some((Input::CodePoint(tail), tail_position));
                expected.tokens.push(Token::Character(tail));
                assert_eq!(
                    execution.step(),
                    Ok(Progress::Continue(State::ScriptDataEscaped))
                );
                assert_eq!(snapshot(&execution), expected);
                if !finished {
                    for _ in 0..3 {
                        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                        assert_eq!(snapshot(&execution), expected);
                    }
                    execution.context_mut().input.finish();
                }
                let eof_position = Position {
                    line: tail_position.line,
                    column: tail_position.column + 1,
                };
                expected.current = Some((Input::Eof, eof_position));
                expected.tokens.push(Token::Eof);
                expected.errors.push(ParseError {
                    code: "eof-in-script-html-comment-like-text",
                    position: eof_position,
                });
                assert_eq!(execution.step(), Ok(Progress::Finished));
                assert_eq!(snapshot(&execution), expected);
                assert_terminal(&mut execution, Ok(Progress::Finished));
            }
        }
    }
}

#[test]
fn conceptual_eof_reports_once_and_open_exhaustion_has_no_effects() {
    for prefix in ["", "-", "\r", "\r\n-", "😀\r\nx"] {
        let mut execution = entry(seeded_context(), prefix);
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
    }
}

#[test]
fn default_context_suspends_then_executes_real_end_tag_or_double_escape_destinations() {
    for (text, target, output) in [
        ("-", State::ScriptDataEscapedDashDash, "-"),
        ("<", State::ScriptDataEscapedLessThanSign, ""),
        ("\0", State::ScriptDataEscaped, "\u{fffd}"),
        (">", State::ScriptDataEscaped, ">"),
        ("&", State::ScriptDataEscaped, "&"),
    ] {
        let mut execution = Execution::new(State::ScriptDataEscapedDash, Context::default());
        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
        assert_eq!(execution.context().input.current(), None);
        execution.context_mut().input.push(points(text)).unwrap();
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
        let mut expected = snapshot(&execution);
        if text == "<" {
            for _ in 0..3 {
                assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                assert_eq!(snapshot(&execution), expected);
            }
            execution.context_mut().input.push(points("/tail")).unwrap();
            expected.state = State::ScriptDataEscapedEndTagOpen;
            expected.current = Some((
                Input::CodePoint('/'.into()),
                Position { line: 1, column: 2 },
            ));
            assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&execution), expected);
            expected.state = State::ScriptDataEscapedEndTagName;
            expected.current = Some((
                Input::CodePoint('t'.into()),
                Position { line: 1, column: 3 },
            ));
            expected.storage.current_tag = Some(TagKind::End);
            expected.storage.end = Some(PendingEndTag::default());
            assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&execution), expected);
            for (index, point) in points("tail").into_iter().enumerate() {
                expected.current = Some((
                    Input::CodePoint(point),
                    Position {
                        line: 1,
                        column: index + 3,
                    },
                ));
                expected.storage.end.as_mut().unwrap().name.push(point);
                expected.storage.buffer.push(point);
                assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                assert_eq!(snapshot(&execution), expected);
            }
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&execution), expected);
            execution.context_mut().input.finish();
            expected.state = State::ScriptDataEscaped;
            expected.current = Some((Input::Eof, Position { line: 1, column: 7 }));
            expected
                .tokens
                .extend(points("</tail").into_iter().map(Token::Character));
            assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&execution), expected);
            expected.tokens.push(Token::Eof);
            expected
                .errors
                .push(error("eof-in-script-html-comment-like-text", 1, 7));
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&execution), expected);
            assert_terminal(&mut execution, Ok(Progress::Finished));
            continue;
        }
        for _ in 0..3 {
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&execution), expected);
        }
        execution
            .context_mut()
            .input
            .push(points("\0<tail"))
            .unwrap();
        expected.current = Some((
            Input::CodePoint('\0'.into()),
            Position { line: 1, column: 2 },
        ));
        expected.state = State::ScriptDataEscaped;
        expected.tokens.push(Token::Character('\u{fffd}'.into()));
        expected
            .errors
            .push(error("unexpected-null-character", 1, 2));
        assert_eq!(
            execution.step(),
            Ok(Progress::Continue(State::ScriptDataEscaped))
        );
        assert_eq!(snapshot(&execution), expected);
        expected.current = Some((
            Input::CodePoint('<'.into()),
            Position { line: 1, column: 3 },
        ));
        expected.state = State::ScriptDataEscapedLessThanSign;
        assert_eq!(
            execution.step(),
            Ok(Progress::Continue(State::ScriptDataEscapedLessThanSign))
        );
        assert_eq!(snapshot(&execution), expected);
        expected.state = State::ScriptDataDoubleEscapeStart;
        expected.current = Some((
            Input::CodePoint('t'.into()),
            Position { line: 1, column: 4 },
        ));
        expected.tokens.push(Token::Character('<'.into()));
        assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
        assert_eq!(snapshot(&execution), expected);
        for (index, point) in points("tail").into_iter().enumerate() {
            expected.current = Some((
                Input::CodePoint(point),
                Position {
                    line: 1,
                    column: 4 + index,
                },
            ));
            expected.storage.buffer.push(point);
            expected.tokens.push(Token::Character(point));
            assert_eq!(
                execution.step(),
                Ok(Progress::Continue(State::ScriptDataDoubleEscapeStart))
            );
            assert_eq!(snapshot(&execution), expected);
        }

        for _ in 0..3 {
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&execution), expected);
        }
        execution.context_mut().input.finish();

        expected.state = State::ScriptDataEscaped;
        expected.current = Some((
            Input::Eof,
            Position {
                line: 1,
                column: 4 + 4,
            },
        ));
        assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
        assert_eq!(snapshot(&execution), expected);
        expected.tokens.push(Token::Eof);
        expected
            .errors
            .push(error("eof-in-script-html-comment-like-text", 1, 4 + 4));
        for _ in 0..3 {
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&execution), expected);
        }
    }
}

#[test]
fn incoming_reconsumption_keeps_positions_and_does_not_duplicate_preprocessing_errors() {
    for value in [0, 0xd800, 0xfdd0, 0x0b, 0x2d, 0x3c] {
        let point = CodePoint::new(value).unwrap();
        let mut context = Context::default();
        context.input.push([point, 'z'.into()]).unwrap();
        assert_eq!(context.input.consume(), Input::CodePoint(point));
        let input_errors = context.input.errors().to_vec();
        context.input.reconsume().unwrap();
        assert_eq!(
            context.input.reconsume(),
            Err(CursorError::AlreadyReconsuming)
        );
        let mut execution = Execution::new(State::ScriptDataEscapedDash, context);
        assert!(matches!(execution.step(), Ok(Progress::Continue(_))));
        assert_eq!(
            execution.context().input.current(),
            Some((Input::CodePoint(point), Position { line: 1, column: 1 }))
        );
        assert_eq!(execution.context().input.errors(), input_errors);
        assert_eq!(
            execution.context_mut().input.consume(),
            Input::CodePoint('z'.into())
        );
    }
    let mut context = Context::default();
    context.input.finish();
    assert_eq!(context.input.consume(), Input::Eof);
    context.input.reconsume().unwrap();
    let mut execution = Execution::new(State::ScriptDataEscapedDash, context);
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(
        execution.context().errors,
        [error("eof-in-script-html-comment-like-text", 1, 1)]
    );
    assert_eq!(execution.context().tokens, [Token::Eof]);
    assert_terminal(&mut execution, Ok(Progress::Finished));
}

#[test]
fn split_crlf_is_suppressed_before_nul_or_conceptual_eof_at_normalized_position() {
    for nul in [false, true] {
        let mut execution = entry(Context::default(), "\r");
        execution.context_mut().input.push(points("\n")).unwrap();
        let before = snapshot(&execution);
        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
        assert_eq!(snapshot(&execution), before);
        if nul {
            execution.context_mut().input.push(points("\0")).unwrap();
        }
        execution.context_mut().input.finish();
        let result = execution.step();
        assert_eq!(
            execution.context().input.position(),
            Position { line: 2, column: 1 }
        );
        assert!(execution.context().input.errors().is_empty());
        if nul {
            assert_eq!(result, Ok(Progress::Continue(State::ScriptDataEscaped)));
            assert_eq!(
                execution.context().tokens,
                [Token::Character('\u{fffd}'.into())]
            );
            assert_eq!(
                execution.context().errors,
                [error("unexpected-null-character", 2, 1)]
            );
            let mut expected = snapshot(&execution);
            expected.current = Some((Input::Eof, Position { line: 2, column: 2 }));
            expected
                .errors
                .push(error("eof-in-script-html-comment-like-text", 2, 2));
            expected.tokens.push(Token::Eof);
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&execution), expected);
            assert_terminal(&mut execution, Ok(Progress::Finished));
        } else {
            assert_eq!(result, Ok(Progress::Finished));
            assert_eq!(execution.context().tokens, [Token::Eof]);
            assert_eq!(
                execution.context().errors,
                [error("eof-in-script-html-comment-like-text", 2, 1)]
            );
            assert_terminal(&mut execution, Ok(Progress::Finished));
        }
    }
}

#[test]
fn script_data_entry_reaches_real_escaped_dash_dash_and_reports_escaped_eof() {
    let mut context = Context::default();
    context.input.push(points("<!---")).unwrap();
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
    assert!(execution.context().errors.is_empty());
    let mut expected = snapshot(&execution);
    for (column, state) in [
        (3, State::ScriptDataEscapeStartDash),
        (4, State::ScriptDataEscapedDashDash),
        (5, State::ScriptDataEscapedDashDash),
    ] {
        expected.state = state;
        expected.current = Some((Input::CodePoint('-'.into()), Position { line: 1, column }));
        expected.tokens.push(Token::Character('-'.into()));
        assert_eq!(execution.step(), Ok(Progress::Continue(state)));
        assert_eq!(snapshot(&execution), expected);
    }
    expected.current = Some((Input::Eof, Position { line: 1, column: 6 }));
    expected
        .errors
        .push(error("eof-in-script-html-comment-like-text", 1, 6));
    expected.tokens.push(Token::Eof);
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(snapshot(&execution), expected);
    assert_terminal(&mut execution, Ok(Progress::Finished));
}

type Trace = Vec<(Result<Progress, UnimplementedState>, Snapshot)>;

fn drain(execution: &mut Execution, trace: &mut Trace, bound: usize) -> bool {
    // Trace real EscapedDash/Escaped/DashDash steps until suspension, EOF, or a stub.
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
    panic!("source step did not suspend or terminate");
}

fn trace(chunks: &[&[CodePoint]], closed: bool, prefix: &str) -> Trace {
    let mut execution = entry(seeded_context(), prefix);
    let mut trace = vec![];
    let bound = chunks.iter().map(|chunk| chunk.len()).sum::<usize>() + 3;
    for chunk in chunks {
        execution
            .context_mut()
            .input
            .push(chunk.iter().copied())
            .unwrap();
        if !closed && drain(&mut execution, &mut trace, bound) {
            // Never feed/reconstruct/resume after a terminal block.
            return trace;
        }
    }
    execution.context_mut().input.finish();
    assert!(drain(&mut execution, &mut trace, bound));
    trace
}

#[test]
fn all_two_chunk_splits_and_single_point_chunks_match_closed_source_step_traces() {
    let mut inputs: Vec<_> = [
        "",
        "-",
        "-->",
        "<",
        "</script>",
        ">",
        "&amp;",
        "!",
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
        "x-\0a-\r\n\0&😀\r",
        "x-a--literal\0",
        "x-a-<unconsumed\0",
        "--a--\0--->\r\n\0",
        "--a--\0---<\r\n\0",
        "x--\0--a---\0--<tail",
        "x--\0--a---><!tail",
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
    for input in inputs {
        for prefix in ["", "\r", "\r\n-"] {
            let expected = trace(&[&input], true, prefix);
            for split in 0..=input.len() {
                assert_eq!(
                    trace(&[&input[..split], &[], &input[split..]], false, prefix),
                    expected,
                    "split {split} of {input:?}, prefix {prefix:?}"
                );
            }
            assert_eq!(
                trace(&input.chunks(1).collect::<Vec<_>>(), false, prefix),
                expected,
                "{input:?}, prefix {prefix:?}"
            );
        }
    }
}
