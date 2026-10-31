//! State-local evidence: pinned source lines 143486–143502, revision
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358. Not upstream conformance counts.

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
        name: points("a\r\0"),
        value: vec![CodePoint::new(0xd800).unwrap()],
    };
    Context {
        tokens: vec![Token::Character('p'.into())],
        errors: vec![error("prior-diagnostic", 7, 9)],
        return_state: Some(State::AttributeValueDoubleQuoted),
        last_start_tag: Some(points("title")),
        current_tag: Some(TagKind::End),
        pending_start_tag: Some(PendingStartTag {
            name: points("Start\r\0"),
            self_closing: true,
            attributes: vec![attribute.clone()],
            discarded_current_attribute: Some(attribute.clone()),
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("End\r\0"),
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

// Isolated entry has the preceding '<' already consumed. The raw CRLF makes
// positions sensitive to preprocessing; no prefix tokenization is simulated.
fn isolated(mut context: Context) -> Execution {
    context.input.push(points("\r\n<")).unwrap();
    assert_eq!(context.input.consume(), Input::CodePoint('\n'.into()));
    assert_eq!(context.input.consume(), Input::CodePoint('<'.into()));
    assert_eq!(context.input.position(), Position { line: 2, column: 1 });
    Execution::new(State::RcdataLessThanSign, context)
}

#[test]
fn solidus_clears_only_buffer_consumes_once_and_blocks_without_creating_a_tag() {
    for seeded in [false, true] {
        for selector in [None, Some(TagKind::Start), Some(TagKind::End)] {
            for closed in [false, true] {
                for with_tail in [false, true] {
                    let mut context = if seeded {
                        seeded_context()
                    } else {
                        Context::default()
                    };
                    // Defaults require no selected/pending token. Seeded storage can
                    // retain either selection, or no selection, without being replaced.
                    if seeded {
                        context.current_tag = selector;
                    } else {
                        assert!(context.temporary_buffer.is_empty());
                        assert!(context.current_tag.is_none());
                        assert!(context.pending_start_tag.is_none());
                        assert!(context.pending_end_tag.is_none());
                        assert!(context.pending_comment.is_none());
                        assert!(context.last_start_tag.is_none());
                        assert!(context.return_state.is_none());
                    }
                    let mut execution = isolated(context);
                    execution.context_mut().input.push(points("/")).unwrap();
                    let tail = CodePoint::new(0xd800).unwrap();
                    if with_tail {
                        execution.context_mut().input.push([tail]).unwrap();
                    }
                    if closed {
                        execution.context_mut().input.finish();
                    }
                    let mut expected = snapshot(&execution);
                    expected.state = State::RcdataEndTagOpen;
                    expected.current = Some((
                        Input::CodePoint('/'.into()),
                        Position { line: 2, column: 2 },
                    ));
                    expected.temporary_buffer.clear();
                    assert_eq!(
                        execution.step(),
                        Ok(Progress::Continue(State::RcdataEndTagOpen))
                    );
                    assert_eq!(snapshot(&execution), expected);
                    let blocked = Err(UnimplementedState {
                        state: State::RcdataEndTagOpen,
                    });
                    for _ in 0..3 {
                        assert_eq!(execution.step(), blocked);
                        assert_eq!(snapshot(&execution), expected);
                    }
                    // Cursor inspection only; never reconstruct/resume blocked execution.
                    let next = if with_tail {
                        Input::CodePoint(tail)
                    } else if closed {
                        Input::Eof
                    } else {
                        Input::NeedMoreInput
                    };
                    assert_eq!(execution.context_mut().input.consume(), next);
                    if with_tail {
                        assert_eq!(
                            execution.context().input.errors(),
                            [error("surrogate-in-input-stream", 2, 3)]
                        );
                    }
                    if !closed {
                        execution.context_mut().input.push(points("later")).unwrap();
                        execution.context_mut().input.finish();
                    }
                    let inspected = snapshot(&execution);
                    assert_eq!(execution.step(), blocked);
                    assert_eq!(snapshot(&execution), inspected);
                }
            }
        }
    }
}

#[test]
fn anything_else_emits_only_less_than_then_rcdata_reconsumes_with_exact_diagnostics() {
    let mut values: Vec<_> = (0..=0x7f).filter(|&v| v != 0x2f).collect();
    values.extend([
        0x85, 0xa0, 0xe9, 0x301, 0xfeff, 0x1f600, 0xd800, 0xdfff, 0xfdd0, 0x10ffff,
    ]);
    for value in values {
        for seeded in [false, true] {
            let point = CodePoint::new(value).unwrap();
            let mut execution = isolated(if seeded {
                seeded_context()
            } else {
                Context::default()
            });
            execution
                .context_mut()
                .input
                .push([point, 'z'.into()])
                .unwrap();
            execution.context_mut().input.finish();
            let mut expected = snapshot(&execution);
            let normalized = if value == 0x0d { '\n'.into() } else { point };
            expected.state = State::Rcdata;
            expected.current = Some((
                Input::CodePoint(normalized),
                Position { line: 2, column: 2 },
            ));
            expected.tokens.push(Token::Character('<'.into()));
            let diagnostic = match value {
                0xd800 | 0xdfff => Some("surrogate-in-input-stream"),
                0xfdd0 | 0x10ffff => Some("noncharacter-in-input-stream"),
                0x01..=0x08 | 0x0b | 0x0e..=0x1f | 0x7f | 0x85 => {
                    Some("control-character-in-input-stream")
                }
                _ => None,
            };
            if let Some(code) = diagnostic {
                expected.input_errors.push(error(code, 2, 2));
            }
            assert_eq!(
                execution.step(),
                Ok(Progress::Continue(State::Rcdata)),
                "{value:x}"
            );
            assert_eq!(snapshot(&execution), expected);
            assert_eq!(
                execution.context_mut().input.reconsume(),
                Err(CursorError::AlreadyReconsuming)
            );
            // Source did not replace NUL, append the current character, change
            // return state, clear scratch storage, or create/emit a tag.
            let target = match value {
                0x26 => {
                    expected.return_state = Some(State::Rcdata);
                    State::CharacterReference
                }
                0x3c => State::RcdataLessThanSign,
                _ => {
                    if value == 0 {
                        expected
                            .errors
                            .push(error("unexpected-null-character", 2, 2));
                    }
                    expected.tokens.push(Token::Character(if value == 0 {
                        '\u{fffd}'.into()
                    } else {
                        normalized
                    }));
                    State::Rcdata
                }
            };
            expected.state = target;
            assert_eq!(execution.step(), Ok(Progress::Continue(target)));
            assert_eq!(snapshot(&execution), expected);
            if target == State::CharacterReference {
                for _ in 0..3 {
                    assert_eq!(execution.step(), Err(UnimplementedState { state: target }));
                    assert_eq!(snapshot(&execution), expected);
                }
            }
            // The destination reconsumed the same item/position exactly once;
            // preprocessing errors did not duplicate and the tail is untouched.
            assert_eq!(
                execution.context_mut().input.consume(),
                Input::CodePoint('z'.into())
            );
            let position = if normalized == '\n'.into() {
                Position { line: 3, column: 1 }
            } else {
                Position { line: 2, column: 3 }
            };
            assert_eq!(execution.context().input.position(), position);
        }
    }
}

#[test]
fn open_exhaustion_is_effect_free_but_explicit_eof_falls_back_and_is_reconsumed() {
    for seeded in [false, true] {
        let mut execution = isolated(if seeded {
            seeded_context()
        } else {
            Context::default()
        });
        let mut expected = snapshot(&execution);
        for _ in 0..3 {
            execution.context_mut().input.push([]).unwrap();
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&execution), expected);
        }
        execution.context_mut().input.finish();
        expected.state = State::Rcdata;
        expected.current = Some((Input::Eof, Position { line: 2, column: 2 }));
        expected.tokens.push(Token::Character('<'.into()));
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Rcdata)));
        assert_eq!(snapshot(&execution), expected);
        assert_eq!(
            execution.context_mut().input.reconsume(),
            Err(CursorError::AlreadyReconsuming)
        );
        expected.tokens.push(Token::Eof);
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&execution), expected);
        execution.context_mut().input.reconsume().unwrap();
        assert_eq!(
            execution.context_mut().input.push([]),
            Err(CursorError::InputAlreadyFinished)
        );
        for _ in 0..3 {
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&execution), expected);
        }
    }
}

