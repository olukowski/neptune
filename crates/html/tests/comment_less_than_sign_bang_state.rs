//! Branch oracle: pinned WHATWG source 404b3eb14a7a936dddefb2d78e224dbeb01d2358,
//! lines 144610–144624. These are isolated source-step checks, not conformance.

use neptune_html::tokenizer::{
    Attribute, CodePoint, Context, CursorError, Execution, Input, ParseError, PendingComment,
    PendingEndTag, PendingStartTag, Position, Progress, State, TagKind, Token, UnimplementedState,
};

fn points(text: &str) -> Vec<CodePoint> {
    text.chars().map(CodePoint::from).collect()
}

fn point(value: u32) -> CodePoint {
    CodePoint::new(value).unwrap()
}

fn error(code: &'static str, line: usize, column: usize) -> ParseError {
    ParseError {
        code,
        position: Position { line, column },
    }
}

fn literal_seed() -> Vec<CodePoint> {
    // Stored data is already token data: do not preprocess, normalize, or replace it.
    [points("seed\0\r<!"), vec![point(0xd800), point(0xffff)]].concat()
}

fn seeded(kind: TagKind, data: Vec<CodePoint>) -> Context {
    let attribute = Attribute {
        name: points("kept"),
        value: vec![point(0xdfff), '\0'.into()],
    };
    let mut context = Context {
        tokens: vec![
            Token::Comment { data: vec![] },
            Token::Character('p'.into()),
        ],
        errors: vec![error("prior-state-error", 1, 1)],
        current_tag: Some(kind),
        pending_start_tag: Some(PendingStartTag {
            name: points("start"),
            self_closing: true,
            attributes: vec![attribute.clone()],
            discarded_current_attribute: Some(attribute.clone()),
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("end"),
            attributes: vec![attribute.clone()],
            discarded_current_attribute: Some(attribute),
            ..Default::default()
        }),
        pending_comment: Some(PendingComment { data }),
        temporary_buffer: literal_seed(),
        return_state: Some(State::AttributeValueSingleQuoted),
        last_start_tag: Some(points("title")),
        ..Default::default()
    };
    // Establish prior preprocessing diagnostics and a consumed predecessor '!'.
    // Normal entry has already appended '<!' to comment data; empty and other
    // seeded data are also legal direct-entry preservation probes.
    context
        .input
        .push([vec![point(0xdfff)], points("\r\n<!")].concat())
        .unwrap();
    for expected in [point(0xdfff), '\n'.into(), '<'.into(), '!'.into()] {
        assert_eq!(context.input.consume(), Input::CodePoint(expected));
    }
    assert_eq!(context.input.position(), Position { line: 2, column: 2 });
    context
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Snapshot {
    state: State,
    current: Option<(Input, Position)>,
    position: Position,
    input_errors: Vec<ParseError>,
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
    current_tag: Option<TagKind>,
    start: Option<PendingStartTag>,
    end: Option<PendingEndTag>,
    comment: Option<PendingComment>,
    last_start_tag: Option<Vec<CodePoint>>,
    return_state: Option<State>,
    scratch: Vec<CodePoint>,
}

fn snapshot(run: &Execution) -> Snapshot {
    let context = run.context();
    Snapshot {
        state: run.state(),
        current: context.input.current(),
        position: context.input.position(),
        input_errors: context.input.errors().to_vec(),
        tokens: context.tokens.clone(),
        errors: context.errors.clone(),
        current_tag: context.current_tag,
        start: context.pending_start_tag.clone(),
        end: context.pending_end_tag.clone(),
        comment: context.pending_comment.clone(),
        last_start_tag: context.last_start_tag.clone(),
        return_state: context.return_state,
        scratch: context.temporary_buffer.clone(),
    }
}

fn terminal(run: &mut Execution, state: State) {
    let before = snapshot(run);
    for _ in 0..3 {
        assert_eq!(run.step(), Err(UnimplementedState { state }));
        assert_eq!(snapshot(run), before);
    }
}

fn values() -> Vec<u32> {
    (0..=0x7f)
        .chain([
            0x80, 0x9f, 0xa0, 0xc9, 0x2010, 0x2212, 0xd800, 0xdfff, 0xfdd0, 0xffff, 0x1f600,
            0x10ffff,
        ])
        .collect()
}

fn preprocessing_error(value: u32) -> Option<&'static str> {
    match value {
        0x01..=0x08 | 0x0b | 0x0e..=0x1f | 0x7f..=0x9f => Some("control-character-in-input-stream"),
        0xd800 | 0xdfff => Some("surrogate-in-input-stream"),
        0xfdd0 | 0xffff | 0x10ffff => Some("noncharacter-in-input-stream"),
        _ => None,
    }
}

