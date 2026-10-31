//! Source-step evidence for Script data state (#script-data-state), pinned HTML
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358, source lines 143338–143359.
//! See docs/tokenizer-script-data.md. Local checks are not conformance runs.

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

fn seeded_context(return_state: Option<State>) -> Context {
    let attribute = Attribute {
        name: points("a"),
        value: points("retained"),
    };
    let discarded = Attribute {
        name: points("a"),
        value: points("discarded"),
    };
    Context {
        return_state,
        last_start_tag: Some(points("script")),
        current_tag: Some(TagKind::End),
        pending_start_tag: Some(PendingStartTag {
            name: points("start"),
            self_closing: true,
            attributes: vec![attribute.clone()],
            discarded_current_attribute: Some(discarded.clone()),
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("end"),
            self_closing: true,
            attributes: vec![attribute],
            discarded_current_attribute: Some(discarded),
        }),
        pending_comment: Some(PendingComment {
            data: points("comment"),
        }),
        temporary_buffer: points("scratch"),
        tokens: vec![Token::Character('p'.into())],
        errors: vec![error("prior-diagnostic", 1, 1)],
        ..Default::default()
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Retained {
    return_state: Option<State>,
    last_start_tag: Option<Vec<CodePoint>>,
    current_tag: Option<TagKind>,
    start: Option<PendingStartTag>,
    end: Option<PendingEndTag>,
    comment: Option<PendingComment>,
    temporary_buffer: Vec<CodePoint>,
}

fn retained(context: &Context) -> Retained {
    Retained {
        return_state: context.return_state,
        last_start_tag: context.last_start_tag.clone(),
        current_tag: context.current_tag,
        start: context.pending_start_tag.clone(),
        end: context.pending_end_tag.clone(),
        comment: context.pending_comment.clone(),
        temporary_buffer: context.temporary_buffer.clone(),
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Snapshot {
    state: State,
    current: Option<(Input, Position)>,
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
    input_errors: Vec<ParseError>,
    retained: Retained,
}

fn snapshot(execution: &Execution) -> Snapshot {
    let context = execution.context();
    Snapshot {
        state: execution.state(),
        current: context.input.current(),
        tokens: context.tokens.clone(),
        errors: context.errors.clone(),
        input_errors: context.input.errors().to_vec(),
        retained: retained(context),
    }
}

#[test]
fn less_than_consumes_only_the_delimiter_before_destination_dispatch() {
    for finished in [false, true] {
        for suffix in [
            vec![],
            points("/script>"),
            points("!"),
            points("\0"),
            vec![CodePoint::new(0xd800).unwrap()],
        ] {
            let mut context = seeded_context(Some(State::Rcdata));
            context.input.push(points("<")).unwrap();
            context.input.push(suffix.iter().copied()).unwrap();
            if finished {
                context.input.finish();
            }
            let preserved = retained(&context);
            let tokens = context.tokens.clone();
            let errors = context.errors.clone();
            let mut execution = Execution::new(State::ScriptData, context);
            assert_eq!(
                execution.step(),
                Ok(Progress::Continue(State::ScriptDataLessThanSign))
            );
            assert_eq!(
                execution.context().input.current(),
                Some((
                    Input::CodePoint('<'.into()),
                    Position { line: 1, column: 1 }
                ))
            );
            assert_eq!(retained(execution.context()), preserved);
            assert_eq!(execution.context().tokens, tokens);
            assert_eq!(execution.context().errors, errors);
            assert!(execution.context().input.errors().is_empty());
            // Inspect the untouched cursor at the source boundary, without
            // executing the destination. This proves no '<' reconsumption and
            // no eager preprocessing of the tail. Real destination execution is
            // covered separately in script_data_less_than_sign_state.rs.
            let next = suffix.first().copied().map_or(
                if finished {
                    Input::Eof
                } else {
                    Input::NeedMoreInput
                },
                Input::CodePoint,
            );
            assert_eq!(execution.context_mut().input.consume(), next);
        }
    }
    // The real less-than-sign destination emits both '<' and '!' before the
    // escape-start source suspends, then the real escape chain reaches its '<' stub.
    let mut context = Context::default();
    context.input.push(points("<!")).unwrap();
    let mut execution = Execution::new(State::ScriptData, context);
    assert_eq!(
        execution.step(),
        Ok(Progress::Continue(State::ScriptDataLessThanSign))
    );
    assert_eq!(
        execution.step(),
        Ok(Progress::Continue(State::ScriptDataEscapeStart))
    );
    assert_eq!(
        execution.context().tokens,
        [Token::Character('<'.into()), Token::Character('!'.into())]
    );
    assert_eq!(
        execution.context().input.current(),
        Some((
            Input::CodePoint('!'.into()),
            Position { line: 1, column: 2 }
        ))
    );
    assert!(execution.context().errors.is_empty());
    assert!(execution.context().input.errors().is_empty());
    let boundary = snapshot(&execution);
    for _ in 0..3 {
        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
        assert_eq!(snapshot(&execution), boundary);
    }
    execution
        .context_mut()
        .input
        .push(points("--<tail\0"))
        .unwrap();
    let mut expected = boundary;
    expected.state = State::ScriptDataEscapeStartDash;
    expected.current = Some((
        Input::CodePoint('-'.into()),
        Position { line: 1, column: 3 },
    ));
    expected.tokens.push(Token::Character('-'.into()));
    assert_eq!(
        execution.step(),
        Ok(Progress::Continue(State::ScriptDataEscapeStartDash))
    );
    assert_eq!(snapshot(&execution), expected);
    expected.state = State::ScriptDataEscapedDashDash;
    expected.current = Some((
        Input::CodePoint('-'.into()),
        Position { line: 1, column: 4 },
    ));
    expected.tokens.push(Token::Character('-'.into()));
    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
    assert_eq!(snapshot(&execution), expected);
    expected.state = State::ScriptDataEscapedLessThanSign;
    expected.current = Some((
        Input::CodePoint('<'.into()),
        Position { line: 1, column: 5 },
    ));
    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
    assert_eq!(snapshot(&execution), expected);
    expected.state = State::ScriptDataDoubleEscapeStart;
    expected.current = Some((
        Input::CodePoint('t'.into()),
        Position { line: 1, column: 6 },
    ));
    expected.retained.temporary_buffer.clear();
    expected.tokens.push(Token::Character('<'.into()));
    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
    assert_eq!(snapshot(&execution), expected);
    let blocked = Err(UnimplementedState {
        state: State::ScriptDataDoubleEscapeStart,
    });
    for _ in 0..3 {
        assert_eq!(execution.step(), blocked);
        assert_eq!(snapshot(&execution), expected);
    }
    execution.context_mut().input.push(points("more")).unwrap();
    execution.context_mut().input.finish();
    assert_eq!(execution.step(), blocked);
    assert_eq!(snapshot(&execution), expected);
}

#[test]
fn real_end_tag_open_entry_suspends_then_creates_or_falls_back_at_exact_boundaries() {
    for suffix in ["A", " ", "\0", ""] {
        for finished in [false, true] {
            let mut context = seeded_context(Some(State::Rcdata));
            context.current_tag = Some(TagKind::Start);
            context.input.push(points("</")).unwrap();
            let mut execution = Execution::new(State::ScriptData, context);
            let mut expected = snapshot(&execution);
            expected.state = State::ScriptDataLessThanSign;
            expected.current = Some((
                Input::CodePoint('<'.into()),
                Position { line: 1, column: 1 },
            ));
            assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&execution), expected);
            expected.state = State::ScriptDataEndTagOpen;
            expected.current = Some((
                Input::CodePoint('/'.into()),
                Position { line: 1, column: 2 },
            ));
            expected.retained.temporary_buffer.clear();
            assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&execution), expected);
            for _ in 0..3 {
                assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                assert_eq!(snapshot(&execution), expected);
            }
            execution.context_mut().input.push(points(suffix)).unwrap();
            if finished {
                execution.context_mut().input.finish();
            } else if suffix.is_empty() {
                assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                assert_eq!(snapshot(&execution), expected);
                execution.context_mut().input.finish();
            }
            expected.current = Some((
                suffix
                    .chars()
                    .next()
                    .map_or(Input::Eof, |c| Input::CodePoint(c.into())),
                Position { line: 1, column: 3 },
            ));
            if suffix == "A" {
                expected.state = State::ScriptDataEndTagName;
                expected.retained.current_tag = Some(TagKind::End);
                expected.retained.end = Some(PendingEndTag {
                    name: vec![],
                    self_closing: false,
                    attributes: vec![],
                    discarded_current_attribute: None,
                });
            } else {
                expected.state = State::ScriptData;
                expected
                    .tokens
                    .extend([Token::Character('<'.into()), Token::Character('/'.into())]);
            }
            assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&execution), expected);
            assert_eq!(
                execution.context_mut().input.reconsume(),
                Err(CursorError::AlreadyReconsuming)
            );
            if suffix == "A" {
                // The name state reconsumes A at the same position, normalizes
                // only the candidate and preserves original spelling in scratch.
                expected
                    .retained
                    .end
                    .as_mut()
                    .unwrap()
                    .name
                    .push('a'.into());
                expected.retained.temporary_buffer.push('A'.into());
                assert_eq!(
                    execution.step(),
                    Ok(Progress::Continue(State::ScriptDataEndTagName))
                );
                assert_eq!(snapshot(&execution), expected);
                if !finished {
                    for _ in 0..3 {
                        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                        assert_eq!(snapshot(&execution), expected);
                    }
                    execution.context_mut().input.finish();
                }
                expected.state = State::ScriptData;
                expected.current = Some((Input::Eof, Position { line: 1, column: 4 }));
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
                continue;
            }
            if !suffix.is_empty() {
                expected.tokens.push(Token::Character(if suffix == "\0" {
                    expected
                        .errors
                        .push(error("unexpected-null-character", 1, 3));
                    '\u{fffd}'.into()
                } else {
                    ' '.into()
                }));
                assert_eq!(execution.step(), Ok(Progress::Continue(State::ScriptData)));
                assert_eq!(snapshot(&execution), expected);
                if !finished {
                    assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                    assert_eq!(snapshot(&execution), expected);
                    execution.context_mut().input.finish();
                }
                expected.current = Some((Input::Eof, Position { line: 1, column: 4 }));
            }
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

#[test]
fn null_replacement_and_exact_normalized_diagnostics_preserve_context() {
    let mut context = seeded_context(Some(State::CharacterReference));
    context.input.push(points("\0\r\n\0")).unwrap();
    context.input.finish();
    let preserved = retained(&context);
    let mut execution = Execution::new(State::ScriptData, context);
    for (consumed, emitted, line, column) in [
        ('\0', '\u{fffd}', 1, 1),
        ('\n', '\n', 1, 2),
        ('\0', '\u{fffd}', 2, 1),
    ] {
        assert_eq!(execution.step(), Ok(Progress::Continue(State::ScriptData)));
        assert_eq!(
            execution.context().input.current(),
            Some((Input::CodePoint(consumed.into()), Position { line, column }))
        );
        assert_eq!(
            execution.context().tokens.last(),
            Some(&Token::Character(emitted.into()))
        );
        assert_eq!(retained(execution.context()), preserved);
    }
    assert_eq!(
        execution.context().errors,
        [
            error("prior-diagnostic", 1, 1),
            error("unexpected-null-character", 1, 1),
            error("unexpected-null-character", 2, 1),
        ]
    );
    assert!(execution.context().input.errors().is_empty());
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(
        execution.context().input.current(),
        Some((Input::Eof, Position { line: 2, column: 2 }))
    );
    assert_eq!(
        execution.context().tokens,
        [
            Token::Character('p'.into()),
            Token::Character('\u{fffd}'.into()),
            Token::Character('\n'.into()),
            Token::Character('\u{fffd}'.into()),
            Token::Eof,
        ]
    );
    assert_eq!(retained(execution.context()), preserved);
    let done = snapshot(&execution);
    for _ in 0..3 {
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&execution), done);
    }
}