#[test]
fn recovery_normalizes_crlf_once_and_suspends_after_ignored_lf() {
    let mut execution = isolated(seeded_context());
    execution.context_mut().input.push(points("\r")).unwrap();
    assert_eq!(execution.step(), Ok(Progress::Continue(State::Rcdata)));
    let source = snapshot(&execution);
    assert_eq!(
        source.current,
        Some((
            Input::CodePoint('\n'.into()),
            Position { line: 2, column: 2 }
        ))
    );
    assert_eq!(execution.step(), Ok(Progress::Continue(State::Rcdata)));
    let mut expected = source;
    expected.tokens.push(Token::Character('\n'.into()));
    assert_eq!(snapshot(&execution), expected);
    execution.context_mut().input.push(points("\n")).unwrap();
    for _ in 0..3 {
        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
        assert_eq!(snapshot(&execution), expected);
    }
    execution.context_mut().input.push(points("\0")).unwrap();
    expected.current = Some((
        Input::CodePoint('\0'.into()),
        Position { line: 3, column: 1 },
    ));
    expected
        .errors
        .push(error("unexpected-null-character", 3, 1));
    expected.tokens.push(Token::Character('\u{fffd}'.into()));
    assert_eq!(execution.step(), Ok(Progress::Continue(State::Rcdata)));
    assert_eq!(snapshot(&execution), expected);
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
    panic!("state failed to progress, suspend or terminate");
}

