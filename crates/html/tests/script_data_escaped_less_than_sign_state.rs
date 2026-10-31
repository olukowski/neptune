//! Pinned HTML 404b3eb14a7a936dddefb2d78e224dbeb01d2358, source
//! 143872–143892 (#script-data-escaped-less-than-sign-state).
//! Local source-step/streaming evidence, not upstream conformance runs.
use neptune_html::tokenizer::{
    Attribute, CodePoint, Context, Execution, Input, ParseError, PendingComment, PendingEndTag,
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

fn direct(mut context: Context) -> Execution {
    // Explicit entry setup: an escaped predecessor already consumed '<'.
    context.input.push(['<'.into()]).unwrap();
    assert_eq!(context.input.consume(), Input::CodePoint('<'.into()));
    Execution::new(State::ScriptDataEscapedLessThanSign, context)
}

#[test]
fn every_ascii_branch_and_non_scalars_preserve_exact_source_effects_and_spelling() {
    // Includes all 52 letters, their boundaries, punctuation, NUL and non-scalars.
    for value in (0..=0x7f).chain([
        0x80, 0x9f, 0xa0, 0xc0, 0xe9, 0x301, 0xd800, 0xdfff, 0xfdd0, 0xfeff, 0xfffd, 0x1f600,
        0x1fffe, 0x10ffff,
    ]) {
        for seeded in [false, true] {
            let mut run = direct(if seeded {
                seeded_context()
            } else {
                Context::default()
            });
            let point = CodePoint::new(value).unwrap();
            run.context_mut().input.push([point, '!'.into()]).unwrap();
            run.context_mut().input.finish();
            let mut expected = snapshot(&run);
            let normalized = if value == 0x0d { '\n'.into() } else { point };
            let alpha = matches!(value, 0x41..=0x5a | 0x61..=0x7a);
            expected.state = if value == 0x2f {
                State::ScriptDataEscapedEndTagOpen
            } else if alpha {
                State::ScriptDataDoubleEscapeStart
            } else {
                State::ScriptDataEscaped
            };
            expected.current = Some((
                Input::CodePoint(normalized),
                Position { line: 1, column: 2 },
            ));
            if value == 0x2f || alpha {
                expected.temporary_buffer.clear();
            }
            if value != 0x2f {
                expected.tokens.push(Token::Character('<'.into()));
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
                expected.input_errors.push(error(code, 1, 2));
            }
            assert_eq!(run.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&run), expected, "U+{value:04X}, seeded={seeded}");
            // Probe the source boundary only; never dispatch/reconstruct a stub.
            // Uppercase is still uppercase; no name or scratch append belongs here.
            if value != 0x2f {
                assert_eq!(
                    run.context_mut().input.consume(),
                    Input::CodePoint(normalized)
                );
                assert_eq!(snapshot(&run), expected);
            }
            assert_eq!(
                run.context_mut().input.consume(),
                Input::CodePoint('!'.into())
            );
            assert_eq!(
                run.context().input.position(),
                if matches!(value, 0x0a | 0x0d) {
                    Position { line: 2, column: 1 }
                } else {
                    Position { line: 1, column: 3 }
                }
            );
        }
    }
}

#[test]
fn slash_and_alpha_destinations_are_terminal_with_no_destination_effects() {
    for suffix in ["/", "/ScRiPt>", "A", "sCrIpT ", "Z\0"] {
        for finished in [false, true] {
            let mut run = direct(seeded_context());
            run.context_mut().input.push(points(suffix)).unwrap();
            if finished {
                run.context_mut().input.finish();
            }
            let mut expected = snapshot(&run);
            let slash = suffix.starts_with('/');
            expected.state = if slash {
                State::ScriptDataEscapedEndTagOpen
            } else {
                State::ScriptDataDoubleEscapeStart
            };
            expected.current = Some((
                Input::CodePoint(suffix.chars().next().unwrap().into()),
                Position { line: 1, column: 2 },
            ));
            expected.temporary_buffer.clear();
            if !slash {
                expected.tokens.push(Token::Character('<'.into()));
            }
            assert_eq!(run.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&run), expected);
            let blocked = Err(UnimplementedState {
                state: expected.state,
            });
            for _ in 0..3 {
                assert_eq!(run.step(), blocked);
                assert_eq!(snapshot(&run), expected);
            }
            // A terminality assertion, not an attempt to resume execution.
            run.context_mut().input.finish();
            assert_eq!(run.step(), blocked);
            assert_eq!(snapshot(&run), expected);
        }
    }
}