#[test]
fn anything_else_including_entity_like_text_and_non_scalars_is_literal() {
    let mut input =
        points("&amp;&#0;&#xD800;&bogus;>/!?=\"'Azé😀\u{0301}\t\u{000c}\u{fffd}\u{feff}");
    let prefix_len = input.len();
    input.extend(
        [0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x0b, 0x7f].map(|value| CodePoint::new(value).unwrap()),
    );
    for return_state in [None, Some(State::Rcdata), Some(State::ScriptData)] {
        let mut context = seeded_context(return_state);
        context.input.push(input.iter().copied()).unwrap();
        context.input.finish();
        let preserved = retained(&context);
        let mut execution = Execution::new(State::ScriptData, context);
        for (i, &point) in input.iter().enumerate() {
            assert_eq!(execution.step(), Ok(Progress::Continue(State::ScriptData)));
            assert_eq!(
                execution.context().input.current(),
                Some((
                    Input::CodePoint(point),
                    Position {
                        line: 1,
                        column: i + 1
                    }
                ))
            );
            assert_eq!(execution.context().tokens.len(), i + 2);
            assert_eq!(execution.context().tokens[i + 1], Token::Character(point));
            assert_eq!(
                execution.context().errors,
                [error("prior-diagnostic", 1, 1)]
            );
            assert_eq!(retained(execution.context()), preserved);
        }
        assert_eq!(
            execution.context().input.errors(),
            [
                error("surrogate-in-input-stream", 1, prefix_len + 1),
                error("surrogate-in-input-stream", 1, prefix_len + 2),
                error("noncharacter-in-input-stream", 1, prefix_len + 3),
                error("noncharacter-in-input-stream", 1, prefix_len + 4),
                error("control-character-in-input-stream", 1, prefix_len + 5),
                error("control-character-in-input-stream", 1, prefix_len + 6),
            ]
        );
        assert_eq!(execution.step(), Ok(Progress::Finished));
        let expected: Vec<_> = [Token::Character('p'.into())]
            .into_iter()
            .chain(input.iter().copied().map(Token::Character))
            .chain([Token::Eof])
            .collect();
        assert_eq!(execution.context().tokens, expected);
        assert_eq!(
            execution.context().input.position(),
            Position {
                line: 1,
                column: input.len() + 1
            }
        );
        assert_eq!(retained(execution.context()), preserved);
    }
}

