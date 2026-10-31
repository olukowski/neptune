//! Local branch evidence, not upstream conformance runs.
//! Pinned WHATWG HTML 404b3eb14a7a936dddefb2d78e224dbeb01d2358:
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143913-L143956>

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

fn attribute(name: &str, value: &str) -> Attribute {
    Attribute {
        name: points(name),
        value: points(value),
    }
}

// Fresh isolated entry, NEVER reconstruction after an UnimplementedState.
fn context(name: &str, spelling: &str, last: Option<&str>) -> Context {
    Context {
        current_tag: Some(TagKind::End),
        pending_end_tag: Some(PendingEndTag {
            name: points(name),
            self_closing: false,
            attributes: vec![],
            discarded_current_attribute: None,
        }),
        temporary_buffer: points(spelling),
        last_start_tag: last.map(points),
        pending_start_tag: Some(PendingStartTag {
            name: points("unrelated"),
            self_closing: true,
            attributes: vec![attribute("retained", "start")],
            discarded_current_attribute: Some(attribute("detached", "start")),
        }),
        pending_comment: Some(PendingComment {
            data: points("untouched"),
        }),
        return_state: Some(State::Rcdata),
        ..Context::default()
    }
}

fn execution(mut context: Context, input: &[CodePoint], finished: bool) -> Execution {
    context.input.push(input.iter().copied()).unwrap();
    if finished {
        context.input.finish();
    }
    Execution::new(State::ScriptDataEscapedEndTagName, context)
}

#[derive(Debug, Eq, PartialEq)]
struct Snapshot {
    state: State,
    current: Option<(Input, Position)>,
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
    input_errors: Vec<ParseError>,
    selected: Option<TagKind>,
    start: Option<PendingStartTag>,
    end: Option<PendingEndTag>,
    comment: Option<PendingComment>,
    buffer: Vec<CodePoint>,
    last: Option<Vec<CodePoint>>,
    return_state: Option<State>,
}

fn snapshot(execution: &Execution) -> Snapshot {
    let c = execution.context();
    Snapshot {
        state: execution.state(),
        current: c.input.current(),
        tokens: c.tokens.clone(),
        errors: c.errors.clone(),
        input_errors: c.input.errors().to_vec(),
        selected: c.current_tag,
        start: c.pending_start_tag.clone(),
        end: c.pending_end_tag.clone(),
        comment: c.pending_comment.clone(),
        buffer: c.temporary_buffer.clone(),
        last: c.last_start_tag.clone(),
        return_state: c.return_state,
    }
}

fn literal(buffer: &[CodePoint]) -> Vec<Token> {
    points("</")
        .into_iter()
        .chain(buffer.iter().copied())
        .map(Token::Character)
        .collect()
}

fn end_tag(attributes: Vec<Attribute>, self_closing: bool) -> Token {
    Token::EndTag {
        name: points("script"),
        attributes,
        self_closing,
    }
}

fn terminal(execution: &mut Execution, result: Result<Progress, UnimplementedState>) {
    let before = snapshot(execution);
    for _ in 0..3 {
        assert_eq!(execution.step(), result);
        assert_eq!(snapshot(execution), before);
    }
}

#[test]
fn initialized_end_and_original_spelling_are_required_entry_context_not_creation() {
    let default = Context::default();
    assert_eq!(default.current_tag, None);
    assert_eq!(default.pending_end_tag, None);
    assert!(default.temporary_buffer.is_empty());
    assert_eq!(default.last_start_tag, None);
    assert_eq!(
        PendingEndTag::default(),
        PendingEndTag {
            name: vec![],
            self_closing: false,
            attributes: vec![],
            discarded_current_attribute: None,
        }
    );

    let mut seeded = context("pre", "PrE", Some("script"));
    let end = seeded.pending_end_tag.as_mut().unwrap();
    end.self_closing = true;
    end.attributes = vec![attribute("a", "one"), attribute("b", "two")];
    end.discarded_current_attribute = Some(attribute("a", "detached"));
    seeded.tokens.push(Token::Character('!'.into()));
    seeded.errors.push(error("prior", 7, 2));
    let input = points("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz");
    let mut run = execution(seeded, &input, false);
    for (i, point) in input.into_iter().enumerate() {
        let mut expected = snapshot(&run);
        expected.current = Some((
            Input::CodePoint(point),
            Position {
                line: 1,
                column: i + 1,
            },
        ));
        let normalized = CodePoint::new(if point.value() <= 0x5a {
            point.value() + 0x20
        } else {
            point.value()
        })
        .unwrap();
        expected.end.as_mut().unwrap().name.push(normalized);
        expected.buffer.push(point);
        assert_eq!(
            run.step(),
            Ok(Progress::Continue(State::ScriptDataEscapedEndTagName))
        );
        assert_eq!(snapshot(&run), expected);
    }
    // Selection and both distinct names survive accumulation, not token creation.
    assert_eq!(run.context().current_tag, Some(TagKind::End));
    assert_eq!(
        run.context().pending_start_tag.as_ref().unwrap().name,
        points("unrelated")
    );
    let before = snapshot(&run);
    for _ in 0..3 {
        run.context_mut().input.push([]).unwrap();
        assert_eq!(run.step(), Ok(Progress::NeedMoreInput));
        assert_eq!(snapshot(&run), before);
    }
}

