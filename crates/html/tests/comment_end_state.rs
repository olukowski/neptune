//! Local branch evidence for pinned HTML Comment end (#comment-end-state):
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144681-L144706>
//! Not upstream conformance. No sibling state is implemented or simulated here.

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

fn seeded() -> Context {
    let attribute = Attribute {
        name: points("attr"),
        value: points("value"),
    };
    Context {
        tokens: vec![Token::Character('p'.into())],
        errors: vec![error("prior", 1, 1)],
        pending_comment: Some(PendingComment {
            // Existing data is literal: no NUL replacement, scalar conversion,
            // newline normalization or preprocessing diagnostics on seeded data.
            data: vec!['\0'.into(), CodePoint::new(0xd800).unwrap(), '\r'.into()],
        }),
        current_tag: Some(TagKind::End),
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
    Execution::new(State::CommentEnd, context)
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

fn assert_blocked(execution: &mut Execution, state: State) {
    let before = snapshot(execution);
    for _ in 0..2 {
        assert_eq!(execution.step(), Err(UnimplementedState { state }));
        assert_eq!(snapshot(execution), before);
    }
}

#[test]
fn anything_else_restores_only_two_hyphens_and_reconsumes_without_comment_processing() {
    let values = (0..=0x7f)
        .filter(|v| !matches!(v, 0x21 | 0x2d | 0x3e))
        .chain([
            0x85, 0xa0, 0xe9, 0x391, 0x301, 0x1f600, 0xd800, 0xdfff, 0xfdd0, 0x10ffff,
        ]);
    for value in values {
        let point = CodePoint::new(value).unwrap();
        let normalized = if value == 0x0d { '\n'.into() } else { point };
        let mut execution = isolated(&[point, 'z'.into()], true);
        let mut expected = snapshot(&execution);
        expected.state = State::Comment;
        expected.current = Some((
            Input::CodePoint(normalized),
            Position { line: 1, column: 1 },
        ));
        expected.comment.as_mut().unwrap().data.extend(points("--"));
        let preprocessing = match value {
            0xd800 | 0xdfff => Some("surrogate-in-input-stream"),
            0xfdd0 | 0x10ffff => Some("noncharacter-in-input-stream"),
            0x01..=0x08 | 0x0b | 0x0e..=0x1f | 0x7f | 0x85 => {
                Some("control-character-in-input-stream")
            }
            _ => None,
        };
        if let Some(code) = preprocessing {
            expected.input_errors.push(error(code, 1, 1));
        }
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Comment)));
        assert_eq!(snapshot(&execution), expected, "U+{value:04X}");
        // Source scheduled reconsumption; even NUL has no state error/replacement yet.
        assert_eq!(
            execution.context_mut().input.reconsume(),
            Err(CursorError::AlreadyReconsuming)
        );
        assert_blocked(&mut execution, State::Comment);
        // Inspect retained cursor only; never reconstruct/resume blocked execution.
        let cursor = &mut execution.context_mut().input;
        assert_eq!(cursor.consume(), Input::CodePoint(normalized));
        assert_eq!(cursor.current(), expected.current);
        assert_eq!(cursor.errors(), expected.input_errors);
        assert_eq!(cursor.consume(), Input::CodePoint('z'.into()));
        assert_eq!(
            cursor.position(),
            if normalized == '\n'.into() {
                Position { line: 2, column: 1 }
            } else {
                Position { line: 1, column: 2 }
            }
        );
    }
}

#[test]
fn hyphens_append_one_each_then_bang_consumes_without_appending_or_reconsuming() {
    let mut execution = isolated(&points("---!z"), true);
    let mut expected = snapshot(&execution);
    for column in 1..=3 {
        assert_eq!(execution.step(), Ok(Progress::Continue(State::CommentEnd)));
        expected.current = Some((Input::CodePoint('-'.into()), Position { line: 1, column }));
        expected.comment.as_mut().unwrap().data.push('-'.into());
        assert_eq!(snapshot(&execution), expected);
    }
    assert_eq!(
        execution.step(),
        Ok(Progress::Continue(State::CommentEndBang))
    );
    expected.state = State::CommentEndBang;
    expected.current = Some((
        Input::CodePoint('!'.into()),
        Position { line: 1, column: 4 },
    ));
    assert_eq!(snapshot(&execution), expected);
    assert_blocked(&mut execution, State::CommentEndBang);
    assert_eq!(
        execution.context_mut().input.consume(),
        Input::CodePoint('z'.into())
    );
    assert_eq!(
        execution.context().input.position(),
        Position { line: 1, column: 5 }
    );
}

