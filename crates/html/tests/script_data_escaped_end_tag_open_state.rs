//! Pinned HTML 404b3eb14a7a936dddefb2d78e224dbeb01d2358, lines 143895–143911.
//! Inspect source effects before the real name destination. Predecessor less-than
//! sign remains a terminal stub; no blocked execution is reconstructed or resumed.

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
        name: points("name"),
        value: vec![CodePoint::new(0xd800).unwrap(), '\0'.into()],
    };
    Context {
        tokens: vec![Token::Character('p'.into())],
        errors: vec![error("prior-diagnostic", 1, 1)],
        last_start_tag: Some(points("script")),
        return_state: Some(State::Rcdata),
        current_tag: Some(TagKind::Start),
        pending_start_tag: Some(PendingStartTag {
            name: points("start"),
            self_closing: true,
            attributes: vec![attribute.clone()],
            discarded_current_attribute: Some(attribute.clone()),
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("stale-end"),
            self_closing: true,
            attributes: vec![attribute.clone()],
            discarded_current_attribute: Some(attribute),
        }),
        pending_comment: Some(PendingComment {
            data: points("comment"),
        }),
        // Normally cleared by the predecessor. This state must not clear/append it.
        temporary_buffer: vec![CodePoint::new(0xdfff).unwrap(), '\0'.into(), 'Z'.into()],
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
    last_start_tag: Option<Vec<CodePoint>>,
    return_state: Option<State>,
    current_tag: Option<TagKind>,
    start: Option<PendingStartTag>,
    end: Option<PendingEndTag>,
    comment: Option<PendingComment>,
    buffer: Vec<CodePoint>,
}

fn snapshot(execution: &Execution) -> Snapshot {
    let context = execution.context();
    Snapshot {
        state: execution.state(),
        current: context.input.current(),
        tokens: context.tokens.clone(),
        errors: context.errors.clone(),
        input_errors: context.input.errors().to_vec(),
        last_start_tag: context.last_start_tag.clone(),
        return_state: context.return_state,
        current_tag: context.current_tag,
        start: context.pending_start_tag.clone(),
        end: context.pending_end_tag.clone(),
        comment: context.pending_comment.clone(),
        buffer: context.temporary_buffer.clone(),
    }
}

// Position the isolated cursor, not a simulation of predecessor algorithms.
fn entry(mut context: Context, prefix: &str) -> Context {
    context.input.push(points(prefix)).unwrap();
    for point in points(&prefix.replace("\r\n", "\n").replace('\r', "\n")) {
        assert_eq!(context.input.consume(), Input::CodePoint(point));
    }
    context
}

fn assert_terminal_stub(execution: &mut Execution, state: State) {
    let before = snapshot(execution);
    for _ in 0..3 {
        assert_eq!(execution.step(), Err(UnimplementedState { state }));
        assert_eq!(snapshot(execution), before);
    }
}

