//! Local source-step evidence, not upstream conformance. Pinned HTML revision
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, source lines 144093–144108.
//! DoubleEscapeEnd and both Dash/DashDash predecessors are implemented on the
//! integrated base. Ordinary Script data entry now reaches the complete double-escaped loop.

use neptune_html::tokenizer::{
    Attribute, CodePoint, Context, CursorError, Execution, Input, ParseError, PendingComment,
    PendingEndTag, PendingStartTag, Position, Progress, State, TagKind, Token, UnimplementedState,
};

const SOURCE: State = State::ScriptDataDoubleEscapedLessThanSign;

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
        name: points("MiXeD"),
        value: vec![CodePoint::new(0xd800).unwrap(), '\0'.into()],
    };
    Context {
        tokens: vec![Token::Character('p'.into())],
        errors: vec![error("prior-diagnostic", 1, 1)],
        last_start_tag: Some(points("script")),
        return_state: Some(State::AttributeValueSingleQuoted),
        current_tag: Some(TagKind::End),
        pending_start_tag: Some(PendingStartTag {
            name: points("StArT"),
            self_closing: true,
            attributes: vec![attribute.clone()],
            discarded_current_attribute: Some(attribute.clone()),
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("EnD"),
            self_closing: true,
            attributes: vec![attribute.clone()],
            discarded_current_attribute: Some(attribute),
        }),
        pending_comment: Some(PendingComment {
            data: points("comment"),
        }),
        temporary_buffer: vec!['S'.into(), CodePoint::new(0xdfff).unwrap(), '\0'.into()],
        ..Default::default()
    }
}

#[derive(Debug, Eq, PartialEq)]
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
    buffer: Vec<CodePoint>,
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
        buffer: context.temporary_buffer.clone(),
    }
}

// Isolated context setup, not execution of a sibling algorithm: all three
// immediate predecessors have consumed AND emitted '<' on entry here.
fn entry(mut context: Context, newline_prefix: bool) -> Execution {
    if newline_prefix {
        context.input.push(points("\r\n")).unwrap();
        assert_eq!(context.input.consume(), Input::CodePoint('\n'.into()));
    }
    context.input.push(points("<")).unwrap();
    assert_eq!(context.input.consume(), Input::CodePoint('<'.into()));
    context.tokens.push(Token::Character('<'.into()));
    Execution::new(SOURCE, context)
}

fn finish_at(execution: &mut Execution, expected: &mut Snapshot, line: usize, column: usize) {
    expected.current = Some((Input::Eof, Position { line, column }));
    expected.tokens.push(Token::Eof);
    expected
        .errors
        .push(error("eof-in-script-html-comment-like-text", line, column));
    for _ in 0..3 {
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(execution), *expected);
    }
}

