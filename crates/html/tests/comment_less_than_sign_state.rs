use neptune_html::tokenizer::{
    Attribute, CodePoint, Context, CursorError, Execution, Input, ParseError, PendingComment,
    PendingEndTag, PendingStartTag, Position, Progress, State, TagKind, Token, UnimplementedState,
};

const SOURCE: State = State::CommentLessThanSign;

fn points(text: &str) -> Vec<CodePoint> {
    text.chars().map(Into::into).collect()
}

fn error(code: &'static str, line: usize, column: usize) -> ParseError {
    ParseError {
        code,
        position: Position { line, column },
    }
}

fn seeded(kind: TagKind) -> Context {
    let attribute = Attribute {
        name: points("name"),
        value: points("value"),
    };
    let mut data = points("seed\0\r");
    data.extend([CodePoint::new(0xd800).unwrap(), '<'.into()]);
    let mut context = Context {
        tokens: vec![
            Token::Character('p'.into()),
            Token::Comment { data: vec![] },
        ],
        errors: vec![error("prior", 1, 1)],
        current_tag: Some(kind),
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
        last_start_tag: Some(points("last")),
        return_state: Some(State::Rcdata),
        temporary_buffer: points("scratch\0\r"),
        ..Context::default()
    };
    // Model only the already-consumed predecessor input, not its algorithm.
    context.input.push(points("\r\n<")).unwrap();
    assert_eq!(context.input.consume(), Input::CodePoint('\n'.into()));
    assert_eq!(context.input.consume(), Input::CodePoint('<'.into()));
    context
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Snapshot {
    state: State,
    current: Option<(Input, Position)>,
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
    input_errors: Vec<ParseError>,
    comment: Option<PendingComment>,
    start: Option<PendingStartTag>,
    end: Option<PendingEndTag>,
    kind: Option<TagKind>,
    last: Option<Vec<CodePoint>>,
    return_state: Option<State>,
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
        comment: context.pending_comment.clone(),
        start: context.pending_start_tag.clone(),
        end: context.pending_end_tag.clone(),
        kind: context.current_tag,
        last: context.last_start_tag.clone(),
        return_state: context.return_state,
        buffer: context.temporary_buffer.clone(),
    }
}

fn assert_terminal_stub(execution: &mut Execution, expected: &Snapshot) {
    let blocked = Err(UnimplementedState {
        state: expected.state,
    });
    assert_eq!(execution.step(), blocked);
    assert_eq!(snapshot(execution), *expected);
    assert_eq!(execution.step(), blocked);
    assert_eq!(snapshot(execution), *expected);
}

#[test]
fn every_ascii_branch_and_unicode_fallback_have_exact_source_effects() {
    for kind in [TagKind::Start, TagKind::End] {
        for value in (0..=0x7f).chain([
            0x80, 0x9f, 0xa0, 0xc9, 0x3a9, 0xd800, 0xdfff, 0xfdd0, 0xfffe, 0x1fffe, 0x1f600,
            0x10ffff,
        ]) {
            let point = CodePoint::new(value).unwrap();
            let mut context = seeded(kind);
            context.input.push([point, 'z'.into()]).unwrap();
            context.input.finish();
            let mut execution = Execution::new(SOURCE, context);
            let mut expected = snapshot(&execution);
            let normalized = if value == 0x0d { '\n'.into() } else { point };
            expected.current = Some((
                Input::CodePoint(normalized),
                Position { line: 2, column: 2 },
            ));
            let preprocessing = match value {
                0xd800..=0xdfff => Some("surrogate-in-input-stream"),
                0xfdd0 | 0xfffe | 0x1fffe | 0x10ffff => Some("noncharacter-in-input-stream"),
                0x01..=0x08 | 0x0b | 0x0e..=0x1f | 0x7f..=0x9f => {
                    Some("control-character-in-input-stream")
                }
                _ => None,
            };
            if let Some(code) = preprocessing {
                expected.input_errors.push(error(code, 2, 2));
            }
            expected.state = match value {
                0x21 => State::CommentLessThanSignBang,
                0x3c => SOURCE,
                _ => State::Comment,
            };
            if matches!(value, 0x21 | 0x3c) {
                expected.comment.as_mut().unwrap().data.push(point);
            }
            assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
            assert_eq!(snapshot(&execution), expected, "U+{value:04X}");
            if expected.state != SOURCE {
                // Inspect source data/errors before the unsupported destination.
                assert_terminal_stub(&mut execution, &expected);
            }
            // Inspect the cursor only; never reconstruct/resume a blocked execution.
            // Append branches consumed once; fallback scheduled literal reconsumption.
            if !matches!(value, 0x21 | 0x3c) {
                assert_eq!(
                    execution.context_mut().input.reconsume(),
                    Err(CursorError::AlreadyReconsuming)
                );
                assert_eq!(
                    execution.context_mut().input.consume(),
                    Input::CodePoint(normalized)
                );
                assert_eq!(execution.context().input.current(), expected.current);
                assert_eq!(execution.context().input.errors(), expected.input_errors);
            }
            assert_eq!(
                execution.context_mut().input.consume(),
                Input::CodePoint('z'.into())
            );
            assert_eq!(
                execution.context().input.position(),
                if normalized == '\n'.into() {
                    Position { line: 3, column: 1 }
                } else {
                    Position { line: 2, column: 3 }
                }
            );
            assert_eq!(execution.context_mut().input.consume(), Input::Eof);
        }
    }
}

