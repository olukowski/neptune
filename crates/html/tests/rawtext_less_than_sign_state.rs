//! State-local branch evidence for HTML revision 404b3eb14a7a936dddefb2d78e224dbeb01d2358:
//! #rawtext-less-than-sign-state (source lines 143570–143586).
//! Local tests are not upstream conformance runs; end tag name stays a stub.

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
        last_start_tag: Some(points("style")),
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
        tokens: vec![Token::Character('p'.into())],
        errors: vec![error("prior-diagnostic", 1, 1)],
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

// Isolated entry has already consumed '<', but no following input. No blocked
// execution is reconstructed: setup initializes only the existing input cursor.
fn entry(mut context: Context, newline_prefix: bool) -> Execution {
    if newline_prefix {
        context.input.push(points("\r\n")).unwrap();
        assert_eq!(context.input.consume(), Input::CodePoint('\n'.into()));
    }
    context.input.push(points("<")).unwrap();
    assert_eq!(context.input.consume(), Input::CodePoint('<'.into()));
    Execution::new(State::RawtextLessThanSign, context)
}

#[test]
fn solidus_clears_only_buffer_before_end_tag_open_fallback_and_rawtext_null_effects() {
    for finished in [false, true] {
        for selected in [None, Some(TagKind::Start), Some(TagKind::End)] {
            let mut context = seeded_context();
            context.current_tag = selected;
            let mut execution = entry(context, true);
            execution.context_mut().input.push(points("/\0z")).unwrap();
            if finished {
                execution.context_mut().input.finish();
            }
            let mut expected = snapshot(&execution);
            assert_eq!(
                execution.step(),
                Ok(Progress::Continue(State::RawtextEndTagOpen))
            );
            expected.state = State::RawtextEndTagOpen;
            expected.current = Some((
                Input::CodePoint('/'.into()),
                Position { line: 2, column: 2 },
            ));
            expected.storage.buffer.clear();
            assert_eq!(snapshot(&execution), expected);
            // Only the destination consumes NUL and emits the literal prefix.
            assert_eq!(execution.step(), Ok(Progress::Continue(State::Rawtext)));
            expected.state = State::Rawtext;
            expected.current = Some((
                Input::CodePoint('\0'.into()),
                Position { line: 2, column: 3 },
            ));
            expected
                .tokens
                .extend([Token::Character('<'.into()), Token::Character('/'.into())]);
            assert_eq!(snapshot(&execution), expected);
            assert_eq!(
                execution.context_mut().input.reconsume(),
                Err(CursorError::AlreadyReconsuming)
            );
            // RAWTEXT, not either source state, replaces NUL and diagnoses it.
            assert_eq!(execution.step(), Ok(Progress::Continue(State::Rawtext)));
            expected.tokens.push(Token::Character('\u{fffd}'.into()));
            expected
                .errors
                .push(error("unexpected-null-character", 2, 3));
            assert_eq!(snapshot(&execution), expected);
            assert_eq!(execution.step(), Ok(Progress::Continue(State::Rawtext)));
            expected.current = Some((
                Input::CodePoint('z'.into()),
                Position { line: 2, column: 4 },
            ));
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
            expected.current = Some((Input::Eof, Position { line: 2, column: 5 }));
            expected.tokens.push(Token::Eof);
            for _ in 0..3 {
                assert_eq!(execution.step(), Ok(Progress::Finished));
                assert_eq!(snapshot(&execution), expected);
            }
        }
    }
}

#[test]
fn default_entry_initializes_empty_buffer_without_creating_any_token_storage() {
    for input in ["/", "x", ""] {
        let mut execution = entry(Context::default(), false);
        execution.context_mut().input.push(points(input)).unwrap();
        execution.context_mut().input.finish();
        let expected = storage(execution.context());
        assert!(expected.buffer.is_empty());
        assert_eq!(expected.current_tag, None);
        assert_eq!(expected.start, None);
        assert_eq!(expected.end, None);
        assert_eq!(expected.comment, None);
        assert_eq!(expected.last_start_tag, None);
        assert_eq!(expected.return_state, None);
        let target = if input == "/" {
            State::RawtextEndTagOpen
        } else {
            State::Rawtext
        };
        assert_eq!(execution.step(), Ok(Progress::Continue(target)));
        assert_eq!(storage(execution.context()), expected);
        assert!(execution.context().errors.is_empty());
        assert!(execution.context().input.errors().is_empty());
        assert_eq!(
            execution.context().tokens,
            if input == "/" {
                vec![]
            } else {
                vec![Token::Character('<'.into())]
            }
        );
    }
}