#[test]
fn isolated_entry_accepts_a_scheduled_first_letter_without_advancing_or_resetting_context() {
    let mut seeded = context("", "", Some("script"));
    seeded.input.push(points("ScRiPt>tail")).unwrap();
    seeded.input.finish();
    assert_eq!(seeded.input.consume(), Input::CodePoint('S'.into()));
    seeded.input.reconsume().unwrap();
    let mut run = Execution::new(State::ScriptDataEscapedEndTagName, seeded);
    let mut expected = snapshot(&run);
    expected.end.as_mut().unwrap().name.push('s'.into());
    expected.buffer.push('S'.into());
    assert_eq!(
        run.step(),
        Ok(Progress::Continue(State::ScriptDataEscapedEndTagName))
    );
    assert_eq!(snapshot(&run), expected);
    assert_eq!(
        run.context_mut().input.consume(),
        Input::CodePoint('c'.into())
    );

    // A preconsumed non-scalar gets no second preprocessing error when the
    // source reconsumes it, nor when its recovery schedules reconsumption again.
    let point = CodePoint::new(0xd800).unwrap();
    let mut seeded = context("script", "ScRiPt", Some("script"));
    seeded.input.push([point]).unwrap();
    assert_eq!(seeded.input.consume(), Input::CodePoint(point));
    seeded.input.reconsume().unwrap();
    let mut run = Execution::new(State::ScriptDataEscapedEndTagName, seeded);
    assert_eq!(run.step(), Ok(Progress::Continue(State::ScriptDataEscaped)));
    assert_eq!(
        run.context().input.errors(),
        [error("surrogate-in-input-stream", 1, 1)]
    );
    assert_eq!(run.context().tokens, literal(&points("ScRiPt")));
    assert!(run.context().errors.is_empty());
    let mut expected = snapshot(&run);
    expected.tokens.push(Token::Character(point));
    assert_eq!(run.step(), Ok(Progress::Continue(State::ScriptDataEscaped)));
    assert_eq!(snapshot(&run), expected);
    assert_eq!(run.step(), Ok(Progress::NeedMoreInput));
    assert_eq!(snapshot(&run), expected);
    assert_eq!(
        run.context().input.errors(),
        [error("surrogate-in-input-stream", 1, 1)]
    );
}

#[test]
fn each_appropriate_delimiter_consumes_once_and_only_greater_than_emits() {
    for (delimiter, target) in [
        ('\t', State::BeforeAttributeName),
        ('\n', State::BeforeAttributeName),
        ('\u{c}', State::BeforeAttributeName),
        (' ', State::BeforeAttributeName),
        ('\r', State::BeforeAttributeName),
        ('/', State::SelfClosingStartTag),
        ('>', State::Data),
    ] {
        let mut run = execution(
            context("script", "ScRiPt", Some("script")),
            &[delimiter.into(), '!'.into()],
            true,
        );
        let mut expected = snapshot(&run);
        expected.state = target;
        expected.current = Some((
            Input::CodePoint(if delimiter == '\r' { '\n' } else { delimiter }.into()),
            Position { line: 1, column: 1 },
        ));
        if delimiter == '>' {
            expected.end = None;
            expected.selected = None;
            expected.tokens.push(end_tag(vec![], false));
        }
        assert_eq!(run.step(), Ok(Progress::Continue(target)));
        assert_eq!(snapshot(&run), expected);
        assert_eq!(
            run.context_mut().input.consume(),
            Input::CodePoint('!'.into())
        );
        assert_eq!(
            run.context().input.position(),
            if matches!(delimiter, '\n' | '\r') {
                Position { line: 2, column: 1 }
            } else {
                Position { line: 1, column: 2 }
            }
        );
    }
}

