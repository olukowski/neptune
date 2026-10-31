//! Local evidence, not upstream conformance. HTML 404b3eb14a7a936dddefb2d78e224dbeb01d2358
//! source 143958–143991; definitions and entry boundary in the state-local doc.
use neptune_html::tokenizer::{
    Attribute, CodePoint, Context, CursorError, Execution, Input, ParseError, PendingComment,
    PendingEndTag, PendingStartTag, Position, Progress, State, TagKind, Token, UnimplementedState,
};

fn points(s: &str) -> Vec<CodePoint> {
    s.chars().map(Into::into).collect()
}
fn error(code: &'static str, line: usize, column: usize) -> ParseError {
    ParseError {
        code,
        position: Position { line, column },
    }
}
fn seeded(buffer: Vec<CodePoint>) -> Context {
    let attribute = Attribute {
        name: points("a"),
        value: vec![CodePoint::new(0xd800).unwrap(), '\0'.into()],
    };
    Context {
        tokens: vec![Token::Character('p'.into())],
        errors: vec![error("prior", 1, 1)],
        last_start_tag: Some(points("unrelated")),
        return_state: Some(State::AttributeValueSingleQuoted),
        current_tag: Some(TagKind::End),
        pending_start_tag: Some(PendingStartTag {
            name: points("start"),
            self_closing: true,
            attributes: vec![attribute.clone()],
            discarded_current_attribute: Some(attribute.clone()),
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("end"),
            self_closing: true,
            attributes: vec![attribute.clone()],
            discarded_current_attribute: Some(attribute),
        }),
        pending_comment: Some(PendingComment {
            data: points("comment"),
        }),
        temporary_buffer: buffer,
        ..Default::default()
    }
}
#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    state: State,
    current: Option<(Input, Position)>,
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
    input_errors: Vec<ParseError>,
    last_start_tag: Option<Vec<CodePoint>>,
    return_state: Option<State>,
    current_tag: Option<TagKind>,
    start: Option<PendingStartTag>,
    end: Option<PendingEndTag>,
    comment: Option<PendingComment>,
    buffer: Vec<CodePoint>,
}
fn snapshot(e: &Execution) -> Snapshot {
    let c = e.context();
    Snapshot {
        state: e.state(),
        current: c.input.current(),
        tokens: c.tokens.clone(),
        errors: c.errors.clone(),
        input_errors: c.input.errors().to_vec(),
        last_start_tag: c.last_start_tag.clone(),
        return_state: c.return_state,
        current_tag: c.current_tag,
        start: c.pending_start_tag.clone(),
        end: c.pending_end_tag.clone(),
        comment: c.pending_comment.clone(),
        buffer: c.temporary_buffer.clone(),
    }
}
fn entry(buffer: Vec<CodePoint>) -> Execution {
    let mut context = seeded(buffer);
    // Cursor history only, not execution/simulation of the predecessor stub.
    context.input.push(points("\r\n<")).unwrap();
    assert_eq!(context.input.consume(), Input::CodePoint('\n'.into()));
    assert_eq!(context.input.consume(), Input::CodePoint('<'.into()));
    Execution::new(State::ScriptDataDoubleEscapeStart, context)
}
fn terminal_stub(e: &mut Execution, target: State) {
    let before = snapshot(e);
    assert_eq!(before.state, target);
    for _ in 0..3 {
        assert_eq!(e.step(), Err(UnimplementedState { state: target }));
        assert_eq!(snapshot(e), before);
    }
    assert!(!before.tokens.contains(&Token::Eof));
}

