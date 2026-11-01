//! Pinned HTML 404b3eb14a7a936dddefb2d78e224dbeb01d2358, source
//! 144708–144732 (#comment-end-bang-state). Local tests, not upstream runs.
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

fn context(kind: Option<TagKind>) -> Context {
    let mut context = Context {
        pending_comment: Some(PendingComment::default()),
        ..Default::default()
    };
    if let Some(kind) = kind {
        let retained = Attribute {
            name: points("a"),
            value: points("retained"),
        };
        let detached = Attribute {
            name: points("a"),
            value: points("detached"),
        };
        context.current_tag = Some(kind);
        context.pending_start_tag = Some(PendingStartTag {
            name: points("start"),
            self_closing: true,
            attributes: vec![retained.clone()],
            discarded_current_attribute: Some(detached.clone()),
        });
        context.pending_end_tag = Some(PendingEndTag {
            name: points("end"),
            self_closing: true,
            attributes: vec![retained],
            discarded_current_attribute: Some(detached),
        });
        context.last_start_tag = Some(points("last"));
        context.return_state = Some(State::Rcdata);
        context.temporary_buffer = points("scratch");
        context.tokens = vec![
            Token::Comment { data: vec![] },
            Token::Character('p'.into()),
        ];
        context.errors = vec![error("prior", 1, 1)];
        let mut data = points("seed\0\r--!<&");
        data.extend([
            CodePoint::new(0xdfff).unwrap(),
            CodePoint::new(0xffff).unwrap(),
        ]);
        context.pending_comment = Some(PendingComment { data });
    }
    context
}

#[derive(Debug, Eq, PartialEq)]
struct Snapshot {
    state: State,
    current: Option<(Input, Position)>,
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
    input_errors: Vec<ParseError>,
    current_tag: Option<TagKind>,
    start: Option<PendingStartTag>,
    end: Option<PendingEndTag>,
    comment: Option<PendingComment>,
    last: Option<Vec<CodePoint>>,
    return_state: Option<State>,
    scratch: Vec<CodePoint>,
}

fn snapshot(run: &Execution) -> Snapshot {
    let context = run.context();
    Snapshot {
        state: run.state(),
        current: context.input.current(),
        tokens: context.tokens.clone(),
        errors: context.errors.clone(),
        input_errors: context.input.errors().to_vec(),
        current_tag: context.current_tag,
        start: context.pending_start_tag.clone(),
        end: context.pending_end_tag.clone(),
        comment: context.pending_comment.clone(),
        last: context.last_start_tag.clone(),
        return_state: context.return_state,
        scratch: context.temporary_buffer.clone(),
    }
}

#[test]
fn every_ascii_branch_and_non_scalars_have_exact_source_effects() {
    for value in (0..=0x7f).chain([
        0x80, 0x9f, 0xa0, 0xc0, 0xe9, 0x301, 0xd800, 0xdfff, 0xfdd0, 0xfeff, 0xfffd, 0x1f600,
        0x1fffe, 0x10ffff,
    ]) {
        for kind in [None, Some(TagKind::Start), Some(TagKind::End)] {
            let point = CodePoint::new(value).unwrap();
            let normalized = if value == 0x0d { '\n'.into() } else { point };
            let mut run = Execution::new(State::CommentEndBang, context(kind));
            run.context_mut().input.push([point, 'z'.into()]).unwrap();
            run.context_mut().input.finish();
            let mut expected = snapshot(&run);
            expected.state = match value {
                0x2d => State::CommentEndDash,
                0x3e => State::Data,
                _ => State::Comment,
            };
            expected.current = Some((
                Input::CodePoint(normalized),
                Position { line: 1, column: 1 },
            ));
            if value == 0x3e {
                let data = expected.comment.take().unwrap().data;
                expected.tokens.push(Token::Comment { data });
                expected
                    .errors
                    .push(error("incorrectly-closed-comment", 1, 1));
            } else {
                expected
                    .comment
                    .as_mut()
                    .unwrap()
                    .data
                    .extend(points("--!"));
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
            assert_eq!(run.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&run), expected, "U+{value:04X}, {kind:?}");
            // Inspect the cursor at the source boundary, not by running a sibling.
            if !matches!(value, 0x2d | 0x3e) {
                assert_eq!(
                    run.context_mut().input.reconsume(),
                    Err(CursorError::AlreadyReconsuming)
                );
                assert_eq!(
                    run.context_mut().input.consume(),
                    Input::CodePoint(normalized)
                );
                assert_eq!(snapshot(&run), expected); // No duplicate preprocessing error.
            }
            assert_eq!(
                run.context_mut().input.consume(),
                Input::CodePoint('z'.into())
            );
            assert_eq!(
                run.context().input.position(),
                if matches!(value, 0x0a | 0x0d) {
                    Position { line: 2, column: 1 }
                } else {
                    Position { line: 1, column: 2 }
                }
            );
        }
    }
}