#[test]
fn every_ascii_fallback_and_non_scalar_preserves_buffer_and_reconsumes_in_rawtext() {
    let values = (0..=0x7f).filter(|v| *v != 0x2f).chain([
        0x80, 0x9f, 0xa0, 0xc9, 0x3a9, 0xfeff, 0x1f600, 0xd800, 0xdfff, 0xfdd0, 0xffff, 0x10ffff,
    ]);
    for value in values {
        for finished in [false, true] {
            let point = CodePoint::new(value).unwrap();
            let normalized = if value == 0x0d { '\n'.into() } else { point };
            let mut execution = entry(seeded_context(), true);
            execution
                .context_mut()
                .input
                .push([point, 'z'.into()])
                .unwrap();
            if finished {
                execution.context_mut().input.finish();
            }
            let preserved = storage(execution.context());
            let diagnostic = match value {
                0xd800..=0xdfff => Some("surrogate-in-input-stream"),
                0xfdd0..=0xfdef | 0xffff | 0x10ffff => Some("noncharacter-in-input-stream"),
                1..=8 | 0x0b | 0x0e..=0x1f | 0x7f..=0x9f => {
                    Some("control-character-in-input-stream")
                }
                _ => None,
            };
            let expected_input_errors: Vec<_> = diagnostic
                .into_iter()
                .map(|code| error(code, 2, 2))
                .collect();
            assert_eq!(execution.step(), Ok(Progress::Continue(State::Rawtext)));
            assert_eq!(
                execution.context().tokens,
                [Token::Character('p'.into()), Token::Character('<'.into())]
            );
            assert_eq!(
                execution.context().errors,
                [error("prior-diagnostic", 1, 1)]
            );
            assert_eq!(execution.context().input.errors(), expected_input_errors);
            assert_eq!(
                execution.context().input.current(),
                Some((
                    Input::CodePoint(normalized),
                    Position { line: 2, column: 2 }
                ))
            );
            assert_eq!(storage(execution.context()), preserved);
            assert_eq!(
                execution.context_mut().input.reconsume(),
                Err(CursorError::AlreadyReconsuming)
            );

            // Destination effects are deliberately inspected separately: only
            // RAWTEXT replaces NUL/reports its error, or branches on a second '<'.
            let target = if value == 0x3c {
                State::RawtextLessThanSign
            } else {
                State::Rawtext
            };
            assert_eq!(execution.step(), Ok(Progress::Continue(target)));
            let mut tokens = vec![Token::Character('p'.into()), Token::Character('<'.into())];
            if value != 0x3c {
                tokens.push(Token::Character(if value == 0 {
                    '\u{fffd}'.into()
                } else {
                    normalized
                }));
            }
            assert_eq!(execution.context().tokens, tokens);
            let mut errors = vec![error("prior-diagnostic", 1, 1)];
            if value == 0 {
                errors.push(error("unexpected-null-character", 2, 2));
            }
            assert_eq!(execution.context().errors, errors);
            assert_eq!(execution.context().input.errors(), expected_input_errors);
            assert_eq!(
                execution.context().input.current(),
                Some((
                    Input::CodePoint(normalized),
                    Position { line: 2, column: 2 }
                ))
            );
            assert_eq!(storage(execution.context()), preserved);
            assert_eq!(
                execution.context_mut().input.consume(),
                Input::CodePoint('z'.into())
            );
        }
    }
}

#[test]
fn open_exhaustion_repeatedly_suspends_but_eof_falls_back_then_rawtext_finishes_once() {
    let mut execution = entry(seeded_context(), true);
    let before = snapshot(&execution);
    for _ in 0..3 {
        execution.context_mut().input.push([]).unwrap();
        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
        assert_eq!(snapshot(&execution), before);
    }
    execution.context_mut().input.finish();
    execution.context_mut().input.finish();
    assert_eq!(execution.step(), Ok(Progress::Continue(State::Rawtext)));
    let mut expected = before;
    expected.state = State::Rawtext;
    expected.current = Some((Input::Eof, Position { line: 2, column: 2 }));
    expected.tokens.push(Token::Character('<'.into()));
    assert_eq!(snapshot(&execution), expected);
    assert_eq!(
        execution.context_mut().input.reconsume(),
        Err(CursorError::AlreadyReconsuming)
    );
    assert_eq!(execution.step(), Ok(Progress::Finished));
    expected.tokens.push(Token::Eof);
    assert_eq!(snapshot(&execution), expected);
    for _ in 0..3 {
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&execution), expected);
    }
}

