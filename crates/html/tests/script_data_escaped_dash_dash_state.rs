//! State-local evidence for pinned HTML #script-data-escaped-dash-dash-state.
//! See docs/tokenizer-script-data-escaped-dash-dash.md; these are not upstream runs.

use neptune_html::tokenizer::{
    Attribute, CodePoint, Context, Execution, Input, ParseError, PendingComment, PendingEndTag,
    PendingStartTag, Position, Progress, State, TagKind, Token, UnimplementedState,
};

const SOURCE: State = State::ScriptDataEscapedDashDash;

fn points(text: &str) -> Vec<CodePoint> {
    text.chars().map(Into::into).collect()
}

fn error(code: &'static str, line: usize, column: usize) -> ParseError {
    ParseError {
        code,
        position: Position { line, column },
    }
}

fn seeded() -> Context {
    let attribute = Attribute {
        name: points("a"),
        value: points("retained"),
    };
    let discarded = Attribute {
        name: points("a"),
        value: points("discarded"),
    };
    Context {
        tokens: vec![Token::Character('p'.into())],
        errors: vec![error("prior-diagnostic", 1, 1)],
        return_state: Some(State::Rcdata),
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
    buffer: Vec<CodePoint>,
}

fn retained(context: &Context) -> Retained {
    Retained {
        return_state: context.return_state,
        last_start_tag: context.last_start_tag.clone(),
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
fn every_ascii_branch_consumes_once_with_exact_source_effects_and_no_reconsumption() {
    for value in 0..=0x7f {
        for finished in [false, true] {
            let point = CodePoint::new(value).unwrap();
            let mut context = seeded();
            context.input.push([point, 'z'.into()]).unwrap();
            if finished {
                context.input.finish();
            }
            let preserved = retained(&context);
            let normalized = if value == 0x0d { '\n'.into() } else { point };
            let (target, output) = match value {
                0x2d => (SOURCE, Some(point)),
                0x3c => (State::ScriptDataEscapedLessThanSign, None),
                0x3e => (State::ScriptData, Some(point)),
                0 => (State::ScriptDataEscaped, Some('\u{fffd}'.into())),
                _ => (State::ScriptDataEscaped, Some(normalized)),
            };
            let mut execution = Execution::new(SOURCE, context);
            assert_eq!(
                execution.step(),
                Ok(Progress::Continue(target)),
                "{value:x}"
            );
            assert_eq!(retained(execution.context()), preserved);
            let expected: Vec<_> = [Token::Character('p'.into())]
                .into_iter()
                .chain(output.map(Token::Character))
                .collect();
            assert_eq!(execution.context().tokens, expected);
            let mut errors = vec![error("prior-diagnostic", 1, 1)];
            if value == 0 {
                errors.push(error("unexpected-null-character", 1, 1));
            }
            assert_eq!(execution.context().errors, errors);
            let preprocessing = if matches!(value, 1..=8 | 0x0b | 0x0e..=0x1f | 0x7f) {
                vec![error("control-character-in-input-stream", 1, 1)]
            } else {
                vec![]
            };
            assert_eq!(execution.context().input.errors(), preprocessing);
            assert_eq!(
                execution.context().input.current(),
                Some((
                    Input::CodePoint(normalized),
                    Position { line: 1, column: 1 }
                ))
            );
            // Inspect only the source boundary: the next raw point is untouched,
            // and no branch scheduled reconsumption, including NUL and '<'.
            assert_eq!(
                execution.context_mut().input.consume(),
                Input::CodePoint('z'.into())
            );
            assert_eq!(execution.context().input.errors(), preprocessing);
        }
    }
}

#[test]
fn unicode_non_scalars_and_preprocessing_are_preserved_not_replaced() {
    for (value, diagnostic) in [
        (0x85, Some("control-character-in-input-stream")),
        (0xa0, None),
        (0xc9, None),
        (0x2014, None),
        (0xd800, Some("surrogate-in-input-stream")),
        (0xdfff, Some("surrogate-in-input-stream")),
        (0xfdd0, Some("noncharacter-in-input-stream")),
        (0xffff, Some("noncharacter-in-input-stream")),
        (0x10ffff, Some("noncharacter-in-input-stream")),
        (0xfeff, None),
        (0xfffd, None),
        (0x1f600, None),
    ] {
        let point = CodePoint::new(value).unwrap();
        let mut context = seeded();
        context.input.push(points("--")).unwrap();
        context.input.push([point, '\0'.into()]).unwrap();
        context.input.finish();
        let preserved = retained(&context);
        let mut execution = Execution::new(SOURCE, context);
        for _ in 0..2 {
            assert_eq!(execution.step(), Ok(Progress::Continue(SOURCE)));
        }
        assert_eq!(
            execution.step(),
            Ok(Progress::Continue(State::ScriptDataEscaped))
        );
        assert_eq!(
            execution.context().tokens,
            [
                Token::Character('p'.into()),
                Token::Character('-'.into()),
                Token::Character('-'.into()),
                Token::Character(point),
            ]
        );
        assert_eq!(
            execution.context().errors,
            [error("prior-diagnostic", 1, 1)]
        );
        let expected: Vec<_> = diagnostic
            .into_iter()
            .map(|code| error(code, 1, 3))
            .collect();
        assert_eq!(execution.context().input.errors(), expected);
        assert_eq!(
            execution.context().input.position(),
            Position { line: 1, column: 3 }
        );
        assert_eq!(retained(execution.context()), preserved);
        // The real Escaped destination consumes the untouched NUL exactly once.
        let mut expected = snapshot(&execution);
        expected.current = Some((
            Input::CodePoint('\0'.into()),
            Position { line: 1, column: 4 },
        ));
        expected.tokens.push(Token::Character('\u{fffd}'.into()));
        expected
            .errors
            .push(error("unexpected-null-character", 1, 4));
        assert_eq!(
            execution.step(),
            Ok(Progress::Continue(State::ScriptDataEscaped))
        );
        assert_eq!(snapshot(&execution), expected);
        expected.current = Some((Input::Eof, Position { line: 1, column: 5 }));
        expected.tokens.push(Token::Eof);
        expected
            .errors
            .push(error("eof-in-script-html-comment-like-text", 1, 5));
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&execution), expected);
        for _ in 0..3 {
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&execution), expected);
        }
    }
}

