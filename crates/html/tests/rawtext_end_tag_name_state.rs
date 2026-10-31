//! State-local evidence for pinned source lines 143608–143651; not upstream runs.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143608-L143651>

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

// Isolated entry, not a reconstruction/resumption of a blocked execution.
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
            attributes: vec![attribute("retained", "value")],
            discarded_current_attribute: Some(attribute("detached", "start")),
        }),
        pending_comment: Some(PendingComment {
            data: points("comment"),
        }),
        return_state: Some(State::Rcdata),
        ..Default::default()
    }
}

fn execution(mut context: Context, input: &[CodePoint], finished: bool) -> Execution {
    context.input.push(input.iter().copied()).unwrap();
    if finished {
        context.input.finish();
    }
    Execution::new(State::RawtextEndTagName, context)
}

#[derive(Debug, Eq, PartialEq)]
struct Snapshot {
    state: State,
    current: Option<(Input, Position)>,
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
    input_errors: Vec<ParseError>,
    current_tag: Option<TagKind>,
    end: Option<PendingEndTag>,
    start: Option<PendingStartTag>,
    comment: Option<PendingComment>,
    buffer: Vec<CodePoint>,
    last: Option<Vec<CodePoint>>,
    return_state: Option<State>,
}

fn snapshot(execution: &Execution) -> Snapshot {
    let context = execution.context();
    Snapshot {
        state: execution.state(),
        current: context.input.current(),
        tokens: context.tokens.clone(),
        errors: context.errors.clone(),
        input_errors: context.input.errors().to_vec(),
        current_tag: context.current_tag,
        end: context.pending_end_tag.clone(),
        start: context.pending_start_tag.clone(),
        comment: context.pending_comment.clone(),
        buffer: context.temporary_buffer.clone(),
        last: context.last_start_tag.clone(),
        return_state: context.return_state,
    }
}

fn literal(buffer: &[CodePoint]) -> Vec<Token> {
    points("</")
        .into_iter()
        .chain(buffer.iter().copied())
        .map(Token::Character)
        .collect()
}

fn terminal(execution: &mut Execution, result: Result<Progress, UnimplementedState>) {
    let before = snapshot(execution);
    for _ in 0..3 {
        assert_eq!(execution.step(), result);
        assert_eq!(snapshot(execution), before);
    }
}

#[test]
fn creation_defaults_and_all_ascii_letters_append_without_replacing_storage() {
    let default = Context::default();
    assert!(default.temporary_buffer.is_empty());
    assert!(default.pending_end_tag.is_none());
    assert!(default.current_tag.is_none());
    assert!(default.last_start_tag.is_none());
    assert_eq!(
        PendingEndTag::default(),
        PendingEndTag {
            name: vec![],
            self_closing: false,
            attributes: vec![],
            discarded_current_attribute: None,
        }
    );
    let input = points("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz");
    let mut context = context("pre", "PrE", Some("style"));
    // Preexisting code points are storage, not fresh input to preprocess/fold.
    context
        .pending_end_tag
        .as_mut()
        .unwrap()
        .name
        .push(CodePoint::new(0xd800).unwrap());
    context
        .temporary_buffer
        .push(CodePoint::new(0xfdd0).unwrap());
    let end = context.pending_end_tag.as_mut().unwrap();
    end.self_closing = true;
    end.attributes = vec![attribute("a", "b"), attribute("c", "d")];
    end.discarded_current_attribute = Some(attribute("a", "detached"));
    context.tokens.push(Token::Character('!'.into()));
    context.errors.push(error("prior", 9, 2));
    let mut execution = execution(context, &input, false);
    for (i, point) in input.into_iter().enumerate() {
        let mut expected = snapshot(&execution);
        expected.current = Some((
            Input::CodePoint(point),
            Position {
                line: 1,
                column: i + 1,
            },
        ));
        expected.end.as_mut().unwrap().name.push(
            CodePoint::new(if point.value() <= 0x5a {
                point.value() + 0x20
            } else {
                point.value()
            })
            .unwrap(),
        );
        expected.buffer.push(point);
        assert_eq!(
            execution.step(),
            Ok(Progress::Continue(State::RawtextEndTagName))
        );
        assert_eq!(snapshot(&execution), expected);
    }
    let before = snapshot(&execution);
    for _ in 0..3 {
        execution.context_mut().input.push([]).unwrap();
        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
        assert_eq!(snapshot(&execution), before);
    }
}