#[test]
fn exhaustive_source_branches_preserve_context_spelling_and_current_input() {
    for value in (0..=0x7f).chain([
        0x80, 0x9f, 0xa0, 0xc9, 0x130, 0x212a, 0xfeff, 0x1f600, 0xd800, 0xdfff, 0xfdd0, 0xffff,
        0x10ffff,
    ]) {
        for buffer in [
            points("script"),
            points("SCRIPT"),
            vec!['x'.into(), CodePoint::new(0xd800).unwrap(), '\0'.into()],
        ] {
            for selected in [None, Some(TagKind::Start), Some(TagKind::End)] {
                for finished in [false, true] {
                    let point = CodePoint::new(value).unwrap();
                    let normalized = if value == 0x0d { '\n'.into() } else { point };
                    let mut e = entry(buffer.clone());
                    e.context_mut().current_tag = selected;
                    e.context_mut().input.push([point, '!'.into()]).unwrap();
                    if finished {
                        e.context_mut().input.finish();
                    }
                    let mut expected = snapshot(&e);
                    expected.current = Some((
                        Input::CodePoint(normalized),
                        Position { line: 2, column: 2 },
                    ));
                    let diagnostic = match value {
                        0xd800..=0xdfff => Some("surrogate-in-input-stream"),
                        0xfdd0 | 0xffff | 0x10ffff => Some("noncharacter-in-input-stream"),
                        0x01..=0x08 | 0x0b | 0x0e..=0x1f | 0x7f..=0x9f => {
                            Some("control-character-in-input-stream")
                        }
                        _ => None,
                    };
                    if let Some(code) = diagnostic {
                        expected.input_errors.push(error(code, 2, 2));
                    }
                    let alpha = matches!(value, 0x41..=0x5a | 0x61..=0x7a);
                    let delimiter = matches!(value, 0x09 | 0x0a | 0x0c | 0x0d | 0x20 | 0x2f | 0x3e);
                    expected.state = if alpha {
                        State::ScriptDataDoubleEscapeStart
                    } else if delimiter && buffer == points("script") {
                        State::ScriptDataDoubleEscaped
                    } else {
                        State::ScriptDataEscaped
                    };
                    if alpha {
                        expected.buffer.push(
                            CodePoint::new(if value <= 0x5a { value + 0x20 } else { value })
                                .unwrap(),
                        );
                    }
                    if alpha || delimiter {
                        expected.tokens.push(Token::Character(normalized));
                    }
                    assert_eq!(
                        e.step(),
                        Ok(Progress::Continue(expected.state)),
                        "{value:x}"
                    );
                    assert_eq!(snapshot(&e), expected, "{value:x}");
                    // Inspect scheduled consumption before any destination execution.
                    // These probes never dispatch/reconstruct/resume a terminal run.
                    assert_eq!(
                        e.context_mut().input.consume(),
                        if alpha || delimiter {
                            Input::CodePoint('!'.into())
                        } else {
                            Input::CodePoint(normalized)
                        }
                    );
                    assert_eq!(e.context().input.errors(), expected.input_errors);
                    if !alpha && !delimiter {
                        assert_eq!(
                            e.context_mut().input.consume(),
                            Input::CodePoint('!'.into())
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn all_case_spellings_accumulate_lowercase_but_emit_original_and_decide_only_on_delimiter() {
    for mask in 0..64 {
        let spelling: String = "script"
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if mask & (1 << i) != 0 {
                    c.to_ascii_uppercase()
                } else {
                    c
                }
            })
            .collect();
        for delimiter in ['\t', '\n', '\x0c', ' ', '/', '>', '\r'] {
            let mut e = entry(vec![]);
            e.context_mut()
                .input
                .push(points(&format!("{spelling}{delimiter}TAIL")))
                .unwrap();
            e.context_mut().input.finish();
            let mut expected = snapshot(&e);
            for (i, point) in points(&spelling).into_iter().enumerate() {
                expected.current = Some((
                    Input::CodePoint(point),
                    Position {
                        line: 2,
                        column: i + 2,
                    },
                ));
                expected
                    .buffer
                    .push("script".chars().nth(i).unwrap().into());
                expected.tokens.push(Token::Character(point));
                assert_eq!(
                    e.step(),
                    Ok(Progress::Continue(State::ScriptDataDoubleEscapeStart))
                );
                assert_eq!(snapshot(&e), expected);
            }
            expected.state = State::ScriptDataDoubleEscaped;
            let normalized = if delimiter == '\r' { '\n' } else { delimiter };
            expected.current = Some((
                Input::CodePoint(normalized.into()),
                Position { line: 2, column: 8 },
            ));
            expected.tokens.push(Token::Character(normalized.into()));
            assert_eq!(
                e.step(),
                Ok(Progress::Continue(State::ScriptDataDoubleEscaped))
            );
            assert_eq!(snapshot(&e), expected);
            // Preserve the complete source snapshot above, then execute the
            // merged destination: it consumes TAIL, not the delimiter again.
            for (i, point) in points("TAIL").into_iter().enumerate() {
                expected.current = Some((
                    Input::CodePoint(point),
                    if normalized == '\n' {
                        Position {
                            line: 3,
                            column: i + 1,
                        }
                    } else {
                        Position {
                            line: 2,
                            column: i + 9,
                        }
                    },
                ));
                expected.tokens.push(Token::Character(point));
                assert_eq!(
                    e.step(),
                    Ok(Progress::Continue(State::ScriptDataDoubleEscaped))
                );
                assert_eq!(snapshot(&e), expected);
            }
            let eof_position = if normalized == '\n' {
                Position { line: 3, column: 5 }
            } else {
                Position {
                    line: 2,
                    column: 13,
                }
            };
            expected.current = Some((Input::Eof, eof_position));
            expected.tokens.push(Token::Eof);
            expected.errors.push(ParseError {
                code: "eof-in-script-html-comment-like-text",
                position: eof_position,
            });
            assert_eq!(e.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&e), expected);
            assert_eq!(e.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&e), expected);
        }
    }
}

#[test]
fn exact_buffer_equality_not_prefix_case_folding_or_tag_history() {
    for buffer in [
        points(""),
        points("scrip"),
        points("scriptt"),
        points("xscript"),
        points("SCRIPT"),
        points("script\0"),
        points("ſcript"),
        vec!['s'.into(), CodePoint::new(0xd800).unwrap()],
    ] {
        for delimiter in ['\t', '\n', '\x0c', ' ', '/', '>'] {
            let mut e = entry(buffer.clone());
            e.context_mut().last_start_tag = Some(points("script"));
            e.context_mut()
                .input
                .push([delimiter.into(), 'Z'.into()])
                .unwrap();
            let mut expected = snapshot(&e);
            expected.state = State::ScriptDataEscaped;
            expected.current = Some((
                Input::CodePoint(delimiter.into()),
                Position { line: 2, column: 2 },
            ));
            expected.tokens.push(Token::Character(delimiter.into()));
            assert_eq!(e.step(), Ok(Progress::Continue(State::ScriptDataEscaped)));
            assert_eq!(snapshot(&e), expected);
            expected.current = Some((
                Input::CodePoint('Z'.into()),
                if delimiter == '\n' {
                    Position { line: 3, column: 1 }
                } else {
                    Position { line: 2, column: 3 }
                },
            ));
            expected.tokens.push(Token::Character('Z'.into()));
            assert_eq!(e.step(), Ok(Progress::Continue(State::ScriptDataEscaped)));
            assert_eq!(snapshot(&e), expected);
        }
    }
}

#[test]
fn fallback_reconsumes_nul_and_non_scalars_without_duplicate_preprocessing_errors() {
    for (value, diagnostic) in [
        (0, None),
        (0xd800, Some("surrogate-in-input-stream")),
        (0xfdd0, Some("noncharacter-in-input-stream")),
        (0x0b, Some("control-character-in-input-stream")),
        (0x26, None),
    ] {
        let point = CodePoint::new(value).unwrap();
        let mut e = entry(points("script"));
        e.context_mut().input.push([point, '\0'.into()]).unwrap();
        e.context_mut().input.finish();
        let mut expected = snapshot(&e);
        expected.state = State::ScriptDataEscaped;
        expected.current = Some((Input::CodePoint(point), Position { line: 2, column: 2 }));
        if let Some(code) = diagnostic {
            expected.input_errors.push(error(code, 2, 2));
        }
        assert_eq!(e.step(), Ok(Progress::Continue(State::ScriptDataEscaped)));
        assert_eq!(snapshot(&e), expected); // No source emission, replacement, or state error.
        expected.tokens.push(Token::Character(if value == 0 {
            '\u{fffd}'.into()
        } else {
            point
        }));
        if value == 0 {
            expected
                .errors
                .push(error("unexpected-null-character", 2, 2));
        }
        assert_eq!(e.step(), Ok(Progress::Continue(State::ScriptDataEscaped)));
        assert_eq!(snapshot(&e), expected);
        expected.current = Some((
            Input::CodePoint('\0'.into()),
            Position { line: 2, column: 3 },
        ));
        expected.tokens.push(Token::Character('\u{fffd}'.into()));
        expected
            .errors
            .push(error("unexpected-null-character", 2, 3));
        assert_eq!(e.step(), Ok(Progress::Continue(State::ScriptDataEscaped)));
        assert_eq!(snapshot(&e), expected);
        expected.current = Some((Input::Eof, Position { line: 2, column: 4 }));
        expected
            .errors
            .push(error("eof-in-script-html-comment-like-text", 2, 4));
        expected.tokens.push(Token::Eof);
        assert_eq!(e.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&e), expected);
        assert_eq!(e.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&e), expected);
    }
}

#[test]
fn open_exhaustion_is_effect_free_eof_is_reconsumed_even_after_exact_script() {
    for text in ["", "scrip", "ScRiPt"] {
        let mut e = entry(vec![]);
        e.context_mut().input.push(points(text)).unwrap();
        for _ in text.chars() {
            assert_eq!(
                e.step(),
                Ok(Progress::Continue(State::ScriptDataDoubleEscapeStart))
            );
        }
        let before = snapshot(&e);
        for _ in 0..3 {
            e.context_mut().input.push([]).unwrap();
            assert_eq!(e.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&e), before);
        }
        e.context_mut().input.finish();
        let mut expected = before;
        expected.state = State::ScriptDataEscaped;
        expected.current = Some((
            Input::Eof,
            Position {
                line: 2,
                column: text.len() + 2,
            },
        ));
        assert_eq!(e.step(), Ok(Progress::Continue(State::ScriptDataEscaped)));
        assert_eq!(snapshot(&e), expected);
        assert_eq!(
            e.context_mut().input.reconsume(),
            Err(CursorError::AlreadyReconsuming)
        );
        expected.tokens.push(Token::Eof);
        expected.errors.push(error(
            "eof-in-script-html-comment-like-text",
            2,
            text.len() + 2,
        ));
        assert_eq!(e.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&e), expected);
        assert_eq!(e.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&e), expected);
    }
}