#[test]
fn open_exhaustion_and_reconsumed_eof_have_distinct_exact_effects() {
    for seeded in [false, true] {
        let mut run = direct(if seeded {
            seeded_context()
        } else {
            Context::default()
        });
        let mut expected = snapshot(&run);
        for _ in 0..3 {
            run.context_mut().input.push([]).unwrap();
            assert_eq!(run.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&run), expected);
        }
        run.context_mut().input.finish();
        expected.state = State::ScriptDataEscaped;
        expected.current = Some((Input::Eof, Position { line: 1, column: 2 }));
        expected.tokens.push(Token::Character('<'.into()));
        assert_eq!(run.step(), Ok(Progress::Continue(State::ScriptDataEscaped)));
        // No EOF token or error until the real escaped destination reconsumes EOF.
        assert_eq!(snapshot(&run), expected);
        expected.tokens.push(Token::Eof);
        expected
            .errors
            .push(error("eof-in-script-html-comment-like-text", 1, 2));
        for _ in 0..3 {
            assert_eq!(run.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&run), expected);
        }
    }
}

#[test]
fn fallback_reconsumption_defers_null_handling_and_never_duplicates_preprocessing() {
    for value in [0, 0x0d, 0x0b, 0x3c, 0x26, 0xc0, 0xd800, 0xfdd0, 0x10ffff] {
        let mut run = direct(seeded_context());
        let point = CodePoint::new(value).unwrap();
        run.context_mut().input.push([point]).unwrap();
        run.context_mut().input.finish();
        let mut expected = snapshot(&run);
        let normalized = if value == 0x0d { '\n'.into() } else { point };
        expected.state = State::ScriptDataEscaped;
        expected.current = Some((
            Input::CodePoint(normalized),
            Position { line: 1, column: 2 },
        ));
        expected.tokens.push(Token::Character('<'.into()));
        let code = match value {
            0x0b => Some("control-character-in-input-stream"),
            0xd800 => Some("surrogate-in-input-stream"),
            0xfdd0 | 0x10ffff => Some("noncharacter-in-input-stream"),
            _ => None,
        };
        if let Some(code) = code {
            expected.input_errors.push(error(code, 1, 2));
        }
        assert_eq!(run.step(), Ok(Progress::Continue(State::ScriptDataEscaped)));
        assert_eq!(snapshot(&run), expected);
        if value == 0x3c {
            expected.state = State::ScriptDataEscapedLessThanSign;
        } else {
            expected.tokens.push(Token::Character(if value == 0 {
                '\u{fffd}'.into()
            } else {
                normalized
            }));
            if value == 0 {
                expected
                    .errors
                    .push(error("unexpected-null-character", 1, 2));
            }
        }
        assert_eq!(run.step(), Ok(Progress::Continue(expected.state)));
        assert_eq!(snapshot(&run), expected);
        // Exact equality above includes the original diagnostic's multiplicity/position.
    }
}

#[test]
fn all_real_predecessors_reach_the_gateway_before_its_source_step() {
    for (prefix, predecessor) in [
        ("<!--x", State::ScriptDataEscaped),
        ("<!--x-", State::ScriptDataEscapedDash),
        ("<!--", State::ScriptDataEscapedDashDash),
    ] {
        for suffix in ["/ScRiPt>", "ScRiPt ", "\0", "!", ""] {
            let mut context = seeded_context();
            context.input.push(points(prefix)).unwrap();
            let mut run = Execution::new(State::ScriptData, context);
            for _ in 0..20 {
                if run.state() == predecessor
                    && run.context().input.current().map(|c| c.1.column) == Some(prefix.len())
                {
                    break;
                }
                assert!(matches!(run.step(), Ok(Progress::Continue(_))));
            }
            assert_eq!(run.state(), predecessor);
            run.context_mut().input.push(points("<")).unwrap();
            let mut expected = snapshot(&run);
            expected.state = State::ScriptDataEscapedLessThanSign;
            expected.current = Some((
                Input::CodePoint('<'.into()),
                Position {
                    line: 1,
                    column: prefix.len() + 1,
                },
            ));
            assert_eq!(run.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&run), expected);
            for _ in 0..2 {
                assert_eq!(run.step(), Ok(Progress::NeedMoreInput));
                assert_eq!(snapshot(&run), expected);
            }
            run.context_mut().input.push(points(suffix)).unwrap();
            run.context_mut().input.finish();
            let slash = suffix.starts_with('/');
            let alpha = suffix.starts_with('S');
            expected.state = if slash {
                State::ScriptDataEscapedEndTagOpen
            } else if alpha {
                State::ScriptDataDoubleEscapeStart
            } else {
                State::ScriptDataEscaped
            };
            expected.current = Some((
                suffix
                    .chars()
                    .next()
                    .map_or(Input::Eof, |p| Input::CodePoint(p.into())),
                Position {
                    line: 1,
                    column: prefix.len() + 2,
                },
            ));
            if slash || alpha {
                expected.temporary_buffer.clear();
            }
            if !slash {
                expected.tokens.push(Token::Character('<'.into()));
            }
            assert_eq!(run.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&run), expected);
            if slash || alpha {
                assert_eq!(
                    run.step(),
                    Err(UnimplementedState {
                        state: expected.state
                    })
                );
                assert_eq!(snapshot(&run), expected);
            }
        }
    }
}