#[test]
fn appropriate_means_exact_candidate_name_not_scratch_or_unemitted_storage() {
    for last in [
        None,
        Some("SCRIPT"),
        Some("scrip"),
        Some("scripts"),
        Some("scrıpt"),
        Some("unrelated"),
    ] {
        for delimiter in ['\t', '\n', '\u{c}', ' ', '\r', '/', '>'] {
            let mut run = execution(
                context("script", "ScRiPt", last),
                &[delimiter.into(), '!'.into()],
                true,
            );
            let mut expected = snapshot(&run);
            expected.state = State::ScriptDataEscaped;
            expected.current = Some((
                Input::CodePoint(if delimiter == '\r' { '\n' } else { delimiter }.into()),
                Position { line: 1, column: 1 },
            ));
            expected.tokens = literal(&expected.buffer);
            assert_eq!(run.step(), Ok(Progress::Continue(State::ScriptDataEscaped)));
            assert_eq!(snapshot(&run), expected);
            assert_eq!(
                run.context_mut().input.reconsume(),
                Err(CursorError::AlreadyReconsuming)
            );
            expected
                .tokens
                .push(Token::Character(match expected.current.unwrap().0 {
                    Input::CodePoint(point) => point,
                    _ => unreachable!(),
                }));
            assert_eq!(run.step(), Ok(Progress::Continue(State::ScriptDataEscaped)));
            assert_eq!(snapshot(&run), expected);
            expected.current = Some((
                Input::CodePoint('!'.into()),
                if matches!(delimiter, '\r' | '\n') {
                    Position { line: 2, column: 1 }
                } else {
                    Position { line: 1, column: 2 }
                },
            ));
            expected.tokens.push(Token::Character('!'.into()));
            assert_eq!(run.step(), Ok(Progress::Continue(State::ScriptDataEscaped)));
            assert_eq!(snapshot(&run), expected);
            expected.current = Some((
                Input::Eof,
                Position {
                    line: expected.current.unwrap().1.line,
                    column: expected.current.unwrap().1.column + 1,
                },
            ));
            expected.tokens.push(Token::Eof);
            expected.errors.push(ParseError {
                code: "eof-in-script-html-comment-like-text",
                position: expected.current.unwrap().1,
            });
            assert_eq!(run.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&run), expected);
            terminal(&mut run, Ok(Progress::Finished));
        }
    }
    // Seeded storage probes isolate the definition from scratch and scalar strings.
    for (name, spelling) in [("script", "different-scratch"), ("", ""), ("É", "É")] {
        let mut run = execution(context(name, spelling, Some(name)), &points(">"), true);
        assert_eq!(run.step(), Ok(Progress::Continue(State::Data)));
        assert_eq!(
            run.context().tokens,
            [Token::EndTag {
                name: points(name),
                attributes: vec![],
                self_closing: false,
            }]
        );
    }
    let non_scalar = CodePoint::new(0xd800).unwrap();
    let mut seeded = context("", "", None);
    seeded.pending_end_tag.as_mut().unwrap().name = vec![non_scalar];
    seeded.last_start_tag = Some(vec![non_scalar]);
    let mut run = execution(seeded, &points(">"), true);
    assert_eq!(run.step(), Ok(Progress::Continue(State::Data)));
    assert_eq!(
        run.context().tokens,
        [Token::EndTag {
            name: vec![non_scalar],
            attributes: vec![],
            self_closing: false,
        }]
    );
    assert!(run.context().input.errors().is_empty());
}

#[test]
fn scratch_is_emitted_in_order_without_preprocessing_and_without_including_mismatch() {
    let mut seeded = context("script", "ScRiPt", Some("script"));
    seeded
        .temporary_buffer
        .extend([0xd, 0, 0xd800, 0xfdd0].map(|v| CodePoint::new(v).unwrap()));
    let buffer = seeded.temporary_buffer.clone();
    let mut run = execution(seeded, &points("!tail"), true);
    assert_eq!(run.step(), Ok(Progress::Continue(State::ScriptDataEscaped)));
    assert_eq!(run.context().tokens, literal(&buffer));
    assert_eq!(run.context().temporary_buffer, buffer);
    assert!(run.context().errors.is_empty());
    assert!(run.context().input.errors().is_empty());
    assert_eq!(
        run.context_mut().input.reconsume(),
        Err(CursorError::AlreadyReconsuming)
    );
    let mut expected = snapshot(&run);
    for (i, point) in points("!tail").into_iter().enumerate() {
        expected.current = Some((
            Input::CodePoint(point),
            Position {
                line: 1,
                column: i + 1,
            },
        ));
        expected.tokens.push(Token::Character(point));
        assert_eq!(run.step(), Ok(Progress::Continue(State::ScriptDataEscaped)));
        assert_eq!(snapshot(&run), expected);
    }
    expected.current = Some((Input::Eof, Position { line: 1, column: 6 }));
    expected.tokens.push(Token::Eof);
    expected.errors.push(ParseError {
        code: "eof-in-script-html-comment-like-text",
        position: expected.current.unwrap().1,
    });
    assert_eq!(run.step(), Ok(Progress::Finished));
    assert_eq!(snapshot(&run), expected);
    terminal(&mut run, Ok(Progress::Finished));
}