#[test]
fn all_ascii_letters_create_empty_selected_end_tag_and_reconsume_without_normalizing() {
    for value in (0x41..=0x5a).chain(0x61..=0x7a) {
        for selected in [None, Some(TagKind::Start), Some(TagKind::End)] {
            for stale in [false, true] {
                for closed in [false, true] {
                    let point = CodePoint::new(value).unwrap();
                    let mut context = entry(seeded_context(), "\r\n</");
                    context.current_tag = selected;
                    if !stale {
                        context.pending_end_tag = None;
                    }
                    context.input.push([point, '!'.into()]).unwrap();
                    if closed {
                        context.input.finish();
                    }
                    let mut execution = Execution::new(State::ScriptDataEscapedEndTagOpen, context);
                    let mut expected = snapshot(&execution);
                    expected.state = State::ScriptDataEscapedEndTagName;
                    expected.current =
                        Some((Input::CodePoint(point), Position { line: 2, column: 3 }));
                    expected.current_tag = Some(TagKind::End);
                    expected.end = Some(PendingEndTag {
                        name: vec![],
                        self_closing: false,
                        attributes: vec![],
                        discarded_current_attribute: None,
                    });
                    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                    assert_eq!(snapshot(&execution), expected, "U+{value:04X}");
                    assert_eq!(
                        execution.context_mut().input.reconsume(),
                        Err(CursorError::AlreadyReconsuming)
                    );
                    // Only the following real name step normalizes/appends;
                    // the first letter is reconsumed at the SAME position.
                    expected.end.as_mut().unwrap().name.push(
                        CodePoint::new(if value <= 0x5a { value + 0x20 } else { value }).unwrap(),
                    );
                    expected.buffer.push(point);
                    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                    assert_eq!(snapshot(&execution), expected);
                    expected.state = State::ScriptDataEscaped;
                    expected.current = Some((
                        Input::CodePoint('!'.into()),
                        Position { line: 2, column: 4 },
                    ));
                    expected.tokens.extend(
                        points("</")
                            .into_iter()
                            .chain(expected.buffer.iter().copied())
                            .map(Token::Character),
                    );
                    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                    assert_eq!(snapshot(&execution), expected);
                    assert_eq!(
                        execution.context_mut().input.reconsume(),
                        Err(CursorError::AlreadyReconsuming)
                    );
                    expected.tokens.push(Token::Character('!'.into()));
                    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                    assert_eq!(snapshot(&execution), expected);
                    if !closed {
                        for _ in 0..3 {
                            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                            assert_eq!(snapshot(&execution), expected);
                        }
                        execution.context_mut().input.finish();
                    }
                    expected.current = Some((Input::Eof, Position { line: 2, column: 5 }));
                    expected.tokens.push(Token::Eof);
                    expected
                        .errors
                        .push(error("eof-in-script-html-comment-like-text", 2, 5));
                    for _ in 0..3 {
                        assert_eq!(execution.step(), Ok(Progress::Finished));
                        assert_eq!(snapshot(&execution), expected);
                    }
                }
            }
        }
    }
}

#[test]
fn all_non_alpha_ascii_and_non_scalars_fall_back_then_execute_real_escaped_step() {
    let inputs = (0..=0x7f)
        .filter(|v| !matches!(v, 0x41..=0x5a | 0x61..=0x7a))
        .chain([
            0x80, 0x9f, 0xa0, 0xc9, 0x3b1, 0xff21, 0xfeff, 0x1f600, 0xd800, 0xdfff, 0xfdd0,
            0x1fffe, 0x10ffff,
        ]);
    for value in inputs {
        for selected in [None, Some(TagKind::Start), Some(TagKind::End)] {
            for closed in [false, true] {
                let point = CodePoint::new(value).unwrap();
                let normalized = if value == 0x0d { '\n'.into() } else { point };
                let mut context = entry(seeded_context(), "\r\n</");
                context.current_tag = selected;
                context.input.push([point, '!'.into()]).unwrap();
                if closed {
                    context.input.finish();
                }
                let mut execution = Execution::new(State::ScriptDataEscapedEndTagOpen, context);
                let mut expected = snapshot(&execution);
                expected.state = State::ScriptDataEscaped;
                expected.current = Some((
                    Input::CodePoint(normalized),
                    Position { line: 2, column: 3 },
                ));
                expected
                    .tokens
                    .extend(points("</").into_iter().map(Token::Character));
                expected.input_errors = match value {
                    0xd800..=0xdfff => vec![error("surrogate-in-input-stream", 2, 3)],
                    0xfdd0 | 0x1fffe | 0x10ffff => {
                        vec![error("noncharacter-in-input-stream", 2, 3)]
                    }
                    0x01..=0x08 | 0x0b | 0x0e..=0x1f | 0x7f..=0x9f => {
                        vec![error("control-character-in-input-stream", 2, 3)]
                    }
                    _ => vec![],
                };
                assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                assert_eq!(snapshot(&execution), expected, "U+{value:04X}");
                assert_eq!(
                    execution.context_mut().input.reconsume(),
                    Err(CursorError::AlreadyReconsuming)
                );
                // The destination alone handles NUL or transitions on '-'/'<'.
                match value {
                    0x3c => expected.state = State::ScriptDataEscapedLessThanSign,
                    0x2d => {
                        expected.state = State::ScriptDataEscapedDash;
                        expected.tokens.push(Token::Character(point));
                    }
                    0 => {
                        expected
                            .errors
                            .push(error("unexpected-null-character", 2, 3));
                        expected.tokens.push(Token::Character('\u{fffd}'.into()));
                    }
                    _ => expected.tokens.push(Token::Character(normalized)),
                }
                assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                assert_eq!(snapshot(&execution), expected);
                if value == 0x3c {
                    assert_terminal_stub(&mut execution, State::ScriptDataEscapedLessThanSign);
                    continue;
                }
                let tail_position = if normalized == '\n'.into() {
                    Position { line: 3, column: 1 }
                } else {
                    Position { line: 2, column: 4 }
                };
                expected.state = State::ScriptDataEscaped;
                expected.current = Some((Input::CodePoint('!'.into()), tail_position));
                expected.tokens.push(Token::Character('!'.into()));
                assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
                assert_eq!(snapshot(&execution), expected);
                if !closed {
                    for _ in 0..3 {
                        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                        assert_eq!(snapshot(&execution), expected);
                    }
                    execution.context_mut().input.finish();
                }
                let eof_position = Position {
                    line: tail_position.line,
                    column: tail_position.column + 1,
                };
                expected.current = Some((Input::Eof, eof_position));
                expected.errors.push(ParseError {
                    code: "eof-in-script-html-comment-like-text",
                    position: eof_position,
                });
                expected.tokens.push(Token::Eof);
                for _ in 0..3 {
                    assert_eq!(execution.step(), Ok(Progress::Finished));
                    assert_eq!(snapshot(&execution), expected);
                }
            }
        }
    }
}

