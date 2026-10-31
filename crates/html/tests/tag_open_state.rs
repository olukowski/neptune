//! Branch evidence for pinned HTML "Tag open state" (#tag-open-state):
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>
//! Focused tests, not upstream conformance counts.

use neptune_html::tokenizer::{
    CodePoint, Context, Execution, Input, ParseError, PendingStartTag, Position, Progress, State,
    Token, UnimplementedState,
};

fn points(text: &str) -> Vec<CodePoint> {
    text.chars().map(CodePoint::from).collect()
}

fn error(code: &'static str, line: usize, column: usize) -> ParseError {
    ParseError {
        code,
        position: Position { line, column },
    }
}

fn seeded() -> Context {
    Context {
        tokens: vec![Token::Character('p'.into())],
        errors: vec![error("prior-diagnostic", 1, 1)],
        last_start_tag: Some(points("title")),
        return_state: Some(State::Rcdata),
        pending_start_tag: Some(PendingStartTag {
            name: points("old"),
            ..PendingStartTag::default()
        }),
        temporary_buffer: vec![CodePoint::new(0xd800).unwrap(), 'x'.into()],
        ..Context::default()
    }
}

#[test]
fn tag_open_switch_branches_consume_once_and_preserve_effects_at_terminal_stubs() {
    let mut branches = vec![
        ('!', State::MarkupDeclarationOpen),
        ('/', State::EndTagOpen),
        ('?', State::ProcessingInstructionOpen),
    ];
    // Every ASCII alpha, both cases: no initial lowercasing or name append.
    branches.extend(('A'..='Z').chain('a'..='z').map(|c| (c, State::TagName)));
    for (matched, target) in branches {
        for initial in [State::Data, State::TagOpen] {
            for finished in [false, true] {
                let mut context = seeded();
                if initial == State::Data {
                    context.input.push(points("<")).unwrap();
                }
                let tail = CodePoint::new(0xdfff).unwrap();
                context.input.push([matched.into(), tail]).unwrap();
                if finished {
                    context.input.finish();
                }
                let mut execution = Execution::new(initial, context);
                if initial == State::Data {
                    assert_eq!(execution.step(), Ok(Progress::Continue(State::TagOpen)));
                    assert_eq!(
                        execution.context().input.current(),
                        Some((
                            Input::CodePoint('<'.into()),
                            Position { line: 1, column: 1 }
                        ))
                    );
                }
                assert_eq!(execution.step(), Ok(Progress::Continue(target)));
                let column = if initial == State::Data { 2 } else { 1 };
                assert_eq!(
                    execution.context().input.current(),
                    Some((
                        Input::CodePoint(matched.into()),
                        Position { line: 1, column }
                    ))
                );
                let context = execution.context();
                assert_eq!(context.tokens, [Token::Character('p'.into())]);
                assert_eq!(context.errors, [error("prior-diagnostic", 1, 1)]);
                assert!(context.input.errors().is_empty());
                assert_eq!(context.last_start_tag, Some(points("title")));
                assert_eq!(context.return_state, Some(State::Rcdata));
                assert_eq!(
                    context.pending_start_tag,
                    Some(PendingStartTag {
                        name: if target == State::TagName {
                            vec![]
                        } else {
                            points("old")
                        },
                        attributes: vec![],
                        self_closing: false,
                        discarded_current_attribute: None,
                    })
                );
                assert_eq!(
                    context.temporary_buffer,
                    if matched == '?' {
                        vec![]
                    } else {
                        vec![CodePoint::new(0xd800).unwrap(), 'x'.into()]
                    }
                );
                let effects = snapshot(&execution);
                let blocked = Err(UnimplementedState { state: target });
                // End tag open and Tag name now execute; only other targets are stubs.
                if !matches!(target, State::EndTagOpen | State::TagName) {
                    for _ in 0..2 {
                        assert_eq!(execution.step(), blocked);
                        assert_eq!(snapshot(&execution), effects);
                    }
                }
                // Inspection only, NOT supported resumption after a stub.
                // Letters reconsume; punctuation switches without reconsuming.
                let expected = if target == State::TagName {
                    matched.into()
                } else {
                    tail
                };
                assert_eq!(
                    execution.context_mut().input.consume(),
                    Input::CodePoint(expected)
                );
                assert_eq!(
                    execution.context().input.position(),
                    Position {
                        line: 1,
                        column: column + usize::from(target != State::TagName)
                    }
                );
                if !matches!(target, State::EndTagOpen | State::TagName) {
                    assert_eq!(execution.step(), blocked);
                }
            }
        }
    }
    // Creation also works from the default absent context, without emitting it.
    let mut context = Context::default();
    context.input.push(points("A")).unwrap();
    let mut execution = Execution::new(State::TagOpen, context);
    assert_eq!(execution.step(), Ok(Progress::Continue(State::TagName)));
    assert_eq!(
        execution.context().pending_start_tag,
        Some(PendingStartTag::default())
    );
    assert!(execution.context().tokens.is_empty());
    assert_eq!(execution.context().last_start_tag, None);
}

