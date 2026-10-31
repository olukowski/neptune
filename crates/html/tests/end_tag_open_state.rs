//! Branch evidence for pinned HTML "End tag open state" (#end-tag-open-state):
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>
//! Focused tests, not upstream conformance counts.

use neptune_html::tokenizer::{
    CodePoint, Context, Execution, Input, ParseError, PendingComment, PendingEndTag,
    PendingStartTag, Position, Progress, State, Token, UnimplementedState,
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
            name: points("start"),
            ..PendingStartTag::default()
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("end"),
            ..PendingEndTag::default()
        }),
        pending_comment: Some(PendingComment {
            data: points("old comment"),
        }),
        temporary_buffer: vec![CodePoint::new(0xd800).unwrap(), 'x'.into()],
        ..Context::default()
    }
}

// Exercise the actual Data -> Tag open -> End tag open path as well as both
// isolated entry points. Inspect each source before the following state runs.
fn enter(initial: State, mut context: Context, tail: &[CodePoint], closed: bool) -> Execution {
    let prefix = match initial {
        State::Data => "\r\n</",
        State::TagOpen => "/",
        State::EndTagOpen => "",
        _ => unreachable!(),
    };
    context.input.push(points(prefix)).unwrap();
    context.input.push(tail.iter().copied()).unwrap();
    if closed {
        context.input.finish();
    }
    let mut execution = Execution::new(initial, context);
    if initial == State::Data {
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
        assert_eq!(execution.step(), Ok(Progress::Continue(State::TagOpen)));
        assert_eq!(
            execution.context().input.current(),
            Some((
                Input::CodePoint('<'.into()),
                Position { line: 2, column: 1 }
            ))
        );
    }
    if initial != State::EndTagOpen {
        let before = snapshot(&execution);
        assert_eq!(execution.step(), Ok(Progress::Continue(State::EndTagOpen)));
        assert_eq!(
            execution.context().input.current(),
            Some((Input::CodePoint('/'.into()), position(initial, false)))
        );
        let mut expected = before;
        expected.state = State::EndTagOpen;
        expected.current = execution.context().input.current();
        assert_eq!(snapshot(&execution), expected);
    }
    execution
}

fn position(initial: State, next: bool) -> Position {
    let (line, column) = match initial {
        State::Data => (2, 2),
        State::TagOpen => (1, 1),
        State::EndTagOpen => (1, 0),
        _ => unreachable!(),
    };
    Position {
        line,
        column: column + usize::from(next),
    }
}

#[test]
fn end_tag_open_letters_create_empty_end_tags_without_lowercasing_then_reconsume_in_tag_name() {
    for matched in ('A'..='Z').chain('a'..='z') {
        for initial in [State::Data, State::TagOpen, State::EndTagOpen] {
            for closed in [false, true] {
                let mut execution = enter(initial, seeded(), &[matched.into(), '>'.into()], closed);
                let mut expected = snapshot(&execution);
                assert_eq!(execution.step(), Ok(Progress::Continue(State::TagName)));
                expected.state = State::TagName;
                expected.current =
                    Some((Input::CodePoint(matched.into()), position(initial, true)));
                expected.end = Some(PendingEndTag::default());
                assert_eq!(snapshot(&execution), expected);
                // The next source step reconsumes and appends exactly once.
                assert_eq!(execution.step(), Ok(Progress::Continue(State::TagName)));
                assert_eq!(
                    execution.context().pending_end_tag.as_ref().unwrap().name,
                    points(&matched.to_ascii_lowercase().to_string())
                );
                assert_eq!(
                    execution.context().input.current().unwrap().0,
                    Input::CodePoint(matched.into())
                );
                assert_eq!(
                    execution.context().input.position(),
                    position(initial, true)
                );
                assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
                assert_eq!(
                    execution.context().input.current().unwrap().0,
                    Input::CodePoint('>'.into())
                );
                assert_eq!(
                    execution.context().tokens.last(),
                    Some(&Token::EndTag {
                        name: points(&matched.to_ascii_lowercase().to_string()),
                        attributes: vec![],
                        self_closing: false
                    })
                );
            }
        }
    }
    let mut execution = enter(State::EndTagOpen, Context::default(), &points("Z"), false);
    assert_eq!(execution.step(), Ok(Progress::Continue(State::TagName)));
    assert_eq!(
        execution.context().pending_end_tag,
        Some(PendingEndTag::default())
    );
    assert!(execution.context().pending_start_tag.is_none());
    assert!(execution.context().pending_comment.is_none());
    assert!(execution.context().tokens.is_empty());
    assert_eq!(execution.context().last_start_tag, None);
}