#[test]
fn every_two_chunk_split_and_single_point_chunks_match_closed_source_step_traces() {
    let mut nonscalars = points("é😀\r\n\0<");
    nonscalars.extend([0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x0b].map(|v| CodePoint::new(v).unwrap()));
    let samples = [
        vec![],
        points("/"),
        points("/title>ignored"),
        points("a>!\0"),
        points("\0\r\n\0\r"),
        points("\r"),
        points("\r\n"),
        points("\r\r"),
        points("<<"),
        points("<!--x--><?x>"),
        points("<&amp;"),
        points("<</x>"),
        points("abc<\0</title>"),
        nonscalars,
    ];
    for input in samples {
        // Exercise isolated correctly initialized entry and real RCDATA entry.
        for isolated_entry in [false, true] {
            let mut raw = input.clone();
            if !isolated_entry {
                raw.insert(0, '<'.into());
            }
            let run = |chunks: Vec<&[CodePoint]>, closed: bool| {
                let mut execution = if isolated_entry {
                    isolated(seeded_context())
                } else {
                    Execution::new(State::Rcdata, seeded_context())
                };
                let mut trace = Vec::new();
                let bound = raw.len() * 2 + 6;
                if closed {
                    execution
                        .context_mut()
                        .input
                        .push(raw.iter().copied())
                        .unwrap();
                    execution.context_mut().input.finish();
                } else {
                    assert!(!drain(&mut execution, &mut trace, bound));
                    for chunk in chunks {
                        execution
                            .context_mut()
                            .input
                            .push(chunk.iter().copied())
                            .unwrap();
                        if drain(&mut execution, &mut trace, bound) {
                            return trace; // No reconstruction or resumption of a stub.
                        }
                        execution.context_mut().input.push([]).unwrap();
                        assert!(!drain(&mut execution, &mut trace, bound));
                    }
                    execution.context_mut().input.finish();
                }
                assert!(drain(&mut execution, &mut trace, bound));
                trace
            };
            let expected = run(vec![], true);
            for split in 0..=raw.len() {
                assert_eq!(
                    run(vec![&raw[..split], &raw[split..]], false),
                    expected,
                    "split {split}, input {raw:?}"
                );
            }
            assert_eq!(
                run(raw.chunks(1).collect(), false),
                expected,
                "input {raw:?}"
            );
        }
    }
}