#[test]
fn destinations_execute_or_reach_terminal_double_escape_start() {
    for (text, target, output) in [
        ("--\0tail", State::ScriptDataEscaped, Some('\u{fffd}')),
        ("--<tail", State::ScriptDataEscapedLessThanSign, None),
        ("--&amp;", State::ScriptDataEscaped, Some('&')),
    ] {
        let mut context = Context::default();
        context.input.push(points(text)).unwrap();
        context.input.finish();
        let mut execution = Execution::new(SOURCE, context);
        for _ in 0..2 {
            assert_eq!(execution.step(), Ok(Progress::Continue(SOURCE)));
        }
        assert_eq!(execution.step(), Ok(Progress::Continue(target)));
        let expected: Vec<_> = points("--")
            .into_iter()
            .chain(output.map(Into::into))
            .map(Token::Character)
            .collect();
        assert_eq!(execution.context().tokens, expected);
        let errors = if output == Some('\u{fffd}') {
            vec![error("unexpected-null-character", 1, 3)]
        } else {
            vec![]
        };
        assert_eq!(execution.context().errors, errors);
        assert!(execution.context().input.errors().is_empty());
        let mut expected = snapshot(&execution);
        if text == "--<tail" {
            expected.state = State::ScriptDataDoubleEscapeStart;
            expected.current = Some((
                Input::CodePoint('t'.into()),
                Position { line: 1, column: 4 },
            ));
            expected.tokens.push(Token::Character('<'.into()));
            assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&execution), expected);
            for _ in 0..3 {
                assert_eq!(
                    execution.step(),
                    Err(UnimplementedState {
                        state: State::ScriptDataDoubleEscapeStart
                    })
                );
                assert_eq!(snapshot(&execution), expected);
            }
            continue;
        }
        // NUL and '&' have consumed their source point, not scheduled reconsumption.
        for (index, point) in points(text).into_iter().enumerate().skip(3) {
            expected.current = Some((
                Input::CodePoint(point),
                Position {
                    line: 1,
                    column: index + 1,
                },
            ));
            expected.tokens.push(Token::Character(point));
            assert_eq!(
                execution.step(),
                Ok(Progress::Continue(State::ScriptDataEscaped))
            );
            assert_eq!(snapshot(&execution), expected);
        }
        expected.current = Some((Input::Eof, Position { line: 1, column: 8 }));
        expected.tokens.push(Token::Eof);
        expected
            .errors
            .push(error("eof-in-script-html-comment-like-text", 1, 8));
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&execution), expected);
    }
}

