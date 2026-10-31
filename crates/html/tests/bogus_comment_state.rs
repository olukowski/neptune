//! Every branch of pinned HTML "Bogus comment state" (#bogus-comment-state):
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>
//! Local branch/streaming evidence, not upstream conformance counts.

use neptune_html::tokenizer::{
    CodePoint, Context, Execution, Input, ParseError, PendingComment, PendingEndTag,
    PendingStartTag, Position, Progress, State, TagKind, Token, UnimplementedState,
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
        errors: vec![error("prior", 1, 1)],
        pending_comment: Some(PendingComment {
            // Existing data is preserved literally, not preprocessed/replaced.
            data: vec!['\0'.into(), CodePoint::new(0xd800).unwrap(), 's'.into()],
        }),
        current_tag: Some(TagKind::End),
        pending_start_tag: Some(PendingStartTag {
            name: points("start"),
            ..PendingStartTag::default()
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("end"),
            ..PendingEndTag::default()
        }),
        last_start_tag: Some(points("title")),
        return_state: Some(State::Rcdata),
        temporary_buffer: points("scratch"),
        ..Context::default()
    }
}

fn isolated(input: &[CodePoint], closed: bool) -> Execution {
    let mut context = seeded();
    context.input.push(input.iter().copied()).unwrap();
    if closed {
        context.input.finish();
    }
    Execution::new(State::BogusComment, context)
}

#[derive(Debug, Eq, PartialEq)]
struct Snapshot {
    state: State,
    current: Option<(Input, Position)>,
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
    input_errors: Vec<ParseError>,
    comment: Option<PendingComment>,
    current_tag: Option<TagKind>,
    start: Option<PendingStartTag>,
    end: Option<PendingEndTag>,
    last_start_tag: Option<Vec<CodePoint>>,
    return_state: Option<State>,
    buffer: Vec<CodePoint>,
}

fn snapshot(execution: &Execution) -> Snapshot {
    let c = execution.context();
    Snapshot {
        state: execution.state(),
        current: c.input.current(),
        tokens: c.tokens.clone(),
        errors: c.errors.clone(),
        input_errors: c.input.errors().to_vec(),
        comment: c.pending_comment.clone(),
        current_tag: c.current_tag,
        start: c.pending_start_tag.clone(),
        end: c.pending_end_tag.clone(),
        last_start_tag: c.last_start_tag.clone(),
        return_state: c.return_state,
        buffer: c.temporary_buffer.clone(),
    }
}

#[test]
fn bogus_comment_anything_else_appends_exact_code_points_without_reconsuming() {
    for value in [
        0x09, 0x0a, 0x0c, 0x0d, 0x20, 0x21, 0x26, 0x2d, 0x2f, 0x30, 0x3c, 0x3f, 0x41, 0x5a, 0x61,
        0x7a, 0xe9, 0x391, 0x301, 0x1f600, 0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x0b, 0x7f,
    ] {
        let point = CodePoint::new(value).unwrap();
        let normalized = if value == 0x0d { '\n'.into() } else { point };
        let mut execution = isolated(&[point, '>'.into(), 'x'.into()], true);
        let mut expected = snapshot(&execution);
        assert_eq!(
            execution.step(),
            Ok(Progress::Continue(State::BogusComment))
        );
        expected.current = Some((
            Input::CodePoint(normalized),
            Position { line: 1, column: 1 },
        ));
        expected.comment.as_mut().unwrap().data.push(normalized);
        let preprocessing = match value {
            0xd800 | 0xdfff => Some("surrogate-in-input-stream"),
            0xfdd0 | 0x10ffff => Some("noncharacter-in-input-stream"),
            0x0b | 0x7f => Some("control-character-in-input-stream"),
            _ => None,
        };
        if let Some(code) = preprocessing {
            expected.input_errors.push(error(code, 1, 1));
        }
        assert_eq!(snapshot(&execution), expected);
        // The next step consumes '>', not the previous point a second time.
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
        expected.state = State::Data;
        expected.current = Some((
            Input::CodePoint('>'.into()),
            if normalized == '\n'.into() {
                Position { line: 2, column: 1 }
            } else {
                Position { line: 1, column: 2 }
            },
        ));
        expected.tokens.push(Token::Comment {
            data: expected.comment.take().unwrap().data,
        });
        assert_eq!(snapshot(&execution), expected);
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
        assert_eq!(
            execution.context().tokens.last(),
            Some(&Token::Character('x'.into()))
        );
        assert_eq!(execution.step(), Ok(Progress::Finished));
        let terminal = snapshot(&execution);
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&execution), terminal);
    }
}

