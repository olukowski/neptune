//! Branch evidence for the pinned HTML Standard's "RAWTEXT state" (#rawtext-state):
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>
//! These focused tests are not upstream conformance counts.

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
    Context {
        return_state,
        last_start_tag: Some(points("style")),
        current_tag: Some(TagKind::End),
        pending_start_tag: Some(PendingStartTag {
            name: points("pending-start"),
            self_closing: true,
            attributes: vec![Attribute {
                name: points("name"),
                value: points("value"),
            }],
            discarded_current_attribute: Some(Attribute {
                name: points("name"),
                value: points("discarded-value"),
            }),
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("pending-end"),
            ..Default::default()
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

// Every unrelated context field is observable, including selected/pending tags.
#[derive(Debug, Eq, PartialEq)]
struct Retained {
    return_state: Option<State>,
    last_start_tag: Option<Vec<CodePoint>>,
    current_tag: Option<TagKind>,
    pending_start_tag: Option<PendingStartTag>,
    pending_end_tag: Option<PendingEndTag>,
    pending_comment: Option<PendingComment>,
    temporary_buffer: Vec<CodePoint>,
}

fn retained(context: &Context) -> Retained {
    Retained {
        return_state: context.return_state,
        last_start_tag: context.last_start_tag.clone(),
        current_tag: context.current_tag,
        pending_start_tag: context.pending_start_tag.clone(),
        pending_end_tag: context.pending_end_tag.clone(),
        pending_comment: context.pending_comment.clone(),
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
fn rawtext_less_than_consumes_once_without_output_error_or_reconsumption() {
    for finished in [false, true] {
        for suffix in [
            vec![],
            points("/style>"),
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
            let mut execution = Execution::new(State::Rawtext, context);
            assert_eq!(
                execution.step(),
                Ok(Progress::Continue(State::RawtextLessThanSign))
            );
            assert_eq!(execution.state(), State::RawtextLessThanSign);
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
            // Inspect the source boundary only. Destination behavior is covered in
            // rawtext_less_than_sign_state.rs; RAWTEXT neither consumed the tail
            // nor reconsumed '<'.
            let next = suffix.first().copied().map_or(
                if finished {
                    Input::Eof
                } else {
                    Input::NeedMoreInput
                },
                Input::CodePoint,
            );
            assert_eq!(execution.context_mut().input.consume(), next);
            assert_eq!(execution.context().tokens, tokens);
        }
    }
}

#[test]
fn rawtext_null_reports_at_consumed_location_then_emits_replacement_not_current_null() {
    let mut context = seeded_context(Some(State::CharacterReference));
    context.input.push(points("\0\r\n\0")).unwrap();
    context.input.finish();
    let preserved = retained(&context);
    let mut execution = Execution::new(State::Rawtext, context);
    for (consumed, emitted, line, column, count) in [
        ('\0', '\u{fffd}', 1, 1, 2),
        ('\n', '\n', 1, 2, 2),
        ('\0', '\u{fffd}', 2, 1, 3),
    ] {
        let before = execution.context().tokens.len();
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Rawtext)));
        assert_eq!(
            execution.context().input.current(),
            Some((Input::CodePoint(consumed.into()), Position { line, column }))
        );
        assert_eq!(execution.context().tokens.len(), before + 1);
        assert_eq!(
            execution.context().tokens[before],
            Token::Character(emitted.into())
        );
        assert_eq!(execution.context().errors.len(), count);
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
    let completed = snapshot(&execution);
    for _ in 0..3 {
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&execution), completed);
    }
}

#[test]
fn rawtext_anything_else_is_literal_including_ampersands_and_non_scalar_input() {
    let mut input = points("&amp;&#0;&foo;>/!?=\"'é😀\u{0301}\t\u{000c}\u{fffd}\u{feff}");
    let prefix_len = input.len();
    input
        .extend([0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x0b, 0x7f].map(|v| CodePoint::new(v).unwrap()));
    for return_state in [None, Some(State::Rcdata), Some(State::Rawtext)] {
        let mut context = seeded_context(return_state);
        context.input.push(input.iter().copied()).unwrap();
        context.input.finish();
        let preserved = retained(&context);
        let mut execution = Execution::new(State::Rawtext, context);
        for (i, &point) in input.iter().enumerate() {
            assert_eq!(execution.step(), Ok(Progress::Continue(State::Rawtext)));
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
fn rawtext_empty_input_suspends_repeatedly_and_only_explicit_eof_finishes_once() {
    let mut execution = Execution::new(State::Rawtext, Context::default());
    let empty = snapshot(&execution);
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
    assert!(execution.context().errors.is_empty());
    assert!(execution.context().input.errors().is_empty());
    assert_eq!(execution.context().return_state, None);
    let completed = snapshot(&execution);
    assert_eq!(
        execution.context_mut().input.push([]),
        Err(CursorError::InputAlreadyFinished)
    );
    for _ in 0..3 {
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&execution), completed);
    }
    let mut context = Context::default();
    context.input.finish();
    let mut closed = Execution::new(State::Rawtext, context);
    assert_eq!(closed.step(), Ok(Progress::Finished));
    assert_eq!(snapshot(&closed), completed);
}

type Trace = Vec<(Result<Progress, UnimplementedState>, Snapshot)>;

// Record every source step and terminal result; suspension must be effect-free.
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
            let completed = snapshot(execution);
            for _ in 0..2 {
                assert_eq!(execution.step(), result);
                assert_eq!(snapshot(execution), completed);
            }
            return true;
        }
    }
    panic!("RAWTEXT failed to consume or terminate");
}

#[test]
fn rawtext_all_two_chunk_splits_and_single_points_preserve_source_step_traces() {
    let mut mixed = points("&amp;\r\n\0é😀\r\r\nb\r");
    mixed.extend([0xd800, 0xdfff, 0xfdd0, 0x01].map(|v| CodePoint::new(v).unwrap()));
    let mut blocked = mixed.clone();
    blocked.extend(points("</style>unconsumed\0"));
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
        points("<\r"),
    ] {
        let run = |chunks: Vec<&[CodePoint]>, finish_first: bool| {
            let mut execution = Execution::new(State::Rawtext, seeded_context(Some(State::Rcdata)));
            let mut trace = Vec::new();
            if finish_first {
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
                        // A stub is terminal, never feed/reconstruct/resume it.
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