#[test]
fn all_branches_preserve_comment_and_unrelated_context_before_terminal_destinations() {
    for value in values() {
        for kind in [TagKind::Start, TagKind::End] {
            for data in [vec![], points("<!"), literal_seed()] {
                for finished in [false, true] {
                    let mut context = seeded(kind, data.clone());
                    context.input.push([point(value), 'z'.into()]).unwrap();
                    if finished {
                        context.input.finish();
                    }
                    let mut run = Execution::new(State::CommentLessThanSignBang, context);
                    let mut expected = snapshot(&run);
                    let target = if value == 0x2d {
                        State::CommentLessThanSignBangDash
                    } else {
                        State::Comment
                    };
                    let normalized = if value == 0x0d { 0x0a } else { value };
                    expected.state = target;
                    expected.position = Position { line: 2, column: 3 };
                    expected.current =
                        Some((Input::CodePoint(point(normalized)), expected.position));
                    if let Some(code) = preprocessing_error(value) {
                        expected.input_errors.push(error(code, 2, 3));
                    }
                    assert_eq!(run.step(), Ok(Progress::Continue(target)), "U+{value:04X}");
                    // No output (including EOF), state diagnostic, append, clear,
                    // replacement, or take, even for NUL/non-scalars/closing syntax.
                    assert_eq!(snapshot(&run), expected, "U+{value:04X}");
                    if value != 0x2d {
                        assert_eq!(
                            run.context_mut().input.reconsume(),
                            Err(CursorError::AlreadyReconsuming)
                        );
                    }
                    terminal(&mut run, target);
                }
            }
        }
    }
}

#[test]
fn cursor_probe_proves_consumed_hyphen_and_reconsumed_fallback_without_running_destinations() {
    for value in values() {
        let mut context = seeded(TagKind::Start, literal_seed());
        context.input.push([point(value), 'z'.into()]).unwrap();
        context.input.finish();
        let mut run = Execution::new(State::CommentLessThanSignBang, context);
        assert!(matches!(run.step(), Ok(Progress::Continue(_))));
        let after_source = snapshot(&run);
        let cursor = &mut run.context_mut().input;
        if value != 0x2d {
            let normalized = if value == 0x0d { 0x0a } else { value };
            assert_eq!(cursor.consume(), Input::CodePoint(point(normalized)));
            assert_eq!(cursor.current(), after_source.current);
            assert_eq!(cursor.errors(), after_source.input_errors);
        }
        // Cursor-only probes happen before any destination attempt. They are not
        // resumed/reconstructed blocked tokenizer executions.
        assert_eq!(cursor.consume(), Input::CodePoint('z'.into()));
        assert_eq!(
            cursor.position(),
            if matches!(value, 0x0a | 0x0d) {
                Position { line: 3, column: 1 }
            } else {
                Position { line: 2, column: 4 }
            }
        );
        assert_eq!(cursor.consume(), Input::Eof);
        assert_eq!(cursor.errors(), after_source.input_errors);
    }
}

#[test]
fn open_exhaustion_is_effect_free_but_explicit_eof_is_reconsumed_not_emitted_or_taken() {
    for predecessor in [false, true] {
        let context = if predecessor {
            seeded(TagKind::End, literal_seed())
        } else {
            Context {
                pending_comment: Some(PendingComment::default()),
                ..Default::default()
            }
        };
        let mut run = Execution::new(State::CommentLessThanSignBang, context);
        let before = snapshot(&run);
        for _ in 0..3 {
            run.context_mut().input.push([]).unwrap();
            assert_eq!(run.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&run), before);
        }
        run.context_mut().input.finish();
        let position = if predecessor {
            Position { line: 2, column: 3 }
        } else {
            Position { line: 1, column: 1 }
        };
        let mut expected = before;
        expected.state = State::Comment;
        expected.position = position;
        expected.current = Some((Input::Eof, position));
        assert_eq!(run.step(), Ok(Progress::Continue(State::Comment)));
        assert_eq!(snapshot(&run), expected);
        assert_eq!(
            run.context_mut().input.reconsume(),
            Err(CursorError::AlreadyReconsuming)
        );
        terminal(&mut run, State::Comment);
    }
    // Independently inspect scheduled conceptual EOF before a destination attempt.
    let mut context = seeded(TagKind::Start, points("<!"));
    context.input.finish();
    let mut run = Execution::new(State::CommentLessThanSignBang, context);
    assert_eq!(run.step(), Ok(Progress::Continue(State::Comment)));
    let before = snapshot(&run);
    assert_eq!(run.context_mut().input.consume(), Input::Eof);
    assert_eq!(snapshot(&run), before);
    // Successful scheduling proves the first scheduled reconsume was consumed.
    assert_eq!(run.context_mut().input.reconsume(), Ok(()));
}