#[test]
fn bogus_comment_null_replaces_only_new_input_and_reports_each_normalized_location() {
    let mut execution = isolated(&points("\r\n\0😀\0>\0"), true);
    for (point, line, column) in [('\n', 1, 1), ('\0', 2, 1), ('😀', 2, 2), ('\0', 2, 3)] {
        let mut expected = snapshot(&execution);
        assert_eq!(
            execution.step(),
            Ok(Progress::Continue(State::BogusComment))
        );
        expected.current = Some((Input::CodePoint(point.into()), Position { line, column }));
        expected
            .comment
            .as_mut()
            .unwrap()
            .data
            .push(if point == '\0' {
                '\u{fffd}'.into()
            } else {
                point.into()
            });
        if point == '\0' {
            expected
                .errors
                .push(error("unexpected-null-character", line, column));
        }
        assert_eq!(snapshot(&execution), expected);
    }
    assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
    let mut data = seeded().pending_comment.unwrap().data;
    data.extend(points("\n�😀�"));
    assert_eq!(
        execution.context().tokens,
        [Token::Character('p'.into()), Token::Comment { data }]
    );
    assert!(execution.context().pending_comment.is_none());
    // Real Data continues after '>' and emits the original NUL, not U+FFFD.
    assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
    assert_eq!(
        execution.context().tokens.last(),
        Some(&Token::Character('\0'.into()))
    );
    assert_eq!(
        execution.context().errors.last(),
        Some(&error("unexpected-null-character", 2, 5))
    );
    assert_eq!(execution.step(), Ok(Progress::Finished));
}

#[test]
fn bogus_comment_open_exhaustion_preserves_context_until_explicit_eof_emits_comment_then_eof() {
    for initial_data in [vec![], seeded().pending_comment.unwrap().data] {
        for input in ["", "<&", "\0", "\r\n", "\r"] {
            let mut execution = isolated(&points(input), false);
            execution
                .context_mut()
                .pending_comment
                .as_mut()
                .unwrap()
                .data = initial_data.clone();
            assert!(!drain(&mut execution, &mut vec![], input.len() + 2));
            let mut expected = snapshot(&execution);
            for _ in 0..2 {
                execution.context_mut().input.push([]).unwrap();
                assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                assert_eq!(snapshot(&execution), expected);
            }
            let eof_position = match input {
                "" => Position { line: 1, column: 1 },
                "<&" => Position { line: 1, column: 3 },
                "\0" => Position { line: 1, column: 2 },
                _ => Position { line: 2, column: 1 },
            };
            execution.context_mut().input.finish();
            assert_eq!(execution.step(), Ok(Progress::Finished));
            expected.current = Some((Input::Eof, eof_position));
            expected.tokens.push(Token::Comment {
                data: expected.comment.take().unwrap().data,
            });
            expected.tokens.push(Token::Eof);
            // EOF does not switch state or generate eof-in-comment.
            assert_eq!(snapshot(&execution), expected);
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&execution), expected);
        }
    }
    // Empty data also emits on '>', without treating it as EOF.
    let mut execution = isolated(&points(">"), false);
    execution.context_mut().pending_comment = Some(PendingComment::default());
    assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
    assert_eq!(
        execution.context().tokens.last(),
        Some(&Token::Comment { data: vec![] })
    );
    assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
    execution.context_mut().input.finish();
    assert_eq!(execution.step(), Ok(Progress::Finished));
}

