//! State-local evidence for source lines 143505–143522 at WHATWG HTML revision
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358. These are not upstream conformance runs.

use neptune_html::tokenizer::{
    Attribute, CodePoint, Context, CursorError, Execution, Input, ParseError, PendingComment,
    PendingEndTag, PendingStartTag, Position, Progress, State, TagKind, Token, UnimplementedState,
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

fn context(seeded: bool) -> Context {
    let mut context = if seeded {
        let attribute = Attribute {
            name: points("kept\r\0"),
            value: vec![CodePoint::new(0xd800).unwrap()],
        };
        Context {
            tokens: vec![Token::Character('p'.into())],
            errors: vec![error("prior-diagnostic", 4, 5)],
            return_state: Some(State::AttributeValueDoubleQuoted),
            last_start_tag: Some(points("title")),
            current_tag: Some(TagKind::Start),
            pending_start_tag: Some(PendingStartTag {
                name: points("start\r\0"),
                self_closing: true,
                attributes: vec![attribute.clone()],
                discarded_current_attribute: Some(attribute.clone()),
            }),
            pending_end_tag: Some(PendingEndTag {
                name: points("old-end\r\0"),
                self_closing: true,
                attributes: vec![attribute.clone()],
                discarded_current_attribute: Some(attribute),
            }),
            pending_comment: Some(PendingComment {
                data: points("comment\r\0"),
            }),
            temporary_buffer: vec!['\r'.into(), '\0'.into(), CodePoint::new(0xdfff).unwrap()],
            ..Context::default()
        }
    } else {
        // Real entry from the less-than-sign state has an empty temporary buffer;
        // this isolated state does not require any pending tag or last start tag.
        Context::default()
    };
    // Initialize the cursor as if the preceding prefix was already consumed.
    // Raw CRLF counts as one normalized point; the next input is at (2, 3).
    context.input.push(points("q\r\n</")).unwrap();
    for ch in ['q', '\n', '<', '/'] {
        assert_eq!(context.input.consume(), Input::CodePoint(ch.into()));
    }
    context
}

#[derive(Debug, Eq, PartialEq)]
struct Snapshot {
    state: State,
    current: Option<(Input, Position)>,
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
    input_errors: Vec<ParseError>,
    return_state: Option<State>,
    last_start_tag: Option<Vec<CodePoint>>,
    current_tag: Option<TagKind>,
    pending_start_tag: Option<PendingStartTag>,
    pending_end_tag: Option<PendingEndTag>,
    pending_comment: Option<PendingComment>,
    temporary_buffer: Vec<CodePoint>,
}

fn snapshot(execution: &Execution) -> Snapshot {
    let context = execution.context();
    Snapshot {
        state: execution.state(),
        current: context.input.current(),
        tokens: context.tokens.clone(),
        errors: context.errors.clone(),
        input_errors: context.input.errors().to_vec(),
        return_state: context.return_state,
        last_start_tag: context.last_start_tag.clone(),
        current_tag: context.current_tag,
        pending_start_tag: context.pending_start_tag.clone(),
        pending_end_tag: context.pending_end_tag.clone(),
        pending_comment: context.pending_comment.clone(),
        temporary_buffer: context.temporary_buffer.clone(),
    }
}

fn execution(input: &[CodePoint], seeded: bool, closed: bool) -> Execution {
    let mut context = context(seeded);
    context.input.push(input.iter().copied()).unwrap();
    if closed {
        context.input.finish();
    }
    Execution::new(State::RcdataEndTagOpen, context)
}

fn assert_terminal(execution: &mut Execution, result: Result<Progress, UnimplementedState>) {
    let before = snapshot(execution);
    for _ in 0..3 {
        assert_eq!(execution.step(), result);
        assert_eq!(snapshot(execution), before);
    }
}

#[test]
fn ascii_alpha_creates_and_selects_empty_end_tag_before_terminal_name_stub() {
    for value in (0x41..=0x5a).chain(0x61..=0x7a) {
        let point = CodePoint::new(value).unwrap();
        for seeded in [false, true] {
            for selector in [None, Some(TagKind::Start), Some(TagKind::End)] {
                let mut run = execution(&[point, '!'.into()], seeded, true);
                // Supply matching storage for every selected-tag context.
                if selector.is_some() && !seeded {
                    run.context_mut().pending_start_tag = Some(PendingStartTag::default());
                    run.context_mut().pending_end_tag = Some(PendingEndTag::default());
                }
                run.context_mut().current_tag = selector;
                let mut expected = snapshot(&run);
                expected.state = State::RcdataEndTagName;
                expected.current = Some((Input::CodePoint(point), Position { line: 2, column: 3 }));
                expected.current_tag = Some(TagKind::End);
                expected.pending_end_tag = Some(PendingEndTag {
                    name: vec![],
                    self_closing: false,
                    attributes: vec![],
                    discarded_current_attribute: None,
                });
                assert_eq!(run.step(), Ok(Progress::Continue(State::RcdataEndTagName)));
                assert_eq!(snapshot(&run), expected);
                assert_eq!(
                    run.context_mut().input.reconsume(),
                    Err(CursorError::AlreadyReconsuming)
                );
                let blocked = Err(UnimplementedState {
                    state: State::RcdataEndTagName,
                });
                assert_eq!(run.step(), blocked);
                assert_eq!(snapshot(&run), expected);
                assert_terminal(&mut run, blocked);
            }
        }
        // Inspect cursor source effects independently; do NOT execute/resume a stub.
        let mut run = execution(&[point, '!'.into()], true, true);
        assert_eq!(run.step(), Ok(Progress::Continue(State::RcdataEndTagName)));
        assert_eq!(run.context_mut().input.consume(), Input::CodePoint(point));
        assert_eq!(
            run.context().input.position(),
            Position { line: 2, column: 3 }
        );
        assert_eq!(
            run.context_mut().input.consume(),
            Input::CodePoint('!'.into())
        );
        assert_eq!(
            run.context().input.position(),
            Position { line: 2, column: 4 }
        );
    }
}

#[test]
fn anything_else_emits_only_literal_prefix_then_rcdata_reconsumes() {
    let values = (0..=0x7f)
        .filter(|v| !matches!(v, 0x41..=0x5a | 0x61..=0x7a))
        .chain([
            0x80, 0x9f, 0xa0, 0xc9, 0x3b1, 0xd800, 0xdfff, 0xfdd0, 0xfffe, 0x10ffff, 0x1f600,
        ]);
    for value in values {
        let point = CodePoint::new(value).unwrap();
        let normalized = if value == 0x0d { '\n'.into() } else { point };
        let preprocessing = match value {
            0xd800 | 0xdfff => Some("surrogate-in-input-stream"),
            0xfdd0 | 0xfffe | 0x10ffff => Some("noncharacter-in-input-stream"),
            0x01..=0x08 | 0x0b | 0x0e..=0x1f | 0x7f..=0x9f => {
                Some("control-character-in-input-stream")
            }
            _ => None,
        };
        for seeded in [false, true] {
            let mut run = execution(&[point, '!'.into()], seeded, true);
            let mut expected = snapshot(&run);
            expected.state = State::Rcdata;
            expected.current = Some((
                Input::CodePoint(normalized),
                Position { line: 2, column: 3 },
            ));
            expected
                .tokens
                .extend([Token::Character('<'.into()), Token::Character('/'.into())]);
            if let Some(code) = preprocessing {
                expected.input_errors.push(error(code, 2, 3));
            }
            assert_eq!(run.step(), Ok(Progress::Continue(State::Rcdata)));
            assert_eq!(snapshot(&run), expected, "source U+{value:04X}");
            assert_eq!(
                run.context_mut().input.reconsume(),
                Err(CursorError::AlreadyReconsuming)
            );

            // RCDATA, not this source, handles NUL, ampersand, and '<'.
            let destination = match value {
                0x26 => {
                    expected.state = State::CharacterReference;
                    expected.return_state = Some(State::Rcdata);
                    State::CharacterReference
                }
                0x3c => {
                    expected.state = State::RcdataLessThanSign;
                    State::RcdataLessThanSign
                }
                _ => {
                    let emitted = if value == 0 {
                        expected
                            .errors
                            .push(error("unexpected-null-character", 2, 3));
                        '\u{fffd}'.into()
                    } else {
                        normalized
                    };
                    expected.tokens.push(Token::Character(emitted));
                    State::Rcdata
                }
            };
            assert_eq!(run.step(), Ok(Progress::Continue(destination)));
            assert_eq!(snapshot(&run), expected, "destination U+{value:04X}");
            if destination != State::Rcdata {
                let blocked = Err(UnimplementedState { state: destination });
                assert_eq!(run.step(), blocked);
                assert_eq!(snapshot(&run), expected);
                assert_terminal(&mut run, blocked);
            } else {
                // Tail is still unconsumed after the reconsuming destination step.
                assert_eq!(run.step(), Ok(Progress::Continue(State::Rcdata)));
                expected.tokens.push(Token::Character('!'.into()));
                expected.current = Some((
                    Input::CodePoint('!'.into()),
                    if normalized == '\n'.into() {
                        Position { line: 3, column: 1 }
                    } else {
                        Position { line: 2, column: 4 }
                    },
                ));
                assert_eq!(snapshot(&run), expected);
                assert_eq!(run.step(), Ok(Progress::Finished));
                expected.tokens.push(Token::Eof);
                expected.current = Some((
                    Input::Eof,
                    if normalized == '\n'.into() {
                        Position { line: 3, column: 2 }
                    } else {
                        Position { line: 2, column: 5 }
                    },
                ));
                assert_eq!(snapshot(&run), expected);
                assert_terminal(&mut run, Ok(Progress::Finished));
            }
        }
    }
}

#[test]
fn open_exhaustion_repeatedly_suspends_but_eof_falls_back_and_is_reconsumed() {
    for seeded in [false, true] {
        let mut run = execution(&[], seeded, false);
        let mut expected = snapshot(&run);
        for _ in 0..3 {
            assert_eq!(run.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&run), expected);
            run.context_mut().input.push([]).unwrap();
        }
        run.context_mut().input.finish();
        run.context_mut().input.finish();
        assert_eq!(run.step(), Ok(Progress::Continue(State::Rcdata)));
        expected.state = State::Rcdata;
        expected.current = Some((Input::Eof, Position { line: 2, column: 3 }));
        expected
            .tokens
            .extend([Token::Character('<'.into()), Token::Character('/'.into())]);
        assert_eq!(snapshot(&run), expected);
        assert_eq!(
            run.context_mut().input.reconsume(),
            Err(CursorError::AlreadyReconsuming)
        );
        assert_eq!(run.step(), Ok(Progress::Finished));
        expected.tokens.push(Token::Eof);
        assert_eq!(snapshot(&run), expected);
        assert_terminal(&mut run, Ok(Progress::Finished));
        assert_eq!(
            run.context_mut().input.push([]),
            Err(CursorError::InputAlreadyFinished)
        );

        let mut closed = execution(&[], seeded, true);
        assert_eq!(closed.step(), Ok(Progress::Continue(State::Rcdata)));
        assert_eq!(closed.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&closed), expected);

        // EOF reconsumption itself is inspected separately from destination effects.
        let mut probe = execution(&[], seeded, true);
        assert_eq!(probe.step(), Ok(Progress::Continue(State::Rcdata)));
        assert_eq!(probe.context_mut().input.consume(), Input::Eof);
        assert_eq!(
            probe.context().input.position(),
            Position { line: 2, column: 3 }
        );
    }
}