#[test]
fn tag_open_anything_else_emits_less_than_and_reconsumes_unchanged_in_data() {
    // Include ASCII-alpha boundaries, non-ASCII letters, NUL, actual surrogates,
    // noncharacters, controls, and the other Data transition characters.
    for value in [
        0, 0x09, 0x0a, 0x0c, 0x20, 0x22, 0x26, 0x30, 0x39, 0x3c, 0x3e, 0x40, 0x5b, 0x60, 0x7b,
        0xe9, 0x391, 0x301, 0x1f600, 0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x0b,
    ] {
        let point = CodePoint::new(value).unwrap();
        let mut context = seeded();
        context.input.push(['<'.into(), point, 'z'.into()]).unwrap();
        context.input.finish();
        let mut execution = Execution::new(State::Data, context);
        assert_eq!(execution.step(), Ok(Progress::Continue(State::TagOpen)));
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
        let position = Position { line: 1, column: 2 };
        assert_eq!(
            execution.context().input.current(),
            Some((Input::CodePoint(point), position))
        );
        assert_eq!(
            execution.context().tokens,
            [Token::Character('p'.into()), Token::Character('<'.into())]
        );
        assert_eq!(
            execution.context().errors,
            [
                error("prior-diagnostic", 1, 1),
                error("invalid-first-character-of-tag-name", 1, 2)
            ]
        );
        assert_eq!(
            execution.context().pending_start_tag,
            seeded().pending_start_tag
        );
        assert_eq!(
            execution.context().temporary_buffer,
            seeded().temporary_buffer
        );
        assert_eq!(execution.context().last_start_tag, seeded().last_start_tag);
        assert_eq!(execution.context().return_state, seeded().return_state);
        let preprocessing = match value {
            0xd800 | 0xdfff => vec![error("surrogate-in-input-stream", 1, 2)],
            0xfdd0 | 0x10ffff => vec![error("noncharacter-in-input-stream", 1, 2)],
            0x0b => vec![error("control-character-in-input-stream", 1, 2)],
            _ => vec![],
        };
        assert_eq!(execution.context().input.errors(), preprocessing);
        let next = match value {
            0x26 => State::CharacterReference,
            0x3c => State::TagOpen,
            _ => State::Data,
        };
        assert_eq!(execution.step(), Ok(Progress::Continue(next)));
        // Data processed exactly the same code point at the same location.
        assert_eq!(
            execution.context().input.current(),
            Some((Input::CodePoint(point), position))
        );
        assert_eq!(execution.context().input.errors(), preprocessing);
        if !matches!(value, 0x26 | 0x3c) {
            assert_eq!(
                execution.context().tokens.last(),
                Some(&Token::Character(point))
            );
            assert_eq!(
                execution.context().errors.len(),
                if value == 0 { 3 } else { 2 }
            );
            if value == 0 {
                assert_eq!(
                    execution.context().errors[2],
                    error("unexpected-null-character", 1, 2)
                );
            }
            assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
            assert_eq!(
                execution.context().tokens.last(),
                Some(&Token::Character('z'.into()))
            );
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(execution.context().tokens.last(), Some(&Token::Eof));
        }
    }
}

#[test]
fn tag_open_open_exhaustion_is_not_eof_and_eof_emits_less_than_then_one_eof() {
    for initial in [State::Data, State::TagOpen] {
        let mut context = seeded();
        if initial == State::Data {
            context.input.push(points("\r\n<")).unwrap();
        }
        let mut execution = Execution::new(initial, context);
        if initial == State::Data {
            assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
            assert_eq!(execution.step(), Ok(Progress::Continue(State::TagOpen)));
        }
        let suspended = snapshot(&execution);
        for _ in 0..2 {
            execution.context_mut().input.push([]).unwrap();
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&execution), suspended);
        }
        execution.context_mut().input.finish();
        assert_eq!(execution.step(), Ok(Progress::Finished));
        let (line, column) = if initial == State::Data {
            (2, 2)
        } else {
            (1, 1)
        };
        assert_eq!(
            execution.context().input.current(),
            Some((Input::Eof, Position { line, column }))
        );
        assert_eq!(
            execution.context().errors,
            [
                error("prior-diagnostic", 1, 1),
                error("eof-before-tag-name", line, column)
            ]
        );
        let mut expected = suspended.tokens;
        expected.extend([Token::Character('<'.into()), Token::Eof]);
        assert_eq!(execution.context().tokens, expected);
        assert_eq!(
            execution.context().pending_start_tag,
            seeded().pending_start_tag
        );
        assert_eq!(
            execution.context().temporary_buffer,
            seeded().temporary_buffer
        );
        assert!(execution.context().input.errors().is_empty());
        let completed = snapshot(&execution);
        for _ in 0..2 {
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&execution), completed);
        }
    }
}