#[test]
fn unsupported_destinations_are_terminal_after_inspectable_source_effects() {
    for (text, target) in [
        ("-tail", State::CommentEndDash),
        ("\0tail", State::Comment),
        ("!tail", State::Comment),
        ("<tail", State::Comment),
    ] {
        let mut run = Execution::new(State::CommentEndBang, context(Some(TagKind::End)));
        run.context_mut().input.push(points(text)).unwrap();
        run.context_mut().input.finish();
        assert_eq!(run.step(), Ok(Progress::Continue(target)));
        let source = snapshot(&run);
        assert!(
            source
                .comment
                .as_ref()
                .unwrap()
                .data
                .ends_with(&points("--!"))
        );
        assert_eq!(source.errors, [error("prior", 1, 1)]);
        let blocked = Err(UnimplementedState { state: target });
        for _ in 0..3 {
            assert_eq!(run.step(), blocked);
            assert_eq!(snapshot(&run), source);
        }
        // No reconstruction or resumption; this execution remains blocked.
    }
}

#[test]
fn eof_emits_only_existing_comment_then_one_eof_at_normalized_position() {
    for kind in [None, Some(TagKind::Start), Some(TagKind::End)] {
        let mut context = context(kind);
        let surrogate = CodePoint::new(0xd800).unwrap();
        context
            .input
            .push([surrogate, '\r'.into(), '\n'.into()])
            .unwrap();
        assert_eq!(context.input.consume(), Input::CodePoint(surrogate));
        assert_eq!(context.input.consume(), Input::CodePoint('\n'.into()));
        context.input.finish();
        let mut run = Execution::new(State::CommentEndBang, context);
        let mut expected = snapshot(&run);
        expected.current = Some((Input::Eof, Position { line: 2, column: 1 }));
        let data = expected.comment.take().unwrap().data;
        expected
            .tokens
            .extend([Token::Comment { data }, Token::Eof]);
        expected.errors.push(error("eof-in-comment", 2, 1));
        for _ in 0..3 {
            assert_eq!(run.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&run), expected);
        }
        // EOF was consumed, not scheduled for reconsumption by the state.
        assert_eq!(run.context_mut().input.reconsume(), Ok(()));
        assert_eq!(run.context_mut().input.consume(), Input::Eof);
        assert_eq!(snapshot(&run), expected);
    }
}

#[test]
fn open_exhaustion_is_effect_free_until_explicit_eof() {
    let mut context = context(Some(TagKind::Start));
    context.input.push(points("\r")).unwrap();
    assert_eq!(context.input.consume(), Input::CodePoint('\n'.into()));
    let mut run = Execution::new(State::CommentEndBang, context);
    let mut expected = snapshot(&run);
    // A split CRLF's suppressed LF is not a CommentEndBang input character.
    for chunk in [points(""), points("\n"), points("")] {
        run.context_mut().input.push(chunk).unwrap();
        for _ in 0..3 {
            assert_eq!(run.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&run), expected);
        }
    }
    run.context_mut().input.finish();
    expected.current = Some((Input::Eof, Position { line: 2, column: 1 }));
    let data = expected.comment.take().unwrap().data;
    expected
        .tokens
        .extend([Token::Comment { data }, Token::Eof]);
    expected.errors.push(error("eof-in-comment", 2, 1));
    assert_eq!(run.step(), Ok(Progress::Finished));
    assert_eq!(snapshot(&run), expected);
}

