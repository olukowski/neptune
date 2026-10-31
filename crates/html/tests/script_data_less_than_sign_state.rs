//! Local branch evidence, not upstream conformance runs. HTML revision
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, #script-data-less-than-sign-state,
//! source lines 143654–143673. Script data and end tag open execute; only
//! end tag name and escape start remain terminal stubs in this chain.

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

// Initialize the cursor as if '<' had already been consumed. This is isolated
// entry, not a synthetic implementation of ScriptData or resumption after a stub.
fn entry(mut context: Context, newline_prefix: bool) -> Execution {
    if newline_prefix {
        context.input.push(points("\r\n")).unwrap();
        assert_eq!(context.input.consume(), Input::CodePoint('\n'.into()));
    }
    context.input.push(points("<")).unwrap();
    assert_eq!(context.input.consume(), Input::CodePoint('<'.into()));
    Execution::new(State::ScriptDataLessThanSign, context)
}

fn assert_terminal_stub(execution: &mut Execution, target: State) {
    let expected = snapshot(execution);
    assert_eq!(execution.state(), target);
    for _ in 0..3 {
        assert_eq!(execution.step(), Err(UnimplementedState { state: target }));
        assert_eq!(snapshot(execution), expected);
    }
    assert!(!execution.context().tokens.contains(&Token::Eof));
}

#[test]
fn every_ascii_branch_and_non_scalar_has_exact_effects_before_destination_dispatch() {
    let values = (0..=0x7f).chain([
        0x80, 0x9f, 0xa0, 0xc9, 0x3a9, 0xfeff, 0x1f600, 0xd800, 0xdfff, 0xfdd0, 0xffff, 0x10ffff,
    ]);
    for value in values {
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
                    Position { line: 2, column: 2 },
                ));
                let target = match value {
                    0x2f => {
                        expected.storage.buffer.clear();
                        State::ScriptDataEndTagOpen
                    }
                    0x21 => {
                        expected
                            .tokens
                            .extend([Token::Character('<'.into()), Token::Character('!'.into())]);
                        State::ScriptDataEscapeStart
                    }
                    _ => {
                        expected.tokens.push(Token::Character('<'.into()));
                        State::ScriptData
                    }
                };
                expected.state = target;
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
                    .map(|code| error(code, 2, 2))
                    .collect();
                assert_eq!(execution.step(), Ok(Progress::Continue(target)));
                assert_eq!(snapshot(&execution), expected, "input U+{value:04X}");
                let tail_position = if normalized == '\n'.into() {
                    Position { line: 3, column: 1 }
                } else {
                    Position { line: 2, column: 3 }
                };
                if value == 0x2f {
                    // Slash was consumed, not reconsumed: the real end-tag-open
                    // step consumes z and creates/selects an EMPTY candidate.
                    expected.state = State::ScriptDataEndTagName;
                    expected.current = Some((Input::CodePoint('z'.into()), tail_position));
                    expected.storage.current_tag = Some(TagKind::End);
                    expected.storage.end = Some(PendingEndTag {
                        name: vec![],
                        self_closing: false,
                        attributes: vec![],
                        discarded_current_attribute: None,
                    });
                    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                    assert_eq!(snapshot(&execution), expected);
                    assert_eq!(
                        execution.context_mut().input.reconsume(),
                        Err(CursorError::AlreadyReconsuming)
                    );
                    assert_terminal_stub(&mut execution, State::ScriptDataEndTagName);
                    assert_eq!(
                        execution.context_mut().input.consume(),
                        Input::CodePoint('z'.into())
                    );
                    assert_eq!(execution.context().input.position(), tail_position);
                    continue;
                }
                if value == 0x21 {
                    assert_terminal_stub(&mut execution, State::ScriptDataEscapeStart);
                    // Inspect the untouched tail only, never resume a stub.
                    assert_eq!(
                        execution.context_mut().input.consume(),
                        Input::CodePoint('z'.into())
                    );
                    assert_eq!(execution.context().input.position(), tail_position);
                    assert_eq!(execution.context().input.errors(), expected.input_errors);
                    continue;
                }

                assert_eq!(
                    execution.context_mut().input.reconsume(),
                    Err(CursorError::AlreadyReconsuming)
                );
                // The real Script data destination reconsumes at the SAME position.
                // Only that step diagnoses/replaces NUL or handles another '<'.
                if value == 0x3c {
                    assert_eq!(
                        execution.step(),
                        Ok(Progress::Continue(State::ScriptDataLessThanSign))
                    );
                    expected.state = State::ScriptDataLessThanSign;
                    assert_eq!(snapshot(&execution), expected);
                    assert_eq!(execution.step(), Ok(Progress::Continue(State::ScriptData)));
                    expected.state = State::ScriptData;
                    expected.current = Some((Input::CodePoint('z'.into()), tail_position));
                    expected.tokens.push(Token::Character('<'.into()));
                    assert_eq!(snapshot(&execution), expected);
                } else {
                    assert_eq!(execution.step(), Ok(Progress::Continue(State::ScriptData)));
                    expected.tokens.push(Token::Character(if value == 0 {
                        expected
                            .errors
                            .push(error("unexpected-null-character", 2, 2));
                        '\u{fffd}'.into()
                    } else {
                        normalized
                    }));
                    assert_eq!(snapshot(&execution), expected);
                }
                assert_eq!(execution.step(), Ok(Progress::Continue(State::ScriptData)));
                expected.current = Some((Input::CodePoint('z'.into()), tail_position));
                expected.tokens.push(Token::Character('z'.into()));
                assert_eq!(snapshot(&execution), expected);
                if !finished {
                    for _ in 0..3 {
                        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                        assert_eq!(snapshot(&execution), expected);
                    }
                    execution.context_mut().input.finish();
                }
                assert_eq!(execution.step(), Ok(Progress::Finished));
                expected.current = Some((
                    Input::Eof,
                    Position {
                        line: tail_position.line,
                        column: tail_position.column + 1,
                    },
                ));
                expected.tokens.push(Token::Eof);
                assert_eq!(snapshot(&execution), expected);
                for _ in 0..3 {
                    assert_eq!(execution.step(), Ok(Progress::Finished));
                    assert_eq!(snapshot(&execution), expected);
                }
            }
        }
    }
}