#[test]
fn greater_than_takes_only_comment_then_real_data_consumes_tail_and_eof() {
    for data in [vec![], seeded().pending_comment.unwrap().data] {
        for kind in [TagKind::Start, TagKind::End] {
            let mut execution = isolated(&points("-->\0"), true);
            execution.context_mut().pending_comment = Some(PendingComment { data: data.clone() });
            execution.context_mut().current_tag = Some(kind);
            for _ in 0..2 {
                assert_eq!(execution.step(), Ok(Progress::Continue(State::CommentEnd)));
            }
            let mut expected = snapshot(&execution);
            assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
            expected.state = State::Data;
            expected.current = Some((
                Input::CodePoint('>'.into()),
                Position { line: 1, column: 3 },
            ));
            expected.tokens.push(Token::Comment {
                data: expected.comment.take().unwrap().data,
            });
            assert_eq!(snapshot(&execution), expected);
            let mut emitted = data.clone();
            emitted.extend(points("--"));
            assert_eq!(
                expected.tokens.last(),
                Some(&Token::Comment { data: emitted })
            );
            assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
            expected.current = Some((
                Input::CodePoint('\0'.into()),
                Position { line: 1, column: 4 },
            ));
            expected.tokens.push(Token::Character('\0'.into()));
            expected
                .errors
                .push(error("unexpected-null-character", 1, 4));
            assert_eq!(snapshot(&execution), expected);
            assert_eq!(execution.step(), Ok(Progress::Finished));
            expected.current = Some((Input::Eof, Position { line: 1, column: 5 }));
            expected.tokens.push(Token::Eof);
            assert_eq!(snapshot(&execution), expected);
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&execution), expected);
        }
    }
    // No implicit hyphens are added when closing an empty comment directly.
    let mut execution = isolated(&points(">"), true);
    execution.context_mut().pending_comment = Some(PendingComment::default());
    assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
    assert_eq!(
        execution.context().tokens.last(),
        Some(&Token::Comment { data: vec![] })
    );
}

#[test]
fn open_exhaustion_suspends_until_explicit_eof_diagnoses_and_emits_comment_then_eof() {
    for initial in [vec![], seeded().pending_comment.unwrap().data] {
        for input in ["", "-", "---"] {
            let mut execution = isolated(&points(input), false);
            execution.context_mut().pending_comment = Some(PendingComment {
                data: initial.clone(),
            });
            for _ in input.chars() {
                assert_eq!(execution.step(), Ok(Progress::Continue(State::CommentEnd)));
            }
            let mut expected = snapshot(&execution);
            for _ in 0..2 {
                execution.context_mut().input.push([]).unwrap();
                assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
                assert_eq!(snapshot(&execution), expected);
            }
            execution.context_mut().input.finish();
            assert_eq!(execution.step(), Ok(Progress::Finished));
            let column = input.len() + 1;
            expected.current = Some((Input::Eof, Position { line: 1, column }));
            expected.errors.push(error("eof-in-comment", 1, column));
            expected.tokens.push(Token::Comment {
                data: expected.comment.take().unwrap().data,
            });
            expected.tokens.push(Token::Eof);
            assert_eq!(snapshot(&execution), expected);
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&execution), expected);
            // EOF was consumed, not scheduled for reconsumption by CommentEnd.
            assert_eq!(execution.context_mut().input.reconsume(), Ok(()));
        }
    }
}