#[test]
fn greater_than_takes_comment_once_then_data_consumes_tail_without_reconsumption() {
    for kind in [None, Some(TagKind::Start), Some(TagKind::End)] {
        let mut run = Execution::new(State::CommentEndBang, context(kind));
        let surrogate = CodePoint::new(0xd800).unwrap();
        let mut input = points(">\0\r\n");
        input.extend([surrogate, '!'.into()]);
        run.context_mut().input.push(input).unwrap();
        run.context_mut().input.finish();
        let mut expected = snapshot(&run);
        expected.state = State::Data;
        expected.current = Some((
            Input::CodePoint('>'.into()),
            Position { line: 1, column: 1 },
        ));
        let data = expected.comment.take().unwrap().data;
        expected.tokens.push(Token::Comment { data });
        expected
            .errors
            .push(error("incorrectly-closed-comment", 1, 1));
        assert_eq!(run.step(), Ok(Progress::Continue(State::Data)));
        assert_eq!(snapshot(&run), expected);
        for (point, line, column) in [
            ('\0'.into(), 1, 2),
            ('\n'.into(), 1, 3),
            (surrogate, 2, 1),
            ('!'.into(), 2, 2),
        ] {
            expected.tokens.push(Token::Character(point));
            expected.current = Some((Input::CodePoint(point), Position { line, column }));
            if point.value() == 0 {
                expected
                    .errors
                    .push(error("unexpected-null-character", line, column));
            }
            if point == surrogate {
                expected
                    .input_errors
                    .push(error("surrogate-in-input-stream", line, column));
            }
            assert_eq!(run.step(), Ok(Progress::Continue(State::Data)));
            assert_eq!(snapshot(&run), expected);
        }
        expected.tokens.push(Token::Eof);
        expected.current = Some((Input::Eof, Position { line: 2, column: 3 }));
        for _ in 0..3 {
            assert_eq!(run.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&run), expected);
        }
    }
}

#[test]
fn ordinary_data_comment_entry_still_blocks_at_markup_declaration_open() {
    let mut context = Context::default();
    context.input.push(points("<!--x--!>")).unwrap();
    context.input.finish();
    let mut run = Execution::new(State::Data, context);
    assert_eq!(run.step(), Ok(Progress::Continue(State::TagOpen)));
    assert_eq!(
        run.step(),
        Ok(Progress::Continue(State::MarkupDeclarationOpen))
    );
    let source = snapshot(&run);
    assert_eq!(
        source.current,
        Some((
            Input::CodePoint('!'.into()),
            Position { line: 1, column: 2 }
        ))
    );
    assert!(source.tokens.is_empty());
    assert!(source.errors.is_empty());
    assert!(source.comment.is_none());
    for _ in 0..3 {
        assert_eq!(
            run.step(),
            Err(UnimplementedState {
                state: State::MarkupDeclarationOpen
            })
        );
        assert_eq!(snapshot(&run), source);
    }
}

type Step = (Result<Progress, UnimplementedState>, Snapshot);

// Never feed/resume after a terminal result. A source trace includes its intended
// transition BEFORE the destination's terminal stub attempt.
fn drain(run: &mut Execution, trace: &mut Vec<Step>) -> bool {
    loop {
        let before = snapshot(run);
        let result = run.step();
        if result == Ok(Progress::NeedMoreInput) {
            assert_eq!(snapshot(run), before);
            assert_eq!(run.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(run), before);
            return false;
        }
        let terminal = matches!(result, Ok(Progress::Finished) | Err(_));
        trace.push((result, snapshot(run)));
        if terminal {
            let before = snapshot(run);
            assert_eq!(run.step(), result);
            assert_eq!(snapshot(run), before);
            return true;
        }
    }
}

fn trace(chunks: &[&[CodePoint]], closed: bool) -> Vec<Step> {
    let mut run = Execution::new(State::CommentEndBang, context(Some(TagKind::End)));
    let mut result = vec![];
    if closed {
        assert_eq!(chunks.len(), 1);
        run.context_mut()
            .input
            .push(chunks[0].iter().copied())
            .unwrap();
        run.context_mut().input.finish();
    } else {
        for chunk in chunks {
            run.context_mut().input.push(chunk.iter().copied()).unwrap();
            if drain(&mut run, &mut result) {
                return result;
            }
        }
        run.context_mut().input.finish();
    }
    assert!(drain(&mut run, &mut result));
    result
}

#[test]
fn every_two_chunk_split_and_single_point_chunks_match_closed_source_traces() {
    let mut non_scalars = points(">\r\n\0");
    non_scalars.extend([
        CodePoint::new(0xd800).unwrap(),
        CodePoint::new(0x1fffe).unwrap(),
        '\u{b}'.into(),
    ]);
    for input in [
        points(""),
        points("-tail"),
        points("!tail"),
        points("\0tail"),
        points("\r\ntail"),
        vec![CodePoint::new(0xdfff).unwrap(), 'x'.into()],
        points(">"),
        points(">text\r\n\r\r\0🙂\r"),
        points(">&tail"),
        points("><!--x-->"),
        points("></?x>"),
        non_scalars,
    ] {
        let expected = trace(&[&input], true);
        for split in 0..=input.len() {
            assert_eq!(
                trace(&[&input[..split], &input[split..]], false),
                expected,
                "split {split}: {input:?}"
            );
        }
        assert_eq!(
            trace(&input.chunks(1).collect::<Vec<_>>(), false),
            expected,
            "single points: {input:?}"
        );
    }
}
