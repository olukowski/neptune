//! Source-local evidence for pinned RAWTEXT end tag open, lines 143589–143607:
//! https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source
//! Isolated entry only: neighboring entry/name states intentionally remain stubs.

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
        last_start_tag: Some(points("style")),
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
        // A normal less-than-sign entry would have cleared this already.
        temporary_buffer: vec![],
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

// Initialize the cursor at the assigned state's entry, without implementing or
// executing either neighboring entry algorithm. Prefix can include raw CRLF.
fn entry(mut context: Context, prefix: &str) -> Context {
    context.input.push(points(prefix)).unwrap();
    let normalized = prefix.replace("\r\n", "\n").replace('\r', "\n");
    for point in points(&normalized) {
        assert_eq!(context.input.consume(), Input::CodePoint(point));
    }
    context
}

fn assert_terminal(execution: &mut Execution, result: Result<Progress, UnimplementedState>) {
    let before = snapshot(execution);
    for _ in 0..3 {
        assert_eq!(execution.step(), result);
        assert_eq!(snapshot(execution), before);
    }
}

#[test]
fn every_ascii_alpha_creates_empty_selected_end_tag_then_blocks_without_name_effects() {
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
                let mut expected = storage(&context);
                expected.current_tag = Some(TagKind::End);
                expected.end = Some(PendingEndTag {
                    name: vec![],
                    self_closing: false,
                    attributes: vec![],
                    discarded_current_attribute: None,
                });
                context.input.push([point, '!'.into()]).unwrap();
                if finished {
                    context.input.finish();
                }
                let mut execution = Execution::new(State::RawtextEndTagOpen, context);
                assert_eq!(
                    execution.step(),
                    Ok(Progress::Continue(State::RawtextEndTagName))
                );
                assert_eq!(storage(execution.context()), expected);
                assert_eq!(execution.context().tokens, [Token::Character('p'.into())]);
                assert_eq!(
                    execution.context().errors,
                    [error("prior-diagnostic", 1, 1)]
                );
                assert!(execution.context().input.errors().is_empty());
                assert_eq!(
                    execution.context().input.current(),
                    Some((Input::CodePoint(point), Position { line: 1, column: 3 }))
                );
                assert_terminal(
                    &mut execution,
                    Err(UnimplementedState {
                        state: State::RawtextEndTagName,
                    }),
                );
                // Cursor inspection only, never reconstruct/resume blocked execution.
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
                assert_terminal(
                    &mut execution,
                    Err(UnimplementedState {
                        state: State::RawtextEndTagName,
                    }),
                );
            }
        }
    }
}