#[test]
fn appropriate_delimiters_are_consumed_and_only_greater_than_emits() {
    for (delimiter, target) in [
        ('\t', State::BeforeAttributeName),
        ('\n', State::BeforeAttributeName),
        ('\u{c}', State::BeforeAttributeName),
        (' ', State::BeforeAttributeName),
        ('\r', State::BeforeAttributeName),
        ('/', State::SelfClosingStartTag),
        ('>', State::Data),
    ] {
        let mut execution = execution(
            context("style", "StYlE", Some("style")),
            &[delimiter.into(), '!'.into()],
            true,
        );
        let mut expected = snapshot(&execution);
        expected.state = target;
        expected.current = Some((
            Input::CodePoint(if delimiter == '\r' { '\n' } else { delimiter }.into()),
            Position { line: 1, column: 1 },
        ));
        if delimiter == '>' {
            expected.end = None;
            expected.current_tag = None;
            expected.tokens.push(Token::EndTag {
                name: points("style"),
                attributes: vec![],
                self_closing: false,
            });
        }
        assert_eq!(execution.step(), Ok(Progress::Continue(target)));
        assert_eq!(snapshot(&execution), expected);
        // Cursor-only inspection proves no reconsumption and no overconsumption.
        assert_eq!(
            execution.context_mut().input.consume(),
            Input::CodePoint('!'.into())
        );
        assert_eq!(
            execution.context().input.position(),
            Position {
                line: if matches!(delimiter, '\r' | '\n') {
                    2
                } else {
                    1
                },
                column: if matches!(delimiter, '\r' | '\n') {
                    1
                } else {
                    2
                }
            }
        );
    }
}

#[test]
fn appropriateness_is_exact_name_equality_with_a_prior_emitted_start_tag() {
    for last in [
        None,
        Some("STYLE"),
        Some("styl"),
        Some("styles"),
        Some("stýle"),
        Some("other"),
    ] {
        for delimiter in ['\t', '\n', '\u{c}', ' ', '\r', '/', '>'] {
            let mut execution = execution(
                context("style", "StYlE", last),
                &[delimiter.into(), '!'.into()],
                true,
            );
            let mut expected = snapshot(&execution);
            expected.state = State::Rawtext;
            expected.current = Some((
                Input::CodePoint(if delimiter == '\r' { '\n' } else { delimiter }.into()),
                Position { line: 1, column: 1 },
            ));
            expected.tokens = literal(&expected.buffer);
            assert_eq!(execution.step(), Ok(Progress::Continue(State::Rawtext)));
            assert_eq!(snapshot(&execution), expected);
            assert_eq!(execution.step(), Ok(Progress::Continue(State::Rawtext)));
            assert_eq!(
                execution.context().tokens.last(),
                Some(&Token::Character(
                    if delimiter == '\r' { '\n' } else { delimiter }.into()
                ))
            );
            assert_eq!(execution.context().input.current(), expected.current);
            assert_eq!(
                execution.context_mut().input.consume(),
                Input::CodePoint('!'.into())
            );
        }
    }
    // Comparison uses the normalized candidate, never original spelling; it is
    // code-point equality, not Unicode folding or scalar-string conversion.
    for (name, spelling) in [("style", "NOT-THE-NAME"), ("", ""), ("É", "É")] {
        let mut execution = execution(context(name, spelling, Some(name)), &points(">"), true);
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
        assert_eq!(
            execution.context().tokens,
            [Token::EndTag {
                name: points(name),
                attributes: vec![],
                self_closing: false
            }]
        );
    }
}