#[test]
fn isolated_entry_accepts_explicit_predecessor_contract_without_executing_stub() {
    let mut context = seeded(vec![]);
    // Explicit direct-entry setup, not a resumed or simulated predecessor run:
    // scratch is empty, '<' was emitted, first ASCII alpha is current/reconsuming.
    context.tokens.push(Token::Character('<'.into()));
    context.input.push(points("Script>")).unwrap();
    assert_eq!(context.input.consume(), Input::CodePoint('S'.into()));
    context.input.reconsume().unwrap();
    let mut e = Execution::new(State::ScriptDataDoubleEscapeStart, context);
    let mut expected = snapshot(&e);
    expected.tokens.push(Token::Character('S'.into()));
    expected.buffer.push('s'.into());
    assert_eq!(
        e.step(),
        Ok(Progress::Continue(State::ScriptDataDoubleEscapeStart))
    );
    assert_eq!(snapshot(&e), expected);
    assert_eq!(
        e.context_mut().input.consume(),
        Input::CodePoint('c'.into())
    );
}

#[test]
fn predecessor_remains_terminal_no_fake_real_entry() {
    let mut context = seeded(points("sentinel"));
    context.input.push(points("<!--<ScRiPt>tail")).unwrap();
    context.input.finish();
    let mut e = Execution::new(State::ScriptData, context);
    for state in [
        State::ScriptDataLessThanSign,
        State::ScriptDataEscapeStart,
        State::ScriptDataEscapeStartDash,
        State::ScriptDataEscapedDashDash,
        State::ScriptDataEscapedLessThanSign,
    ] {
        assert_eq!(e.step(), Ok(Progress::Continue(state)));
    }
    assert_eq!(
        e.context().input.current(),
        Some((
            Input::CodePoint('<'.into()),
            Position { line: 1, column: 5 }
        ))
    );
    assert_eq!(e.context().temporary_buffer, points("sentinel"));
    assert_eq!(
        e.context().tokens,
        [
            vec![Token::Character('p'.into())],
            points("<!--").into_iter().map(Token::Character).collect()
        ]
        .concat()
    );
    terminal_stub(&mut e, State::ScriptDataEscapedLessThanSign);
}