#[test]
fn every_other_ascii_and_non_ascii_point_falls_back_before_rawtext_destination_effects() {
    let inputs = (0..=0x7f)
        .filter(|v| !matches!(v, 0x41..=0x5a | 0x61..=0x7a))
        .chain([
            0x80, 0x9f, 0xa0, 0xc9, 0x3b1, 0xff21, 0xfeff, 0x1f600, 0xd800, 0xdfff, 0xfdd0,
            0x1fffe, 0x10ffff,
        ]);
    for value in inputs {
        let point = CodePoint::new(value).unwrap();
        let normalized = if value == 0x0d { '\n'.into() } else { point };
        let mut context = entry(seeded_context(), "\r\n</");
        context.input.push([point, '!'.into()]).unwrap();
        context.input.finish();
        let preserved = storage(&context);
        let preprocessing = match value {
            0xd800..=0xdfff => vec![error("surrogate-in-input-stream", 2, 3)],
            0xfdd0 | 0x1fffe | 0x10ffff => vec![error("noncharacter-in-input-stream", 2, 3)],
            0x01..=0x08 | 0x0b | 0x0e..=0x1f | 0x7f..=0x9f => {
                vec![error("control-character-in-input-stream", 2, 3)]
            }
            _ => vec![],
        };
        let mut execution = Execution::new(State::RawtextEndTagOpen, context);
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Rawtext)));
        assert_eq!(
            execution.context().tokens,
            [
                Token::Character('p'.into()),
                Token::Character('<'.into()),
                Token::Character('/'.into())
            ]
        );
        assert_eq!(
            execution.context().errors,
            [error("prior-diagnostic", 1, 1)]
        );
        assert_eq!(execution.context().input.errors(), preprocessing);
        let current = Some((
            Input::CodePoint(normalized),
            Position { line: 2, column: 3 },
        ));
        assert_eq!(execution.context().input.current(), current);
        assert_eq!(storage(execution.context()), preserved);
        let target = if value == 0x3c {
            State::RawtextLessThanSign
        } else {
            State::Rawtext
        };
        assert_eq!(execution.step(), Ok(Progress::Continue(target)));
        assert_eq!(execution.context().input.current(), current);
        assert_eq!(execution.context().input.errors(), preprocessing);
        assert_eq!(storage(execution.context()), preserved);
        let mut errors = vec![error("prior-diagnostic", 1, 1)];
        let mut tokens = vec![
            Token::Character('p'.into()),
            Token::Character('<'.into()),
            Token::Character('/'.into()),
        ];
        if value == 0 {
            errors.push(error("unexpected-null-character", 2, 3));
            tokens.push(Token::Character('\u{fffd}'.into()));
        } else if value != 0x3c {
            tokens.push(Token::Character(normalized));
        }
        assert_eq!(execution.context().errors, errors);
        assert_eq!(execution.context().tokens, tokens);
        if value == 0x3c {
            assert_terminal(&mut execution, Err(UnimplementedState { state: target }));
            assert_eq!(
                execution.context_mut().input.consume(),
                Input::CodePoint('!'.into())
            );
        } else {
            assert_eq!(execution.step(), Ok(Progress::Continue(State::Rawtext)));
            tokens.push(Token::Character('!'.into()));
            assert_eq!(execution.context().tokens, tokens);
            assert_eq!(
                execution.context().input.position(),
                if normalized == '\n'.into() {
                    Position { line: 3, column: 1 }
                } else {
                    Position { line: 2, column: 4 }
                }
            );
            assert_eq!(execution.step(), Ok(Progress::Finished));
            tokens.push(Token::Eof);
            assert_eq!(execution.context().tokens, tokens);
            assert_eq!(execution.context().errors, errors);
            assert_eq!(execution.context().input.errors(), preprocessing);
            assert_eq!(storage(execution.context()), preserved);
            assert_terminal(&mut execution, Ok(Progress::Finished));
        }
    }
}

#[test]
fn open_exhaustion_repeatedly_suspends_but_explicit_eof_falls_back_then_is_reconsumed() {
    let context = entry(seeded_context(), "\r\n</");
    let mut execution = Execution::new(State::RawtextEndTagOpen, context);
    let before = snapshot(&execution);
    for _ in 0..3 {
        execution.context_mut().input.push([]).unwrap();
        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
        assert_eq!(snapshot(&execution), before);
    }
    execution.context_mut().input.finish();
    execution.context_mut().input.finish();
    assert_eq!(execution.step(), Ok(Progress::Continue(State::Rawtext)));
    assert_eq!(
        execution.context().tokens,
        [
            Token::Character('p'.into()),
            Token::Character('<'.into()),
            Token::Character('/'.into())
        ]
    );
    let eof = Some((Input::Eof, Position { line: 2, column: 3 }));
    assert_eq!(execution.context().input.current(), eof);
    // The cursor confirms conceptual EOF is scheduled, not merely read again.
    assert_eq!(
        execution.context_mut().input.reconsume(),
        Err(CursorError::AlreadyReconsuming)
    );
    assert_eq!(execution.context().errors, before.errors);
    assert!(execution.context().input.errors().is_empty());
    assert_eq!(storage(execution.context()), before.storage);
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(execution.context().input.current(), eof);
    assert_eq!(
        execution.context().tokens,
        [
            Token::Character('p'.into()),
            Token::Character('<'.into()),
            Token::Character('/'.into()),
            Token::Eof
        ]
    );
    assert_terminal(&mut execution, Ok(Progress::Finished));
    assert_eq!(
        execution.context_mut().input.push([]),
        Err(CursorError::InputAlreadyFinished)
    );
}

