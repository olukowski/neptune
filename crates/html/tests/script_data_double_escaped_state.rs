//! Pinned HTML 404b3eb14a7a936dddefb2d78e224dbeb01d2358, source
//! 143993–144021 (#script-data-double-escaped-state). Local checks, not conformance runs.

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
        temporary_buffer: points("sCrİpT"),
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
        for (seeded, selector) in [
            (false, None),
            (true, None),
            (true, Some(TagKind::Start)),
            (true, Some(TagKind::End)),
        ] {
            let mut context = if seeded {
                seeded_context()
            } else {
                Context::default()
            };
            context.current_tag = selector;
            let point = CodePoint::new(value).unwrap();
            context.input.push([point, 'z'.into()]).unwrap();
            context.input.finish();
            let mut execution = Execution::new(State::ScriptDataDoubleEscaped, context);
            let mut expected = snapshot(&execution);
            let normalized = if value == 0x0d { '\n'.into() } else { point };
            expected.current = Some((
                Input::CodePoint(normalized),
                Position { line: 1, column: 1 },
            ));
            expected.state = match value {
                0x2d => State::ScriptDataDoubleEscapedDash,
                0x3c => State::ScriptDataDoubleEscapedLessThanSign,
                _ => State::ScriptDataDoubleEscaped,
            };
            expected.tokens.push(Token::Character(if value == 0 {
                '\u{fffd}'.into()
            } else {
                normalized
            }));
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
fn destinations_execute_exact_surrogate_null_and_eof_steps_after_source_effects() {
    for (delimiter, destination) in [
        ('-', State::ScriptDataDoubleEscapedDash),
        ('<', State::ScriptDataDoubleEscapedLessThanSign),
    ] {
        for tail in [vec![], vec![CodePoint::new(0xd800).unwrap(), '\0'.into()]] {
            for finished in [false, true] {
                let mut context = seeded_context();
                context.input.push([delimiter.into()]).unwrap();
                context.input.push(tail.iter().copied()).unwrap();
                if finished {
                    context.input.finish();
                }
                let mut execution = Execution::new(State::ScriptDataDoubleEscaped, context);
                let mut expected = snapshot(&execution);
                expected.state = destination;
                expected.current = Some((
                    Input::CodePoint(delimiter.into()),
                    Position { line: 1, column: 1 },
                ));
                expected.tokens.push(Token::Character(delimiter.into()));
                assert_eq!(execution.step(), Ok(Progress::Continue(destination)));
                assert_eq!(snapshot(&execution), expected);
                if !tail.is_empty() {
                    let surrogate = tail[0];
                    expected.state = State::ScriptDataDoubleEscaped;
                    expected.current =
                        Some((Input::CodePoint(surrogate), Position { line: 1, column: 2 }));
                    expected
                        .input_errors
                        .push(error("surrogate-in-input-stream", 1, 2));
                    if delimiter == '<' {
                        // The actual less-than state schedules reconsumption;
                        // only the next DoubleEscaped step emits the surrogate.
                        assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                        assert_eq!(snapshot(&execution), expected);
                        assert_eq!(
                            execution.context_mut().input.reconsume(),
                            Err(CursorError::AlreadyReconsuming)
                        );
                    }
                    expected.tokens.push(Token::Character(surrogate));
                    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                    assert_eq!(snapshot(&execution), expected);
                    expected.current = Some((
                        Input::CodePoint('\0'.into()),
                        Position { line: 1, column: 3 },
                    ));
                    expected.tokens.push(Token::Character('\u{fffd}'.into()));
                    expected
                        .errors
                        .push(error("unexpected-null-character", 1, 3));
                    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                    assert_eq!(snapshot(&execution), expected);
                }
                if !finished {
                    for _ in 0..3 {
                        execution.context_mut().input.push([]).unwrap();
                        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                        assert_eq!(snapshot(&execution), expected);
                    }
                    execution.context_mut().input.finish();
                }
                let column = tail.len() + 2;
                expected.current = Some((Input::Eof, Position { line: 1, column }));
                if tail.is_empty() && delimiter == '<' {
                    expected.state = State::ScriptDataDoubleEscaped;
                    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                    assert_eq!(snapshot(&execution), expected);
                    assert_eq!(
                        execution.context_mut().input.reconsume(),
                        Err(CursorError::AlreadyReconsuming)
                    );
                }
                expected.tokens.push(Token::Eof);
                expected
                    .errors
                    .push(error("eof-in-script-html-comment-like-text", 1, column));
                for _ in 0..3 {
                    assert_eq!(execution.step(), Ok(Progress::Finished));
                    assert_eq!(snapshot(&execution), expected);
                }
            }
        }
    }
}

#[test]
fn nulls_newlines_and_eof_report_exact_normalized_positions_once() {
    let mut context = seeded_context();
    context.input.push(points("\0\0\r\n\0\r\r\n")).unwrap();
    context.input.finish();
    let mut execution = Execution::new(State::ScriptDataDoubleEscaped, context);
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
            Ok(Progress::Continue(State::ScriptDataDoubleEscaped))
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
            let mut execution = Execution::new(State::ScriptDataDoubleEscaped, context);
            execution.context_mut().input.push(points(prefix)).unwrap();
            for _ in 0..if prefix.is_empty() { 0 } else { 2 } {
                assert_eq!(
                    execution.step(),
                    Ok(Progress::Continue(State::ScriptDataDoubleEscaped))
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
    let mut execution = Execution::new(State::ScriptDataDoubleEscaped, context);
    for (index, &point) in input.iter().enumerate() {
        assert_eq!(
            execution.step(),
            Ok(Progress::Continue(State::ScriptDataDoubleEscaped))
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

#[test]
fn incoming_reconsumption_keeps_positions_and_once_only_preprocessing() {
    for item in [
        Some(0),
        Some(0x2d),
        Some(0x3c),
        Some(0x2f),
        Some(0x26),
        Some(0x3e),
        Some(0x41),
        Some(0xd800),
        Some(0x10ffff),
        Some(0x01),
        Some(0x0d),
        None,
    ] {
        let mut context = seeded_context();
        context.input.push(points("\r\n")).unwrap();
        if let Some(value) = item {
            context
                .input
                .push([CodePoint::new(value).unwrap(), 'z'.into()])
                .unwrap();
        }
        context.input.finish();
        assert_eq!(context.input.consume(), Input::CodePoint('\n'.into()));
        let consumed = context.input.consume();
        let position = Position { line: 2, column: 1 };
        assert_eq!(context.input.position(), position);
        context.input.reconsume().unwrap();
        let mut execution = Execution::new(State::ScriptDataDoubleEscaped, context);
        let mut expected = snapshot(&execution);
        let result = match consumed {
            Input::Eof => {
                expected
                    .errors
                    .push(error("eof-in-script-html-comment-like-text", 2, 1));
                expected.tokens.push(Token::Eof);
                Progress::Finished
            }
            Input::CodePoint(point) => {
                expected.state = match point.value() {
                    0x2d => State::ScriptDataDoubleEscapedDash,
                    0x3c => State::ScriptDataDoubleEscapedLessThanSign,
                    _ => State::ScriptDataDoubleEscaped,
                };
                expected
                    .tokens
                    .push(Token::Character(if point.value() == 0 {
                        expected
                            .errors
                            .push(error("unexpected-null-character", 2, 1));
                        '\u{fffd}'.into()
                    } else {
                        point
                    }));
                Progress::Continue(expected.state)
            }
            Input::NeedMoreInput => panic!("closed input has an item or EOF"),
        };
        assert_eq!(execution.step(), Ok(result));
        assert_eq!(snapshot(&execution), expected);
        if item.is_none() {
            for _ in 0..3 {
                assert_eq!(execution.step(), Ok(Progress::Finished));
                assert_eq!(snapshot(&execution), expected);
            }
        } else {
            // Source-only cursor probe: no outgoing reconsumption, no suffix loss.
            assert_eq!(
                execution.context_mut().input.consume(),
                Input::CodePoint('z'.into())
            );
            assert_eq!(
                execution.context().input.position(),
                if item == Some(0x0d) {
                    Position { line: 3, column: 1 }
                } else {
                    Position { line: 2, column: 2 }
                }
            );
            assert_eq!(execution.context().input.errors(), expected.input_errors);
        }
    }
}

#[test]
fn real_script_entry_reaches_double_escaped_with_exact_null_and_eof_ownership() {
    let mut context = seeded_context();
    context.input.push(points("<!--<script>\0")).unwrap();
    context.input.finish();
    let mut execution = Execution::new(State::ScriptData, context);
    let mut expected = snapshot(&execution);
    for (column, consumed, emitted, target) in [
        (1, '<', "", State::ScriptDataLessThanSign),
        (2, '!', "<!", State::ScriptDataEscapeStart),
        (3, '-', "-", State::ScriptDataEscapeStartDash),
        (4, '-', "-", State::ScriptDataEscapedDashDash),
        (5, '<', "", State::ScriptDataEscapedLessThanSign),
    ] {
        expected.state = target;
        expected.current = Some((
            Input::CodePoint(consumed.into()),
            Position { line: 1, column },
        ));
        expected
            .tokens
            .extend(points(emitted).into_iter().map(Token::Character));
        assert_eq!(execution.step(), Ok(Progress::Continue(target)));
        assert_eq!(snapshot(&execution), expected);
    }
    expected.state = State::ScriptDataDoubleEscapeStart;
    expected.current = Some((
        Input::CodePoint('s'.into()),
        Position { line: 1, column: 6 },
    ));
    expected.temporary_buffer.clear();
    expected.tokens.push(Token::Character('<'.into()));
    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
    assert_eq!(snapshot(&execution), expected);
    for (index, point) in points("script").into_iter().enumerate() {
        expected.current = Some((
            Input::CodePoint(point),
            Position {
                line: 1,
                column: index + 6,
            },
        ));
        expected.temporary_buffer.push(
            CodePoint::new(if (0x41..=0x5a).contains(&point.value()) {
                point.value() + 0x20
            } else {
                point.value()
            })
            .unwrap(),
        );
        expected.tokens.push(Token::Character(point));
        assert_eq!(
            execution.step(),
            Ok(Progress::Continue(State::ScriptDataDoubleEscapeStart))
        );
        assert_eq!(snapshot(&execution), expected);
    }
    expected.state = State::ScriptDataDoubleEscaped;
    expected.current = Some((
        Input::CodePoint('>'.into()),
        Position {
            line: 1,
            column: 12,
        },
    ));
    expected.tokens.push(Token::Character('>'.into()));
    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
    assert_eq!(snapshot(&execution), expected);
    expected.state = State::ScriptDataDoubleEscaped;
    expected.current = Some((
        Input::CodePoint('\0'.into()),
        Position {
            line: 1,
            column: 13,
        },
    ));
    expected.tokens.push(Token::Character('\u{fffd}'.into()));
    expected
        .errors
        .push(error("unexpected-null-character", 1, 13));
    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
    assert_eq!(snapshot(&execution), expected);
    expected.current = Some((
        Input::Eof,
        Position {
            line: 1,
            column: 14,
        },
    ));
    expected.tokens.push(Token::Eof);
    expected
        .errors
        .push(error("eof-in-script-html-comment-like-text", 1, 14));
    assert_eq!(execution.step(), Ok(Progress::Finished));
    assert_eq!(snapshot(&execution), expected);
}

#[test]
fn closed_double_escaped_cycles_preserve_source_steps_scratch_and_return_to_escaped_then_script() {
    use State::*;
    let mut context = seeded_context();
    context
        .input
        .push(points("-a--\0<\0</X>--b</ScRiPt>-->"))
        .unwrap();
    context.input.finish();
    let mut execution = Execution::new(ScriptDataDoubleEscaped, context);
    let mut expected = snapshot(&execution);
    // Pinned 143993–144142 and 143775–143869: fixed source-step expectations,
    // including the repeated current NUL at the less-than fallback boundary.
    for (point, column, target, emitted, buffer) in [
        ('-', 1, ScriptDataDoubleEscapedDash, Some('-'), None),
        ('a', 2, ScriptDataDoubleEscaped, Some('a'), None),
        ('-', 3, ScriptDataDoubleEscapedDash, Some('-'), None),
        ('-', 4, ScriptDataDoubleEscapedDashDash, Some('-'), None),
        ('\0', 5, ScriptDataDoubleEscaped, Some('\u{fffd}'), None),
        ('<', 6, ScriptDataDoubleEscapedLessThanSign, Some('<'), None),
        ('\0', 7, ScriptDataDoubleEscaped, None, None),
        ('\0', 7, ScriptDataDoubleEscaped, Some('\u{fffd}'), None),
        ('<', 8, ScriptDataDoubleEscapedLessThanSign, Some('<'), None),
        ('/', 9, ScriptDataDoubleEscapeEnd, Some('/'), Some("")),
        ('X', 10, ScriptDataDoubleEscapeEnd, Some('X'), Some("x")),
        ('>', 11, ScriptDataDoubleEscaped, Some('>'), None),
        ('-', 12, ScriptDataDoubleEscapedDash, Some('-'), None),
        ('-', 13, ScriptDataDoubleEscapedDashDash, Some('-'), None),
        ('b', 14, ScriptDataDoubleEscaped, Some('b'), None),
        (
            '<',
            15,
            ScriptDataDoubleEscapedLessThanSign,
            Some('<'),
            None,
        ),
        ('/', 16, ScriptDataDoubleEscapeEnd, Some('/'), Some("")),
        ('S', 17, ScriptDataDoubleEscapeEnd, Some('S'), Some("s")),
        ('c', 18, ScriptDataDoubleEscapeEnd, Some('c'), Some("sc")),
        ('R', 19, ScriptDataDoubleEscapeEnd, Some('R'), Some("scr")),
        ('i', 20, ScriptDataDoubleEscapeEnd, Some('i'), Some("scri")),
        ('P', 21, ScriptDataDoubleEscapeEnd, Some('P'), Some("scrip")),
        (
            't',
            22,
            ScriptDataDoubleEscapeEnd,
            Some('t'),
            Some("script"),
        ),
        ('>', 23, ScriptDataEscaped, Some('>'), None),
        ('-', 24, ScriptDataEscapedDash, Some('-'), None),
        ('-', 25, ScriptDataEscapedDashDash, Some('-'), None),
        ('>', 26, ScriptData, Some('>'), None),
    ] {
        expected.state = target;
        expected.current = Some((Input::CodePoint(point.into()), Position { line: 1, column }));
        if let Some(emitted) = emitted {
            expected.tokens.push(Token::Character(emitted.into()));
            if point == '\0' {
                expected
                    .errors
                    .push(error("unexpected-null-character", 1, column));
            }
        }
        if let Some(buffer) = buffer {
            expected.temporary_buffer = points(buffer);
        }
        assert_eq!(execution.step(), Ok(Progress::Continue(target)));
        assert_eq!(snapshot(&execution), expected);
        if point == '\0' && column == 7 && emitted.is_none() {
            assert_eq!(
                execution.context_mut().input.reconsume(),
                Err(CursorError::AlreadyReconsuming)
            );
        }
    }
    expected.current = Some((
        Input::Eof,
        Position {
            line: 1,
            column: 27,
        },
    ));
    expected.tokens.push(Token::Eof);
    // Ordinary ScriptData EOF adds no comment-like EOF diagnostic.
    for _ in 0..3 {
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&execution), expected);
    }
}

#[test]
fn closed_dash_dash_return_to_script_handles_null_crlf_and_ordinary_eof() {
    let mut context = seeded_context();
    context.input.push(points("-->\0\r\n")).unwrap();
    context.input.finish();
    let mut execution = Execution::new(State::ScriptDataDoubleEscaped, context);
    let mut expected = snapshot(&execution);
    for (column, point, target) in [
        (1, '-', State::ScriptDataDoubleEscapedDash),
        (2, '-', State::ScriptDataDoubleEscapedDashDash),
        (3, '>', State::ScriptData),
        (4, '\0', State::ScriptData),
        (5, '\n', State::ScriptData),
    ] {
        expected.state = target;
        expected.current = Some((Input::CodePoint(point.into()), Position { line: 1, column }));
        expected.tokens.push(Token::Character(if point == '\0' {
            expected
                .errors
                .push(error("unexpected-null-character", 1, 4));
            '\u{fffd}'.into()
        } else {
            point.into()
        }));
        assert_eq!(execution.step(), Ok(Progress::Continue(target)));
        assert_eq!(snapshot(&execution), expected);
    }
    expected.current = Some((Input::Eof, Position { line: 2, column: 1 }));
    expected.tokens.push(Token::Eof);
    for _ in 0..3 {
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&execution), expected);
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
    panic!("Script data double escaped failed to consume or terminate");
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
    let cases = [
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
        points("x\0\r\n-tail"),
        points("x\0\r\n<tail"),
        points("-<unconsumed\0"),
        points("<"),
        points("\r\n<\r\n"),
        points("\r-\0"),
        points("<!--"),
        points("</script>"),
        points("-a--\0<\0</X>--b</ScRiPt>-->"),
        points("-->\0\r\n"),
        points("<\r\n<\0<\r<\r\n"),
        points("</X>\r\n\0--a--\0</ScRiPt>-->"),
    ]
    .into_iter()
    .map(|input| (State::ScriptDataDoubleEscaped, input))
    .chain([
        (State::ScriptData, points("<!--<script>\0")),
        (State::ScriptData, points("\r\n<!--a<script>\0")),
        (State::ScriptData, points("<!---<script>\r\n")),
    ]);
    for (initial, input) in cases {
        let run = |chunks: Vec<&[CodePoint]>, closed: bool| {
            let mut execution = Execution::new(initial, seeded_context());
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
                    if drain(&mut execution, &mut trace, input.len() * 3 + 3) {
                        // Never feed, reconstruct, or resume a blocked execution.
                        return trace;
                    }
                    execution.context_mut().input.push([]).unwrap();
                    assert!(!drain(&mut execution, &mut trace, 1));
                }
                execution.context_mut().input.finish();
            }
            assert!(drain(&mut execution, &mut trace, input.len() * 3 + 3));
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