#[test]
fn open_exhaustion_suspends_but_eof_reports_once_and_finishes_without_state_change() {
    for prefix in ["", "-", "---"] {
        let mut context = seeded();
        context.input.push(points(prefix)).unwrap();
        let preserved = retained(&context);
        let mut execution = Execution::new(SOURCE, context);
        for _ in prefix.chars() {
            assert_eq!(execution.step(), Ok(Progress::Continue(SOURCE)));
        }
        let before = snapshot(&execution);
        for _ in 0..3 {
            execution.context_mut().input.push([]).unwrap();
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&execution), before);
        }
        execution.context_mut().input.finish();
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(execution.state(), SOURCE);
        assert_eq!(
            execution.context().errors,
            [
                error("prior-diagnostic", 1, 1),
                error("eof-in-script-html-comment-like-text", 1, prefix.len() + 1)
            ]
        );
        let expected: Vec<_> = [Token::Character('p'.into())]
            .into_iter()
            .chain(points(prefix).into_iter().map(Token::Character))
            .chain([Token::Eof])
            .collect();
        assert_eq!(execution.context().tokens, expected);
        assert_eq!(
            execution.context().input.current(),
            Some((
                Input::Eof,
                Position {
                    line: 1,
                    column: prefix.len() + 1
                }
            ))
        );
        assert_eq!(retained(execution.context()), preserved);
        let done = snapshot(&execution);
        for _ in 0..3 {
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&execution), done);
        }
    }
}

#[test]
fn assigned_null_and_eof_diagnostics_use_normalized_code_point_positions() {
    for suffix in ["-\0", "--"] {
        let mut context = Context::default();
        context.input.push(points("é😀\r\n")).unwrap();
        for point in points("é😀\n") {
            assert_eq!(context.input.consume(), Input::CodePoint(point));
        }
        context.input.push(points(suffix)).unwrap();
        context.input.finish();
        let mut execution = Execution::new(SOURCE, context);
        assert_eq!(execution.step(), Ok(Progress::Continue(SOURCE)));
        let (code, column) = if suffix.ends_with('\0') {
            assert_eq!(
                execution.step(),
                Ok(Progress::Continue(State::ScriptDataEscaped))
            );
            assert_eq!(
                execution.context().tokens,
                [
                    Token::Character('-'.into()),
                    Token::Character('\u{fffd}'.into())
                ]
            );
            ("unexpected-null-character", 2)
        } else {
            assert_eq!(execution.step(), Ok(Progress::Continue(SOURCE)));
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(
                execution.context().tokens,
                [
                    Token::Character('-'.into()),
                    Token::Character('-'.into()),
                    Token::Eof
                ]
            );
            ("eof-in-script-html-comment-like-text", 3)
        };
        assert_eq!(execution.context().errors, [error(code, 2, column)]);
        assert!(execution.context().input.errors().is_empty());
        assert_eq!(
            execution.context().input.position(),
            Position { line: 2, column }
        );
    }
}