#[test]
fn open_exhaustion_is_not_eof_and_eof_is_reconsumed_before_destination_error() {
    for (prefix, position) in [
        ("", Position { line: 1, column: 1 }),
        ("</", Position { line: 1, column: 3 }),
        ("\r\n</", Position { line: 2, column: 3 }),
        ("\r", Position { line: 2, column: 1 }),
    ] {
        let mut execution = Execution::new(
            State::ScriptDataEscapedEndTagOpen,
            entry(seeded_context(), prefix),
        );
        let before = snapshot(&execution);
        for _ in 0..3 {
            execution.context_mut().input.push([]).unwrap();
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&execution), before);
        }
        execution.context_mut().input.finish();
        execution.context_mut().input.finish();
        let mut expected = before;
        expected.state = State::ScriptDataEscaped;
        expected.current = Some((Input::Eof, position));
        expected
            .tokens
            .extend(points("</").into_iter().map(Token::Character));
        assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
        assert_eq!(snapshot(&execution), expected);
        assert_eq!(
            execution.context_mut().input.reconsume(),
            Err(CursorError::AlreadyReconsuming)
        );
        expected.errors.push(ParseError {
            code: "eof-in-script-html-comment-like-text",
            position,
        });
        expected.tokens.push(Token::Eof);
        for _ in 0..3 {
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&execution), expected);
        }
    }
}

#[test]
fn fresh_direct_entry_can_resume_suspension_with_either_branch() {
    for text in ["A", "!", "\0"] {
        // Direct entry supplies an empty predecessor-cleared buffer, no candidate.
        let mut execution = Execution::new(State::ScriptDataEscapedEndTagOpen, Context::default());
        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
        execution.context_mut().input.push(points(text)).unwrap();
        let mut expected = snapshot(&execution);
        expected.current = Some((
            Input::CodePoint(text.chars().next().unwrap().into()),
            Position { line: 1, column: 1 },
        ));
        expected.state = if text == "A" {
            State::ScriptDataEscapedEndTagName
        } else {
            State::ScriptDataEscaped
        };
        if text == "A" {
            expected.current_tag = Some(TagKind::End);
            expected.end = Some(PendingEndTag {
                name: vec![],
                self_closing: false,
                attributes: vec![],
                discarded_current_attribute: None,
            });
        } else {
            expected
                .tokens
                .extend(points("</").into_iter().map(Token::Character));
        }
        assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
        assert_eq!(snapshot(&execution), expected);
        if text == "A" {
            expected.end.as_mut().unwrap().name.push('a'.into());
            expected.buffer.push('A'.into());
            assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&execution), expected);
            for _ in 0..3 {
                assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                assert_eq!(snapshot(&execution), expected);
            }
            execution.context_mut().input.finish();
            expected.state = State::ScriptDataEscaped;
            expected.current = Some((Input::Eof, Position { line: 1, column: 2 }));
            expected
                .tokens
                .extend(points("</A").into_iter().map(Token::Character));
            assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&execution), expected);
            assert_eq!(
                execution.context_mut().input.reconsume(),
                Err(CursorError::AlreadyReconsuming)
            );
            expected.tokens.push(Token::Eof);
            expected
                .errors
                .push(error("eof-in-script-html-comment-like-text", 1, 2));
            for _ in 0..3 {
                assert_eq!(execution.step(), Ok(Progress::Finished));
                assert_eq!(snapshot(&execution), expected);
            }
        } else {
            expected.tokens.push(Token::Character(if text == "\0" {
                '\u{fffd}'.into()
            } else {
                '!'.into()
            }));
            if text == "\0" {
                expected
                    .errors
                    .push(error("unexpected-null-character", 1, 1));
            }
            assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&execution), expected);
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&execution), expected);
            execution.context_mut().input.finish();
            expected.current = Some((Input::Eof, Position { line: 1, column: 2 }));
            expected
                .errors
                .push(error("eof-in-script-html-comment-like-text", 1, 2));
            expected.tokens.push(Token::Eof);
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&execution), expected);
        }
    }
}

