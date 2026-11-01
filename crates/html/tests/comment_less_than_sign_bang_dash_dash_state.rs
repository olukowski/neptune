//! State-local source-step checks, not upstream conformance executions.
//! Pinned HTML 404b3eb14a7a936dddefb2d78e224dbeb01d2358, source
//! 144642–144657, #comment-less-than-sign-bang-dash-dash-state.
//! Direct entry supplies initialized pending-comment data. Source expectations are
//! inspected before real destinations; no blocked execution is reconstructed/resumed.

use neptune_html::tokenizer::{
    Attribute, CodePoint, Context, CursorError, Execution, Input, ParseError, PendingComment,
    PendingEndTag, PendingStartTag, Position, Progress, State, TagKind, Token, UnimplementedState,
};

const SOURCE: State = State::CommentLessThanSignBangDashDash;

fn points(text: &str) -> Vec<CodePoint> {
    text.chars().map(Into::into).collect()
}

fn error(code: &'static str, line: usize, column: usize) -> ParseError {
    ParseError {
        code,
        position: Position { line, column },
    }
}

fn literal_data() -> Vec<CodePoint> {
    let mut data = points("seed<!\0\r\n--&>");
    data.extend([0xd800, 0xdfff, 0xfdd0, 0x10ffff].map(|n| CodePoint::new(n).unwrap()));
    data
}

