//! Source-local checks for #script-data-end-tag-open-state, pinned HTML source
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, lines 143676–143692.
//! Real Script data / less-than-sign steps execute separately from this source;
//! end tag name executes separately; escape start remains an explicit terminal stub.

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
        return_state: Some(State::Rcdata),
        current_tag: Some(TagKind::Start),
        pending_start_tag: Some(PendingStartTag {
            name: points("start"),
            self_closing: true,
            attributes: vec![attribute.clone()],
            discarded_current_attribute: Some(attribute.clone()),
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("stale-end"),
            self_closing: true,
            attributes: vec![attribute.clone()],
            discarded_current_attribute: Some(attribute),
        }),
        pending_comment: Some(PendingComment {
            data: points("comment"),
        }),
        // A predecessor normally clears this. Check that this state does not.
        temporary_buffer: vec![CodePoint::new(0xdfff).unwrap(), '\0'.into(), 'Z'.into()],
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

// Position an isolated cursor, not an execution of either predecessor algorithm.
fn entry(mut context: Context, prefix: &str) -> Context {
    context.input.push(points(prefix)).unwrap();
    for point in points(&prefix.replace("\r\n", "\n").replace('\r', "\n")) {
        assert_eq!(context.input.consume(), Input::CodePoint(point));
    }
    context
}

fn assert_terminal_stub(execution: &mut Execution, target: State) {
    let before = snapshot(execution);
    for _ in 0..3 {
        assert_eq!(execution.step(), Err(UnimplementedState { state: target }));
        assert_eq!(snapshot(execution), before);
    }
}

