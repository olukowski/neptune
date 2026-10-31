//! Branch evidence for the pinned HTML Standard's "PLAINTEXT state" (#plaintext-state):
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>
//! These focused tests are not upstream conformance counts.

use neptune_html::tokenizer::{
    CodePoint, Context, CursorError, Execution, Input, ParseError, PendingStartTag, Position,
    Progress, State, Token,
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
fn plaintext_null_replaces_output_not_current_input_and_reports_each_location() {
    let mut context = Context {
        pending_start_tag: Some(PendingStartTag {
            name: points("pending"),
            ..PendingStartTag::default()
        }),
        temporary_buffer: points("scratch"),
        return_state: Some(State::Rcdata),
        last_start_tag: Some(points("p")),
        tokens: vec![Token::Character('p'.into())],
        errors: vec![error("prior-diagnostic", 1, 1)],
        ..Context::default()
    };
    context.input.push(points("\0\r\n\0")).unwrap();
    context.input.finish();
    let mut execution = Execution::new(State::Plaintext, context);
    for (consumed, emitted, position, errors) in [
        ('\0', '\u{fffd}', Position { line: 1, column: 1 }, 2),
        ('\n', '\n', Position { line: 1, column: 2 }, 2),
        ('\0', '\u{fffd}', Position { line: 2, column: 1 }, 3),
    ] {
        let before = execution.context().tokens.len();
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Plaintext)));
        assert_eq!(execution.state(), State::Plaintext);
        assert_eq!(execution.context().tokens.len(), before + 1);
        assert_eq!(
            execution.context().tokens[before],
            Token::Character(emitted.into())
        );
        assert_eq!(
            execution.context().input.current(),
            Some((Input::CodePoint(consumed.into()), position))
        );
        assert_eq!(execution.context().errors.len(), errors);
        assert_eq!(execution.context().return_state, Some(State::Rcdata));
        assert_eq!(execution.context().last_start_tag, Some(points("p")));
        assert_eq!(
            execution.context().pending_start_tag,
            Some(PendingStartTag {
                name: points("pending"),
                ..PendingStartTag::default()
            })
        );
        assert_eq!(execution.context().temporary_buffer, points("scratch"));
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
    let completed = snapshot(&execution);
    for _ in 0..2 {
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&execution), completed);
    }
}

#[test]
fn plaintext_anything_else_including_markup_and_non_scalar_points_is_literal() {
    let mut input = points("<plaintext>&amp;</plaintext>é😀\u{0301}\t\u{000c}\u{fffd}");
    let prefix_len = input.len();
    input.extend([0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x0b].map(|v| CodePoint::new(v).unwrap()));
    let mut context = Context {
        return_state: Some(State::Rcdata),
        last_start_tag: Some(points("title")),
        ..Context::default()
    };
    context.input.push(input.iter().copied()).unwrap();
    context.input.finish();
    let mut execution = Execution::new(State::Plaintext, context);
    for (i, &point) in input.iter().enumerate() {
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Plaintext)));
        assert_eq!(execution.state(), State::Plaintext);
        assert_eq!(execution.context().tokens.len(), i + 1);
        assert_eq!(execution.context().tokens[i], Token::Character(point));
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
        assert!(execution.context().errors.is_empty());
        assert_eq!(execution.context().return_state, Some(State::Rcdata));
        assert_eq!(execution.context().last_start_tag, Some(points("title")));
    }
    assert_eq!(
        execution.context().input.errors(),
        [
            error("surrogate-in-input-stream", 1, prefix_len + 1),
            error("surrogate-in-input-stream", 1, prefix_len + 2),
            error("noncharacter-in-input-stream", 1, prefix_len + 3),
            error("noncharacter-in-input-stream", 1, prefix_len + 4),
            error("control-character-in-input-stream", 1, prefix_len + 5),
        ]
    );
    assert_eq!(execution.step(), Ok(Progress::Finished));
    let mut expected: Vec<_> = input.iter().copied().map(Token::Character).collect();
    expected.push(Token::Eof);
    assert_eq!(execution.context().tokens, expected);
    assert_eq!(
        execution.context().input.position(),
        Position {
            line: 1,
            column: input.len() + 1
        }
    );
    let completed = snapshot(&execution);
    // Even mutating the retained cursor cannot cause terminal execution to emit again.
    execution.context_mut().input.reconsume().unwrap();
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(snapshot(&execution), completed);
}

#[test]
fn plaintext_empty_open_input_suspends_until_explicit_eof_and_finishes_once() {
    let mut execution = Execution::new(State::Plaintext, Context::default());
    let empty = snapshot(&execution);
    for _ in 0..2 {
        execution.context_mut().input.push([]).unwrap();
        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
        assert_eq!(snapshot(&execution), empty);
    }
    execution.context_mut().input.finish();
    execution.context_mut().input.finish();
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(execution.state(), State::Plaintext);
    assert_eq!(execution.context().tokens, [Token::Eof]);
    assert_eq!(
        execution.context().input.current(),
        Some((Input::Eof, Position { line: 1, column: 1 }))
    );
    assert!(execution.context().errors.is_empty());
    assert!(execution.context().input.errors().is_empty());
    let completed = snapshot(&execution);
    assert_eq!(
        execution.context_mut().input.push([]),
        Err(CursorError::InputAlreadyFinished)
    );
    for _ in 0..2 {
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&execution), completed);
    }
    let mut context = Context::default();
    context.input.finish();
    let mut closed = Execution::new(State::Plaintext, context);
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
    pending_start_tag: Option<PendingStartTag>,
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
        pending_start_tag: context.pending_start_tag.clone(),
        temporary_buffer: context.temporary_buffer.clone(),
    }
}

// Capture every source step, not just final output, excluding suspension.
fn drain(execution: &mut Execution, trace: &mut Vec<(Progress, Snapshot)>, bound: usize) -> bool {
    for _ in 0..bound {
        let before = snapshot(execution);
        let result = execution.step().unwrap();
        if result == Progress::NeedMoreInput {
            assert_eq!(snapshot(execution), before);
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(execution), before);
            return false;
        }
        trace.push((result, snapshot(execution)));
        if result == Progress::Finished {
            let completed = snapshot(execution);
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(execution), completed);
            return true;
        }
    }
    panic!("PLAINTEXT failed to consume or terminate");
}

#[test]
fn plaintext_chunk_boundaries_preserve_every_step_token_diagnostic_and_position() {
    let mut mixed = points("<plaintext>&amp;\r\n\0é😀\r\r\nb\r");
    mixed.extend([0xd800, 0xdfff, 0xfdd0, 0x01].map(|v| CodePoint::new(v).unwrap()));
    for input in [
        vec![],
        mixed,
        points("\r"),
        points("\r\n"),
        points("\r\r"),
        points("\0\r"),
        points("<&"),
    ] {
        let run = |chunks: Vec<&[CodePoint]>, finish_first: bool| {
            let mut execution = Execution::new(State::Plaintext, Context::default());
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
                    assert!(!drain(&mut execution, &mut trace, input.len() + 2));
                    execution.context_mut().input.push([]).unwrap();
                    assert!(!drain(&mut execution, &mut trace, 1));
                }
                execution.context_mut().input.finish();
            }
            assert!(drain(&mut execution, &mut trace, input.len() + 2));
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