#[test]
fn actual_eof_emits_only_less_than_and_schedules_conceptual_eof_reconsumption() {
    for newline_prefix in [false, true] {
        let mut execution = entry(seeded_context(), newline_prefix);
        let before = snapshot(&execution);
        for _ in 0..3 {
            execution.context_mut().input.push([]).unwrap();
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&execution), before);
        }
        execution.context_mut().input.finish();
        execution.context_mut().input.finish();
        let mut expected = before;
        expected.state = State::ScriptData;
        expected.current = Some((
            Input::Eof,
            Position {
                line: if newline_prefix { 2 } else { 1 },
                column: 2,
            },
        ));
        expected.tokens.push(Token::Character('<'.into()));
        assert_eq!(execution.step(), Ok(Progress::Continue(State::ScriptData)));
        assert_eq!(snapshot(&execution), expected);
        assert_eq!(
            execution.context_mut().input.reconsume(),
            Err(CursorError::AlreadyReconsuming)
        );
        // EOF is emitted only by the real Script data destination, at the same
        // conceptual EOF position. The less-than-sign source did not finish.
        assert_eq!(execution.step(), Ok(Progress::Finished));
        expected.tokens.push(Token::Eof);
        for _ in 0..3 {
            assert_eq!(snapshot(&execution), expected);
            assert_eq!(execution.step(), Ok(Progress::Finished));
        }
    }
}

#[test]
fn default_storage_stays_absent_and_no_destination_creation_or_eof_is_invented() {
    for (input, target, output) in [
        ("/", State::ScriptDataEndTagOpen, ""),
        ("!", State::ScriptDataEscapeStart, "<!"),
        ("x", State::ScriptData, "<"),
        ("", State::ScriptData, "<"),
    ] {
        let mut execution = entry(Context::default(), false);
        execution.context_mut().input.push(points(input)).unwrap();
        execution.context_mut().input.finish();
        assert_eq!(execution.step(), Ok(Progress::Continue(target)));
        assert_eq!(storage(execution.context()), storage(&Context::default()));
        assert_eq!(execution.context().current_tag, None);
        assert_eq!(execution.context().pending_start_tag, None);
        assert_eq!(execution.context().pending_end_tag, None);
        assert_eq!(execution.context().pending_comment, None);
        assert_eq!(execution.context().last_start_tag, None);
        assert_eq!(execution.context().return_state, None);
        assert!(execution.context().temporary_buffer.is_empty());
        assert!(execution.context().errors.is_empty());
        assert!(execution.context().input.errors().is_empty());
        assert_eq!(
            execution.context().tokens,
            points(output)
                .into_iter()
                .map(Token::Character)
                .collect::<Vec<_>>()
        );
        if input == "!" {
            assert_terminal_stub(&mut execution, State::ScriptDataEscapeStart);
        } else {
            if input == "/" {
                let mut expected = snapshot(&execution);
                expected.state = State::ScriptData;
                expected.current = Some((Input::Eof, Position { line: 1, column: 3 }));
                expected
                    .tokens
                    .extend([Token::Character('<'.into()), Token::Character('/'.into())]);
                assert_eq!(execution.step(), Ok(Progress::Continue(State::ScriptData)));
                assert_eq!(snapshot(&execution), expected);
                assert_eq!(
                    execution.context_mut().input.reconsume(),
                    Err(CursorError::AlreadyReconsuming)
                );
            }
            if input == "x" {
                assert_eq!(execution.step(), Ok(Progress::Continue(State::ScriptData)));
                assert_eq!(
                    execution.context().tokens,
                    [Token::Character('<'.into()), Token::Character('x'.into())]
                );
            }
            assert_eq!(execution.step(), Ok(Progress::Finished));
            let mut expected: Vec<_> = points(match input {
                "x" => "<x",
                "/" => "</",
                _ => "<",
            })
            .into_iter()
            .map(Token::Character)
            .collect();
            expected.push(Token::Eof);
            assert_eq!(execution.context().tokens, expected);
            assert_eq!(storage(execution.context()), storage(&Context::default()));
        }
    }
}