#[test]
fn empty_open_input_suspends_and_only_explicit_eof_finishes_once_without_creation() {
    let mut execution = Execution::new(State::ScriptData, Context::default());
    let empty = snapshot(&execution);
    let defaults = retained(execution.context());
    for _ in 0..3 {
        execution.context_mut().input.push([]).unwrap();
        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
        assert_eq!(snapshot(&execution), empty);
    }
    execution.context_mut().input.finish();
    execution.context_mut().input.finish();
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(execution.context().tokens, [Token::Eof]);
    assert_eq!(
        execution.context().input.current(),
        Some((Input::Eof, Position { line: 1, column: 1 }))
    );
    assert_eq!(retained(execution.context()), defaults);
    assert!(execution.context().errors.is_empty());
    assert!(execution.context().input.errors().is_empty());
    assert_eq!(
        execution.context_mut().input.push([]),
        Err(CursorError::InputAlreadyFinished)
    );
    let completed = snapshot(&execution);
    for _ in 0..3 {
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&execution), completed);
    }
    let mut context = Context::default();
    context.input.finish();
    let mut closed = Execution::new(State::ScriptData, context);
    assert_eq!(closed.step(), Ok(Progress::Finished));
    assert_eq!(snapshot(&closed), completed);
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
        let terminal = !matches!(result, Ok(Progress::Continue(_)));
        trace.push((result, snapshot(execution)));
        if terminal {
            let done = snapshot(execution);
            assert_eq!(execution.step(), result);
            assert_eq!(snapshot(execution), done);
            return true;
        }
    }
    panic!("Script data failed to consume or terminate");
}