#[test]
fn all_ascii_letters_create_and_select_empty_end_tag_without_appending_or_lowercasing() {
    for value in (0x41..=0x5a).chain(0x61..=0x7a) {
        for selected in [None, Some(TagKind::Start), Some(TagKind::End)] {
            for finished in [false, true] {
                let point = CodePoint::new(value).unwrap();
                let mut context = entry(seeded_context(), "</");
                context.current_tag = selected;
                if selected.is_none() {
                    context.pending_end_tag = None;
                    context.last_start_tag = None;
                    context.return_state = None;
                }
                context.input.push([point, '!'.into()]).unwrap();
                if finished {
                    context.input.finish();
                }
                let mut execution = Execution::new(State::ScriptDataEndTagOpen, context);
                let mut expected = snapshot(&execution);
                expected.state = State::ScriptDataEndTagName;
                expected.current = Some((Input::CodePoint(point), Position { line: 1, column: 3 }));
                expected.current_tag = Some(TagKind::End);
                expected.end = Some(PendingEndTag {
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
                let normalized =
                    CodePoint::new(if value <= 0x5a { value + 0x20 } else { value }).unwrap();
                expected.end.as_mut().unwrap().name.push(normalized);
                expected.buffer.push(point);
                assert_eq!(
                    execution.step(),
                    Ok(Progress::Continue(State::ScriptDataEndTagName))
                );
                assert_eq!(snapshot(&execution), expected);
                expected.state = State::ScriptData;
                expected.current = Some((
                    Input::CodePoint('!'.into()),
                    Position { line: 1, column: 4 },
                ));
                expected.tokens.extend(
                    points("</")
                        .into_iter()
                        .chain(expected.buffer.iter().copied())
                        .map(Token::Character),
                );
                assert_eq!(execution.step(), Ok(Progress::Continue(State::ScriptData)));
                assert_eq!(snapshot(&execution), expected);
                assert_eq!(
                    execution.context_mut().input.reconsume(),
                    Err(CursorError::AlreadyReconsuming)
                );
                expected.tokens.push(Token::Character('!'.into()));
                assert_eq!(execution.step(), Ok(Progress::Continue(State::ScriptData)));
                assert_eq!(snapshot(&execution), expected);
                if !finished {
                    assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                    assert_eq!(snapshot(&execution), expected);
                    execution.context_mut().input.finish();
                }
                expected.current = Some((Input::Eof, Position { line: 1, column: 5 }));
                expected.tokens.push(Token::Eof);
                assert_eq!(execution.step(), Ok(Progress::Finished));
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
fn every_other_ascii_and_representative_non_ascii_point_emits_only_literal_prefix() {
    let inputs = (0..=0x7f)
        .filter(|v| !matches!(v, 0x41..=0x5a | 0x61..=0x7a))
        .chain([
            0x80, 0x9f, 0xa0, 0xc9, 0x3b1, 0xff21, 0xfeff, 0x1f600, 0xd800, 0xdfff, 0xfdd0,
            0x1fffe, 0x10ffff,
        ]);
    for value in inputs {
        for selected in [None, Some(TagKind::Start), Some(TagKind::End)] {
            for finished in [false, true] {
                let point = CodePoint::new(value).unwrap();
                let normalized = if value == 0x0d { '\n'.into() } else { point };
                let mut context = entry(seeded_context(), "\r\n</");
                context.current_tag = selected;
                context.input.push([point, '!'.into()]).unwrap();
                if finished {
                    context.input.finish();
                }
                let mut execution = Execution::new(State::ScriptDataEndTagOpen, context);
                let mut expected = snapshot(&execution);
                expected.state = State::ScriptData;
                expected.current = Some((
                    Input::CodePoint(normalized),
                    Position { line: 2, column: 3 },
                ));
                expected
                    .tokens
                    .extend([Token::Character('<'.into()), Token::Character('/'.into())]);
                expected.input_errors = match value {
                    0xd800..=0xdfff => vec![error("surrogate-in-input-stream", 2, 3)],
                    0xfdd0 | 0x1fffe | 0x10ffff => {
                        vec![error("noncharacter-in-input-stream", 2, 3)]
                    }
                    0x01..=0x08 | 0x0b | 0x0e..=0x1f | 0x7f..=0x9f => {
                        vec![error("control-character-in-input-stream", 2, 3)]
                    }
                    _ => vec![],
                };
                assert_eq!(execution.step(), Ok(Progress::Continue(State::ScriptData)));
                assert_eq!(snapshot(&execution), expected, "U+{value:04X}");
                assert_eq!(
                    execution.context_mut().input.reconsume(),
                    Err(CursorError::AlreadyReconsuming)
                );
                // Only the real destination replaces/diagnoses NUL. Another '<'
                // transitions again without emission; preprocessing is not repeated.
                let tail_position = if normalized == '\n'.into() {
                    Position { line: 3, column: 1 }
                } else {
                    Position { line: 2, column: 4 }
                };
                if value == 0x3c {
                    expected.state = State::ScriptDataLessThanSign;
                    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                    assert_eq!(snapshot(&execution), expected);
                    expected.state = State::ScriptDataEscapeStart;
                    expected.current = Some((Input::CodePoint('!'.into()), tail_position));
                    expected
                        .tokens
                        .extend([Token::Character('<'.into()), Token::Character('!'.into())]);
                    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                    assert_eq!(snapshot(&execution), expected);
                    assert_terminal_stub(&mut execution, State::ScriptDataEscapeStart);
                    continue;
                }
                expected.tokens.push(Token::Character(if value == 0 {
                    expected
                        .errors
                        .push(error("unexpected-null-character", 2, 3));
                    '\u{fffd}'.into()
                } else {
                    normalized
                }));
                assert_eq!(execution.step(), Ok(Progress::Continue(State::ScriptData)));
                assert_eq!(snapshot(&execution), expected);
                expected.current = Some((Input::CodePoint('!'.into()), tail_position));
                expected.tokens.push(Token::Character('!'.into()));
                assert_eq!(execution.step(), Ok(Progress::Continue(State::ScriptData)));
                assert_eq!(snapshot(&execution), expected);
                if !finished {
                    for _ in 0..3 {
                        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                        assert_eq!(snapshot(&execution), expected);
                    }
                    execution.context_mut().input.finish();
                }
                expected.current = Some((
                    Input::Eof,
                    Position {
                        line: tail_position.line,
                        column: tail_position.column + 1,
                    },
                ));
                expected.tokens.push(Token::Eof);
                assert_eq!(execution.step(), Ok(Progress::Finished));
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
fn open_exhaustion_suspends_but_actual_eof_reconsumes_after_literal_prefix_without_eof_token() {
    for prefix in ["", "</", "\r\n</", "\r"] {
        let context = entry(seeded_context(), prefix);
        let mut execution = Execution::new(State::ScriptDataEndTagOpen, context);
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
            match prefix {
                "" => Position { line: 1, column: 1 },
                "</" => Position { line: 1, column: 3 },
                "\r\n</" => Position { line: 2, column: 3 },
                "\r" => Position { line: 2, column: 1 },
                _ => unreachable!(),
            },
        ));
        expected
            .tokens
            .extend([Token::Character('<'.into()), Token::Character('/'.into())]);
        assert_eq!(execution.step(), Ok(Progress::Continue(State::ScriptData)));
        assert_eq!(snapshot(&execution), expected);
        assert_eq!(
            execution.context_mut().input.reconsume(),
            Err(CursorError::AlreadyReconsuming)
        );
        // Only Script data emits terminal EOF, at the same conceptual position.
        expected.tokens.push(Token::Eof);
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&execution), expected);
        for _ in 0..3 {
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&execution), expected);
        }
    }
}

#[test]
fn suspension_can_resume_with_either_branch_using_default_storage() {
    for input in ["A", "!", "\0"] {
        let mut execution = Execution::new(State::ScriptDataEndTagOpen, Context::default());
        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
        execution.context_mut().input.push(points(input)).unwrap();
        let alpha = input == "A";
        let target = if alpha {
            State::ScriptDataEndTagName
        } else {
            State::ScriptData
        };
        let mut expected = snapshot(&execution);
        expected.state = target;
        expected.current = Some((
            Input::CodePoint(input.chars().next().unwrap().into()),
            Position { line: 1, column: 1 },
        ));
        if alpha {
            expected.current_tag = Some(TagKind::End);
            expected.end = Some(PendingEndTag {
                name: vec![],
                self_closing: false,
                attributes: vec![],
                discarded_current_attribute: None,
            });
        } else {
            expected
                .tokens
                .extend([Token::Character('<'.into()), Token::Character('/'.into())]);
        }
        assert_eq!(execution.step(), Ok(Progress::Continue(target)));
        assert_eq!(snapshot(&execution), expected);
        if alpha {
            expected.end.as_mut().unwrap().name.push('a'.into());
            expected.buffer.push('A'.into());
            assert_eq!(
                execution.step(),
                Ok(Progress::Continue(State::ScriptDataEndTagName))
            );
            assert_eq!(snapshot(&execution), expected);
            for _ in 0..3 {
                assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                assert_eq!(snapshot(&execution), expected);
            }
            execution.context_mut().input.finish();
            expected.state = State::ScriptData;
            expected.current = Some((Input::Eof, Position { line: 1, column: 2 }));
            expected
                .tokens
                .extend(points("</A").into_iter().map(Token::Character));
            assert_eq!(execution.step(), Ok(Progress::Continue(State::ScriptData)));
            assert_eq!(snapshot(&execution), expected);
            assert_eq!(
                execution.context_mut().input.reconsume(),
                Err(CursorError::AlreadyReconsuming)
            );
            expected.tokens.push(Token::Eof);
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&execution), expected);
            for _ in 0..3 {
                assert_eq!(execution.step(), Ok(Progress::Finished));
                assert_eq!(snapshot(&execution), expected);
            }
        } else {
            if input == "\0" {
                expected
                    .errors
                    .push(error("unexpected-null-character", 1, 1));
            }
            expected.tokens.push(Token::Character(if input == "\0" {
                '\u{fffd}'.into()
            } else {
                '!'.into()
            }));
            assert_eq!(execution.step(), Ok(Progress::Continue(State::ScriptData)));
            assert_eq!(snapshot(&execution), expected);
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&execution), expected);
            execution.context_mut().input.finish();
            expected.current = Some((Input::Eof, Position { line: 1, column: 2 }));
            expected.tokens.push(Token::Eof);
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&execution), expected);
        }
    }
}

