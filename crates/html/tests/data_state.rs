//! Branch evidence for the pinned HTML Standard's "Data state" (#data-state):
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>
//! These focused tests are not upstream conformance counts.

use neptune_html::tokenizer::{
    CodePoint, Context, Execution, Input, ParseError, Position, Progress, State, Token,
    UnimplementedState,
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

#[test]
fn data_ampersand_and_less_than_complete_source_effects_before_destination_blocks() {
    for (point, destination, return_state) in [
        ('&', State::CharacterReference, Some(State::Data)),
        ('<', State::TagOpen, Some(State::Rcdata)),
    ] {
        // Existing context must survive. Ampersand overwrites the return state;
        // less-than must not change it or the last emitted start tag.
        let tail = CodePoint::new(0xd800).unwrap();
        // The source transition is the same with open input or final input.
        for finished in [false, true] {
            let mut context = Context {
                return_state: Some(State::Rcdata),
                last_start_tag: Some(points("title")),
                tokens: vec![Token::Character('p'.into())],
                errors: vec![error("prior-diagnostic", 1, 1)],
                ..Context::default()
            };
            context.input.push([point.into(), tail]).unwrap();
            if finished {
                context.input.finish();
            }
            let mut execution = Execution::new(State::Data, context);
            assert_eq!(execution.step(), Ok(Progress::Continue(destination)));
            assert_eq!(execution.state(), destination);
            assert_eq!(execution.context().return_state, return_state);
            assert_eq!(execution.context().last_start_tag, Some(points("title")));
            assert_eq!(execution.context().tokens, [Token::Character('p'.into())]);
            assert_eq!(
                execution.context().errors,
                [error("prior-diagnostic", 1, 1)]
            );
            assert!(execution.context().input.errors().is_empty());
            assert_eq!(
                execution.context().input.current(),
                Some((
                    Input::CodePoint(point.into()),
                    Position { line: 1, column: 1 }
                ))
            );
            let source_effects = snapshot(&execution);
            for _ in 0..2 {
                assert_eq!(
                    execution.step(),
                    Err(UnimplementedState { state: destination })
                );
                assert_eq!(snapshot(&execution), source_effects);
            }
            // Inspection only, not an attempt to resume blocked execution:
            // neither source nor stub consumed the tail, nor scheduled reconsume.
            assert_eq!(
                execution.context_mut().input.consume(),
                Input::CodePoint(tail)
            );
            assert_eq!(
                execution.context().input.position(),
                Position { line: 1, column: 2 }
            );
        }
    }
}

#[test]
fn data_null_is_emitted_unchanged_with_one_state_error_per_occurrence() {
    let mut context = Context {
        return_state: Some(State::Rcdata),
        last_start_tag: Some(points("p")),
        ..Context::default()
    };
    context.input.push(points("\0\r\n\0")).unwrap();
    context.input.finish();
    let mut execution = Execution::new(State::Data, context);
    for (point, position, errors) in [
        ('\0', Position { line: 1, column: 1 }, 1),
        ('\n', Position { line: 1, column: 2 }, 1),
        ('\0', Position { line: 2, column: 1 }, 2),
    ] {
        let before = execution.context().tokens.len();
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
        assert_eq!(execution.state(), State::Data);
        assert_eq!(execution.context().tokens.len(), before + 1);
        assert_eq!(
            execution.context().tokens[before],
            Token::Character(point.into())
        );
        assert_eq!(
            execution.context().input.current(),
            Some((Input::CodePoint(point.into()), position))
        );
        assert_eq!(execution.context().errors.len(), errors);
        assert_eq!(execution.context().return_state, Some(State::Rcdata));
        assert_eq!(execution.context().last_start_tag, Some(points("p")));
    }
    assert_eq!(
        execution.context().errors,
        [
            error("unexpected-null-character", 1, 1),
            error("unexpected-null-character", 2, 1),
        ]
    );
    assert!(execution.context().input.errors().is_empty());
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(
        execution.context().tokens,
        [
            Token::Character('\0'.into()),
            Token::Character('\n'.into()),
            Token::Character('\0'.into()),
            Token::Eof,
        ]
    );
}

#[test]
fn data_anything_else_preserves_code_points_and_preprocessing_diagnostics() {
    // Surrogates are actual CodePoints, not scalar strings or U+FFFD substitutes.
    let input: Vec<_> = [
        0x61, 0xe9, 0x1f600, 0x0301, 0xd800, 0xdfff, 0x10ffff, 0x0b, 0x09, 0x0c,
    ]
    .map(|value| CodePoint::new(value).unwrap())
    .into();
    let mut context = Context {
        return_state: Some(State::Rcdata),
        last_start_tag: Some(points("p")),
        ..Context::default()
    };
    context.input.push(input.iter().copied()).unwrap();
    context.input.finish();
    let mut execution = Execution::new(State::Data, context);
    for (i, &point) in input.iter().enumerate() {
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
        assert_eq!(execution.state(), State::Data);
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
        assert_eq!(execution.context().tokens.len(), i + 1);
        assert_eq!(execution.context().tokens[i], Token::Character(point));
        assert!(execution.context().errors.is_empty());
        assert_eq!(execution.context().return_state, Some(State::Rcdata));
        assert_eq!(execution.context().last_start_tag, Some(points("p")));
    }
    assert_eq!(
        execution.context().input.errors(),
        [
            error("surrogate-in-input-stream", 1, 5),
            error("surrogate-in-input-stream", 1, 6),
            error("noncharacter-in-input-stream", 1, 7),
            error("control-character-in-input-stream", 1, 8),
        ]
    );
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(
        execution.context().input.current(),
        Some((
            Input::Eof,
            Position {
                line: 1,
                column: 11
            }
        ))
    );
    let mut expected: Vec<_> = input.into_iter().map(Token::Character).collect();
    expected.push(Token::Eof);
    assert_eq!(execution.context().tokens, expected);
    assert!(execution.context().errors.is_empty());
    assert_eq!(execution.context().return_state, Some(State::Rcdata));
    assert_eq!(execution.context().last_start_tag, Some(points("p")));
    let completed = snapshot(&execution);
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(snapshot(&execution), completed);
}

#[test]
fn data_empty_open_input_suspends_and_only_final_input_emits_terminal_eof() {
    let mut execution = Execution::new(State::Data, Context::default());
    assert_eq!(execution.context().return_state, None);
    for _ in 0..2 {
        execution.context_mut().input.push([]).unwrap();
        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
        assert_eq!(execution.state(), State::Data);
        assert_eq!(execution.context().input.current(), None);
        assert!(execution.context().tokens.is_empty());
        assert!(execution.context().errors.is_empty());
        assert!(execution.context().input.errors().is_empty());
    }
    execution.context_mut().input.finish();
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(execution.state(), State::Data);
    assert_eq!(execution.context().tokens, [Token::Eof]);
    assert_eq!(
        execution.context().input.current(),
        Some((Input::Eof, Position { line: 1, column: 1 }))
    );
    assert_eq!(execution.context().return_state, None);
    let completed = snapshot(&execution);
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(snapshot(&execution), completed);

    // Closed from the outset is equivalent; empty input must really execute Data.
    let mut context = Context::default();
    context.input.finish();
    let mut closed = Execution::new(State::Data, context);
    assert_eq!(closed.step(), Ok(Progress::Finished));
    assert_eq!(snapshot(&closed), completed);
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
    }
}