#[test]
fn malformed_fallback_stream_has_exact_normalized_diagnostic_positions_and_multiplicities() {
    let mut run = direct(seeded_context());
    let mut input = points("\0\r\n\0");
    input.extend([0xd800, 0xfdd0, 0x0b].map(|v| CodePoint::new(v).unwrap()));
    input.extend(points("\r<"));
    run.context_mut().input.push(input).unwrap();
    run.context_mut().input.finish();
    let mut expected = snapshot(&run);
    expected.state = State::ScriptDataEscaped;
    expected.current = Some((Input::Eof, Position { line: 3, column: 2 }));
    expected.tokens.extend(
        points("<\u{fffd}\n\u{fffd}")
            .into_iter()
            .map(Token::Character),
    );
    expected
        .tokens
        .extend([0xd800, 0xfdd0, 0x0b].map(|v| Token::Character(CodePoint::new(v).unwrap())));
    expected
        .tokens
        .extend(points("\n<").into_iter().map(Token::Character));
    expected.tokens.push(Token::Eof);
    expected.errors.extend([
        error("unexpected-null-character", 1, 2),
        error("unexpected-null-character", 2, 1),
        error("eof-in-script-html-comment-like-text", 3, 2),
    ]);
    expected.input_errors.extend([
        error("surrogate-in-input-stream", 2, 2),
        error("noncharacter-in-input-stream", 2, 3),
        error("control-character-in-input-stream", 2, 4),
    ]);
    let mut steps = vec![];
    assert!(drain(&mut run, &mut steps));
    assert_eq!(steps.last().unwrap().0, Ok(Progress::Finished));
    assert_eq!(snapshot(&run), expected);
    for _ in 0..3 {
        assert_eq!(run.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&run), expected);
    }
}

type Trace = Vec<(Result<Progress, UnimplementedState>, Snapshot)>;

fn drain(run: &mut Execution, trace: &mut Trace) -> bool {
    for _ in 0..1000 {
        let before = snapshot(run);
        let result = run.step();
        if result == Ok(Progress::NeedMoreInput) {
            assert_eq!(snapshot(run), before);
            assert_eq!(run.step(), result);
            assert_eq!(snapshot(run), before);
            return false;
        }
        trace.push((result, snapshot(run)));
        if !matches!(result, Ok(Progress::Continue(_))) {
            return true;
        }
    }
    panic!("source-step limit");
}

fn trace(input: &[CodePoint], chunks: &[usize], real: bool) -> Trace {
    let mut run = if real {
        Execution::new(State::ScriptData, seeded_context())
    } else {
        direct(seeded_context())
    };
    let mut result = vec![];
    let mut offset = 0;
    for &length in chunks {
        run.context_mut()
            .input
            .push(input[offset..offset + length].iter().copied())
            .unwrap();
        offset += length;
        if drain(&mut run, &mut result) {
            // Stop immediately; never feed/reconstruct/resume a terminal stub.
            return result;
        }
    }
    assert_eq!(offset, input.len());
    run.context_mut().input.finish();
    assert!(drain(&mut run, &mut result));
    result
}

#[test]
fn every_two_chunk_split_and_single_point_chunking_match_every_source_snapshot() {
    let mut samples = vec![
        points(""),
        points("/ScRiPt>"),
        points("ScRiPt "),
        points("z"),
        points("!\0&>--><!"),
        points("<"),
        points("<!\r\n\0--><"),
        points("\r\n\0\r\r\n!<"),
        points("\r"),
        points("!<A"),
        points("!< /"),
        points("\0\0"),
        points("--><"),
    ];
    samples.push(
        [0xd800, 0xfdd0, 0x0b, 0x0d, 0x0a, 0, 0x10ffff]
            .into_iter()
            .map(|v| CodePoint::new(v).unwrap())
            .collect(),
    );
    for suffix in samples {
        for prefix in ["", "<!--", "<!--x", "<!--x-"] {
            let real = !prefix.is_empty();
            let mut input = points(prefix);
            if real {
                input.push('<'.into());
            }
            input.extend_from_slice(&suffix);
            let expected = trace(&input, &[input.len()], real);
            for split in 0..=input.len() {
                assert_eq!(
                    trace(&input, &[split, input.len() - split], real),
                    expected,
                    "{input:?} split {split}"
                );
            }
            assert_eq!(
                trace(&input, &vec![1; input.len()], real),
                expected,
                "{input:?} single points"
            );
        }
    }
}