#[test]
fn every_split_and_single_point_chunking_matches_closed_source_step_traces() {
    let mut mixed = points("&amp;\r\n\0é😀\r\r\nb\r");
    mixed.extend([0xd800, 0xdfff, 0xfdd0, 0x01].map(|v| CodePoint::new(v).unwrap()));
    let mut blocked = mixed.clone();
    blocked.extend(points("</script>unconsumed\0"));
    for input in [
        vec![],
        mixed,
        blocked,
        points("\r"),
        points("\r\n"),
        points("\r\r"),
        points("\0\r"),
        points("&"),
        points("<"),
        points("&<"),
        points("\r\n<\r\n"),
        points("\r<\0"),
        points("<!"),
        points("</"),
        points("<\r"),
    ] {
        let run = |chunks: Vec<&[CodePoint]>, closed: bool| {
            let mut execution =
                Execution::new(State::ScriptData, seeded_context(Some(State::Rcdata)));
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
                    if drain(&mut execution, &mut trace, input.len() * 3 + 4) {
                        // A terminal stub is never fed, reconstructed or resumed.
                        return trace;
                    }
                    execution.context_mut().input.push([]).unwrap();
                    assert!(!drain(&mut execution, &mut trace, 1));
                }
                execution.context_mut().input.finish();
            }
            assert!(drain(&mut execution, &mut trace, input.len() * 3 + 4));
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