#[test]
fn open_input_suspends_but_explicit_eof_recovers_literally_even_for_matching_name() {
    for (name, spelling, last) in [
        ("", "", None),
        ("script", "ScRiPt", Some("script")),
        ("other", "OtHeR", Some("script")),
    ] {
        let mut run = execution(context(name, spelling, last), &[], false);
        let before = snapshot(&run);
        for _ in 0..3 {
            run.context_mut().input.push([]).unwrap();
            assert_eq!(run.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&run), before);
        }
        run.context_mut().input.finish();
        run.context_mut().input.finish();
        let mut expected = before;
        expected.state = State::ScriptDataEscaped;
        expected.current = Some((Input::Eof, Position { line: 1, column: 1 }));
        expected.tokens = literal(&expected.buffer);
        assert_eq!(run.step(), Ok(Progress::Continue(State::ScriptDataEscaped)));
        assert_eq!(snapshot(&run), expected);
        assert_eq!(
            run.context_mut().input.reconsume(),
            Err(CursorError::AlreadyReconsuming)
        );
        expected.tokens.push(Token::Eof);
        expected.errors.push(ParseError {
            code: "eof-in-script-html-comment-like-text",
            position: expected.current.unwrap().1,
        });
        assert_eq!(run.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&run), expected);
        terminal(&mut run, Ok(Progress::Finished));
        assert_eq!(
            run.context_mut().input.push([]),
            Err(CursorError::InputAlreadyFinished)
        );
    }
    let mut run = execution(context("", "", Some("script")), &points("ScRiPt"), false);
    for _ in 0..6 {
        assert_eq!(
            run.step(),
            Ok(Progress::Continue(State::ScriptDataEscapedEndTagName))
        );
    }
    let before = snapshot(&run);
    assert_eq!(run.step(), Ok(Progress::NeedMoreInput));
    assert_eq!(snapshot(&run), before);
    run.context_mut().input.finish();
    assert_eq!(run.step(), Ok(Progress::Continue(State::ScriptDataEscaped)));
    assert_eq!(
        run.context().input.current(),
        Some((Input::Eof, Position { line: 1, column: 7 }))
    );
    assert_eq!(run.context().tokens, literal(&points("ScRiPt")));
    assert!(run.context().errors.is_empty());
    let mut expected = snapshot(&run);
    expected.tokens.push(Token::Eof);
    expected.errors.push(ParseError {
        code: "eof-in-script-html-comment-like-text",
        position: expected.current.unwrap().1,
    });
    assert_eq!(run.step(), Ok(Progress::Finished));
    assert_eq!(snapshot(&run), expected);
    terminal(&mut run, Ok(Progress::Finished));
}

#[test]
fn emission_uses_selected_tag_helper_including_both_independent_errors() {
    for retained in [false, true] {
        let mut seeded = context("script", "ScRiPt", Some("script"));
        let end = seeded.pending_end_tag.as_mut().unwrap();
        end.self_closing = true;
        let attributes = if retained {
            vec![attribute("a", "one"), attribute("b", "two")]
        } else {
            vec![]
        };
        end.attributes = attributes.clone();
        end.discarded_current_attribute = Some(attribute("a", "not-emitted"));
        seeded.tokens.push(Token::Character('!'.into()));
        seeded.errors.push(error("prior", 8, 3));
        seeded.input.push(points("\r\n")).unwrap();
        assert_eq!(seeded.input.consume(), Input::CodePoint('\n'.into()));
        assert_eq!(seeded.input.consume(), Input::NeedMoreInput);
        let mut run = execution(seeded, &points(">tail"), true);
        let mut expected = snapshot(&run);
        expected.state = State::Data;
        expected.current = Some((
            Input::CodePoint('>'.into()),
            Position { line: 2, column: 1 },
        ));
        expected.end = None;
        expected.selected = None;
        expected.tokens.push(end_tag(attributes, true));
        if retained {
            expected.errors.push(error("end-tag-with-attributes", 2, 1));
        }
        expected
            .errors
            .push(error("end-tag-with-trailing-solidus", 2, 1));
        assert_eq!(run.step(), Ok(Progress::Continue(State::Data)));
        assert_eq!(snapshot(&run), expected);
        assert_eq!(run.step(), Ok(Progress::Continue(State::Data)));
        assert_eq!(
            run.context().tokens.last(),
            Some(&Token::Character('t'.into()))
        );
    }
}

type Trace = Vec<(Result<Progress, UnimplementedState>, Snapshot)>;

fn drain(run: &mut Execution, trace: &mut Trace) -> bool {
    for _ in 0..512 {
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
            terminal(run, result);
            return true;
        }
    }
    panic!("execution did not suspend or terminate");
}