// Real paths cannot reach the assigned state until the immediate predecessor is
// implemented. Assert the genuine boundary rather than injecting/resuming there.
#[test]
fn real_script_entry_stops_at_unimplemented_predecessor_without_consuming_solidus() {
    let mut context = seeded_context();
    context.input.push(points("<!--</A>")).unwrap();
    context.input.finish();
    let mut execution = Execution::new(State::ScriptData, context);
    for (state, point, column) in [
        (State::ScriptDataLessThanSign, '<', 1),
        (State::ScriptDataEscapeStart, '!', 2),
        (State::ScriptDataEscapeStartDash, '-', 3),
        (State::ScriptDataEscapedDashDash, '-', 4),
        (State::ScriptDataEscapedLessThanSign, '<', 5),
    ] {
        let mut expected = snapshot(&execution);
        expected.state = state;
        expected.current = Some((Input::CodePoint(point.into()), Position { line: 1, column }));
        match column {
            2 => expected
                .tokens
                .extend(points("<!").into_iter().map(Token::Character)),
            3 | 4 => expected.tokens.push(Token::Character('-'.into())),
            _ => {}
        }
        assert_eq!(execution.step(), Ok(Progress::Continue(state)));
        assert_eq!(snapshot(&execution), expected);
    }
    assert_terminal_stub(&mut execution, State::ScriptDataEscapedLessThanSign);
}

#[test]
fn repeated_nul_and_non_scalar_after_crlf_have_exact_once_only_diagnostics() {
    let surrogate = CodePoint::new(0xd800).unwrap();
    let mut context = entry(seeded_context(), "\r\n</");
    context
        .input
        .push([
            '\0'.into(),
            '\r'.into(),
            '\n'.into(),
            '\0'.into(),
            surrogate,
        ])
        .unwrap();
    context.input.finish();
    let mut execution = Execution::new(State::ScriptDataEscapedEndTagOpen, context);
    let mut expected = snapshot(&execution);
    expected.state = State::ScriptDataEscaped;
    expected.current = Some((
        Input::CodePoint('\0'.into()),
        Position { line: 2, column: 3 },
    ));
    expected
        .tokens
        .extend(points("</").into_iter().map(Token::Character));
    assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
    assert_eq!(snapshot(&execution), expected);
    for (input, emitted, line, column, nul) in [
        ('\0'.into(), '\u{fffd}'.into(), 2, 3, true),
        ('\n'.into(), '\n'.into(), 2, 4, false),
        ('\0'.into(), '\u{fffd}'.into(), 3, 1, true),
        (surrogate, surrogate, 3, 2, false),
    ] {
        expected.current = Some((Input::CodePoint(input), Position { line, column }));
        expected.tokens.push(Token::Character(emitted));
        if nul {
            expected
                .errors
                .push(error("unexpected-null-character", line, column));
        }
        if input == surrogate {
            expected
                .input_errors
                .push(error("surrogate-in-input-stream", line, column));
        }
        assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
        assert_eq!(snapshot(&execution), expected);
    }
    expected.current = Some((Input::Eof, Position { line: 3, column: 3 }));
    expected.tokens.push(Token::Eof);
    expected
        .errors
        .push(error("eof-in-script-html-comment-like-text", 3, 3));
    for _ in 0..3 {
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&execution), expected);
    }
}