#[test]
fn open_empty_input_suspends_without_a_current_item_and_can_then_accept_input() {
    let mut execution = Execution::new(State::ScriptDataLessThanSign, Context::default());
    let before = snapshot(&execution);
    assert_eq!(before.current, None);
    for _ in 0..3 {
        execution.context_mut().input.push([]).unwrap();
        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
        assert_eq!(snapshot(&execution), before);
    }
    execution.context_mut().input.push(points("!")).unwrap();
    assert_eq!(
        execution.step(),
        Ok(Progress::Continue(State::ScriptDataEscapeStart))
    );
    assert_eq!(
        execution.context().tokens,
        [Token::Character('<'.into()), Token::Character('!'.into())]
    );
    // Open exhaustion at the unimplemented destination is still a stub, not suspension.
    assert_terminal_stub(&mut execution, State::ScriptDataEscapeStart);
    execution.context_mut().input.push(points("--")).unwrap();
    execution.context_mut().input.finish();
    assert_terminal_stub(&mut execution, State::ScriptDataEscapeStart);
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

fn trace(chunks: &[&[CodePoint]], closed: bool, newline_prefix: bool, real_entry: bool) -> Trace {
    let mut execution = if real_entry {
        let mut context = seeded_context();
        if newline_prefix {
            context.input.push(points("\r\n")).unwrap();
            assert_eq!(context.input.consume(), Input::CodePoint('\n'.into()));
        }
        Execution::new(State::ScriptData, context)
    } else {
        entry(seeded_context(), newline_prefix)
    };
    let mut snapshots = Vec::new();
    let bound = chunks.iter().map(|chunk| chunk.len()).sum::<usize>() * 3 + 4;
    for chunk in chunks {
        execution
            .context_mut()
            .input
            .push(chunk.iter().copied())
            .unwrap();
        if !closed && drain(&mut execution, &mut snapshots, bound) {
            // Stop immediately at a real terminal stub. Never feed or resume it.
            return snapshots;
        }
    }
    execution.context_mut().input.finish();
    assert!(drain(&mut execution, &mut snapshots, bound));
    snapshots
}

#[test]
fn every_split_and_single_code_point_chunking_matches_closed_source_and_stub_snapshots() {
    let mut inputs: Vec<Vec<CodePoint>> = [
        "",
        "/",
        "/ScRiPt>",
        "/ XMP>",
        "/\0tail",
        "/<tail",
        "/\r\n",
        "!",
        "!--escape",
        "x",
        "<",
        "&",
        "\0tail",
        "\r\nX",
        "\r\r\n",
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
    ]);
    inputs.push(vec![CodePoint::new(0x10ffff).unwrap(), '\0'.into()]);
    for input in &inputs {
        for real_entry in [false, true] {
            let input = if real_entry {
                [vec!['<'.into()], input.clone()].concat()
            } else {
                input.clone()
            };
            for newline_prefix in [false, true] {
                let expected = trace(&[&input], true, newline_prefix, real_entry);
                for split in 0..=input.len() {
                    assert_eq!(
                        trace(
                            &[&input[..split], &[], &input[split..]],
                            false,
                            newline_prefix,
                            real_entry
                        ),
                        expected,
                        "split {split} of {input:?}, real entry: {real_entry}"
                    );
                }
                let chunks: Vec<_> = input.chunks(1).collect();
                assert_eq!(
                    trace(&chunks, false, newline_prefix, real_entry),
                    expected,
                    "{input:?}"
                );
            }
        }
    }
}