// Record every completed source step and terminal result, excluding suspension.
// A bound also makes accidentally non-consuming Data loops fail promptly.
fn drain(
    execution: &mut Execution,
    trace: &mut Vec<(Result<Progress, UnimplementedState>, Snapshot)>,
    bound: usize,
) -> bool {
    for _ in 0..bound {
        let before = snapshot(execution);
        let result = execution.step();
        if result == Ok(Progress::NeedMoreInput) {
            let suspended = snapshot(execution);
            assert_eq!(suspended, before);
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(execution), suspended);
            return false;
        }
        let terminal = !matches!(result, Ok(Progress::Continue(_)));
        trace.push((result, snapshot(execution)));
        if terminal {
            return true;
        }
    }
    panic!("Data failed to consume or terminate");
}

#[test]
fn data_chunk_boundaries_preserve_all_source_steps_outputs_diagnostics_and_transitions() {
    let mut ordinary = points("a\r\n\0é😀\r\r\nb\r");
    ordinary.extend([0xd800, 0xdfff, 0xfdd0, 0x01].map(|value| CodePoint::new(value).unwrap()));
    let mut samples = vec![
        vec![],
        ordinary.clone(),
        points("\r"),
        points("\r\n"),
        points("\r\r"),
        points("\0\r"),
    ];
    for transition in ['&', '<'] {
        let mut sample = ordinary.clone();
        sample.push(transition.into());
        samples.push(sample);
        samples.push(points(&transition.to_string()));
    }
    for input in samples {
        let run = |chunks: Vec<&[CodePoint]>, finish_first: bool| {
            let mut execution = Execution::new(State::Data, Context::default());
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
                    if drain(&mut execution, &mut trace, input.len() + 3) {
                        return trace;
                    }
                    // Empty chunks cannot turn exhaustion into EOF.
                    execution.context_mut().input.push([]).unwrap();
                    assert!(!drain(&mut execution, &mut trace, 1));
                }
                execution.context_mut().input.finish();
            }
            assert!(drain(&mut execution, &mut trace, input.len() + 3));
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