#[test]
fn greater_than_returns_to_real_script_data_with_normalized_diagnostics_and_later_blocks() {
    let mut context = seeded();
    context.input.push(points("-->\r\n\0\0")).unwrap();
    context
        .input
        .push([CodePoint::new(0xd800).unwrap()])
        .unwrap();
    context.input.finish();
    let preserved = retained(&context);
    let mut execution = Execution::new(SOURCE, context);
    for _ in 0..2 {
        assert_eq!(execution.step(), Ok(Progress::Continue(SOURCE)));
    }
    for _ in 0..5 {
        assert_eq!(execution.step(), Ok(Progress::Continue(State::ScriptData)));
    }
    assert_eq!(execution.step(), Ok(Progress::Finished));
    let expected: Vec<_> = points("p-->\n\u{fffd}\u{fffd}")
        .into_iter()
        .chain([CodePoint::new(0xd800).unwrap()])
        .map(Token::Character)
        .chain([Token::Eof])
        .collect();
    assert_eq!(execution.context().tokens, expected);
    assert_eq!(
        execution.context().errors,
        [
            error("prior-diagnostic", 1, 1),
            error("unexpected-null-character", 2, 1),
            error("unexpected-null-character", 2, 2)
        ]
    );
    assert_eq!(
        execution.context().input.errors(),
        [error("surrogate-in-input-stream", 2, 3)]
    );
    assert_eq!(
        execution.context().input.position(),
        Position { line: 2, column: 4 }
    );
    assert_eq!(retained(execution.context()), preserved);

    let mut context = Context::default();
    context.input.push(points(">x<!--<tail")).unwrap();
    context.input.finish();
    let mut execution = Execution::new(SOURCE, context);
    for target in [
        State::ScriptData,
        State::ScriptData,
        State::ScriptDataLessThanSign,
        State::ScriptDataEscapeStart,
    ] {
        assert_eq!(execution.step(), Ok(Progress::Continue(target)));
    }
    assert_eq!(
        execution.context().tokens,
        points(">x<!")
            .into_iter()
            .map(Token::Character)
            .collect::<Vec<_>>()
    );
    assert!(execution.context().errors.is_empty());
    let mut boundary = snapshot(&execution);
    for (column, state, emitted) in [
        (5, State::ScriptDataEscapeStartDash, Some('-')),
        (6, SOURCE, Some('-')),
        (7, State::ScriptDataEscapedLessThanSign, None),
    ] {
        boundary.state = state;
        boundary.current = Some((
            Input::CodePoint(emitted.unwrap_or('<').into()),
            Position { line: 1, column },
        ));
        if let Some(point) = emitted {
            boundary.tokens.push(Token::Character(point.into()));
        }
        assert_eq!(execution.step(), Ok(Progress::Continue(state)));
        assert_eq!(snapshot(&execution), boundary);
    }
    boundary.state = State::ScriptDataDoubleEscapeStart;
    boundary.current = Some((
        Input::CodePoint('t'.into()),
        Position { line: 1, column: 8 },
    ));
    boundary.tokens.push(Token::Character('<'.into()));
    assert_eq!(execution.step(), Ok(Progress::Continue(boundary.state)));
    assert_eq!(snapshot(&execution), boundary);
    for _ in 0..3 {
        assert_eq!(
            execution.step(),
            Err(UnimplementedState {
                state: State::ScriptDataDoubleEscapeStart
            })
        );
        assert_eq!(snapshot(&execution), boundary);
    }
}