// Pinned HTML 143958–144142: fixed cross-state expectations, not a second
// tokenizer or support detection. Every row asserts the full source snapshot.
#[test]
fn real_double_escaped_loop_preserves_each_source_step_and_returns_via_escaped_to_script_data() {
    use State::*;
    let mut e = entry(vec![]);
    e.context_mut()
        .input
        .push(points("ScRiPt>-a--\0<\0</X>--b</ScRiPt>-->\0\r\n"))
        .unwrap();
    e.context_mut().input.finish();
    let mut expected = snapshot(&e);
    let rows = [
        ('S', 2, ScriptDataDoubleEscapeStart, Some('S'), "s", false),
        ('c', 3, ScriptDataDoubleEscapeStart, Some('c'), "sc", false),
        ('R', 4, ScriptDataDoubleEscapeStart, Some('R'), "scr", false),
        (
            'i',
            5,
            ScriptDataDoubleEscapeStart,
            Some('i'),
            "scri",
            false,
        ),
        (
            'P',
            6,
            ScriptDataDoubleEscapeStart,
            Some('P'),
            "scrip",
            false,
        ),
        (
            't',
            7,
            ScriptDataDoubleEscapeStart,
            Some('t'),
            "script",
            false,
        ),
        ('>', 8, ScriptDataDoubleEscaped, Some('>'), "script", false),
        (
            '-',
            9,
            ScriptDataDoubleEscapedDash,
            Some('-'),
            "script",
            false,
        ),
        ('a', 10, ScriptDataDoubleEscaped, Some('a'), "script", false),
        (
            '-',
            11,
            ScriptDataDoubleEscapedDash,
            Some('-'),
            "script",
            false,
        ),
        (
            '-',
            12,
            ScriptDataDoubleEscapedDashDash,
            Some('-'),
            "script",
            false,
        ),
        (
            '\0',
            13,
            ScriptDataDoubleEscaped,
            Some('\u{fffd}'),
            "script",
            true,
        ),
        (
            '<',
            14,
            ScriptDataDoubleEscapedLessThanSign,
            Some('<'),
            "script",
            false,
        ),
        ('\0', 15, ScriptDataDoubleEscaped, None, "script", false),
        (
            '\0',
            15,
            ScriptDataDoubleEscaped,
            Some('\u{fffd}'),
            "script",
            true,
        ),
        (
            '<',
            16,
            ScriptDataDoubleEscapedLessThanSign,
            Some('<'),
            "script",
            false,
        ),
        ('/', 17, ScriptDataDoubleEscapeEnd, Some('/'), "", false),
        ('X', 18, ScriptDataDoubleEscapeEnd, Some('X'), "x", false),
        ('>', 19, ScriptDataDoubleEscaped, Some('>'), "x", false),
        ('-', 20, ScriptDataDoubleEscapedDash, Some('-'), "x", false),
        (
            '-',
            21,
            ScriptDataDoubleEscapedDashDash,
            Some('-'),
            "x",
            false,
        ),
        ('b', 22, ScriptDataDoubleEscaped, Some('b'), "x", false),
        (
            '<',
            23,
            ScriptDataDoubleEscapedLessThanSign,
            Some('<'),
            "x",
            false,
        ),
        ('/', 24, ScriptDataDoubleEscapeEnd, Some('/'), "", false),
        ('S', 25, ScriptDataDoubleEscapeEnd, Some('S'), "s", false),
        ('c', 26, ScriptDataDoubleEscapeEnd, Some('c'), "sc", false),
        ('R', 27, ScriptDataDoubleEscapeEnd, Some('R'), "scr", false),
        ('i', 28, ScriptDataDoubleEscapeEnd, Some('i'), "scri", false),
        (
            'P',
            29,
            ScriptDataDoubleEscapeEnd,
            Some('P'),
            "scrip",
            false,
        ),
        (
            't',
            30,
            ScriptDataDoubleEscapeEnd,
            Some('t'),
            "script",
            false,
        ),
        ('>', 31, ScriptDataEscaped, Some('>'), "script", false),
        ('-', 32, ScriptDataEscapedDash, Some('-'), "script", false),
        (
            '-',
            33,
            ScriptDataEscapedDashDash,
            Some('-'),
            "script",
            false,
        ),
        ('>', 34, ScriptData, Some('>'), "script", false),
        ('\0', 35, ScriptData, Some('\u{fffd}'), "script", true),
        ('\n', 36, ScriptData, Some('\n'), "script", false),
    ];
    for (point, column, target, emitted, buffer, null_error) in rows {
        expected.state = target;
        expected.current = Some((Input::CodePoint(point.into()), Position { line: 2, column }));
        expected.buffer = points(buffer);
        if let Some(output) = emitted {
            expected.tokens.push(Token::Character(output.into()));
        }
        if null_error {
            expected
                .errors
                .push(error("unexpected-null-character", 2, column));
        }
        assert_eq!(e.step(), Ok(Progress::Continue(target)), "column {column}");
        assert_eq!(snapshot(&e), expected, "column {column}");
    }
    expected.current = Some((Input::Eof, Position { line: 3, column: 1 }));
    expected.tokens.push(Token::Eof);
    assert_eq!(e.step(), Ok(Progress::Finished));
    assert_eq!(snapshot(&e), expected); // ScriptData EOF has no comment-like error.
    assert_eq!(e.step(), Ok(Progress::Finished));
    assert_eq!(snapshot(&e), expected);
}