#[test]
fn existing_destinations_complete_attributes_flags_malformed_recovery_and_eof() {
    for (input, attributes, flag, errors) in [
        (">", vec![], false, vec![]),
        (" >", vec![], false, vec![]),
        (
            "/>",
            vec![],
            true,
            vec![error("end-tag-with-trailing-solidus", 1, 2)],
        ),
        (
            " A='x'>",
            vec![attribute("a", "x")],
            false,
            vec![error("end-tag-with-attributes", 1, 7)],
        ),
        (
            " A='x'/>",
            vec![attribute("a", "x")],
            true,
            vec![
                error("end-tag-with-attributes", 1, 8),
                error("end-tag-with-trailing-solidus", 1, 8),
            ],
        ),
        (
            "/!>",
            vec![attribute("!", "")],
            false,
            vec![
                error("unexpected-solidus-in-tag", 1, 2),
                error("end-tag-with-attributes", 1, 3),
            ],
        ),
        (
            " a='1' a='2'>",
            vec![attribute("a", "1")],
            false,
            vec![
                error("duplicate-attribute", 1, 9),
                error("end-tag-with-attributes", 1, 13),
            ],
        ),
        (
            " a='\0'>",
            vec![attribute("a", "\u{fffd}")],
            false,
            vec![
                error("unexpected-null-character", 1, 5),
                error("end-tag-with-attributes", 1, 7),
            ],
        ),
    ] {
        let mut run = execution(
            context("script", "ScRiPt", Some("script")),
            &points(input),
            true,
        );
        let before = snapshot(&run);
        let mut trace = vec![];
        assert!(drain(&mut run, &mut trace));
        assert_eq!(trace.last().unwrap().0, Ok(Progress::Finished), "{input:?}");
        assert_eq!(
            run.context().tokens,
            [end_tag(attributes, flag), Token::Eof],
            "{input:?}"
        );
        assert_eq!(run.context().errors, errors, "{input:?}");
        assert_eq!(run.context().pending_start_tag, before.start);
        assert_eq!(run.context().pending_comment, before.comment);
        assert_eq!(run.context().return_state, before.return_state);
        assert_eq!(run.context().last_start_tag, before.last);
        assert_eq!(run.context().temporary_buffer, before.buffer);
        assert_eq!(run.context().pending_end_tag, None);
        assert_eq!(run.context().current_tag, None);
    }
    for (input, expected) in [
        (" \r\n", error("eof-in-tag", 2, 1)),
        ("/", error("eof-in-tag", 1, 2)),
        (" a='", error("eof-in-tag", 1, 5)),
    ] {
        let mut run = execution(
            context("script", "ScRiPt", Some("script")),
            &points(input),
            true,
        );
        let mut trace = vec![];
        assert!(drain(&mut run, &mut trace));
        assert_eq!(trace.last().unwrap().0, Ok(Progress::Finished));
        assert_eq!(run.context().tokens, [Token::Eof]);
        assert_eq!(run.context().errors, [expected]);
        assert_eq!(run.context().pending_end_tag, None);
        assert_eq!(run.context().current_tag, None);
        assert_eq!(
            run.context().pending_start_tag.as_ref().unwrap().name,
            points("unrelated")
        );
    }
}

#[test]
fn all_splits_and_single_code_point_chunks_preserve_every_source_step() {
    let mut cases: Vec<Vec<CodePoint>> = [
        "",
        "ScRiPt",
        "ScRiPt>",
        "ScRiPt\t>",
        "ScRiPt\u{c}>",
        "ScRiPt/>",
        "ScRiPt A='x'/>",
        "ScRiPt a='1' a='2'>",
        "ScRiPt \r\n A='\0'>",
        "ScRiPt\r\r>",
        "ScRiPt\r",
        "ScRiPt/",
        "ScRiPt/!>",
        "ScRiPt a='&tail",
        "ScRiPt>&tail",
        "ScRiPt><!tail",
        "OtHeR>tail",
        "ScRiPtÉ>tail",
        "ScRiPt\0tail",
        "ScRiPt<tail",
        "ScRiPt!tail",
        "ScRiPt!--></ScRiPt>\0",
        "ScRiPt!\r\n--></ScRiPt>\0",
        "ScRiPt a='\r\nx'>",
        "ScRiPt a='",
    ]
    .into_iter()
    .map(points)
    .collect();
    for value in [0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0xb] {
        let mut input = points("ScRiPt");
        input.push(CodePoint::new(value).unwrap());
        input.extend(points("tail"));
        cases.push(input);
    }
    for input in cases {
        for last in [None, Some("script"), Some("SCRIPT")] {
            let mut closed = execution(context("", "", last), &input, true);
            let mut expected = vec![];
            assert!(drain(&mut closed, &mut expected));
            let mut chunkings: Vec<Vec<&[CodePoint]>> = (0..=input.len())
                .map(|split| vec![&input[..split], &input[split..]])
                .collect();
            chunkings.push(input.chunks(1).collect());
            for chunks in chunkings {
                let mut run = execution(context("", "", last), &[], false);
                let mut actual = vec![];
                assert!(!drain(&mut run, &mut actual));
                let mut ended = false;
                for chunk in chunks {
                    if ended {
                        break;
                    } // Never resume/reconstruct a terminal execution.
                    run.context_mut().input.push([]).unwrap();
                    run.context_mut().input.push(chunk.iter().copied()).unwrap();
                    ended = drain(&mut run, &mut actual);
                }
                if !ended {
                    run.context_mut().input.finish();
                    assert!(drain(&mut run, &mut actual));
                }
                assert_eq!(actual, expected, "input={input:?}, last={last:?}");
            }
        }
    }
}