#[test]
fn suspension_can_be_fed_then_hyphen_switches_without_appending_or_reconsuming() {
    let mut run = Execution::new(
        State::CommentLessThanSignBang,
        seeded(TagKind::End, points("<!")),
    );
    assert_eq!(run.step(), Ok(Progress::NeedMoreInput));
    run.context_mut().input.push(points("-")).unwrap();
    assert_eq!(
        run.step(),
        Ok(Progress::Continue(State::CommentLessThanSignBangDash))
    );
    let before = snapshot(&run);
    assert_eq!(before.comment, Some(PendingComment { data: points("<!") }));
    assert_eq!(run.context_mut().input.consume(), Input::NeedMoreInput);
    assert_eq!(snapshot(&run), before);
    // A destination stub blocks even with open exhausted input, never suspends.
    terminal(&mut run, State::CommentLessThanSignBangDash);
}

#[test]
fn normal_data_comment_entry_still_blocks_at_markup_declaration_open() {
    let mut context = Context {
        pending_comment: Some(PendingComment {
            data: literal_seed(),
        }),
        ..Default::default()
    };
    context.input.push(points("<!--tail-->")).unwrap();
    context.input.finish();
    let mut run = Execution::new(State::Data, context);
    let mut expected = snapshot(&run);
    for (column, character, target) in [
        (1, '<', State::TagOpen),
        (2, '!', State::MarkupDeclarationOpen),
    ] {
        expected.state = target;
        expected.position = Position { line: 1, column };
        expected.current = Some((Input::CodePoint(character.into()), expected.position));
        assert_eq!(run.step(), Ok(Progress::Continue(target)));
        assert_eq!(snapshot(&run), expected);
    }
    terminal(&mut run, State::MarkupDeclarationOpen);
}

type Trace = Vec<(Result<Progress, UnimplementedState>, Snapshot)>;

fn drain(run: &mut Execution, trace: &mut Trace) -> bool {
    loop {
        let before = snapshot(run);
        let result = run.step();
        if result == Ok(Progress::NeedMoreInput) {
            assert_eq!(snapshot(run), before);
            for _ in 0..3 {
                assert_eq!(run.step(), result);
                assert_eq!(snapshot(run), before);
            }
            return false;
        }
        trace.push((result, snapshot(run)));
        if let Err(blocked) = result {
            terminal(run, blocked.state);
            return true;
        }
        assert!(matches!(result, Ok(Progress::Continue(_))));
    }
}

fn trace(chunks: &[&[CodePoint]], closed: bool) -> Trace {
    let mut run = Execution::new(
        State::CommentLessThanSignBang,
        seeded(TagKind::End, literal_seed()),
    );
    let mut trace = vec![];
    for chunk in chunks {
        run.context_mut().input.push(chunk.iter().copied()).unwrap();
        if !closed && drain(&mut run, &mut trace) {
            return trace; // Never feed or reconstruct terminal blocked execution.
        }
    }
    run.context_mut().input.finish();
    assert!(drain(&mut run, &mut trace));
    trace
}

#[test]
fn every_two_chunk_split_and_single_point_chunks_match_closed_source_step_trace() {
    let mut inputs: Vec<_> = [
        "",
        "-",
        "-->",
        "-tail\0",
        "x",
        "!",
        "<",
        ">",
        "&amp;",
        "\0tail",
        "\r\nX",
        "\r\r\n",
        "\n",
        "É😀",
        "\u{b}\u{fdd0}\0",
    ]
    .into_iter()
    .map(points)
    .collect();
    inputs.push(vec![
        point(0xd800),
        '\r'.into(),
        '\n'.into(),
        point(0x10ffff),
    ]);
    for input in inputs {
        let expected = trace(&[&input], true);
        for split in 0..=input.len() {
            assert_eq!(
                trace(&[&input[..split], &[], &input[split..]], false),
                expected,
                "split {split}: {input:?}"
            );
        }
        assert_eq!(
            trace(&input.chunks(1).collect::<Vec<_>>(), false),
            expected,
            "single code points: {input:?}"
        );
    }
}