#[test]
fn anything_else_recovers_literally_without_source_errors_or_name_mutation() {
    let mut inputs: Vec<_> = (0..=0x7f)
        .filter(|v| !matches!(v, 0x41..=0x5a | 0x61..=0x7a))
        .map(|v| CodePoint::new(v).unwrap())
        .collect();
    inputs.extend(
        [
            0xa0, 0xc9, 0xe9, 0x130, 0x212a, 0x1f600, 0xd800, 0xdfff, 0xfdd0, 0x10ffff,
        ]
        .map(|v| CodePoint::new(v).unwrap()),
    );
    for point in inputs {
        // Mismatching name ensures delimiters also take literal fallback.
        let mut context = context("styl", "StYl", Some("style"));
        context
            .pending_end_tag
            .as_mut()
            .unwrap()
            .discarded_current_attribute = Some(attribute("kept", "detached"));
        let mut execution = execution(context, &[point, '!'.into()], true);
        let mut expected = snapshot(&execution);
        expected.state = State::Rawtext;
        let normalized = if point.value() == 0xd {
            '\n'.into()
        } else {
            point
        };
        expected.current = Some((
            Input::CodePoint(normalized),
            Position { line: 1, column: 1 },
        ));
        expected.tokens = literal(&expected.buffer);
        let preprocessing = match point.value() {
            0xd800..=0xdfff => Some("surrogate-in-input-stream"),
            0xfdd0 | 0x10ffff => Some("noncharacter-in-input-stream"),
            0x01..=0x08 | 0x0b | 0x0e..=0x1f | 0x7f => Some("control-character-in-input-stream"),
            _ => None,
        };
        if let Some(code) = preprocessing {
            expected.input_errors.push(error(code, 1, 1));
        }
        assert_eq!(
            execution.step(),
            Ok(Progress::Continue(State::Rawtext)),
            "{point:?}"
        );
        assert_eq!(snapshot(&execution), expected);
        // Destination, not the source, handles NUL and '<'. Ampersand is literal.
        let target = if point.value() == 0x3c {
            State::RawtextLessThanSign
        } else {
            State::Rawtext
        };
        assert_eq!(execution.step(), Ok(Progress::Continue(target)));
        expected.state = target;
        if point.value() == 0 {
            expected.tokens.push(Token::Character('\u{fffd}'.into()));
            expected
                .errors
                .push(error("unexpected-null-character", 1, 1));
        } else if point.value() != 0x3c {
            expected.tokens.push(Token::Character(normalized));
        }
        assert_eq!(snapshot(&execution), expected);
        if target == State::RawtextLessThanSign {
            terminal(&mut execution, Err(UnimplementedState { state: target }));
        }
        assert_eq!(
            execution.context_mut().input.consume(),
            Input::CodePoint('!'.into())
        );
    }
    // Even an appropriate candidate falls back on non-delimiters, not just a
    // mismatching one. Non-ASCII letters must not extend a candidate name.
    for value in [
        0, 0x30, 0x40, 0x5b, 0x60, 0x7b, 0xa0, 0xc9, 0x130, 0x212a, 0xd800,
    ] {
        let point = CodePoint::new(value).unwrap();
        let mut execution = execution(context("style", "StYlE", Some("style")), &[point], true);
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Rawtext)));
        assert_eq!(
            execution.context().pending_end_tag.as_ref().unwrap().name,
            points("style")
        );
        assert_eq!(execution.context().temporary_buffer, points("StYlE"));
        assert_eq!(execution.context().tokens, literal(&points("StYlE")));
        assert!(execution.context().errors.is_empty());
    }
}

#[test]
fn open_exhaustion_suspends_but_eof_falls_back_and_is_reconsumed_in_rawtext() {
    for (name, spelling, last) in [
        ("", "", None),
        ("style", "StYlE", Some("style")),
        ("other", "OtHeR", Some("style")),
    ] {
        let mut execution = execution(context(name, spelling, last), &[], false);
        let before = snapshot(&execution);
        for _ in 0..3 {
            execution.context_mut().input.push([]).unwrap();
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&execution), before);
        }
        execution.context_mut().input.finish();
        execution.context_mut().input.finish();
        let mut expected = before;
        expected.state = State::Rawtext;
        expected.current = Some((Input::Eof, Position { line: 1, column: 1 }));
        expected.tokens = literal(&expected.buffer);
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Rawtext)));
        assert_eq!(snapshot(&execution), expected);
        assert_eq!(execution.step(), Ok(Progress::Finished));
        expected.tokens.push(Token::Eof);
        assert_eq!(snapshot(&execution), expected);
        assert_eq!(
            execution.context_mut().input.push([]),
            Err(CursorError::InputAlreadyFinished)
        );
        terminal(&mut execution, Ok(Progress::Finished));
    }
    let mut execution = execution(context("", "", Some("style")), &points("StYlE"), false);
    for _ in 0..5 {
        assert_eq!(
            execution.step(),
            Ok(Progress::Continue(State::RawtextEndTagName))
        );
    }
    let before = snapshot(&execution);
    for _ in 0..3 {
        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
        assert_eq!(snapshot(&execution), before);
    }
    execution.context_mut().input.finish();
    assert_eq!(execution.step(), Ok(Progress::Continue(State::Rawtext)));
    assert_eq!(
        execution.context().input.current(),
        Some((Input::Eof, Position { line: 1, column: 6 }))
    );
    assert_eq!(execution.context().tokens, literal(&points("StYlE")));
    assert!(execution.context().errors.is_empty());
    assert_eq!(execution.step(), Ok(Progress::Finished));
    terminal(&mut execution, Ok(Progress::Finished));
}