#[test]
fn absent_storage_stays_absent_on_fallback_and_only_end_storage_is_created_on_alpha() {
    for text in ["a", "!", ""] {
        let mut context = entry(Context::default(), "</");
        context.input.push(points(text)).unwrap();
        context.input.finish();
        let mut expected = storage(&context);
        if text == "a" {
            expected.current_tag = Some(TagKind::End);
            expected.end = Some(PendingEndTag {
                name: vec![],
                self_closing: false,
                attributes: vec![],
                discarded_current_attribute: None,
            });
        }
        let mut execution = Execution::new(State::RawtextEndTagOpen, context);
        let target = if text == "a" {
            State::RawtextEndTagName
        } else {
            State::Rawtext
        };
        assert_eq!(execution.step(), Ok(Progress::Continue(target)));
        assert_eq!(storage(execution.context()), expected);
        assert!(execution.context().errors.is_empty());
        assert!(execution.context().input.errors().is_empty());
        assert_eq!(
            execution.context().tokens,
            if text == "a" {
                vec![]
            } else {
                vec![Token::Character('<'.into()), Token::Character('/'.into())]
            }
        );
        if text == "a" {
            assert_terminal(&mut execution, Err(UnimplementedState { state: target }));
        }
    }
}

#[test]
fn temporary_buffer_is_neither_cleared_nor_appended_by_either_branch() {
    for input in ["A", "!", ""] {
        let mut context = entry(seeded_context(), "</");
        context.temporary_buffer = vec![CodePoint::new(0xdfff).unwrap(), '\0'.into(), 'Z'.into()];
        let expected = context.temporary_buffer.clone();
        context.input.push(points(input)).unwrap();
        context.input.finish();
        let mut execution = Execution::new(State::RawtextEndTagOpen, context);
        assert_eq!(
            execution.step(),
            Ok(Progress::Continue(if input == "A" {
                State::RawtextEndTagName
            } else {
                State::Rawtext
            }))
        );
        assert_eq!(execution.context().temporary_buffer, expected);
        assert!(execution.context().input.errors().is_empty());
    }
}

// Compare every productive source step and terminal result, not just final output.
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
            assert_terminal(execution, result);
            return true;
        }
    }
    panic!("RAWTEXT end tag open failed to progress or terminate");
}

#[test]
fn all_two_chunk_splits_and_single_point_chunks_match_closed_input_source_traces() {
    let mut malformed = points("!\r\n\0&é😀\r\r\nx\r");
    malformed.extend([0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x01].map(|v| CodePoint::new(v).unwrap()));
    for input in [
        vec![],
        points("Aname>tail"),
        points("z"),
        points("!"),
        points("\0"),
        points("&amp;"),
        points("<tail"),
        points("!<tail"),
        points("\r\n"),
        points("\r\r\n\0\r"),
        malformed,
        vec![CodePoint::new(0xd800).unwrap(), '!'.into()],
    ] {
        let run = |chunks: Vec<&[CodePoint]>, closed: bool| {
            let context = entry(seeded_context(), "</");
            let mut execution = Execution::new(State::RawtextEndTagOpen, context);
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
                    if drain(&mut execution, &mut trace, input.len() + 5) {
                        // Never feed or reconstruct a terminally blocked execution.
                        return trace;
                    }
                    execution.context_mut().input.push([]).unwrap();
                    assert!(!drain(&mut execution, &mut trace, 1));
                }
                execution.context_mut().input.finish();
            }
            assert!(drain(&mut execution, &mut trace, input.len() + 5));
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
        assert_eq!(run(input.chunks(1).collect(), false), expected);
    }
}