fn seeded(data: Vec<CodePoint>, selected: Option<TagKind>) -> Context {
    let attribute = Attribute {
        name: points("name"),
        value: literal_data(),
    };
    Context {
        tokens: vec![
            Token::Comment { data: vec![] },
            Token::Character('p'.into()),
            Token::Comment {
                data: points("prior"),
            },
        ],
        errors: vec![error("prior-diagnostic", 1, 1)],
        last_start_tag: Some(points("title")),
        return_state: Some(State::AttributeValueSingleQuoted),
        current_tag: selected,
        pending_start_tag: Some(PendingStartTag {
            name: points("start"),
            attributes: vec![attribute.clone()],
            discarded_current_attribute: Some(attribute.clone()),
            self_closing: true,
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("end"),
            attributes: vec![attribute.clone()],
            discarded_current_attribute: Some(attribute),
            self_closing: true,
        }),
        pending_comment: Some(PendingComment { data }),
        temporary_buffer: literal_data(),
        ..Default::default()
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Storage {
    last_start_tag: Option<Vec<CodePoint>>,
    return_state: Option<State>,
    current_tag: Option<TagKind>,
    start: Option<PendingStartTag>,
    end: Option<PendingEndTag>,
    comment: Option<PendingComment>,
    buffer: Vec<CodePoint>,
}

#[derive(Debug, Eq, PartialEq)]
struct Snapshot {
    state: State,
    current: Option<(Input, Position)>,
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
    input_errors: Vec<ParseError>,
    storage: Storage,
}

fn snapshot(execution: &Execution) -> Snapshot {
    let context = execution.context();
    Snapshot {
        state: execution.state(),
        current: context.input.current(),
        tokens: context.tokens.clone(),
        errors: context.errors.clone(),
        input_errors: context.input.errors().to_vec(),
        storage: Storage {
            last_start_tag: context.last_start_tag.clone(),
            return_state: context.return_state,
            current_tag: context.current_tag,
            start: context.pending_start_tag.clone(),
            end: context.pending_end_tag.clone(),
            comment: context.pending_comment.clone(),
            buffer: context.temporary_buffer.clone(),
        },
    }
}

// Cursor history only; this does not execute any predecessor algorithm.
fn entry(mut context: Context, history: bool) -> Execution {
    if history {
        // Keep an existing preprocessing diagnostic too; source reconsumption
        // must neither erase it nor diagnose the already-consumed point again.
        let surrogate = CodePoint::new(0xd800).unwrap();
        context.input.push([surrogate]).unwrap();
        assert_eq!(context.input.consume(), Input::CodePoint(surrogate));
        context.input.push(points("\r\n<!--")).unwrap();
        for point in points("\n<!--") {
            assert_eq!(context.input.consume(), Input::CodePoint(point));
        }
    }
    Execution::new(SOURCE, context)
}

#[test]
fn every_ascii_branch_and_literal_non_scalar_reconsumes_without_mutating_or_taking_comment() {
    for value in (0..=0x7f).chain([
        0x80, 0x9f, 0xa0, 0xc9, 0x3a9, 0xfeff, 0x1f600, 0xd800, 0xdfff, 0xfdd0, 0xffff, 0x10ffff,
    ]) {
        for finished in [false, true] {
            for selected in [None, Some(TagKind::Start), Some(TagKind::End)] {
                for data in [vec![], literal_data()] {
                    let point = CodePoint::new(value).unwrap();
                    let normalized = if value == 0x0d { '\n'.into() } else { point };
                    let mut execution = entry(seeded(data, selected), true);
                    execution
                        .context_mut()
                        .input
                        .push([point, 'z'.into()])
                        .unwrap();
                    if finished {
                        execution.context_mut().input.finish();
                    }
                    let mut expected = snapshot(&execution);
                    expected.state = State::CommentEnd;
                    expected.current = Some((
                        Input::CodePoint(normalized),
                        Position { line: 2, column: 5 },
                    ));
                    if value != 0x3e {
                        expected.errors.push(error("nested-comment", 2, 5));
                    }
                    let input_error = match value {
                        0xd800..=0xdfff => Some("surrogate-in-input-stream"),
                        0xfdd0 | 0xffff | 0x10ffff => Some("noncharacter-in-input-stream"),
                        0x01..=0x08 | 0x0b | 0x0e..=0x1f | 0x7f..=0x9f => {
                            Some("control-character-in-input-stream")
                        }
                        _ => None,
                    };
                    if let Some(code) = input_error {
                        expected.input_errors.push(error(code, 2, 5));
                    }
                    assert_eq!(execution.step(), Ok(Progress::Continue(State::CommentEnd)));
                    assert_eq!(snapshot(&execution), expected, "U+{value:04X}");
                    assert_eq!(
                        execution.context_mut().input.reconsume(),
                        Err(CursorError::AlreadyReconsuming)
                    );
                    // Cursor-only probe BEFORE attempting the destination. It proves
                    // original/NUL/non-scalar preservation and one scheduled reconsume,
                    // without inventing CommentEnd's algorithm or resuming a block.
                    assert_eq!(
                        execution.context_mut().input.consume(),
                        Input::CodePoint(normalized)
                    );
                    assert_eq!(snapshot(&execution), expected);
                    assert_eq!(
                        execution.context_mut().input.consume(),
                        Input::CodePoint('z'.into())
                    );
                    assert_eq!(
                        execution.context().input.position(),
                        if normalized == '\n'.into() {
                            Position { line: 3, column: 1 }
                        } else {
                            Position { line: 2, column: 6 }
                        }
                    );
                    assert_eq!(execution.context().input.errors(), expected.input_errors);
                }
            }
        }
    }
}

#[test]
fn open_exhaustion_preserves_history_and_only_explicit_eof_reconsumes_without_emission() {
    for history in [false, true] {
        for data in [vec![], literal_data()] {
            let mut execution = entry(seeded(data, None), history);
            let mut expected = snapshot(&execution);
            for _ in 0..3 {
                execution.context_mut().input.push([]).unwrap();
                assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                assert_eq!(snapshot(&execution), expected);
            }
            execution.context_mut().input.finish();
            execution.context_mut().input.finish();
            expected.state = State::CommentEnd;
            expected.current = Some((
                Input::Eof,
                if history {
                    Position { line: 2, column: 5 }
                } else {
                    Position { line: 1, column: 1 }
                },
            ));
            assert_eq!(execution.step(), Ok(Progress::Continue(State::CommentEnd)));
            assert_eq!(snapshot(&execution), expected);
            assert_eq!(
                execution.context_mut().input.reconsume(),
                Err(CursorError::AlreadyReconsuming)
            );
            // No eof-in-comment, nested-comment, comment emission/take, or EOF token.
            assert_eq!(execution.context_mut().input.consume(), Input::Eof);
            assert_eq!(snapshot(&execution), expected);
            assert_eq!(
                execution.context_mut().input.push([]),
                Err(CursorError::InputAlreadyFinished)
            );
        }
    }
}

#[test]
fn suspended_entry_accepts_real_input_without_inventing_predecessor_data() {
    for text in [">", "!", "-", "\0"] {
        let context = Context {
            pending_comment: Some(PendingComment::default()),
            ..Default::default()
        };
        let mut execution = entry(context, false);
        let before = snapshot(&execution);
        assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
        assert_eq!(snapshot(&execution), before);
        execution.context_mut().input.push(points(text)).unwrap();
        assert_eq!(execution.step(), Ok(Progress::Continue(State::CommentEnd)));
        let after = snapshot(&execution);
        assert_eq!(after.storage, before.storage);
        assert!(after.tokens.is_empty());
        assert_eq!(
            after.errors,
            if text == ">" {
                vec![]
            } else {
                vec![error("nested-comment", 1, 1)]
            }
        );
        assert!(after.input_errors.is_empty());
    }
}

// Independent pinned CommentEnd expectations, after the source has been checked.
fn real_end(execution: &mut Execution, expected: &mut Snapshot) {
    assert_eq!(
        execution.context_mut().input.reconsume(),
        Err(CursorError::AlreadyReconsuming)
    );
    let (input, position) = expected.current.unwrap();
    let mut replay_comment = false;
    let result = match input {
        Input::Eof => {
            expected.errors.push(ParseError {
                code: "eof-in-comment",
                position,
            });
            let data = expected.storage.comment.take().unwrap().data;
            expected
                .tokens
                .extend([Token::Comment { data }, Token::Eof]);
            Progress::Finished
        }
        Input::CodePoint(point) => {
            match point.value() {
                0x3e => {
                    expected.state = State::Data;
                    let data = expected.storage.comment.take().unwrap().data;
                    expected.tokens.push(Token::Comment { data });
                }
                0x21 => expected.state = State::CommentEndBang,
                0x2d => expected
                    .storage
                    .comment
                    .as_mut()
                    .unwrap()
                    .data
                    .push('-'.into()),
                _ => {
                    expected.state = State::Comment;
                    expected
                        .storage
                        .comment
                        .as_mut()
                        .unwrap()
                        .data
                        .extend(points("--"));
                    replay_comment = true;
                }
            }
            Progress::Continue(expected.state)
        }
        Input::NeedMoreInput => panic!("source consumed no item"),
    };
    assert_eq!(execution.step(), Ok(result));
    assert_eq!(snapshot(execution), *expected);
    if replay_comment {
        assert_eq!(
            execution.context_mut().input.reconsume(),
            Err(CursorError::AlreadyReconsuming)
        );
        let Input::CodePoint(point) = input else {
            unreachable!()
        };
        let appended = if point.value() == 0 {
            expected.errors.push(ParseError {
                code: "unexpected-null-character",
                position,
            });
            '\u{fffd}'.into()
        } else {
            point
        };
        expected
            .storage
            .comment
            .as_mut()
            .unwrap()
            .data
            .push(appended);
        if point == '<'.into() {
            expected.state = State::CommentLessThanSign;
        }
        assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
        assert_eq!(snapshot(execution), *expected);
    }
    if result == Progress::Finished {
        for _ in 0..3 {
            assert_eq!(execution.step(), Ok(result));
            assert_eq!(snapshot(execution), *expected);
        }
    }
}

#[test]
fn source_effects_precede_real_end_reconsumption_emission_or_delimiter_restoration() {
    for input in [
        vec![],
        points(">tail"),
        points("\0tail"),
        points("-!>"),
        vec![CodePoint::new(0xd800).unwrap()],
    ] {
        for finished in [false, true] {
            if input.is_empty() && !finished {
                continue;
            }
            let mut execution = entry(seeded(literal_data(), Some(TagKind::End)), true);
            execution
                .context_mut()
                .input
                .push(input.iter().copied())
                .unwrap();
            if finished {
                execution.context_mut().input.finish();
            }
            let mut expected = snapshot(&execution);
            expected.state = State::CommentEnd;
            let item = input.first().copied().map_or(Input::Eof, Input::CodePoint);
            expected.current = Some((item, Position { line: 2, column: 5 }));
            if item != Input::Eof && item != Input::CodePoint('>'.into()) {
                expected.errors.push(error("nested-comment", 2, 5));
            }
            if item == Input::CodePoint(CodePoint::new(0xd800).unwrap()) {
                expected
                    .input_errors
                    .push(error("surrogate-in-input-stream", 2, 5));
            }
            assert_eq!(execution.step(), Ok(Progress::Continue(State::CommentEnd)));
            assert_eq!(snapshot(&execution), expected);
            real_end(&mut execution, &mut expected);
        }
    }
}

// Source-local trace through the independently expected immediate destination,
// not the rest of the suffix. Full-family streaming is tested separately below.
// A preceding raw CR leaves LF suppression pending across feed boundaries.
fn trace(chunks: &[&[CodePoint]], closed: bool) -> Vec<Snapshot> {
    let mut context = seeded(literal_data(), Some(TagKind::Start));
    context.input.push(points("\r")).unwrap();
    assert_eq!(context.input.consume(), Input::CodePoint('\n'.into()));
    let mut execution = entry(context, false);
    let mut result = vec![];
    let mut advance = |execution: &mut Execution| {
        let before = snapshot(execution);
        match execution.step().unwrap() {
            Progress::NeedMoreInput => {
                for _ in 0..2 {
                    assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                    assert_eq!(snapshot(execution), before);
                }
                false
            }
            Progress::Continue(State::CommentEnd) => {
                let mut after = snapshot(execution);
                result.push(snapshot(execution));
                real_end(execution, &mut after);
                result.push(after);
                true
            }
            other => panic!("unexpected source result: {other:?}"),
        }
    };
    for (index, chunk) in chunks.iter().enumerate() {
        execution
            .context_mut()
            .input
            .push(chunk.iter().copied())
            .unwrap();
        // The unsplit closed reference finishes BEFORE its source step. Streaming
        // executions may already reach the terminal destination in an earlier chunk.
        if closed && index + 1 == chunks.len() {
            execution.context_mut().input.finish();
        }
        if advance(&mut execution) {
            return result;
        }
    }
    if closed {
        execution.context_mut().input.finish();
        assert!(advance(&mut execution));
    }
    result
}

#[test]
fn every_two_chunk_split_and_single_point_chunks_match_closed_and_open_source_traces() {
    for input in [
        vec![],
        points("\n"),
        points("\n>tail"),
        points("\n\0<--!>"),
        points("\n\r\n\r\r"),
        points("\r"),
        points("\n-!>"),
        points("\n<&>"),
        vec!['\n'.into(), CodePoint::new(0xdfff).unwrap(), '>'.into()],
        vec!['\n'.into(), CodePoint::new(0xfdd0).unwrap(), '\r'.into()],
        points("\n\u{b}&"),
        points("\nΩ😀"),
    ] {
        for closed in [false, true] {
            let expected = trace(&[&input], closed);
            for split in 0..=input.len() {
                assert_eq!(
                    trace(&[&input[..split], &[], &input[split..]], closed),
                    expected,
                    "split {split}: {input:?}"
                );
            }
            assert_eq!(
                trace(&input.chunks(1).collect::<Vec<_>>(), closed),
                expected,
                "single points: {input:?}"
            );
        }
    }
}

#[test]
fn ordinary_data_comment_entry_remains_terminally_blocked_at_markup_declaration_open() {
    let mut context = Context::default();
    context.input.push(points("<!--nested<!--x-->")).unwrap();
    context.input.finish();
    let mut execution = Execution::new(State::Data, context);
    assert_eq!(execution.step(), Ok(Progress::Continue(State::TagOpen)));
    assert_eq!(
        execution.step(),
        Ok(Progress::Continue(State::MarkupDeclarationOpen))
    );
    let expected = snapshot(&execution);
    assert!(expected.storage.comment.is_none());
    assert!(expected.tokens.is_empty());
    assert!(expected.errors.is_empty());
    assert!(expected.input_errors.is_empty());
    assert_eq!(
        expected.current,
        Some((
            Input::CodePoint('!'.into()),
            Position { line: 1, column: 2 }
        ))
    );
    for _ in 0..3 {
        assert_eq!(
            execution.step(),
            Err(UnimplementedState {
                state: State::MarkupDeclarationOpen
            })
        );
        assert_eq!(snapshot(&execution), expected);
    }
}