#[test]
fn greater_than_reuses_shared_emission_and_preserves_unrelated_storage() {
    let mut context = context("style", "StYlE", Some("style"));
    let end = context.pending_end_tag.as_mut().unwrap();
    end.self_closing = true;
    end.attributes = vec![attribute("a", "one"), attribute("b", "two")];
    end.discarded_current_attribute = Some(attribute("a", "not-emitted"));
    context.tokens.push(Token::Character('!'.into()));
    context.errors.push(error("prior", 8, 3));
    // Prefix preprocessing gives the emission delimiter a normalized position.
    context.input.push(points("\r\n")).unwrap();
    assert_eq!(context.input.consume(), Input::CodePoint('\n'.into()));
    assert_eq!(context.input.consume(), Input::NeedMoreInput);
    let mut execution = execution(context, &points(">tail"), true);
    let mut expected = snapshot(&execution);
    expected.state = State::Data;
    expected.current = Some((
        Input::CodePoint('>'.into()),
        Position { line: 2, column: 1 },
    ));
    expected.end = None;
    expected.current_tag = None;
    expected.tokens.push(Token::EndTag {
        name: points("style"),
        attributes: vec![attribute("a", "one"), attribute("b", "two")],
        self_closing: true,
    });
    expected.errors.extend([
        error("end-tag-with-attributes", 2, 1),
        error("end-tag-with-trailing-solidus", 2, 1),
    ]);
    assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
    assert_eq!(snapshot(&execution), expected);
    assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
    assert_eq!(
        execution.context().tokens.last(),
        Some(&Token::Character('t'.into()))
    );
}

type Trace = Vec<(Result<Progress, UnimplementedState>, Snapshot)>;

fn drain(execution: &mut Execution, trace: &mut Trace) -> bool {
    for _ in 0..512 {
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
            terminal(execution, result);
            return true;
        }
    }
    panic!("state failed to consume or terminate");
}

#[test]
fn implemented_destinations_apply_their_own_errors_and_remaining_stubs_are_terminal() {
    for (input, errors, attributes, self_closing) in [
        ("StYlE >", vec![], vec![], false),
        (
            "StYlE/>",
            vec![error("end-tag-with-trailing-solidus", 1, 7)],
            vec![],
            true,
        ),
        (
            "StYlE a='x'>",
            vec![error("end-tag-with-attributes", 1, 12)],
            vec![attribute("a", "x")],
            false,
        ),
        (
            "StYlE\r\n a='x'/>",
            vec![
                error("end-tag-with-attributes", 2, 8),
                error("end-tag-with-trailing-solidus", 2, 8),
            ],
            vec![attribute("a", "x")],
            true,
        ),
    ] {
        let mut execution = execution(context("", "", Some("style")), &points(input), true);
        let mut trace = vec![];
        assert!(drain(&mut execution, &mut trace));
        assert_eq!(trace.last().unwrap().0, Ok(Progress::Finished));
        assert_eq!(
            execution.context().tokens,
            [
                Token::EndTag {
                    name: points("style"),
                    attributes,
                    self_closing
                },
                Token::Eof
            ]
        );
        assert_eq!(execution.context().errors, errors);
        assert_eq!(execution.context().last_start_tag, Some(points("style")));
        assert_eq!(execution.context().temporary_buffer, points("StYlE"));
    }
    for (input, expected_error) in [
        ("StYlE \r\n", error("eof-in-tag", 2, 1)),
        ("StYlE/", error("eof-in-tag", 1, 7)),
    ] {
        let mut execution = execution(context("", "", Some("style")), &points(input), true);
        assert!(drain(&mut execution, &mut vec![]));
        assert_eq!(execution.context().tokens, [Token::Eof]);
        assert_eq!(execution.context().errors, [expected_error]);
        assert!(execution.context().pending_end_tag.is_none());
        assert!(execution.context().current_tag.is_none());
    }
    for (input, state, current, column) in [
        ("StYlE<tail", State::RawtextLessThanSign, '<', 6),
        ("StYlE a='&tail", State::CharacterReference, '&', 10),
        ("StYlE>&tail", State::CharacterReference, '&', 7),
    ] {
        let mut execution = execution(context("", "", Some("style")), &points(input), true);
        let mut trace = vec![];
        assert!(drain(&mut execution, &mut trace));
        assert_eq!(trace.last().unwrap().0, Err(UnimplementedState { state }));
        assert_eq!(
            execution.context().input.current(),
            Some((
                Input::CodePoint(current.into()),
                Position { line: 1, column }
            ))
        );
        assert!(!execution.context().tokens.contains(&Token::Eof));
        assert!(execution.context().errors.is_empty());
        // Mutation/feeding never unblocks a terminal stub.
        execution.context_mut().temporary_buffer.push('!'.into());
        terminal(&mut execution, Err(UnimplementedState { state }));
        assert_eq!(
            execution.context_mut().input.consume(),
            Input::CodePoint('t'.into())
        );
    }
    // Adjacent entries remain stubs even with initialized tag storage.
    for state in [State::RawtextLessThanSign, State::RawtextEndTagOpen] {
        for finished in [false, true] {
            let mut context = context("", "", Some("style"));
            if finished {
                context.input.finish();
            }
            let mut execution = Execution::new(state, context);
            terminal(&mut execution, Err(UnimplementedState { state }));
            if !finished {
                execution.context_mut().input.push(points("later")).unwrap();
                terminal(&mut execution, Err(UnimplementedState { state }));
                assert_eq!(execution.context().input.current(), None);
            }
        }
    }
}