// Real isolated open -> name entry. The helper checks creation BEFORE name
// execution, then every original-spelling/ASCII-normalized accumulation step.
fn through_name(name: &str, suffix: &[CodePoint], last: Option<&str>) -> Execution {
    let mut context = seeded_context();
    context.temporary_buffer.clear();
    context.last_start_tag = last.map(points);
    context
        .input
        .push(points(name).into_iter().chain(suffix.iter().copied()))
        .unwrap();
    context.input.finish();
    let mut run = Execution::new(State::ScriptDataEscapedEndTagOpen, context);
    let mut expected = snapshot(&run);
    expected.state = State::ScriptDataEscapedEndTagName;
    expected.current = Some((
        Input::CodePoint(name.chars().next().unwrap().into()),
        Position { line: 1, column: 1 },
    ));
    expected.current_tag = Some(TagKind::End);
    expected.end = Some(PendingEndTag {
        name: vec![],
        self_closing: false,
        attributes: vec![],
        discarded_current_attribute: None,
    });
    assert_eq!(run.step(), Ok(Progress::Continue(expected.state)));
    assert_eq!(snapshot(&run), expected);
    assert_eq!(
        run.context_mut().input.reconsume(),
        Err(CursorError::AlreadyReconsuming)
    );
    for (index, point) in name.chars().enumerate() {
        assert!(point.is_ascii_alphabetic());
        expected.current = Some((
            Input::CodePoint(point.into()),
            Position {
                line: 1,
                column: index + 1,
            },
        ));
        expected
            .end
            .as_mut()
            .unwrap()
            .name
            .push(point.to_ascii_lowercase().into());
        expected.buffer.push(point.into());
        assert_eq!(run.step(), Ok(Progress::Continue(expected.state)));
        assert_eq!(snapshot(&run), expected);
    }
    run
}

fn finish_real(run: &mut Execution) {
    let mut trace = vec![];
    assert!(drain(run, &mut trace, 200));
    assert_eq!(trace.last().unwrap().0, Ok(Progress::Finished));
}

#[test]
fn created_name_uses_exact_history_at_every_delimiter_before_actual_destination_execution() {
    for last in [Some("script"), None, Some("SCRIPT"), Some("scrip")] {
        for delimiter in ['\t', '\n', '\u{c}', ' ', '/', '>'] {
            let mut run = through_name("ScRiPt", &[delimiter.into()], last);
            let mut expected = snapshot(&run);
            expected.current = Some((
                Input::CodePoint(delimiter.into()),
                Position { line: 1, column: 7 },
            ));
            let appropriate = last == Some("script");
            expected.state = if appropriate {
                match delimiter {
                    '/' => State::SelfClosingStartTag,
                    '>' => State::Data,
                    _ => State::BeforeAttributeName,
                }
            } else {
                State::ScriptDataEscaped
            };
            if appropriate && delimiter == '>' {
                expected.current_tag = None;
                expected.end = None;
                expected.tokens.push(Token::EndTag {
                    name: points("script"),
                    attributes: vec![],
                    self_closing: false,
                });
            } else if !appropriate {
                expected
                    .tokens
                    .extend(points("</ScRiPt").into_iter().map(Token::Character));
            }
            assert_eq!(run.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&run), expected);
            if !appropriate {
                assert_eq!(
                    run.context_mut().input.reconsume(),
                    Err(CursorError::AlreadyReconsuming)
                );
                expected.tokens.push(Token::Character(delimiter.into()));
                assert_eq!(run.step(), Ok(Progress::Continue(State::ScriptDataEscaped)));
                assert_eq!(snapshot(&run), expected);
            }
            let position = if delimiter == '\n' {
                Position { line: 2, column: 1 }
            } else {
                Position { line: 1, column: 8 }
            };
            expected.current = Some((Input::Eof, position));
            expected.tokens.push(Token::Eof);
            if appropriate && delimiter != '>' {
                expected.current_tag = None;
                expected.end = None;
                expected.state = if delimiter == '/' {
                    State::SelfClosingStartTag
                } else {
                    State::AfterAttributeName
                };
                expected.errors.push(ParseError {
                    code: "eof-in-tag",
                    position,
                });
            } else if !appropriate {
                expected.errors.push(ParseError {
                    code: "eof-in-script-html-comment-like-text",
                    position,
                });
            }
            finish_real(&mut run);
            assert_eq!(snapshot(&run), expected, "{last:?}, {delimiter:?}");
        }
    }
}