#[test]
fn every_ascii_branch_unicode_and_non_scalars_have_exact_source_effects() {
    for value in (0..=0x7f).chain([
        0x80, 0x9f, 0xa0, 0xc9, 0x3a9, 0x2028, 0xfeff, 0xfffd, 0x1f600, 0xd800, 0xdfff, 0xfdd0,
        0xfdef, 0xffff, 0x1fffe, 0x10ffff,
    ]) {
        for finished in [false, true] {
            for selected in [None, Some(TagKind::Start), Some(TagKind::End)] {
                let point = CodePoint::new(value).unwrap();
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
                    Input::CodePoint(if value == 0x0d { '\n'.into() } else { point }),
                    Position { line: 2, column: 2 },
                ));
                let target = if value == 0x2f {
                    expected.buffer.clear();
                    expected.tokens.push(Token::Character('/'.into()));
                    State::ScriptDataDoubleEscapeEnd
                } else {
                    State::ScriptDataDoubleEscaped
                };
                expected.state = target;
                let diagnostic = match value {
                    0xd800..=0xdfff => Some("surrogate-in-input-stream"),
                    0xfdd0..=0xfdef | 0xffff | 0x1fffe | 0x10ffff => {
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
                if value != 0x2f {
                    assert_eq!(
                        execution.context_mut().input.reconsume(),
                        Err(CursorError::AlreadyReconsuming)
                    );
                }
                // Source checks above remain separate from destination effects.
                if value == 0x2f {
                    expected.current = Some((
                        Input::CodePoint('z'.into()),
                        Position { line: 2, column: 3 },
                    ));
                    expected.buffer.push('z'.into());
                    expected.tokens.push(Token::Character('z'.into()));
                    assert_eq!(execution.step(), Ok(Progress::Continue(target)));
                    assert_eq!(snapshot(&execution), expected);
                    if !finished {
                        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                        assert_eq!(snapshot(&execution), expected);
                        execution.context_mut().input.finish();
                    }
                    expected.state = State::ScriptDataDoubleEscaped;
                    expected.current = Some((Input::Eof, Position { line: 2, column: 4 }));
                    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                    assert_eq!(snapshot(&execution), expected);
                    assert_eq!(
                        execution.context_mut().input.reconsume(),
                        Err(CursorError::AlreadyReconsuming)
                    );
                    finish_at(&mut execution, &mut expected, 2, 4);
                } else {
                    let normalized = if value == 0x0d { '\n'.into() } else { point };
                    expected.state = match value {
                        0x2d => State::ScriptDataDoubleEscapedDash,
                        0x3c => SOURCE,
                        _ => State::ScriptDataDoubleEscaped,
                    };
                    expected.tokens.push(Token::Character(if value == 0 {
                        expected
                            .errors
                            .push(error("unexpected-null-character", 2, 2));
                        '\u{fffd}'.into()
                    } else {
                        normalized
                    }));
                    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                    assert_eq!(snapshot(&execution), expected);
                    let (line, column) = if value == 0x0a || value == 0x0d {
                        (3, 1)
                    } else {
                        (2, 3)
                    };
                    expected.current =
                        Some((Input::CodePoint('z'.into()), Position { line, column }));
                    expected.state = State::ScriptDataDoubleEscaped;
                    if value == 0x3c {
                        // The emitted '<' leads back here; z is reconsumed once.
                        assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                        assert_eq!(snapshot(&execution), expected);
                        assert_eq!(
                            execution.context_mut().input.reconsume(),
                            Err(CursorError::AlreadyReconsuming)
                        );
                    }
                    expected.tokens.push(Token::Character('z'.into()));
                    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                    assert_eq!(snapshot(&execution), expected);
                    if !finished {
                        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                        assert_eq!(snapshot(&execution), expected);
                        execution.context_mut().input.finish();
                    }
                    finish_at(&mut execution, &mut expected, line, column + 1);
                }
            }
        }
    }
}

#[test]
fn cursor_probes_before_destination_attempt_prove_consumption_and_reconsumption() {
    for value in [0x2f, 0, 0x41, 0x3c, 0x21, 0x26, 0x0d, 0xd800, 0xffff, 0x0b] {
        let point = CodePoint::new(value).unwrap();
        let normalized = if value == 0x0d { '\n'.into() } else { point };
        let mut execution = entry(seeded_context(), false);
        execution
            .context_mut()
            .input
            .push([point, 'z'.into()])
            .unwrap();
        let target = if value == 0x2f {
            State::ScriptDataDoubleEscapeEnd
        } else {
            State::ScriptDataDoubleEscaped
        };
        assert_eq!(execution.step(), Ok(Progress::Continue(target)));
        let errors = execution.context().input.errors().to_vec();
        // Inspect only the cursor, before any stub attempt. This does not
        // implement, reconstruct or resume the destination.
        if value != 0x2f {
            assert_eq!(
                execution.context_mut().input.consume(),
                Input::CodePoint(normalized)
            );
            assert_eq!(
                execution.context().input.position(),
                Position { line: 1, column: 2 }
            );
            assert_eq!(execution.context().input.errors(), errors);
        }
        assert_eq!(
            execution.context_mut().input.consume(),
            Input::CodePoint('z'.into())
        );
        assert_eq!(
            execution.context().input.position(),
            if normalized == '\n'.into() {
                Position { line: 2, column: 1 }
            } else {
                Position { line: 1, column: 3 }
            }
        );
        assert_eq!(execution.context().input.errors(), errors);
    }
}