#[test]
fn exhaustive_nonletters_fall_back_before_escaped_reconsumption_and_diagnostics() {
    let mut values: Vec<_> = (0..=0x7f)
        .filter(|v| !matches!(v, 0x41..=0x5a | 0x61..=0x7a))
        .collect();
    values.extend([
        0xa0, 0xc9, 0xe9, 0x130, 0x212a, 0x1f600, 0xd800, 0xdfff, 0xfdd0, 0x10ffff,
    ]);
    for value in values {
        for matching in [false, true] {
            if matching && matches!(value, 0x09 | 0x0a | 0x0c | 0x0d | 0x20 | 0x2f | 0x3e) {
                continue; // Successful delimiter branches have independent source snapshots.
            }
            let point = CodePoint::new(value).unwrap();
            let mut seeded = context(
                if matching { "script" } else { "scrip" },
                if matching { "ScRiPt" } else { "ScRiP" },
                Some("script"),
            );
            let end = seeded.pending_end_tag.as_mut().unwrap();
            end.self_closing = true;
            end.attributes = vec![attribute("a", "retained")];
            end.discarded_current_attribute = Some(attribute("a", "detached"));
            seeded.tokens.push(Token::Character('!'.into()));
            seeded.errors.push(error("prior", 7, 2));
            seeded.input.push(points("\r\n")).unwrap();
            assert_eq!(seeded.input.consume(), Input::CodePoint('\n'.into()));
            assert_eq!(seeded.input.consume(), Input::NeedMoreInput);
            let mut run = execution(seeded, &[point, '!'.into()], true);
            let mut expected = snapshot(&run);
            expected.state = State::ScriptDataEscaped;
            let normalized = if value == 0xd { '\n'.into() } else { point };
            expected.current = Some((
                Input::CodePoint(normalized),
                Position { line: 2, column: 1 },
            ));
            expected.tokens.extend(literal(&expected.buffer));
            let preprocessing = match value {
                0xd800..=0xdfff => Some("surrogate-in-input-stream"),
                0xfdd0 | 0x10ffff => Some("noncharacter-in-input-stream"),
                0x01..=0x08 | 0x0b | 0x0e..=0x1f | 0x7f => {
                    Some("control-character-in-input-stream")
                }
                _ => None,
            };
            if let Some(code) = preprocessing {
                expected.input_errors.push(error(code, 2, 1));
            }
            assert_eq!(
                run.step(),
                Ok(Progress::Continue(State::ScriptDataEscaped)),
                "{point:?}"
            );
            assert_eq!(snapshot(&run), expected);
            assert_eq!(
                run.context_mut().input.reconsume(),
                Err(CursorError::AlreadyReconsuming)
            );

            // The destination, not the assigned source, diagnoses/replaces NUL.
            expected.state = match value {
                0x3c => State::ScriptDataEscapedLessThanSign,
                0x2d => State::ScriptDataEscapedDash,
                _ => State::ScriptDataEscaped,
            };
            if value == 0 {
                expected.tokens.push(Token::Character('\u{fffd}'.into()));
                expected
                    .errors
                    .push(error("unexpected-null-character", 2, 1));
            } else if value != 0x3c {
                expected.tokens.push(Token::Character(normalized));
            }
            assert_eq!(run.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&run), expected);
            if value == 0x3c {
                let blocked = Err(UnimplementedState {
                    state: State::ScriptDataEscapedLessThanSign,
                });
                assert_eq!(run.step(), blocked);
                assert_eq!(snapshot(&run), expected);
                terminal(&mut run, blocked);
                continue;
            }
            expected.state = State::ScriptDataEscaped;
            let tail_position = if normalized == '\n'.into() {
                Position { line: 3, column: 1 }
            } else {
                Position { line: 2, column: 2 }
            };
            expected.current = Some((Input::CodePoint('!'.into()), tail_position));
            expected.tokens.push(Token::Character('!'.into()));
            assert_eq!(run.step(), Ok(Progress::Continue(State::ScriptDataEscaped)));
            assert_eq!(snapshot(&run), expected);
            let eof_position = Position {
                line: tail_position.line,
                column: tail_position.column + 1,
            };
            expected.current = Some((Input::Eof, eof_position));
            expected.tokens.push(Token::Eof);
            expected.errors.push(ParseError {
                code: "eof-in-script-html-comment-like-text",
                position: eof_position,
            });
            assert_eq!(run.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&run), expected);
            terminal(&mut run, Ok(Progress::Finished));
        }
    }
}

#[test]
fn later_stub_boundaries_are_terminal_without_consuming_tails_or_faking_eof() {
    for (input, state, current, column, emitted) in [
        ("<tail", State::ScriptDataEscapedLessThanSign, '<', 1, false),
        (">&tail", State::CharacterReference, '&', 2, true),
        (" a='&tail", State::CharacterReference, '&', 5, false),
        ("><!tail", State::MarkupDeclarationOpen, '!', 3, true),
    ] {
        let mut run = execution(
            context("script", "ScRiPt", Some("script")),
            &points(input),
            false,
        );
        let mut trace = vec![];
        assert!(drain(&mut run, &mut trace));
        let blocked = Err(UnimplementedState { state });
        assert_eq!(trace.last().unwrap().0, blocked);
        assert_eq!(
            run.context().input.current(),
            Some((
                Input::CodePoint(current.into()),
                Position { line: 1, column }
            ))
        );
        let tokens = if emitted {
            vec![end_tag(vec![], false)]
        } else if state == State::ScriptDataEscapedLessThanSign {
            literal(&points("ScRiPt"))
        } else {
            vec![]
        };
        assert_eq!(run.context().tokens, tokens);
        assert!(run.context().errors.is_empty());
        run.context_mut().input.push(points("more")).unwrap();
        run.context_mut().temporary_buffer.push('!'.into());
        terminal(&mut run, blocked);
        run.context_mut().input.finish();
        terminal(&mut run, blocked);
    }
}