#[test]
fn created_candidate_completes_real_attributes_duplicate_removal_nul_solidus_and_tag_eof() {
    for (suffix, attributes, self_closing, diagnostics, final_state, emitted) in [
        (
            " a='v' A=x />",
            vec![("a", "v")],
            true,
            vec![
                ("duplicate-attribute", 15),
                ("end-tag-with-attributes", 19),
                ("end-tag-with-trailing-solidus", 19),
            ],
            State::Data,
            true,
        ),
        (
            " a='\0' />",
            vec![("a", "\u{fffd}")],
            true,
            vec![
                ("unexpected-null-character", 11),
                ("end-tag-with-attributes", 15),
                ("end-tag-with-trailing-solidus", 15),
            ],
            State::Data,
            true,
        ),
        (
            " /!>",
            vec![("!", "")],
            false,
            vec![
                ("unexpected-solidus-in-tag", 9),
                ("end-tag-with-attributes", 10),
            ],
            State::Data,
            true,
        ),
        (
            " a='v'",
            vec![],
            false,
            vec![("eof-in-tag", 13)],
            State::AfterAttributeValueQuoted,
            false,
        ),
        (
            " a=",
            vec![],
            false,
            vec![("eof-in-tag", 10)],
            State::AttributeValueUnquoted,
            false,
        ),
        (
            ">\0",
            vec![],
            false,
            vec![("unexpected-null-character", 8)],
            State::Data,
            true,
        ),
    ] {
        let mut run = through_name("ScRiPt", &points(suffix), Some("script"));
        let mut expected = snapshot(&run);
        expected.state = final_state;
        expected.current = Some((
            Input::Eof,
            Position {
                line: 1,
                column: 7 + suffix.chars().count(),
            },
        ));
        expected.current_tag = None;
        expected.end = None;
        if emitted {
            expected.tokens.push(Token::EndTag {
                name: points("script"),
                attributes: attributes
                    .into_iter()
                    .map(|(name, value)| Attribute {
                        name: points(name),
                        value: points(value),
                    })
                    .collect(),
                self_closing,
            });
        }
        // After appropriate emission, Data emits original NUL, not U+FFFD.
        if suffix == ">\0" {
            expected.tokens.push(Token::Character('\0'.into()));
        }
        expected.tokens.push(Token::Eof);
        expected.errors.extend(
            diagnostics
                .into_iter()
                .map(|(code, column)| error(code, 1, column)),
        );
        finish_real(&mut run);
        assert_eq!(snapshot(&run), expected, "{suffix:?}");
    }
}