#[test]
fn fallback_keeps_seeded_buffer_literal_and_diagnostics_use_normalized_positions() {
    let mut seeded = context("style", "StYlE", Some("style"));
    seeded
        .temporary_buffer
        .extend([0xd, 0, 0xd800, 0xfdd0].map(|v| CodePoint::new(v).unwrap()));
    let buffer = seeded.temporary_buffer.clone();
    let mut execution = execution(seeded, &points("!\r\n\0"), true);
    assert_eq!(execution.step(), Ok(Progress::Continue(State::Rawtext)));
    assert_eq!(execution.context().tokens, literal(&buffer));
    assert_eq!(execution.context().temporary_buffer, buffer);
    assert!(execution.context().errors.is_empty());
    assert!(execution.context().input.errors().is_empty());
    for _ in 0..3 {
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Rawtext)));
    }
    assert_eq!(
        execution.context().errors,
        [error("unexpected-null-character", 2, 1)]
    );
    assert_eq!(
        execution.context().input.current(),
        Some((
            Input::CodePoint('\0'.into()),
            Position { line: 2, column: 1 }
        ))
    );
    assert_eq!(execution.step(), Ok(Progress::Finished));
    let mut expected = literal(&buffer);
    expected.extend([
        Token::Character('!'.into()),
        Token::Character('\n'.into()),
        Token::Character('\u{fffd}'.into()),
        Token::Eof,
    ]);
    assert_eq!(execution.context().tokens, expected);
    assert_eq!(
        execution.context().input.current(),
        Some((Input::Eof, Position { line: 2, column: 2 }))
    );
    assert!(execution.context().input.errors().is_empty());
    terminal(&mut execution, Ok(Progress::Finished));
}

fn chunk_trace(input: &[CodePoint], chunks: Vec<&[CodePoint]>, closed: bool) -> Trace {
    let mut execution = execution(context("", "", Some("style")), &[], false);
    let mut trace = vec![];
    if closed {
        execution
            .context_mut()
            .input
            .push(input.iter().copied())
            .unwrap();
        execution.context_mut().input.finish();
        assert!(drain(&mut execution, &mut trace));
        return trace;
    }
    assert!(!drain(&mut execution, &mut trace));
    for chunk in chunks {
        execution
            .context_mut()
            .input
            .push(chunk.iter().copied())
            .unwrap();
        if drain(&mut execution, &mut trace) {
            return trace;
        }
    }
    execution.context_mut().input.finish();
    assert!(drain(&mut execution, &mut trace));
    trace
}

#[test]
fn every_two_chunk_split_and_single_code_point_chunk_matches_closed_source_trace() {
    let mut malformed = points("StYlE!");
    malformed
        .extend([0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x0b, 0].map(|v| CodePoint::new(v).unwrap()));
    malformed.extend(points("\r\n\r\r\n&end\r"));
    let mut inputs = vec![malformed];
    inputs.extend(
        [
            "",
            "S",
            "StYlE",
            "Other>",
            "StYlEx>",
            "StYlE>",
            "StYlE>\0\r\ntext",
            "StYlE/>",
            "StYlE a='x'/>",
            "StYlE\r\n a=\"x\"/>",
            "StYlE \r\n",
            "StYlE/",
            "StYlE\0",
            "StYlEÉ>",
            "StYlE\u{b}>",
            "StYlE\u{a0}>",
            "StYlE\r\n\0>",
            "StYlE<tail",
            "StYlE a='&tail",
            "StYlE>&tail",
        ]
        .map(points),
    );
    for input in inputs {
        let expected = chunk_trace(&input, vec![], true);
        for split in 0..=input.len() {
            assert_eq!(
                chunk_trace(&input, vec![&input[..split], &input[split..]], false),
                expected,
                "split {split} in {input:?}"
            );
        }
        assert_eq!(
            chunk_trace(&input, input.chunks(1).collect(), false),
            expected,
            "single points in {input:?}"
        );
    }
}
