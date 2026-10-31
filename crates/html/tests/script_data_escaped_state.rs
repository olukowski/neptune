//! Pinned HTML 404b3eb14a7a936dddefb2d78e224dbeb01d2358, source
//! 143775–143802 (#script-data-escaped-state). Local checks, not conformance runs.

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

fn seeded_context() -> Context {
    let attribute = Attribute {
        name: points("a"),
        value: points("retained"),
    };
    let discarded = Attribute {
        name: points("a"),
        value: points("detached"),
    };
    Context {
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
        tokens: vec![Token::Character('p'.into())],
        errors: vec![error("prior-diagnostic", 1, 1)],
        ..Default::default()
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
    start: Option<PendingStartTag>,
    end: Option<PendingEndTag>,
    comment: Option<PendingComment>,
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
        start: context.pending_start_tag.clone(),
        end: context.pending_end_tag.clone(),
        comment: context.pending_comment.clone(),
        temporary_buffer: context.temporary_buffer.clone(),
    }
}

#[test]
fn every_ascii_branch_and_non_scalars_have_exact_source_effects() {
    for value in (0..=0x7f).chain([
        0x80, 0x9f, 0xa0, 0xe9, 0x301, 0xd800, 0xdfff, 0xfdd0, 0xfeff, 0xfffd, 0x1f600, 0x1fffe,
        0x10ffff,
    ]) {
        for seeded in [false, true] {
            let mut context = if seeded {
                seeded_context()
            } else {
                Context::default()
            };
            let point = CodePoint::new(value).unwrap();
            context.input.push([point, 'z'.into()]).unwrap();
            context.input.finish();
            let mut execution = Execution::new(State::ScriptDataEscaped, context);
            let mut expected = snapshot(&execution);
            let normalized = if value == 0x0d { '\n'.into() } else { point };
            expected.current = Some((
                Input::CodePoint(normalized),
                Position { line: 1, column: 1 },
            ));
            expected.state = match value {
                0x2d => State::ScriptDataEscapedDash,
                0x3c => State::ScriptDataEscapedLessThanSign,
                _ => State::ScriptDataEscaped,
            };
            if value != 0x3c {
                expected.tokens.push(Token::Character(if value == 0 {
                    '\u{fffd}'.into()
                } else {
                    normalized
                }));
            }
            if value == 0 {
                expected
                    .errors
                    .push(error("unexpected-null-character", 1, 1));
            }
            let preprocessing = match value {
                0xd800..=0xdfff => Some("surrogate-in-input-stream"),
                0xfdd0 | 0x1fffe | 0x10ffff => Some("noncharacter-in-input-stream"),
                0x01..=0x08 | 0x0b | 0x0e..=0x1f | 0x7f..=0x9f => {
                    Some("control-character-in-input-stream")
                }
                _ => None,
            };
            if let Some(code) = preprocessing {
                expected.input_errors.push(error(code, 1, 1));
            }
            assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&execution), expected, "U+{value:04X}");
            // Inspect only the source boundary: no branch reconsumes or eats
            // the suffix. Do not execute or reconstruct a destination here.
            assert_eq!(
                execution.context_mut().input.consume(),
                Input::CodePoint('z'.into())
            );
            assert_eq!(
                execution.context().input.position(),
                if value == 0x0a || value == 0x0d {
                    Position { line: 2, column: 1 }
                } else {
                    Position { line: 1, column: 2 }
                }
            );
        }
    }
}