#[test]
fn created_name_eof_and_malformed_mismatch_fall_back_before_escaped_diagnostics() {
    for value in [
        None,
        Some(0),
        Some(0xd800),
        Some(0xfdd0),
        Some(0x0b),
        Some(0x21),
    ] {
        let suffix: Vec<_> = value
            .into_iter()
            .map(|value| CodePoint::new(value).unwrap())
            .collect();
        let mut run = through_name("ScRiPt", &suffix, Some("script"));
        let mut expected = snapshot(&run);
        expected.state = State::ScriptDataEscaped;
        expected.current = Some((
            suffix
                .first()
                .copied()
                .map(Input::CodePoint)
                .unwrap_or(Input::Eof),
            Position { line: 1, column: 7 },
        ));
        expected
            .tokens
            .extend(points("</ScRiPt").into_iter().map(Token::Character));
        match value {
            Some(0xd800) => expected
                .input_errors
                .push(error("surrogate-in-input-stream", 1, 7)),
            Some(0xfdd0) => expected
                .input_errors
                .push(error("noncharacter-in-input-stream", 1, 7)),
            Some(0x0b) => {
                expected
                    .input_errors
                    .push(error("control-character-in-input-stream", 1, 7))
            }
            _ => {}
        }
        assert_eq!(run.step(), Ok(Progress::Continue(expected.state)));
        assert_eq!(snapshot(&run), expected);
        assert_eq!(
            run.context_mut().input.reconsume(),
            Err(CursorError::AlreadyReconsuming)
        );
        if let Some(point) = suffix.first().copied() {
            expected.tokens.push(Token::Character(if value == Some(0) {
                '\u{fffd}'.into()
            } else {
                point
            }));
            if value == Some(0) {
                expected
                    .errors
                    .push(error("unexpected-null-character", 1, 7));
            }
            assert_eq!(run.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&run), expected);
        }
        let column = if value.is_some() { 8 } else { 7 };
        expected.current = Some((Input::Eof, Position { line: 1, column }));
        expected.tokens.push(Token::Eof);
        expected
            .errors
            .push(error("eof-in-script-html-comment-like-text", 1, column));
        finish_real(&mut run);
        assert_eq!(snapshot(&run), expected);
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
        match result {
            Err(UnimplementedState { state }) => {
                assert!(matches!(
                    state,
                    State::ScriptDataEscapedLessThanSign
                        | State::CharacterReference
                        | State::MarkupDeclarationOpen
                ));
                assert_terminal_stub(execution, state);
                return true;
            }
            Ok(Progress::Finished) => {
                let expected = snapshot(execution);
                assert_eq!(execution.step(), result);
                assert_eq!(snapshot(execution), expected);
                return true;
            }
            Ok(Progress::Continue(_)) => {}
            Ok(Progress::NeedMoreInput) => unreachable!(),
        }
    }
    panic!("source/destination steps failed to progress or terminate");
}

#[test]
fn every_two_chunk_split_and_single_point_chunking_matches_closed_source_trace() {
    let mut inputs = vec![vec![]];
    for text in [
        "Aname>tail",
        "z",
        "!",
        "\0",
        "&amp;",
        "<tail",
        " XMP>",
        "0></A>",
        "é",
        "😀",
        "\r\n",
        "\r\r\n\0\r",
        "\nA",
        "-->",
        "--!",
        "-<",
        "<!--",
        "ScRiPt>",
        "ScRiPt>\0",
        "ScRiPt",
        "ScRiP>",
        "ScRiPt/>",
        "ScRiPt a='v' A=duplicate/>",
        "ScRiPt a='\0' />",
        "ScRiPt a='&amp;'>",
        "ScRiPt a='v'",
        "ScRiPt\r\na=1>",
        "ScRiPt/!>",
        "ScRiPt> &amp;",
        "ScRiPt><!--",
    ] {
        inputs.push(points(text));
    }
    for value in [0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x01, 0x0b] {
        inputs.push(vec![CodePoint::new(value).unwrap(), '!'.into()]);
    }
    for prefix in ["", "</", "\r\n</", "\r"] {
        for input in &inputs {
            let run = |chunks: Vec<&[CodePoint]>, closed: bool, last: Option<&str>, stale: bool| {
                let mut context = entry(seeded_context(), prefix);
                context.last_start_tag = last.map(points);
                if !stale {
                    context.temporary_buffer.clear();
                }
                let mut execution = Execution::new(State::ScriptDataEscapedEndTagOpen, context);
                let mut trace = vec![];
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
                        if drain(&mut execution, &mut trace, input.len() * 3 + 10) {
                            // Never feed, reconstruct, or resume terminal execution.
                            return trace;
                        }
                        execution.context_mut().input.push([]).unwrap();
                        assert!(!drain(&mut execution, &mut trace, 1));
                    }
                    execution.context_mut().input.finish();
                }
                assert!(drain(&mut execution, &mut trace, input.len() * 3 + 10));
                trace
            };
            for last in [None, Some("script"), Some("SCRIPT")] {
                for stale in [false, true] {
                    let expected = run(vec![], true, last, stale);
                    for split in 0..=input.len() {
                        assert_eq!(
                            run(vec![&input[..split], &input[split..]], false, last, stale),
                            expected,
                            "{prefix:?}, {last:?}, stale={stale}, split {split} of {input:?}"
                        );
                    }
                    assert_eq!(run(input.chunks(1).collect(), false, last, stale), expected);
                }
            }
        }
    }
}