#[test]
fn entered_double_escape_end_reconsumes_eof_nul_and_non_scalars_only_in_real_destination() {
    for fallback in [
        None,
        Some('\0'.into()),
        Some(CodePoint::new(0xd800).unwrap()),
        Some('&'.into()),
    ] {
        let mut e = entry(vec![]);
        let prefix = points("script></script");
        e.context_mut().input.push(prefix.iter().copied()).unwrap();
        if let Some(point) = fallback {
            e.context_mut().input.push([point, 'z'.into()]).unwrap();
        }
        // The entire prefix is processed by real states, no stub/resumption.
        let mut expected = snapshot(&e);
        for target in [State::ScriptDataDoubleEscapeStart; 6]
            .into_iter()
            .chain([
                State::ScriptDataDoubleEscaped,
                State::ScriptDataDoubleEscapedLessThanSign,
            ])
            .chain([State::ScriptDataDoubleEscapeEnd; 7])
        {
            assert_eq!(e.step(), Ok(Progress::Continue(target)));
        }
        expected.state = State::ScriptDataDoubleEscapeEnd;
        expected.current = Some((
            Input::CodePoint('t'.into()),
            Position {
                line: 2,
                column: 16,
            },
        ));
        expected.buffer = points("script");
        expected
            .tokens
            .extend(prefix.into_iter().map(Token::Character));
        assert_eq!(snapshot(&e), expected);
        if fallback.is_none() {
            assert_eq!(e.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&e), expected);
        }
        e.context_mut().input.finish();
        expected.state = State::ScriptDataDoubleEscaped;
        expected.current = Some((
            fallback.map_or(Input::Eof, Input::CodePoint),
            Position {
                line: 2,
                column: 17,
            },
        ));
        if fallback.is_some_and(|point| point.value() == 0xd800) {
            expected
                .input_errors
                .push(error("surrogate-in-input-stream", 2, 17));
        }
        assert_eq!(
            e.step(),
            Ok(Progress::Continue(State::ScriptDataDoubleEscaped))
        );
        assert_eq!(snapshot(&e), expected); // No premature output or state error.
        assert_eq!(
            e.context_mut().input.reconsume(),
            Err(CursorError::AlreadyReconsuming)
        );
        if let Some(point) = fallback {
            expected
                .tokens
                .push(Token::Character(if point.value() == 0 {
                    '\u{fffd}'.into()
                } else {
                    point
                }));
            if point.value() == 0 {
                expected
                    .errors
                    .push(error("unexpected-null-character", 2, 17));
            }
            assert_eq!(
                e.step(),
                Ok(Progress::Continue(State::ScriptDataDoubleEscaped))
            );
            assert_eq!(snapshot(&e), expected);
            expected.current = Some((
                Input::CodePoint('z'.into()),
                Position {
                    line: 2,
                    column: 18,
                },
            ));
            expected.tokens.push(Token::Character('z'.into()));
            assert_eq!(
                e.step(),
                Ok(Progress::Continue(State::ScriptDataDoubleEscaped))
            );
            assert_eq!(snapshot(&e), expected);
        }
        let eof_column = if fallback.is_some() { 19 } else { 17 };
        expected.current = Some((
            Input::Eof,
            Position {
                line: 2,
                column: eof_column,
            },
        ));
        expected.tokens.push(Token::Eof);
        expected
            .errors
            .push(error("eof-in-script-html-comment-like-text", 2, eof_column));
        assert_eq!(e.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&e), expected);
        assert_eq!(e.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&e), expected);
    }
}