#[test]
fn real_predecessor_block_traces_match_every_split_and_single_point_chunking() {
    for (initial, input, scratch) in [
        (State::ScriptData, "<!--</ScRiPt>tail", "stale"),
        (State::ScriptData, "<!--\r\n</ScRiPt>tail", "stale"),
        (State::ScriptDataEscaped, "</ScRiPt>tail", "stale"),
        (State::ScriptDataEscaped, "\r\n\0</ScRiPt>tail", "stale"),
        (
            State::ScriptDataDoubleEscapeStart,
            "ScRiPt>--a</ScRiPt>\r\n</ScRiPt>tail",
            "", // Explicit entry setup: the real predecessor would clear scratch.
        ),
    ] {
        let input = points(input);
        let mut closed_context = context("stale", scratch, Some("script"));
        closed_context.input.push(input.iter().copied()).unwrap();
        closed_context.input.finish();
        let mut closed = Execution::new(initial, closed_context);
        let mut expected = vec![];
        assert!(drain(&mut closed, &mut expected));
        assert_eq!(
            expected.last().unwrap().0,
            Err(UnimplementedState {
                state: State::ScriptDataEscapedLessThanSign,
            })
        );
        let mut chunkings: Vec<Vec<&[CodePoint]>> = (0..=input.len())
            .map(|split| vec![&input[..split], &input[split..]])
            .collect();
        chunkings.push(input.chunks(1).collect());
        for chunks in chunkings {
            let mut run = Execution::new(initial, context("stale", scratch, Some("script")));
            let mut actual = vec![];
            assert!(!drain(&mut run, &mut actual));
            let mut ended = false;
            for chunk in chunks {
                if ended {
                    break;
                }
                run.context_mut().input.push([]).unwrap();
                run.context_mut().input.push(chunk.iter().copied()).unwrap();
                ended = drain(&mut run, &mut actual);
            }
            assert!(ended);
            assert_eq!(actual, expected, "{initial:?}: {input:?}");
        }
    }
}

// Pinned source 143913–143956 (assigned fallback), 143775–143869
// (Escaped return), and 143654–143738 (ordinary Script data closing tag).
#[test]
fn literal_fallback_follows_real_escaped_return_and_ordinary_closing_tag_into_data() {
    let mut run = execution(
        context("script", "ScRiPt", Some("script")),
        &points("!--></ScRiPt>\0"),
        true,
    );
    let mut expected = snapshot(&run);
    expected.state = State::ScriptDataEscaped;
    expected.current = Some((
        Input::CodePoint('!'.into()),
        Position { line: 1, column: 1 },
    ));
    expected.tokens = literal(&points("ScRiPt"));
    assert_eq!(run.step(), Ok(Progress::Continue(State::ScriptDataEscaped)));
    assert_eq!(snapshot(&run), expected);
    assert_eq!(
        run.context_mut().input.reconsume(),
        Err(CursorError::AlreadyReconsuming)
    );
    for (point, column, target) in [
        ('!', 1, State::ScriptDataEscaped),
        ('-', 2, State::ScriptDataEscapedDash),
        ('-', 3, State::ScriptDataEscapedDashDash),
        ('>', 4, State::ScriptData),
    ] {
        expected.state = target;
        expected.current = Some((Input::CodePoint(point.into()), Position { line: 1, column }));
        expected.tokens.push(Token::Character(point.into()));
        assert_eq!(run.step(), Ok(Progress::Continue(target)));
        assert_eq!(snapshot(&run), expected);
    }
    let mut trace = vec![];
    assert!(drain(&mut run, &mut trace));
    assert_eq!(trace.last().unwrap().0, Ok(Progress::Finished));
    let mut tokens: Vec<_> = points("</ScRiPt!-->")
        .into_iter()
        .map(Token::Character)
        .collect();
    tokens.extend([
        end_tag(vec![], false),
        Token::Character('\0'.into()),
        Token::Eof,
    ]);
    assert_eq!(run.context().tokens, tokens);
    assert_eq!(
        run.context().errors,
        [error("unexpected-null-character", 1, 14)]
    );
    assert!(run.context().input.errors().is_empty());
    assert_eq!(
        run.context().input.current(),
        Some((
            Input::Eof,
            Position {
                line: 1,
                column: 15
            }
        ))
    );
    assert_eq!(run.state(), State::Data);
    assert_eq!(run.context().current_tag, None);
    assert_eq!(run.context().pending_end_tag, None);
    assert_eq!(run.context().temporary_buffer, points("ScRiPt"));
    assert_eq!(run.context().last_start_tag, expected.last);
    assert_eq!(run.context().pending_start_tag, expected.start);
    assert_eq!(run.context().pending_comment, expected.comment);
    assert_eq!(run.context().return_state, expected.return_state);
}