#[test]
fn suspended_entry_receives_solidus_then_alpha_creates_empty_end_tag_before_name_stub() {
    let mut execution = entry(seeded_context(), false);
    assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
    execution.context_mut().input.push(points("/")).unwrap();
    assert_eq!(
        execution.step(),
        Ok(Progress::Continue(State::RawtextEndTagOpen))
    );
    let mut expected = snapshot(&execution);
    for _ in 0..3 {
        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
        assert_eq!(snapshot(&execution), expected);
    }
    execution
        .context_mut()
        .input
        .push(points("style>"))
        .unwrap();
    execution.context_mut().input.finish();
    assert_eq!(
        execution.step(),
        Ok(Progress::Continue(State::RawtextEndTagName))
    );
    expected.state = State::RawtextEndTagName;
    expected.current = Some((
        Input::CodePoint('s'.into()),
        Position { line: 1, column: 3 },
    ));
    expected.storage.current_tag = Some(TagKind::End);
    expected.storage.end = Some(PendingEndTag {
        name: vec![],
        self_closing: false,
        attributes: vec![],
        discarded_current_attribute: None,
    });
    assert_eq!(snapshot(&execution), expected);
    let blocked = Err(UnimplementedState {
        state: State::RawtextEndTagName,
    });
    for _ in 0..3 {
        assert_eq!(execution.step(), blocked);
        assert_eq!(snapshot(&execution), expected);
    }
    // Inspect the retained cursor only; no resumption after the terminal stub.
    assert_eq!(
        execution.context_mut().input.consume(),
        Input::CodePoint('s'.into())
    );
    assert_eq!(
        execution.context_mut().input.consume(),
        Input::CodePoint('t'.into())
    );
    let inspected = snapshot(&execution);
    assert_eq!(execution.step(), blocked);
    assert_eq!(snapshot(&execution), inspected);
}

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
            let terminal = snapshot(execution);
            for _ in 0..3 {
                assert_eq!(execution.step(), result);
                assert_eq!(snapshot(execution), terminal);
            }
            return true;
        }
    }
    panic!("state-local execution did not suspend or terminate");
}

#[test]
fn all_two_chunk_splits_and_single_point_chunks_match_every_source_step() {
    let mut mixed = points("!<&amp;\0\r\n<\r\ré😀");
    mixed.extend([0xd800, 0xdfff, 0xfdd0, 0x1fffe, 0x0b].map(|v| CodePoint::new(v).unwrap()));
    mixed.extend(points("\r"));
    let mut blocked = mixed.clone();
    blocked.extend(points("</style>unconsumed\0"));
    for initial in [State::RawtextLessThanSign, State::Rawtext] {
        for input in [
            vec![],
            points("/"),
            points("/style>"),
            points("<"),
            points("<<"),
            points("&"),
            points("\0"),
            points("\r"),
            points("\r\n"),
            points("\r\r"),
            points("<\0"),
            mixed.clone(),
            blocked.clone(),
        ] {
            let run = |chunks: Vec<&[CodePoint]>, closed: bool| {
                let mut execution = if initial == State::RawtextLessThanSign {
                    entry(seeded_context(), false)
                } else {
                    Execution::new(initial, seeded_context())
                };
                let mut trace = Vec::new();
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
                        if drain(&mut execution, &mut trace, input.len() * 3 + 5) {
                            // Terminal means stop: no feeding/resuming a stub.
                            return trace;
                        }
                        execution.context_mut().input.push([]).unwrap();
                        assert!(!drain(&mut execution, &mut trace, 1));
                    }
                    execution.context_mut().input.finish();
                }
                assert!(drain(&mut execution, &mut trace, input.len() * 3 + 5));
                trace
            };
            let expected = run(vec![], true);
            for split in 0..=input.len() {
                assert_eq!(
                    run(vec![&input[..split], &input[split..]], false),
                    expected,
                    "{initial:?}, split {split}: {input:?}"
                );
            }
            assert_eq!(run(input.chunks(1).collect(), false), expected);
        }
    }
}