#[test]
fn tag_open_normalized_newline_recovery_keeps_locations_and_skips_raw_lf_once() {
    let mut context = Context::default();
    context.input.push(points("\r\n<\r\n\0")).unwrap();
    context.input.finish();
    let mut execution = Execution::new(State::Data, context);
    for state in [
        State::Data,
        State::TagOpen,
        State::Data,
        State::Data,
        State::Data,
    ] {
        assert_eq!(execution.step(), Ok(Progress::Continue(state)));
    }
    assert_eq!(
        execution.context().input.current(),
        Some((
            Input::CodePoint('\0'.into()),
            Position { line: 3, column: 1 }
        ))
    );
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(
        execution.context().tokens,
        [
            Token::Character('\n'.into()),
            Token::Character('<'.into()),
            Token::Character('\n'.into()),
            Token::Character('\0'.into()),
            Token::Eof
        ]
    );
    assert_eq!(
        execution.context().errors,
        [
            error("invalid-first-character-of-tag-name", 2, 2),
            error("unexpected-null-character", 3, 1)
        ]
    );
    assert!(execution.context().input.errors().is_empty());
    assert_eq!(
        execution.context().input.position(),
        Position { line: 3, column: 2 }
    );
}

#[derive(Debug, Eq, PartialEq)]
struct Snapshot {
    state: State,
    current: Option<(Input, Position)>,
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
    input_errors: Vec<ParseError>,
    pending: Option<PendingStartTag>,
    buffer: Vec<CodePoint>,
    return_state: Option<State>,
    last_start_tag: Option<Vec<CodePoint>>,
}

fn snapshot(execution: &Execution) -> Snapshot {
    let context = execution.context();
    Snapshot {
        state: execution.state(),
        current: context.input.current(),
        tokens: context.tokens.clone(),
        errors: context.errors.clone(),
        input_errors: context.input.errors().to_vec(),
        pending: context.pending_start_tag.clone(),
        buffer: context.temporary_buffer.clone(),
        return_state: context.return_state,
        last_start_tag: context.last_start_tag.clone(),
    }
}

fn drain(
    execution: &mut Execution,
    trace: &mut Vec<(Result<Progress, UnimplementedState>, Snapshot)>,
    bound: usize,
) -> bool {
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
            assert_eq!(execution.step(), result);
            assert_eq!(snapshot(execution), terminal);
            return true;
        }
    }
    panic!("Tag open integration failed to consume or terminate");
}

#[test]
fn tag_open_all_chunk_splits_match_closed_source_traces_including_preprocessing() {
    let mut samples = vec![
        points("<"),
        points("<<"),
        points("<\0"),
        points("<é"),
        points("<&"),
        points("<\r\n0"),
        points("<\r"),
        points("<\r\r\n<0\r"),
        points("\r\n<\0é😀\r\n<0"),
        points("<0<9<>"),
    ];
    for c in ['!', '/', '?', 'A', 'Z', 'a', 'z'] {
        samples.push(points(&format!("\r\n<{c}unconsumed")));
    }
    for value in [0xd800, 0xdfff, 0xfdd0, 0x0b, 0x10ffff] {
        samples.push(vec![
            '<'.into(),
            CodePoint::new(value).unwrap(),
            '\r'.into(),
            '\n'.into(),
        ]);
    }
    for initial in [State::Data, State::TagOpen] {
        for input in std::iter::once(vec![]).chain(samples.clone()) {
            let run = |chunks: Vec<&[CodePoint]>, closed: bool| {
                let mut execution = Execution::new(initial, seeded());
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
                        if drain(&mut execution, &mut trace, input.len() * 3 + 5) {
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
                    "{initial:?}: split {split} of {input:?}"
                );
            }
            assert_eq!(run(input.chunks(1).collect(), false), expected);
        }
    }
}