// The six merged algorithms are inherited, not implemented here. This real
// available entry path demonstrates that even their completed loop still stops
// BEFORE EscapedEndTagOpen/Name at the genuine EscapedLessThanSign gateway.
// Pinned source 143958–144142, then 143775–143802.
#[test]
fn merged_double_escaped_loop_returns_to_real_escaped_predecessor_and_blocks() {
    let mut seeded = context("stale", "", Some("script"));
    seeded
        .input
        .push(points("ScRiPt>--a</ScRiPt></ScRiPt>tail"))
        .unwrap();
    seeded.input.finish();
    let mut run = Execution::new(State::ScriptDataDoubleEscapeStart, seeded);
    let mut expected = snapshot(&run);
    for (index, (point, target)) in [
        ('S', State::ScriptDataDoubleEscapeStart),
        ('c', State::ScriptDataDoubleEscapeStart),
        ('R', State::ScriptDataDoubleEscapeStart),
        ('i', State::ScriptDataDoubleEscapeStart),
        ('P', State::ScriptDataDoubleEscapeStart),
        ('t', State::ScriptDataDoubleEscapeStart),
        ('>', State::ScriptDataDoubleEscaped),
        ('-', State::ScriptDataDoubleEscapedDash),
        ('-', State::ScriptDataDoubleEscapedDashDash),
        ('a', State::ScriptDataDoubleEscaped),
        ('<', State::ScriptDataDoubleEscapedLessThanSign),
        ('/', State::ScriptDataDoubleEscapeEnd),
        ('S', State::ScriptDataDoubleEscapeEnd),
        ('c', State::ScriptDataDoubleEscapeEnd),
        ('R', State::ScriptDataDoubleEscapeEnd),
        ('i', State::ScriptDataDoubleEscapeEnd),
        ('P', State::ScriptDataDoubleEscapeEnd),
        ('t', State::ScriptDataDoubleEscapeEnd),
        ('>', State::ScriptDataEscaped),
        ('<', State::ScriptDataEscapedLessThanSign),
    ]
    .into_iter()
    .enumerate()
    {
        let column = index + 1;
        expected.state = target;
        expected.current = Some((Input::CodePoint(point.into()), Position { line: 1, column }));
        if column != 20 {
            expected.tokens.push(Token::Character(point.into()));
        }
        if matches!(column, 1..=6 | 13..=18) {
            expected.buffer.push(point.to_ascii_lowercase().into());
        } else if column == 12 {
            expected.buffer.clear();
        }
        assert_eq!(run.step(), Ok(Progress::Continue(target)));
        assert_eq!(snapshot(&run), expected, "column {column}");
    }
    let blocked = Err(UnimplementedState {
        state: State::ScriptDataEscapedLessThanSign,
    });
    assert_eq!(run.step(), blocked);
    assert_eq!(snapshot(&run), expected);
    assert_eq!(
        run.context().pending_end_tag.as_ref().unwrap().name,
        points("stale")
    );
    assert_eq!(run.context().temporary_buffer, points("script"));
    terminal(&mut run, blocked);
}

#[test]
fn real_script_and_escaped_entries_stop_at_predecessor_not_the_implemented_name_state() {
    for (initial, input, column, prefix) in [
        (State::ScriptData, "<!--</ScRiPt>tail", 5, "<!--"),
        (State::ScriptDataEscaped, "</ScRiPt>tail", 1, ""),
    ] {
        let mut seeded = context("unrelated-end", "stale-buffer", Some("script"));
        seeded.input.push(points(input)).unwrap();
        seeded.input.finish();
        let mut run = Execution::new(initial, seeded);
        let before = snapshot(&run);
        let mut trace = vec![];
        assert!(drain(&mut run, &mut trace));
        let blocked = Err(UnimplementedState {
            state: State::ScriptDataEscapedLessThanSign,
        });
        assert_eq!(trace.last().unwrap().0, blocked);
        let mut expected = before;
        expected.state = State::ScriptDataEscapedLessThanSign;
        expected.tokens = points(prefix).into_iter().map(Token::Character).collect();
        expected.current = Some((Input::CodePoint('<'.into()), Position { line: 1, column }));
        assert_eq!(snapshot(&run), expected);
        terminal(&mut run, blocked);
    }
    // Even isolated predecessor entry remains a typed stub; it cannot create
    // a candidate or arrange first-letter reconsumption in this standalone PR.
    let mut seeded = context("stale", "scratch", Some("script"));
    seeded.input.push(points("ScRiPt>tail")).unwrap();
    seeded.input.finish();
    let mut run = Execution::new(State::ScriptDataEscapedEndTagOpen, seeded);
    let before = snapshot(&run);
    let blocked = Err(UnimplementedState {
        state: State::ScriptDataEscapedEndTagOpen,
    });
    assert_eq!(run.step(), blocked);
    assert_eq!(snapshot(&run), before);
    terminal(&mut run, blocked);
}
