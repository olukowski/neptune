//! Source-local checks for #script-data-end-tag-open-state, pinned HTML source
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, lines 143676–143692.
//! Neither neighboring state is implemented or simulated by this suite.

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
                assert_terminal_stub(&mut execution, State::ScriptDataEndTagName);
                // Inspect cursor scheduling only; never reconstruct/resume the stub.
                assert_eq!(
                    execution.context_mut().input.consume(),
                    Input::CodePoint(point)
                );
                assert_eq!(
                    execution.context().input.position(),
                    Position { line: 1, column: 3 }
                );
                assert_eq!(
                    execution.context_mut().input.consume(),
                    Input::CodePoint('!'.into())
                );
                assert_eq!(
                    execution.context().input.position(),
                    Position { line: 1, column: 4 }
                );
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
                assert_terminal_stub(&mut execution, State::ScriptData);
                // NUL remains NUL here; no destination replacement/error is invented.
                assert_eq!(
                    execution.context_mut().input.consume(),
                    Input::CodePoint(normalized)
                );
                assert_eq!(execution.context().input.errors(), expected.input_errors);
                assert_eq!(
                    execution.context_mut().input.consume(),
                    Input::CodePoint('!'.into())
                );
                assert_eq!(execution.context().input.errors(), expected.input_errors);
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
        assert_terminal_stub(&mut execution, State::ScriptData);
        assert_eq!(
            execution.context_mut().input.reconsume(),
            Err(CursorError::AlreadyReconsuming)
        );
        assert_eq!(execution.context_mut().input.consume(), Input::Eof);
        assert_eq!(execution.context().input.current(), expected.current);
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
        assert_terminal_stub(&mut execution, target);
    }
}

// These traces stop at the first stub. Unread tails are not executed coverage.
type Trace = Vec<(Result<Progress, UnimplementedState>, Snapshot)>;

fn drain(execution: &mut Execution, trace: &mut Trace) -> bool {
    for _ in 0..3 {
        let before = snapshot(execution);
        let result = execution.step();
        if result == Ok(Progress::NeedMoreInput) {
            assert_eq!(snapshot(execution), before);
            assert_eq!(execution.step(), result);
            assert_eq!(snapshot(execution), before);
            return false;
        }
        trace.push((result, snapshot(execution)));
        if let Err(UnimplementedState { state }) = result {
            assert_terminal_stub(execution, state);
            return true;
        }
        assert!(matches!(result, Ok(Progress::Continue(_))));
    }
    panic!("source and destination stub must terminate within two steps");
}

#[test]
fn every_split_and_single_point_chunking_matches_closed_source_and_terminal_stub_trace() {
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
                        if drain(&mut execution, &mut trace) {
                            // Never feed, reconstruct, or resume a blocked execution.
                            return trace;
                        }
                        execution.context_mut().input.push([]).unwrap();
                        assert!(!drain(&mut execution, &mut trace));
                    }
                    execution.context_mut().input.finish();
                }
                assert!(drain(&mut execution, &mut trace));
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