fn drain(
    run: &mut Execution,
    trace: &mut Vec<(Result<Progress, UnimplementedState>, Snapshot)>,
    bound: usize,
) -> bool {
    for _ in 0..bound {
        let before = snapshot(run);
        let result = run.step();
        if result == Ok(Progress::NeedMoreInput) {
            assert_eq!(snapshot(run), before);
            for _ in 0..2 {
                assert_eq!(run.step(), result);
                assert_eq!(snapshot(run), before);
            }
            return false;
        }
        trace.push((result, snapshot(run)));
        if !matches!(result, Ok(Progress::Continue(_))) {
            assert_terminal(run, result);
            return true;
        }
    }
    panic!("RCDATA end tag open did not consume, suspend, or terminate");
}

#[test]
fn all_two_chunk_splits_and_single_point_chunks_preserve_every_source_step() {
    let mut malformed = points("!é😀\r\n\0\r\r\n?\r");
    malformed.extend([0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x01].map(|v| CodePoint::new(v).unwrap()));
    let samples = vec![
        vec![],
        points("A"),
        points("z\0\r\n!"),
        points("TITLE>tail"),
        points("0abc>"),
        points("\0!"),
        points("&tail"),
        points("<tail"),
        points("/title>"),
        points(">tail"),
        points("\r"),
        points("\r\n"),
        points("\r\r\n\0&tail"),
        malformed,
        vec![CodePoint::new(0xd800).unwrap(), '<'.into(), '\0'.into()],
    ];
    for input in samples {
        for seeded in [false, true] {
            let run = |chunks: Vec<&[CodePoint]>, closed: bool| {
                let mut run = execution(if closed { &input } else { &[] }, seeded, closed);
                let mut trace = Vec::new();
                if !closed {
                    for chunk in chunks {
                        run.context_mut().input.push(chunk.iter().copied()).unwrap();
                        if drain(&mut run, &mut trace, input.len() + 5) {
                            // Never reconstruct or resume a blocked execution.
                            return trace;
                        }
                        run.context_mut().input.push([]).unwrap();
                        assert!(!drain(&mut run, &mut trace, 1));
                    }
                    run.context_mut().input.finish();
                }
                assert!(drain(&mut run, &mut trace, input.len() + 5));
                trace
            };
            let expected = run(vec![], true);
            for split in 0..=input.len() {
                assert_eq!(
                    run(vec![&input[..split], &input[split..]], false),
                    expected,
                    "split {split}, seeded={seeded}, input={input:?}"
                );
            }
            assert_eq!(run(input.chunks(1).collect(), false), expected);
        }
    }
}

#[test]
fn neighboring_entry_states_remain_terminal_and_do_not_fake_completion() {
    for state in [State::RcdataLessThanSign, State::RcdataEndTagName] {
        for closed in [false, true] {
            // Fresh isolated entry, not reconstruction of blocked execution.
            let mut context = context(true);
            context.input.push(points("title>")).unwrap();
            if closed {
                context.input.finish();
            }
            context.current_tag = Some(TagKind::End);
            context.pending_end_tag = Some(PendingEndTag::default());
            // Keep the freshly initialized context with its required end-tag storage.
            let mut run = Execution::new(state, context);
            let expected = snapshot(&run);
            let blocked = Err(UnimplementedState { state });
            assert_eq!(run.step(), blocked);
            assert_eq!(snapshot(&run), expected);
            assert_terminal(&mut run, blocked);
            if !closed {
                run.context_mut()
                    .input
                    .push([CodePoint::new(0xd800).unwrap()])
                    .unwrap();
                run.context_mut().input.finish();
                assert_terminal(&mut run, blocked);
                assert_eq!(snapshot(&run), expected);
            }
        }
    }
}