#[test]
fn direct_entry_accepts_reconsumed_characters_and_conceptual_eof_at_normalized_positions() {
    for item in [Some('>'), Some('!'), Some('-'), Some('\0'), None] {
        let mut context = seeded();
        context.input.push(points("\r\n")).unwrap();
        assert_eq!(context.input.consume(), Input::CodePoint('\n'.into()));
        if let Some(point) = item {
            context.input.push([point.into()]).unwrap();
        }
        context.input.finish();
        let input = item.map_or(Input::Eof, |p| Input::CodePoint(p.into()));
        assert_eq!(context.input.consume(), input);
        context.input.reconsume().unwrap();
        let mut execution = Execution::new(State::CommentEnd, context);
        let mut expected = snapshot(&execution);
        let result = match item {
            Some('>') => {
                expected.state = State::Data;
                expected.tokens.push(Token::Comment {
                    data: expected.comment.take().unwrap().data,
                });
                Progress::Continue(State::Data)
            }
            Some('!') => {
                expected.state = State::CommentEndBang;
                Progress::Continue(State::CommentEndBang)
            }
            Some('-') => {
                expected.comment.as_mut().unwrap().data.push('-'.into());
                Progress::Continue(State::CommentEnd)
            }
            Some('\0') => {
                expected.state = State::Comment;
                expected.comment.as_mut().unwrap().data.extend(points("--"));
                Progress::Continue(State::Comment)
            }
            None => {
                expected.errors.push(error("eof-in-comment", 2, 1));
                expected.tokens.push(Token::Comment {
                    data: expected.comment.take().unwrap().data,
                });
                expected.tokens.push(Token::Eof);
                Progress::Finished
            }
            _ => unreachable!(),
        };
        assert_eq!(execution.step(), Ok(result));
        assert_eq!(snapshot(&execution), expected);
        assert_eq!(
            expected.current,
            Some((input, Position { line: 2, column: 1 }))
        );
        if matches!(item, Some('!' | '\0')) {
            assert_blocked(&mut execution, expected.state);
        }
    }
}

#[test]
fn normal_data_comment_entry_still_blocks_before_creation() {
    let mut context = Context::default();
    context.input.push(points("<!--x-->tail")).unwrap();
    context.input.finish();
    let mut execution = Execution::new(State::Data, context);
    assert_eq!(execution.step(), Ok(Progress::Continue(State::TagOpen)));
    assert_eq!(
        execution.step(),
        Ok(Progress::Continue(State::MarkupDeclarationOpen))
    );
    assert_eq!(
        execution.context().input.current(),
        Some((
            Input::CodePoint('!'.into()),
            Position { line: 1, column: 2 }
        ))
    );
    assert!(execution.context().pending_comment.is_none());
    assert!(execution.context().tokens.is_empty());
    assert!(execution.context().errors.is_empty());
    assert_blocked(&mut execution, State::MarkupDeclarationOpen);
    assert_eq!(
        execution.context_mut().input.consume(),
        Input::CodePoint('-'.into())
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
        trace.push((result, snapshot(execution)));
        if !matches!(result, Ok(Progress::Continue(_))) {
            let terminal = snapshot(execution);
            assert_eq!(execution.step(), result);
            assert_eq!(snapshot(execution), terminal);
            return true;
        }
    }
    panic!("CommentEnd source trace did not consume or terminate");
}

#[test]
fn every_two_chunk_split_and_single_point_chunks_match_closed_source_traces() {
    let mut samples: Vec<_> = [
        "",
        "-",
        "---",
        ">",
        "-->",
        "!tail",
        "--!tail",
        "x>",
        "--\0>",
        "--\r\n>",
        "--\r\r\n",
        "--><&",
        "-->\r\n\0😀\r\r\n\r",
        "-->É😀",
        "--><!",
        "-->&",
        "--><?",
        "--><a x='v'>x</a>",
    ]
    .into_iter()
    .map(points)
    .collect();
    for value in [0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x0b] {
        let point = CodePoint::new(value).unwrap();
        let mut fallback = points("--");
        fallback.extend([point, '>'.into()]);
        samples.push(fallback);
        let mut tail = points("-->");
        tail.extend([point, '\r'.into(), '\n'.into()]);
        samples.push(tail);
    }
    for input in samples {
        let run = |chunks: Vec<&[CodePoint]>, closed: bool| {
            let mut execution = isolated(&[], false);
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
                    // A stub ends this run permanently, even if later chunks exist.
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
                "split {split} of {input:?}"
            );
        }
        assert_eq!(run(input.chunks(1).collect(), false), expected, "{input:?}");
    }
}