#[test]
fn open_exhaustion_is_effect_free_but_eof_is_reconsumed_without_emission() {
    for newline_prefix in [false, true] {
        let mut execution = entry(seeded_context(), newline_prefix);
        let before = snapshot(&execution);
        for _ in 0..3 {
            execution.context_mut().input.push([]).unwrap();
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&execution), before);
        }
        execution.context_mut().input.finish();
        let mut expected = before;
        expected.state = State::ScriptDataDoubleEscaped;
        expected.current = Some((
            Input::Eof,
            Position {
                line: if newline_prefix { 2 } else { 1 },
                column: 2,
            },
        ));
        assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
        assert_eq!(snapshot(&execution), expected);
        assert_eq!(
            execution.context_mut().input.reconsume(),
            Err(CursorError::AlreadyReconsuming)
        );
        // Actual destination reconsumes EOF, diagnosing/emitting exactly once.
        finish_at(
            &mut execution,
            &mut expected,
            if newline_prefix { 2 } else { 1 },
            2,
        );
    }
}

#[test]
fn default_storage_and_suspended_entry_need_no_tag_or_scratch_prerequisite() {
    for slash in [false, true] {
        let mut execution = Execution::new(SOURCE, Context::default());
        let before = snapshot(&execution);
        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
        assert_eq!(snapshot(&execution), before);
        if slash {
            execution.context_mut().input.push(points("/")).unwrap();
        } else {
            execution.context_mut().input.finish();
        }
        let mut expected = before;
        expected.state = if slash {
            State::ScriptDataDoubleEscapeEnd
        } else {
            State::ScriptDataDoubleEscaped
        };
        expected.current = Some((
            if slash {
                Input::CodePoint('/'.into())
            } else {
                Input::Eof
            },
            Position { line: 1, column: 1 },
        ));
        if slash {
            expected.tokens.push(Token::Character('/'.into()));
        }
        assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
        assert_eq!(snapshot(&execution), expected);
        if slash {
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&execution), expected);
            execution.context_mut().input.finish();
            expected.state = State::ScriptDataDoubleEscaped;
            expected.current = Some((Input::Eof, Position { line: 1, column: 2 }));
            assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&execution), expected);
            assert_eq!(
                execution.context_mut().input.reconsume(),
                Err(CursorError::AlreadyReconsuming)
            );
        }
        finish_at(&mut execution, &mut expected, 1, if slash { 2 } else { 1 });
    }
}

// Stop immediately at a typed stub or actual completion, never feed/resume it.
// Snapshots retain every real source step and its result, including destination
// effects; only effect-free suspension is excluded from the comparable trace.
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
                        State::CharacterReference | State::MarkupDeclarationOpen
                    ));
                    assert_eq!(snapshot(execution), before);
                }
                let terminal = snapshot(execution);
                assert_eq!(execution.step(), result);
                assert_eq!(snapshot(execution), terminal);
                return true;
            }
            Ok(Progress::NeedMoreInput) => unreachable!(),
        }
    }
}