#[test]
fn end_tag_open_anything_else_creates_empty_comment_and_reconsumes_in_bogus_comment() {
    // ASCII-alpha boundaries, digits, punctuation, whitespace, non-ASCII letters,
    // actual surrogates, noncharacters and controls. NUL is NOT replaced here.
    for value in [
        0, 0x09, 0x0a, 0x0c, 0x0d, 0x20, 0x21, 0x22, 0x26, 0x2f, 0x30, 0x39, 0x3c, 0x3f, 0x40,
        0x5b, 0x60, 0x7b, 0xe9, 0x391, 0x301, 0x1f600, 0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x0b,
    ] {
        let point = CodePoint::new(value).unwrap();
        let normalized = if value == 0x0d { '\n'.into() } else { point };
        for initial in [State::Data, State::TagOpen, State::EndTagOpen] {
            for closed in [false, true] {
                let mut execution = enter(initial, seeded(), &[point, '>'.into()], closed);
                let mut expected = snapshot(&execution);
                let pos = position(initial, true);
                assert_eq!(
                    execution.step(),
                    Ok(Progress::Continue(State::BogusComment))
                );
                expected.state = State::BogusComment;
                expected.current = Some((Input::CodePoint(normalized), pos));
                expected.comment = Some(PendingComment::default());
                expected.errors.push(error(
                    "invalid-first-character-of-tag-name",
                    pos.line,
                    pos.column,
                ));
                let preprocessing = match value {
                    0xd800 | 0xdfff => Some("surrogate-in-input-stream"),
                    0xfdd0 | 0x10ffff => Some("noncharacter-in-input-stream"),
                    0x0b => Some("control-character-in-input-stream"),
                    _ => None,
                };
                if let Some(code) = preprocessing {
                    expected
                        .input_errors
                        .push(error(code, pos.line, pos.column));
                }
                assert_eq!(snapshot(&execution), expected);
                // Bogus comment reconsumes once, appending/replacing only here.
                assert_eq!(
                    execution.step(),
                    Ok(Progress::Continue(State::BogusComment))
                );
                expected.comment = Some(PendingComment {
                    data: vec![if value == 0 {
                        '\u{fffd}'.into()
                    } else {
                        normalized
                    }],
                });
                if value == 0 {
                    expected
                        .errors
                        .push(error("unexpected-null-character", pos.line, pos.column));
                }
                assert_eq!(snapshot(&execution), expected);
                assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
                expected.state = State::Data;
                expected.current = Some((
                    Input::CodePoint('>'.into()),
                    Position {
                        line: pos.line + usize::from(normalized == '\n'.into()),
                        column: if normalized == '\n'.into() {
                            1
                        } else {
                            pos.column + 1
                        },
                    },
                ));
                expected.tokens.push(Token::Comment {
                    data: expected.comment.take().unwrap().data,
                });
                assert_eq!(snapshot(&execution), expected);
            }
        }
    }
    let mut execution = enter(State::EndTagOpen, Context::default(), &points("?"), true);
    assert_eq!(
        execution.step(),
        Ok(Progress::Continue(State::BogusComment))
    );
    assert_eq!(
        execution.context().pending_comment,
        Some(PendingComment::default())
    );
    assert!(execution.context().pending_start_tag.is_none());
    assert!(execution.context().pending_end_tag.is_none());
    assert!(execution.context().tokens.is_empty());
    assert_eq!(execution.context().last_start_tag, None);
}

#[test]
fn end_tag_open_greater_than_reports_missing_name_consumes_once_and_recovers_to_real_data() {
    for initial in [State::Data, State::TagOpen, State::EndTagOpen] {
        let mut execution = enter(initial, seeded(), &points(">\0é\r\n</>x"), true);
        let mut expected = snapshot(&execution);
        let pos = position(initial, true);
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
        expected.state = State::Data;
        expected.current = Some((Input::CodePoint('>'.into()), pos));
        expected
            .errors
            .push(error("missing-end-tag-name", pos.line, pos.column));
        // No tag/comment creation, emission, temporary-buffer clear or reconsume.
        assert_eq!(snapshot(&execution), expected);
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
        assert_eq!(
            execution.context().input.current(),
            Some((
                Input::CodePoint('\0'.into()),
                Position {
                    line: pos.line,
                    column: pos.column + 1
                }
            ))
        );
        assert_eq!(
            execution.context().tokens.last(),
            Some(&Token::Character('\0'.into()))
        );
        assert_eq!(
            execution.context().errors.last(),
            Some(&error(
                "unexpected-null-character",
                pos.line,
                pos.column + 1
            ))
        );
        for target in [
            State::Data,
            State::Data,
            State::TagOpen,
            State::EndTagOpen,
            State::Data,
            State::Data,
        ] {
            assert_eq!(execution.step(), Ok(Progress::Continue(target)));
        }
        assert_eq!(
            execution.context().errors.last(),
            Some(&error("missing-end-tag-name", pos.line + 1, 3))
        );
        let mut tokens = expected.tokens;
        tokens.extend(points("\0é\nx").into_iter().map(Token::Character));
        tokens.push(Token::Eof);
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(execution.context().tokens, tokens);
    }
    // Recovery uses Data's transition behavior as well as its literal output.
    for (tail, target) in [("&", State::CharacterReference), ("<A", State::TagName)] {
        let mut execution = enter(
            State::Data,
            Context::default(),
            &points(&format!(">{tail}")),
            true,
        );
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
        if tail == "&" {
            assert_eq!(
                execution.step(),
                Ok(Progress::Continue(State::CharacterReference))
            );
            assert_eq!(execution.context().return_state, Some(State::Data));
        } else {
            assert_eq!(execution.step(), Ok(Progress::Continue(State::TagOpen)));
            assert_eq!(execution.step(), Ok(Progress::Continue(State::TagName)));
        }
        if target == State::TagName {
            assert_eq!(execution.step(), Ok(Progress::Continue(State::TagName)));
            assert_eq!(
                execution.context().pending_start_tag.as_ref().unwrap().name,
                points("a")
            );
        } else {
            assert_terminal_stub(&mut execution, target);
        }
    }
}