#[test]
fn bogus_comment_recovery_separates_source_diagnostics_and_preserves_stub_failures() {
    let mut context = Context::default();
    context.input.push(points("\r\n</\0<&😀>z</")).unwrap();
    context.input.finish();
    let mut execution = Execution::new(State::Data, context);
    for target in [
        State::Data,
        State::TagOpen,
        State::EndTagOpen,
        State::BogusComment,
    ] {
        assert_eq!(execution.step(), Ok(Progress::Continue(target)));
    }
    // End tag open diagnoses and creates EMPTY data; NUL handling is not its job.
    assert_eq!(
        execution.context().errors,
        [error("invalid-first-character-of-tag-name", 2, 3)]
    );
    assert_eq!(
        execution.context().pending_comment,
        Some(PendingComment::default())
    );
    let current = execution.context().input.current();
    assert_eq!(
        execution.step(),
        Ok(Progress::Continue(State::BogusComment))
    );
    assert_eq!(execution.context().input.current(), current);
    assert_eq!(
        execution.context().errors,
        [
            error("invalid-first-character-of-tag-name", 2, 3),
            error("unexpected-null-character", 2, 3),
        ]
    );
    assert_eq!(
        execution.context().pending_comment.as_ref().unwrap().data,
        points("�")
    );
    let mut trace = vec![];
    assert!(drain(&mut execution, &mut trace, 20));
    assert_eq!(trace.last().unwrap().0, Ok(Progress::Finished));
    assert_eq!(
        execution.context().tokens,
        [
            Token::Character('\n'.into()),
            Token::Comment {
                data: points("�<&😀")
            },
            Token::Character('z'.into()),
            Token::Character('<'.into()),
            Token::Character('/'.into()),
            Token::Eof,
        ]
    );
    assert_eq!(
        execution.context().errors.last(),
        Some(&error("eof-before-tag-name", 2, 11))
    );

    for (tail, target) in [
        ("&", State::CharacterReference),
        ("<!", State::MarkupDeclarationOpen),
        ("<?", State::ProcessingInstructionOpen),
        ("<a ", State::AfterAttributeName),
        ("<a/x", State::AfterAttributeName),
    ] {
        let mut execution = isolated(&points(&format!("x>{tail}")), true);
        let mut trace = vec![];
        assert!(drain(&mut execution, &mut trace, tail.len() * 2 + 6));
        assert_eq!(
            trace.last().unwrap().0,
            Err(UnimplementedState { state: target })
        );
        let effects = snapshot(&execution);
        assert!(
            effects
                .tokens
                .iter()
                .any(|token| matches!(token, Token::Comment { .. }))
        );
        assert_eq!(execution.step(), Err(UnimplementedState { state: target }));
        assert_eq!(snapshot(&execution), effects);
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
        trace.push((result, snapshot(execution)));
        if !matches!(result, Ok(Progress::Continue(_))) {
            let terminal = snapshot(execution);
            assert_eq!(execution.step(), result);
            assert_eq!(snapshot(execution), terminal);
            return true;
        }
    }
    panic!("Bogus comment integration failed to consume or terminate");
}

#[test]
fn bogus_comment_all_two_chunk_splits_and_single_point_chunks_match_closed_source_traces() {
    let mut samples: Vec<_> = [
        "",
        ">",
        "<&",
        "\0\0",
        "😀É<&?>x",
        "\r\n\0>z\r",
        "\r\r\n",
        "</0",
        "</0>x</?&<>",
        "\r\n</\0😀\r\n>x\r",
        "</\r\n>\r\n",
        "</0>&",
        "</0><!",
        "</0><?",
        "</0><a ",
        "</0><a/",
        "</0><A>x</A>",
    ]
    .into_iter()
    .map(points)
    .collect();
    let mut non_scalar = points("</");
    non_scalar.extend([0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x0b].map(|v| CodePoint::new(v).unwrap()));
    non_scalar.extend(points("\r\n\0>"));
    samples.push(non_scalar);
    for initial in [State::BogusComment, State::Data] {
        for input in &samples {
            let run = |chunks: Vec<&[CodePoint]>, closed: bool| {
                let mut execution = Execution::new(initial, seeded());
                let mut trace = vec![];
                let bound = input.len() * 3 + 8;
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
                        // Stop permanently at a stub; never reconstruct/resume it.
                        if drain(&mut execution, &mut trace, bound) {
                            return trace;
                        }
                        execution.context_mut().input.push([]).unwrap();
                        assert!(!drain(&mut execution, &mut trace, 1));
                    }
                    execution.context_mut().input.finish();
                }
                assert!(drain(&mut execution, &mut trace, bound));
                trace
            };
            let expected = run(vec![], true);
            for split in 0..=input.len() {
                assert_eq!(
                    run(vec![&input[..split], &input[split..]], false),
                    expected,
                    "{initial:?} split {split} of {input:?}"
                );
            }
            assert_eq!(run(input.chunks(1).collect(), false), expected);
        }
    }
}