fn trace(
    chunks: &[&[CodePoint]],
    closed_first: bool,
) -> Vec<(Result<Progress, UnimplementedState>, Snapshot)> {
    let mut execution = entry(seeded_context(), true);
    let mut trace = vec![];
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
fn all_two_chunk_splits_and_single_point_chunks_match_closed_source_steps() {
    let mut inputs = [
        "",
        "/",
        "/ScRiPt>",
        "/script>\r\n\0\0-->",
        "/script>\r\r\n",
        "/scrip>",
        "/scriptx>",
        "/script<",
        "/script",
        "/\0",
        "/\r\n",
        "/script>text<",
        "!x",
        "<",
        "&amp;",
        "\0\0",
        "\r\n",
        "\r\r",
        "x\r\n",
        "\r",
    ]
    .map(points)
    .to_vec();
    inputs.push(vec![
        CodePoint::new(0xd800).unwrap(),
        '\r'.into(),
        '\n'.into(),
    ]);
    inputs.push(vec![CodePoint::new(0x1fffe).unwrap(), '\0'.into()]);
    inputs.push(vec!['\u{b}'.into(), '\r'.into()]);
    inputs.push(vec![
        '/'.into(),
        CodePoint::new(0xd800).unwrap(),
        '\0'.into(),
    ]);
    let mut escaped_malformed = points("/ScRiPt>\r\n\0\0");
    escaped_malformed.extend([
        CodePoint::new(0xd800).unwrap(),
        CodePoint::new(0x1fffe).unwrap(),
        '\u{b}'.into(),
    ]);
    inputs.push(escaped_malformed);
    for input in inputs {
        let expected = trace(&[&input], true);
        for split in 0..=input.len() {
            assert_eq!(
                trace(&[&input[..split], &input[split..]], false),
                expected,
                "split {split}, {input:?}"
            );
        }
        assert_eq!(trace(&input.chunks(1).collect::<Vec<_>>(), false), expected);
    }
}

#[test]
fn real_double_escape_end_preserves_spelling_and_normalizes_scratch_for_every_match() {
    for mask in 0..64 {
        let spelling: String = "script"
            .chars()
            .enumerate()
            .map(|(index, letter)| {
                if mask & (1 << index) != 0 {
                    letter.to_ascii_uppercase()
                } else {
                    letter
                }
            })
            .collect();
        for delimiter in ['\t', '\n', '\x0c', ' ', '/', '>', '\r'] {
            let mut execution = entry(seeded_context(), true);
            execution
                .context_mut()
                .input
                .push(points(&format!("/{spelling}{delimiter}x")))
                .unwrap();
            execution.context_mut().input.finish();
            let mut expected = snapshot(&execution);
            expected.state = State::ScriptDataDoubleEscapeEnd;
            expected.current = Some((
                Input::CodePoint('/'.into()),
                Position { line: 2, column: 2 },
            ));
            expected.tokens.push(Token::Character('/'.into()));
            expected.buffer.clear();
            assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&execution), expected);
            for (index, letter) in spelling.chars().enumerate() {
                expected.current = Some((
                    Input::CodePoint(letter.into()),
                    Position {
                        line: 2,
                        column: index + 3,
                    },
                ));
                expected.tokens.push(Token::Character(letter.into()));
                expected.buffer.push(letter.to_ascii_lowercase().into());
                assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                assert_eq!(snapshot(&execution), expected);
            }
            let normalized = if delimiter == '\r' { '\n' } else { delimiter };
            expected.state = State::ScriptDataEscaped;
            expected.current = Some((
                Input::CodePoint(normalized.into()),
                Position { line: 2, column: 9 },
            ));
            expected.tokens.push(Token::Character(normalized.into()));
            assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&execution), expected);
            let (line, column) = if normalized == '\n' { (3, 1) } else { (2, 10) };
            expected.current = Some((Input::CodePoint('x'.into()), Position { line, column }));
            expected.tokens.push(Token::Character('x'.into()));
            assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&execution), expected);
            expected.current = Some((
                Input::Eof,
                Position {
                    line,
                    column: column + 1,
                },
            ));
            expected.tokens.push(Token::Eof);
            expected.errors.push(error(
                "eof-in-script-html-comment-like-text",
                line,
                column + 1,
            ));
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&execution), expected);
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&execution), expected);
        }
    }
}

#[test]
fn integrated_malformed_suffix_has_exact_once_only_destination_diagnostics() {
    let mut execution = entry(seeded_context(), true);
    let mut input = points("/ScRiPt>\r\n\0\0");
    input.extend([
        CodePoint::new(0xd800).unwrap(),
        CodePoint::new(0x1fffe).unwrap(),
        '\u{b}'.into(),
    ]);
    execution.context_mut().input.push(input).unwrap();
    execution.context_mut().input.finish();
    let mut expected = snapshot(&execution);
    expected.state = State::ScriptDataDoubleEscapeEnd;
    expected.current = Some((
        Input::CodePoint('/'.into()),
        Position { line: 2, column: 2 },
    ));
    expected.tokens.push(Token::Character('/'.into()));
    expected.buffer.clear();
    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
    assert_eq!(snapshot(&execution), expected);
    for (index, letter) in "ScRiPt".chars().enumerate() {
        expected.current = Some((
            Input::CodePoint(letter.into()),
            Position {
                line: 2,
                column: index + 3,
            },
        ));
        expected.tokens.push(Token::Character(letter.into()));
        expected.buffer.push(letter.to_ascii_lowercase().into());
        assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
        assert_eq!(snapshot(&execution), expected);
    }
    expected.state = State::ScriptDataEscaped;
    expected.current = Some((
        Input::CodePoint('>'.into()),
        Position { line: 2, column: 9 },
    ));
    expected.tokens.push(Token::Character('>'.into()));
    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
    assert_eq!(snapshot(&execution), expected);
    expected.current = Some((
        Input::CodePoint('\n'.into()),
        Position {
            line: 2,
            column: 10,
        },
    ));
    expected.tokens.push(Token::Character('\n'.into()));
    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
    assert_eq!(snapshot(&execution), expected);
    for column in 1..=2 {
        expected.current = Some((Input::CodePoint('\0'.into()), Position { line: 3, column }));
        expected.tokens.push(Token::Character('\u{fffd}'.into()));
        expected
            .errors
            .push(error("unexpected-null-character", 3, column));
        assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
        assert_eq!(snapshot(&execution), expected);
    }
    for (value, code, column) in [
        (0xd800, "surrogate-in-input-stream", 3),
        (0x1fffe, "noncharacter-in-input-stream", 4),
        (0x0b, "control-character-in-input-stream", 5),
    ] {
        let point = CodePoint::new(value).unwrap();
        expected.current = Some((Input::CodePoint(point), Position { line: 3, column }));
        expected.tokens.push(Token::Character(point));
        expected.input_errors.push(error(code, 3, column));
        assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
        assert_eq!(snapshot(&execution), expected);
    }
    expected.current = Some((Input::Eof, Position { line: 3, column: 6 }));
    expected.tokens.push(Token::Eof);
    expected
        .errors
        .push(error("eof-in-script-html-comment-like-text", 3, 6));
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(snapshot(&execution), expected);
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(snapshot(&execution), expected);
}