#[test]
fn destinations_execute_dash_or_less_than_fallback_after_inspectable_source_effects() {
    for (delimiter, destination) in [
        ('-', State::ScriptDataEscapedDash),
        ('<', State::ScriptDataEscapedLessThanSign),
    ] {
        for finished in [false, true] {
            let mut context = seeded_context();
            context
                .input
                .push([
                    delimiter.into(),
                    CodePoint::new(0xd800).unwrap(),
                    '\0'.into(),
                ])
                .unwrap();
            if finished {
                context.input.finish();
            }
            let mut execution = Execution::new(State::ScriptDataEscaped, context);
            let mut expected = snapshot(&execution);
            expected.state = destination;
            expected.current = Some((
                Input::CodePoint(delimiter.into()),
                Position { line: 1, column: 1 },
            ));
            if delimiter == '-' {
                expected.tokens.push(Token::Character('-'.into()));
            }
            assert_eq!(execution.step(), Ok(Progress::Continue(destination)));
            assert_eq!(snapshot(&execution), expected);
            if delimiter == '<' {
                // The gateway emits '<' and schedules surrogate reconsumption.
                expected.state = State::ScriptDataEscaped;
                expected.current = Some((
                    Input::CodePoint(CodePoint::new(0xd800).unwrap()),
                    Position { line: 1, column: 2 },
                ));
                expected.tokens.push(Token::Character('<'.into()));
                expected
                    .input_errors
                    .push(error("surrogate-in-input-stream", 1, 2));
                assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                assert_eq!(snapshot(&execution), expected);
            }
            // Dash consumes the surrogate once; gateway fallback reconsumes it
            // without duplicating the preprocessing diagnostic.
            let surrogate = CodePoint::new(0xd800).unwrap();
            expected.state = State::ScriptDataEscaped;
            expected.current = Some((Input::CodePoint(surrogate), Position { line: 1, column: 2 }));
            expected.tokens.push(Token::Character(surrogate));
            if delimiter == '-' {
                expected
                    .input_errors
                    .push(error("surrogate-in-input-stream", 1, 2));
            }
            assert_eq!(
                execution.step(),
                Ok(Progress::Continue(State::ScriptDataEscaped))
            );
            assert_eq!(snapshot(&execution), expected);
            expected.current = Some((
                Input::CodePoint('\0'.into()),
                Position { line: 1, column: 3 },
            ));
            expected.tokens.push(Token::Character('\u{fffd}'.into()));
            expected
                .errors
                .push(error("unexpected-null-character", 1, 3));
            assert_eq!(
                execution.step(),
                Ok(Progress::Continue(State::ScriptDataEscaped))
            );
            assert_eq!(snapshot(&execution), expected);
            if !finished {
                for _ in 0..3 {
                    assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                    assert_eq!(snapshot(&execution), expected);
                }
                execution.context_mut().input.finish();
            }
            expected.current = Some((Input::Eof, Position { line: 1, column: 4 }));
            expected.tokens.push(Token::Eof);
            expected
                .errors
                .push(error("eof-in-script-html-comment-like-text", 1, 4));
            for _ in 0..3 {
                assert_eq!(execution.step(), Ok(Progress::Finished));
                assert_eq!(snapshot(&execution), expected);
            }
        }
    }
}

#[test]
fn nulls_newlines_and_eof_report_exact_normalized_positions_once() {
    let mut context = seeded_context();
    context.input.push(points("\0\0\r\n\0\r\r\n")).unwrap();
    context.input.finish();
    let mut execution = Execution::new(State::ScriptDataEscaped, context);
    let mut expected = snapshot(&execution);
    for (consumed, emitted, line, column) in [
        ('\0', '\u{fffd}', 1, 1),
        ('\0', '\u{fffd}', 1, 2),
        ('\n', '\n', 1, 3),
        ('\0', '\u{fffd}', 2, 1),
        ('\n', '\n', 2, 2),
        ('\n', '\n', 3, 1),
    ] {
        expected.current = Some((Input::CodePoint(consumed.into()), Position { line, column }));
        expected.tokens.push(Token::Character(emitted.into()));
        if consumed == '\0' {
            expected
                .errors
                .push(error("unexpected-null-character", line, column));
        }
        assert_eq!(
            execution.step(),
            Ok(Progress::Continue(State::ScriptDataEscaped))
        );
        assert_eq!(snapshot(&execution), expected);
    }
    expected.current = Some((Input::Eof, Position { line: 4, column: 1 }));
    expected
        .errors
        .push(error("eof-in-script-html-comment-like-text", 4, 1));
    expected.tokens.push(Token::Eof);
    for _ in 0..3 {
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&execution), expected);
    }
}