// Traces run all real steps and stop at terminal EOF or the first genuine stub.
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
        match result {
            Err(UnimplementedState { state }) => {
                assert_eq!(state, State::ScriptDataEscapeStart);
                assert_terminal_stub(execution, state);
                return true;
            }
            Ok(Progress::Finished) => {
                let expected = snapshot(execution);
                assert_eq!(execution.step(), result);
                assert_eq!(snapshot(execution), expected);
                return true;
            }
            Ok(Progress::Continue(_)) => {}
            Ok(Progress::NeedMoreInput) => unreachable!(),
        }
    }
    panic!("real source/destination steps failed to progress or terminate");
}

#[test]
fn every_split_and_single_point_chunking_matches_closed_source_and_destination_trace() {
    let mut inputs = vec![vec![]];
    for text in [
        "Aname>tail",
        "z",
        "!",
        "\0",
        "&amp;",
        "<tail",
        " XMP>",
        "0></A>",
        "é",
        "😀",
        "\r\n",
        "\r\r\n\0\r",
        "\nA",
    ] {
        inputs.push(points(text));
    }
    for value in [0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x01, 0x0b] {
        inputs.push(vec![CodePoint::new(value).unwrap(), '!'.into()]);
    }
    for prefix in ["</", "\r\n</", "\r"] {
        for input in &inputs {
            let run = |chunks: Vec<&[CodePoint]>, closed: bool| {
                let context = entry(seeded_context(), prefix);
                let mut execution = Execution::new(State::ScriptDataEndTagOpen, context);
                let mut trace = vec![];
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
                        if drain(&mut execution, &mut trace, input.len() * 3 + 10) {
                            // Never feed, reconstruct, or resume a blocked execution.
                            return trace;
                        }
                        execution.context_mut().input.push([]).unwrap();
                        assert!(!drain(&mut execution, &mut trace, 1));
                    }
                    execution.context_mut().input.finish();
                }
                assert!(drain(&mut execution, &mut trace, input.len() * 3 + 10));
                trace
            };
            let expected = run(vec![], true);
            for split in 0..=input.len() {
                assert_eq!(
                    run(vec![&input[..split], &input[split..]], false),
                    expected,
                    "{prefix:?}, split {split} of {input:?}"
                );
            }
            assert_eq!(run(input.chunks(1).collect(), false), expected);
        }
    }
}