#[test]
fn all_three_real_predecessors_emit_less_than_before_slash_or_eof_processing() {
    for initial in [
        State::ScriptDataDoubleEscaped,
        State::ScriptDataDoubleEscapedDash,
        State::ScriptDataDoubleEscapedDashDash,
    ] {
        for slash in [false, true] {
            let mut context = seeded_context();
            context
                .input
                .push(points(if slash { "</" } else { "<" }))
                .unwrap();
            let mut execution = Execution::new(initial, context);
            let mut expected = snapshot(&execution);
            expected.state = SOURCE;
            expected.current = Some((
                Input::CodePoint('<'.into()),
                Position { line: 1, column: 1 },
            ));
            expected.tokens.push(Token::Character('<'.into()));
            assert_eq!(execution.step(), Ok(Progress::Continue(SOURCE)));
            assert_eq!(snapshot(&execution), expected);
            if slash {
                expected.state = State::ScriptDataDoubleEscapeEnd;
                expected.current = Some((
                    Input::CodePoint('/'.into()),
                    Position { line: 1, column: 2 },
                ));
                expected.tokens.push(Token::Character('/'.into()));
                expected.buffer.clear();
                assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                assert_eq!(snapshot(&execution), expected);
            }
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&execution), expected);
            execution.context_mut().input.finish();
            expected.state = State::ScriptDataDoubleEscaped;
            expected.current = Some((
                Input::Eof,
                Position {
                    line: 1,
                    column: if slash { 3 } else { 2 },
                },
            ));
            assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&execution), expected);
            assert_eq!(
                execution.context_mut().input.reconsume(),
                Err(CursorError::AlreadyReconsuming)
            );
            finish_at(&mut execution, &mut expected, 1, if slash { 3 } else { 2 });
        }
    }
}

#[test]
fn real_script_entry_reaches_double_escape_end_and_returns_to_escaped() {
    let mut context = Context::default();
    context.input.push(points("<!--<script></script>")).unwrap();
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
    assert!(execution.context().errors.is_empty());
    assert!(execution.context().input.errors().is_empty());
    let mut expected = snapshot(&execution);
    expected.state = State::ScriptDataDoubleEscapeStart;
    expected.current = Some((
        Input::CodePoint('s'.into()),
        Position { line: 1, column: 6 },
    ));
    expected.buffer.clear();
    expected.tokens.push(Token::Character('<'.into()));
    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
    assert_eq!(snapshot(&execution), expected);
    let mut trace = vec![];
    assert!(drain(&mut execution, &mut trace));
    expected.state = State::ScriptDataEscaped;
    expected.current = Some((
        Input::Eof,
        Position {
            line: 1,
            column: 22,
        },
    ));
    expected.tokens = points("<!--<script></script>")
        .into_iter()
        .map(Token::Character)
        .chain([Token::Eof])
        .collect();
    expected.buffer = points("script");
    expected
        .errors
        .push(error("eof-in-script-html-comment-like-text", 1, 22));
    assert_eq!(trace.last().unwrap().0, Ok(Progress::Finished));
    assert_eq!(snapshot(&execution), expected);
}