#[test]
fn open_exhaustion_is_effect_free_until_explicit_eof() {
    for seeded in [false, true] {
        for prefix in ["", "x\r\n"] {
            let context = if seeded {
                seeded_context()
            } else {
                Context::default()
            };
            let mut execution = Execution::new(State::ScriptDataEscaped, context);
            execution.context_mut().input.push(points(prefix)).unwrap();
            for _ in 0..if prefix.is_empty() { 0 } else { 2 } {
                assert_eq!(
                    execution.step(),
                    Ok(Progress::Continue(State::ScriptDataEscaped))
                );
            }
            let mut expected = snapshot(&execution);
            for _ in 0..3 {
                execution.context_mut().input.push([]).unwrap();
                assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                assert_eq!(snapshot(&execution), expected);
            }
            execution.context_mut().input.finish();
            execution.context_mut().input.finish();
            let position = if prefix.is_empty() {
                Position { line: 1, column: 1 }
            } else {
                Position { line: 2, column: 1 }
            };
            expected.current = Some((Input::Eof, position));
            expected.tokens.push(Token::Eof);
            expected.errors.push(ParseError {
                code: "eof-in-script-html-comment-like-text",
                position,
            });
            for _ in 0..3 {
                assert_eq!(execution.step(), Ok(Progress::Finished));
                assert_eq!(snapshot(&execution), expected);
            }
            assert_eq!(
                execution.context_mut().input.push([]),
                Err(CursorError::InputAlreadyFinished)
            );
        }
    }
}

#[test]
fn entity_like_and_malformed_script_text_is_literal_not_a_comment_or_reference() {
    let input = points("&amp;&#0;&#xD800;&bogus;>/!?=\"'Azé😀\u{0301}\t\u{000c}\u{fffd}\u{feff}");
    let mut context = Context::default();
    context.input.push(input.iter().copied()).unwrap();
    context.input.finish();
    let mut execution = Execution::new(State::ScriptDataEscaped, context);
    for (index, &point) in input.iter().enumerate() {
        assert_eq!(
            execution.step(),
            Ok(Progress::Continue(State::ScriptDataEscaped))
        );
        assert_eq!(execution.context().tokens[index], Token::Character(point));
        assert!(execution.context().errors.is_empty());
        assert!(execution.context().input.errors().is_empty());
        assert_eq!(execution.context().return_state, None);
        assert_eq!(execution.context().pending_comment, None);
    }
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(
        execution.context().tokens,
        input
            .into_iter()
            .map(Token::Character)
            .chain([Token::Eof])
            .collect::<Vec<_>>()
    );
    assert_eq!(
        execution.context().errors,
        [error(
            "eof-in-script-html-comment-like-text",
            1,
            execution.context().tokens.len()
        )]
    );
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
    panic!("Script data escaped failed to consume or terminate");
}

#[test]
fn every_two_chunk_split_and_single_point_stream_matches_closed_source_steps() {
    let mut mixed = points("&amp;\r\n\0\0é😀\r\r\nb\r");
    mixed
        .extend([0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x01, 0x7f].map(|v| CodePoint::new(v).unwrap()));
    let mut dash = mixed.clone();
    dash.extend(points("-literal\0-a--literal\0"));
    let mut less_than = mixed.clone();
    less_than.extend(points("<unconsumed\0"));
    for input in [
        vec![],
        mixed,
        dash,
        less_than,
        points("\r"),
        points("\r\n"),
        points("\r\r"),
        points("\0\r"),
        points("&"),
        points("-"),
        points("--literal\0"),
        points("--a--\0--->\r\n\0"),
        points("--a--\0---<\r\n\0"),
        points("x--\0--a---\0--<tail"),
        points("x--\0--a---><!tail"),
        points("-<unconsumed\0"),
        points("<"),
        points("\r\n<\r\n"),
        points("\r-\0"),
        points("<!--"),
        points("</script>"),
    ] {
        let run = |chunks: Vec<&[CodePoint]>, closed: bool| {
            let mut execution = Execution::new(State::ScriptDataEscaped, seeded_context());
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
                    if drain(&mut execution, &mut trace, input.len() + 3) {
                        // Never feed, reconstruct, or resume a blocked execution.
                        return trace;
                    }
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