#[test]
fn repeated_less_than_and_bang_append_in_order_without_taking_comment() {
    let mut context = Context {
        pending_comment: Some(PendingComment { data: points("<") }),
        ..Context::default()
    };
    context.input.push(points("<<!tail")).unwrap();
    context.input.finish();
    let mut execution = Execution::new(SOURCE, context);
    let mut expected = snapshot(&execution);
    for (index, point) in points("<<!").into_iter().enumerate() {
        expected.current = Some((
            Input::CodePoint(point),
            Position {
                line: 1,
                column: index + 1,
            },
        ));
        expected.comment.as_mut().unwrap().data.push(point);
        if point == '!'.into() {
            expected.state = State::CommentLessThanSignBang;
        }
        assert_eq!(execution.step(), Ok(Progress::Continue(expected.state)));
        assert_eq!(snapshot(&execution), expected);
    }
    assert_eq!(expected.comment.as_ref().unwrap().data, points("<<<!"));
    assert_terminal_stub(&mut execution, &expected);
    assert_eq!(
        execution.context_mut().input.consume(),
        Input::CodePoint('t'.into())
    );
}

#[test]
fn open_exhaustion_is_effect_free_but_explicit_eof_reconsumes_in_comment() {
    for prefix in ["", "<<<"] {
        let mut context = seeded(TagKind::Start);
        context.input.push(points(prefix)).unwrap();
        let mut execution = Execution::new(SOURCE, context);
        for _ in prefix.chars() {
            assert_eq!(execution.step(), Ok(Progress::Continue(SOURCE)));
        }
        let mut expected = snapshot(&execution);
        for _ in 0..3 {
            execution.context_mut().input.push([]).unwrap();
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&execution), expected);
        }
        execution.context_mut().input.finish();
        expected.state = State::Comment;
        expected.current = Some((
            Input::Eof,
            Position {
                line: 2,
                column: prefix.len() + 2,
            },
        ));
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Comment)));
        // No eof-in-comment error, comment take/emission, or EOF token at source.
        assert_eq!(snapshot(&execution), expected);
        assert_terminal_stub(&mut execution, &expected);
        assert_eq!(
            execution.context_mut().input.reconsume(),
            Err(CursorError::AlreadyReconsuming)
        );
        assert_eq!(execution.context_mut().input.consume(), Input::Eof);
        assert_eq!(execution.context().input.current(), expected.current);
        assert_eq!(execution.context().input.errors(), expected.input_errors);
    }
}

type Trace = Vec<(Result<Progress, UnimplementedState>, Snapshot)>;

fn drain(execution: &mut Execution, trace: &mut Trace) -> bool {
    loop {
        let before = snapshot(execution);
        let result = execution.step();
        if result == Ok(Progress::NeedMoreInput) {
            assert_eq!(snapshot(execution), before);
            assert_eq!(execution.step(), result);
            assert_eq!(snapshot(execution), before);
            return false;
        }
        let after = snapshot(execution);
        trace.push((result, after.clone()));
        if result.is_err() {
            assert_terminal_stub(execution, &after);
            return true;
        }
        assert!(matches!(result, Ok(Progress::Continue(_))));
    }
}

fn trace(input: &[CodePoint], chunks: &[&[CodePoint]], closed: bool) -> Trace {
    let mut execution = Execution::new(SOURCE, seeded(TagKind::End));
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
            if drain(&mut execution, &mut trace) {
                return trace; // Never feed or resume after a terminal block.
            }
        }
        execution.context_mut().input.finish();
    }
    assert!(drain(&mut execution, &mut trace));
    trace
}

#[test]
fn every_two_chunk_split_and_single_point_chunks_preserve_source_step_traces() {
    let mut cases = vec![
        vec![],
        points("<"),
        points("<<<<"),
        points("!tail"),
        points("<<!tail"),
        points("<<x>"),
        points("<<\0tail"),
        points("<<\r\n!"),
        points("<<\r\r"),
        points("<<\r"),
        points("<<&"),
        points("<<->"),
        points("<<\u{1f600}"),
    ];
    for value in [0xd800, 0xdfff, 0xfdd0, 0x1fffe, 0x0b] {
        let mut input = points("<<");
        input.extend([CodePoint::new(value).unwrap(), 'z'.into()]);
        cases.push(input);
    }
    for input in cases {
        let expected = trace(&input, &[], true);
        for split in 0..=input.len() {
            assert_eq!(
                trace(&input, &[&input[..split], &input[split..]], false),
                expected
            );
        }
        assert_eq!(
            trace(&input, &input.chunks(1).collect::<Vec<_>>(), false),
            expected
        );
    }
}

#[test]
fn ordinary_data_comment_entry_still_blocks_before_comment_creation() {
    let mut context = Context::default();
    context.input.push(points("<!--x-->")).unwrap();
    context.input.finish();
    let mut execution = Execution::new(State::Data, context);
    assert_eq!(execution.step(), Ok(Progress::Continue(State::TagOpen)));
    assert_eq!(
        execution.step(),
        Ok(Progress::Continue(State::MarkupDeclarationOpen))
    );
    let expected = snapshot(&execution);
    assert_eq!(
        expected.current,
        Some((
            Input::CodePoint('!'.into()),
            Position { line: 1, column: 2 }
        ))
    );
    assert!(expected.comment.is_none());
    assert!(expected.tokens.is_empty());
    assert!(expected.errors.is_empty());
    assert!(expected.input_errors.is_empty());
    assert_terminal_stub(&mut execution, &expected);
}