#[test]
fn real_escaped_dash_dash_cycles_preserve_every_source_step_and_later_stub() {
    for (initial, first_target) in [
        (State::ScriptDataEscaped, State::ScriptDataEscapedDash),
        (State::ScriptDataEscapedDash, SOURCE),
        (SOURCE, SOURCE),
    ] {
        for exit in ['>', '<'] {
            let mut context = seeded();
            context.input.push(points("--a--\0---")).unwrap();
            context.input.push([exit.into()]).unwrap();
            context.input.push(points("\r\n\0")).unwrap();
            context.input.finish();
            let mut execution = Execution::new(initial, context);
            let mut expected = snapshot(&execution);
            // Fixed specification-derived targets, not inferred from dispatch.
            for (index, (point, target, output)) in [
                ('-', first_target, '-'),
                ('-', SOURCE, '-'),
                ('a', State::ScriptDataEscaped, 'a'),
                ('-', State::ScriptDataEscapedDash, '-'),
                ('-', SOURCE, '-'),
                ('\0', State::ScriptDataEscaped, '\u{fffd}'),
                ('-', State::ScriptDataEscapedDash, '-'),
                ('-', SOURCE, '-'),
                ('-', SOURCE, '-'),
            ]
            .into_iter()
            .enumerate()
            {
                expected.state = target;
                expected.current = Some((
                    Input::CodePoint(point.into()),
                    Position {
                        line: 1,
                        column: index + 1,
                    },
                ));
                expected.tokens.push(Token::Character(output.into()));
                if index == 5 {
                    expected
                        .errors
                        .push(error("unexpected-null-character", 1, 6));
                }
                assert_eq!(execution.step(), Ok(Progress::Continue(target)));
                assert_eq!(snapshot(&execution), expected);
            }
            expected.current = Some((
                Input::CodePoint(exit.into()),
                Position {
                    line: 1,
                    column: 10,
                },
            ));
            if exit == '<' {
                expected.state = State::ScriptDataEscapedLessThanSign;
                assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                assert_eq!(snapshot(&execution), expected);
                expected.state = State::ScriptDataEscaped;
                expected.current = Some((
                    Input::CodePoint('\n'.into()),
                    Position {
                        line: 1,
                        column: 11,
                    },
                ));
                expected.tokens.push(Token::Character('<'.into()));
                assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                assert_eq!(snapshot(&execution), expected);
            } else {
                expected.state = State::ScriptData;
                expected.tokens.push(Token::Character('>'.into()));
                assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                assert_eq!(snapshot(&execution), expected);
            }
            let continuation = expected.state;
            expected.current = Some((
                Input::CodePoint('\n'.into()),
                Position {
                    line: 1,
                    column: 11,
                },
            ));
            expected.tokens.push(Token::Character('\n'.into()));
            assert_eq!(execution.step(), Ok(Progress::Continue(continuation)));
            assert_eq!(snapshot(&execution), expected);
            expected.current = Some((
                Input::CodePoint('\0'.into()),
                Position { line: 2, column: 1 },
            ));
            expected.tokens.push(Token::Character('\u{fffd}'.into()));
            expected
                .errors
                .push(error("unexpected-null-character", 2, 1));
            assert_eq!(execution.step(), Ok(Progress::Continue(continuation)));
            assert_eq!(snapshot(&execution), expected);
            expected.current = Some((Input::Eof, Position { line: 2, column: 2 }));
            expected.tokens.push(Token::Eof);
            if exit == '<' {
                expected
                    .errors
                    .push(error("eof-in-script-html-comment-like-text", 2, 2));
            }
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&execution), expected);
        }
    }
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
    panic!("state failed to consume or terminate");
}

#[test]
fn all_two_chunk_splits_and_single_point_streams_match_closed_source_steps() {
    let mut mixed = points("-->\r\n\0é😀\r\r\nb\r");
    mixed.extend([0xd800, 0xdfff, 0xfdd0, 0x1fffe, 0x01].map(|v| CodePoint::new(v).unwrap()));
    for input in [
        vec![],
        points("-"),
        points("---"),
        points("--<\0tail"),
        points("--\0tail"),
        points("--&amp;"),
        points("--x"),
        points("--a--\0--->\r\n\0"),
        points("--a--\0---<\r\n\0"),
        points("x--\0--a---\0--<tail"),
        points("x--\0--a---><!tail"),
        vec!['-'.into(), CodePoint::new(0xd800).unwrap(), '\0'.into()],
        vec!['-'.into(), CodePoint::new(0x10ffff).unwrap()],
        points("--\u{85}"),
        points("--\r\n"),
        points("\r"),
        points("\r\r"),
        points("--\t"),
        points("-->"),
        mixed,
        points("--><!tail"),
        points("--></script>&"),
        points("--></SCRIPT>\r\n\0"),
        points("-->\r\n\r\r\n\0\r"),
    ] {
        let run = |chunks: Vec<&[CodePoint]>, closed: bool| {
            let mut execution = Execution::new(SOURCE, seeded());
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
                    if drain(&mut execution, &mut trace, input.len() * 3 + 8) {
                        // Never feed, reconstruct, or resume a terminal stub.
                        return trace;
                    }
                    execution.context_mut().input.push([]).unwrap();
                    assert!(!drain(&mut execution, &mut trace, 1));
                }
                execution.context_mut().input.finish();
            }
            assert!(drain(&mut execution, &mut trace, input.len() * 3 + 8));
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