#[test]
fn end_tag_open_open_exhaustion_suspends_until_eof_then_emits_less_than_solidus_and_eof_once() {
    for initial in [State::Data, State::TagOpen, State::EndTagOpen] {
        let mut execution = enter(initial, seeded(), &[], false);
        let mut expected = snapshot(&execution);
        for _ in 0..2 {
            execution.context_mut().input.push([]).unwrap();
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&execution), expected);
        }
        execution.context_mut().input.finish();
        assert_eq!(execution.step(), Ok(Progress::Finished));
        let pos = position(initial, true);
        expected.current = Some((Input::Eof, pos));
        expected
            .errors
            .push(error("eof-before-tag-name", pos.line, pos.column));
        expected.tokens.extend([
            Token::Character('<'.into()),
            Token::Character('/'.into()),
            Token::Eof,
        ]);
        assert_eq!(snapshot(&execution), expected);
        for _ in 0..2 {
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&execution), expected);
        }
        let mut closed = enter(initial, seeded(), &[], true);
        assert_eq!(closed.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&closed), expected);
    }
}

#[test]
fn end_tag_open_pending_storage_is_preserved_by_all_previously_implemented_states() {
    for (initial, samples) in [
        (State::Data, vec!["", "a", "\0", "&", "<"]),
        (State::Plaintext, vec!["", "a", "\0", "&", "<"]),
        (
            State::TagOpen,
            vec!["", "!", "/", "A", "z", "?", "0", "\0", "é"],
        ),
    ] {
        for input in samples {
            let mut context = seeded();
            context.input.push(points(input)).unwrap();
            context.input.finish();
            let mut execution = Execution::new(initial, context);
            assert!(execution.step().is_ok());
            assert_eq!(
                execution.context().pending_end_tag,
                seeded().pending_end_tag
            );
            assert_eq!(
                execution.context().pending_comment,
                seeded().pending_comment
            );
        }
    }
}

fn assert_terminal_stub(execution: &mut Execution, target: State) {
    let effects = snapshot(execution);
    for _ in 0..2 {
        assert_eq!(execution.step(), Err(UnimplementedState { state: target }));
        assert_eq!(snapshot(execution), effects);
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Snapshot {
    state: State,
    current: Option<(Input, Position)>,
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
    input_errors: Vec<ParseError>,
    start: Option<PendingStartTag>,
    end: Option<PendingEndTag>,
    comment: Option<PendingComment>,
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
        start: context.pending_start_tag.clone(),
        end: context.pending_end_tag.clone(),
        comment: context.pending_comment.clone(),
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
    panic!("End tag open integration failed to consume or terminate");
}

#[test]
fn end_tag_open_chunk_splits_preserve_all_source_steps_outputs_context_and_diagnostics() {
    let mut samples = vec![
        points(""),
        points("A"),
        points("z"),
        points(">\0\r\n"),
        points("é"),
        points("?"),
        points("/"),
        points("<"),
        points("</"),
        points("</>"),
        points("</></>"),
        points("</>\0é😀\r\n</>x\r"),
        points("</> &"),
        points("</><Z"),
        points("\r\r\n</\r\n"),
        points("</\r"),
        points("</\r\r\n"),
    ];
    for c in [
        'A', 'Z', 'a', 'z', '0', '9', '?', '!', '/', '<', '&', '\0', 'é', 'Α', '😀',
    ] {
        samples.push(points(&format!("\r\n</{c}>unconsumed")));
    }
    for value in [0xd800, 0xdfff, 0xfdd0, 0x0b, 0x10ffff] {
        let mut input = points("</");
        input.push(CodePoint::new(value).unwrap());
        input.extend(points("\r\n>"));
        samples.push(input);
    }
    for initial in [State::Data, State::TagOpen, State::EndTagOpen] {
        for input in &samples {
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
            // In particular, '<' / '/' / next point are three separate chunks.
            assert_eq!(run(input.chunks(1).collect(), false), expected);
        }
    }
}