type Trace = Vec<(Result<Progress, UnimplementedState>, Snapshot)>;
fn drain(e: &mut Execution, trace: &mut Trace, bound: usize) -> bool {
    for _ in 0..bound {
        let before = snapshot(e);
        let result = e.step();
        if result == Ok(Progress::NeedMoreInput) {
            assert_eq!(snapshot(e), before);
            assert_eq!(e.step(), result);
            assert_eq!(snapshot(e), before);
            return false;
        }
        let terminal = !matches!(result, Ok(Progress::Continue(_)));
        trace.push((result, snapshot(e)));
        if terminal {
            let before = snapshot(e);
            assert_eq!(e.step(), result);
            assert_eq!(snapshot(e), before);
            return true;
        }
    }
    panic!("failed to consume/terminate");
}
#[test]
fn every_two_chunk_split_and_single_point_chunks_match_full_source_traces() {
    let mut malformed = points("ScRiPt\0\0\r\n&bad;\r\r\né😀");
    malformed.extend([0xd800, 0xdfff, 0xfdd0, 0x10ffff, 1].map(|v| CodePoint::new(v).unwrap()));
    for input in [
        vec![],
        points("script"),
        points("SCRIPT>tail"),
        points("scriptt>\r\n\0"),
        points("script/"),
        points("script\t"),
        points("script\x0c"),
        points("script "),
        points("script\r\nTAIL"),
        points("scrip\r\n\0\r\r\nZ\r"),
        points("script0<&tail"),
        points("script--><!tail"),
        points("ScRiPt<tail"),
        points("ScRiPt>-a--\0<\0</X>--b</ScRiPt>-->\0\r\n"),
        points("script>-->\0\r\n"),
        points("script></scriptt>\r\n\0</ScRiPt\r\n-->"),
        points("script></script"),
        points("script><"),
        points("script><\0</\0\r\n"),
        points("script>--<</X>\r\r\n\0-"),
        malformed,
    ] {
        for buffer in [vec![], points("s"), points("script")] {
            let run = |chunks: Vec<&[CodePoint]>, closed: bool| {
                let mut e = entry(buffer.clone());
                let mut trace = Vec::new();
                if closed {
                    e.context_mut().input.push(input.iter().copied()).unwrap();
                    e.context_mut().input.finish();
                } else {
                    for chunk in chunks {
                        e.context_mut().input.push(chunk.iter().copied()).unwrap();
                        if drain(&mut e, &mut trace, input.len() * 3 + 10) {
                            return trace;
                        }
                        e.context_mut().input.push([]).unwrap();
                        assert!(!drain(&mut e, &mut trace, 1));
                    }
                    e.context_mut().input.finish();
                }
                assert!(drain(&mut e, &mut trace, input.len() * 3 + 10));
                trace
            };
            let expected = run(vec![], true);
            for split in 0..=input.len() {
                assert_eq!(
                    run(vec![&input[..split], &input[split..]], false),
                    expected,
                    "{input:?} split {split}"
                );
            }
            assert_eq!(run(input.chunks(1).collect(), false), expected);
        }
    }
}
