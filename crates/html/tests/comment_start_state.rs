//! Local source-step evidence: HTML 404b3eb14a7a936dddefb2d78e224dbeb01d2358,
//! #comment-start-state, source lines 144514–144532. Not upstream conformance.

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
        tokens: vec![
            Token::Character('p'.into()),
            Token::Comment { data: vec![] },
        ],
        errors: vec![error("prior-diagnostic", 1, 1)],
        last_start_tag: Some(points("last")),
        return_state: Some(State::AttributeValueSingleQuoted),
        current_tag: Some(TagKind::End),
        pending_start_tag: Some(PendingStartTag {
            name: points("start"),
            attributes: vec![attribute.clone()],
            discarded_current_attribute: Some(attribute.clone()),
            ..Default::default()
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("end"),
            self_closing: true,
            attributes: vec![attribute.clone()],
            discarded_current_attribute: Some(attribute),
        }),
        // Existing data is literal: no normalization, NUL replacement, or diagnostics.
        pending_comment: Some(PendingComment {
            data: vec![
                'A'.into(),
                '\0'.into(),
                '\r'.into(),
                '<'.into(),
                '&'.into(),
                CodePoint::new(0xdfff).unwrap(),
                CodePoint::new(0x10ffff).unwrap(),
            ],
        }),
        temporary_buffer: points("untouched"),
        ..Default::default()
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Storage {
    last: Option<Vec<CodePoint>>,
    return_state: Option<State>,
    selected: Option<TagKind>,
    start: Option<PendingStartTag>,
    end: Option<PendingEndTag>,
    comment: Option<PendingComment>,
    buffer: Vec<CodePoint>,
}

fn storage(context: &Context) -> Storage {
    Storage {
        last: context.last_start_tag.clone(),
        return_state: context.return_state,
        selected: context.current_tag,
        start: context.pending_start_tag.clone(),
        end: context.pending_end_tag.clone(),
        comment: context.pending_comment.clone(),
        buffer: context.temporary_buffer.clone(),
    }
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

fn snapshot(run: &Execution) -> Snapshot {
    let context = run.context();
    Snapshot {
        state: run.state(),
        current: context.input.current(),
        tokens: context.tokens.clone(),
        errors: context.errors.clone(),
        input_errors: context.input.errors().to_vec(),
        storage: storage(context),
    }
}

// Direct isolated entry: position after a normalized prefix, without implementing
// or simulating MarkupDeclarationOpen. The caller already owns a pending comment.
fn entry(mut context: Context) -> Execution {
    context.input.push(points("\r\n<!--")).unwrap();
    for point in points("\n<!--") {
        assert_eq!(context.input.consume(), Input::CodePoint(point));
    }
    Execution::new(State::CommentStart, context)
}

fn terminal(run: &mut Execution, result: Result<Progress, UnimplementedState>) {
    let before = snapshot(run);
    for _ in 0..3 {
        assert_eq!(run.step(), result);
        assert_eq!(snapshot(run), before);
    }
}

// The predecessor's independently asserted source snapshot schedules this item
// for real Comment. Check that destination separately, including EOF/take order;
// preprocessing diagnostics and unrelated context must remain unchanged.
fn comment_replay(run: &mut Execution, expected: &mut Snapshot) {
    assert_eq!(expected.state, State::Comment);
    let (input, position) = expected.current.unwrap();
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
                0x3c => {
                    expected.storage.comment.as_mut().unwrap().data.push(point);
                    expected.state = State::CommentLessThanSign;
                }
                0x2d => expected.state = State::CommentEndDash,
                0 => {
                    expected.errors.push(ParseError {
                        code: "unexpected-null-character",
                        position,
                    });
                    expected
                        .storage
                        .comment
                        .as_mut()
                        .unwrap()
                        .data
                        .push('\u{fffd}'.into());
                }
                _ => expected.storage.comment.as_mut().unwrap().data.push(point),
            }
            Progress::Continue(expected.state)
        }
        Input::NeedMoreInput => panic!("a predecessor replay requires current input"),
    };
    assert_eq!(run.step(), Ok(result));
    assert_eq!(snapshot(run), *expected);
    if result == Progress::Finished {
        assert_eq!(run.step(), Ok(result));
        assert_eq!(snapshot(run), *expected);
    }
}

#[test]
fn all_ascii_and_representative_non_scalars_have_exact_source_effects() {
    for value in (0..=0x7f).chain([
        0x80, 0x9f, 0xa0, 0xc9, 0x3a9, 0x2010, 0x2212, 0xfeff, 0x1f600, 0xd800, 0xdfff, 0xfdd0,
        0xffff, 0x10ffff,
    ]) {
        for finished in [false, true] {
            for selected in [None, Some(TagKind::Start), Some(TagKind::End)] {
                let mut context = seeded_context();
                context.current_tag = selected;
                let mut run = entry(context);
                let point = CodePoint::new(value).unwrap();
                let normalized = if value == 0xd { '\n'.into() } else { point };
                run.context_mut().input.push([point, 'z'.into()]).unwrap();
                if finished {
                    run.context_mut().input.finish();
                }
                let mut expected = snapshot(&run);
                expected.current = Some((
                    Input::CodePoint(normalized),
                    Position { line: 2, column: 5 },
                ));
                let diagnostic = match value {
                    0xd800..=0xdfff => Some("surrogate-in-input-stream"),
                    0xfdd0 | 0xffff | 0x10ffff => Some("noncharacter-in-input-stream"),
                    1..=8 | 0xb | 0xe..=0x1f | 0x7f..=0x9f => {
                        Some("control-character-in-input-stream")
                    }
                    _ => None,
                };
                expected.input_errors = diagnostic
                    .into_iter()
                    .map(|code| error(code, 2, 5))
                    .collect();
                expected.state = match value {
                    0x2d => State::CommentStartDash,
                    0x3e => State::Data,
                    _ => State::Comment,
                };
                if value == 0x3e {
                    let comment = expected.storage.comment.take().unwrap();
                    expected.tokens.push(Token::Comment { data: comment.data });
                    expected
                        .errors
                        .push(error("abrupt-closing-of-empty-comment", 2, 5));
                }
                assert_eq!(run.step(), Ok(Progress::Continue(expected.state)));
                assert_eq!(snapshot(&run), expected, "U+{value:04X}");
                if expected.state == State::Comment {
                    assert_eq!(
                        run.context_mut().input.reconsume(),
                        Err(CursorError::AlreadyReconsuming)
                    );
                }
                if value == 0x2d {
                    // CommentStart's source effects above are unchanged. The
                    // real CommentStartDash now consumes the next 'z', appends
                    // only the deferred hyphen, and reconsumes in Comment.
                    expected.state = State::Comment;
                    expected.current = Some((
                        Input::CodePoint('z'.into()),
                        Position { line: 2, column: 6 },
                    ));
                    expected
                        .storage
                        .comment
                        .as_mut()
                        .unwrap()
                        .data
                        .push('-'.into());
                    assert_eq!(run.step(), Ok(Progress::Continue(State::Comment)));
                    assert_eq!(snapshot(&run), expected);
                    assert_eq!(
                        run.context_mut().input.reconsume(),
                        Err(CursorError::AlreadyReconsuming)
                    );
                    comment_replay(&mut run, &mut expected);
                } else if value != 0x3e {
                    comment_replay(&mut run, &mut expected);
                } else {
                    expected.current = Some((
                        Input::CodePoint('z'.into()),
                        Position { line: 2, column: 6 },
                    ));
                    expected.tokens.push(Token::Character('z'.into()));
                    assert_eq!(run.step(), Ok(Progress::Continue(State::Data)));
                    assert_eq!(snapshot(&run), expected);
                    if !finished {
                        for _ in 0..3 {
                            assert_eq!(run.step(), Ok(Progress::NeedMoreInput));
                            assert_eq!(snapshot(&run), expected);
                        }
                        run.context_mut().input.finish();
                    }
                    expected.current = Some((Input::Eof, Position { line: 2, column: 7 }));
                    expected.tokens.push(Token::Eof);
                    assert_eq!(run.step(), Ok(Progress::Finished));
                    assert_eq!(snapshot(&run), expected);
                    terminal(&mut run, Ok(Progress::Finished));
                }
            }
        }
    }
}

#[test]
fn hyphen_entry_executes_all_real_start_dash_branches_after_exact_source_effects() {
    for seeded in [false, true] {
        for finished in [false, true] {
            // These are destination inputs after CommentStart consumes '-'.
            for next in [
                None,
                Some(0x2d),
                Some(0x3e),
                Some(0x78),
                Some(0),
                Some(0x0d),
                Some(0xd800),
                Some(0xfdd0),
                Some(0x0b),
            ] {
                let context = if seeded {
                    seeded_context()
                } else {
                    Context {
                        pending_comment: Some(PendingComment::default()),
                        ..Default::default()
                    }
                };
                let mut run = entry(context);
                let mut input = points("-");
                if let Some(value) = next {
                    input.push(CodePoint::new(value).unwrap());
                    input.extend(points("z\r\n\0"));
                }
                run.context_mut().input.push(input).unwrap();
                if finished {
                    run.context_mut().input.finish();
                }
                let mut expected = snapshot(&run);
                expected.state = State::CommentStartDash;
                expected.current = Some((
                    Input::CodePoint('-'.into()),
                    Position { line: 2, column: 5 },
                ));
                assert_eq!(run.step(), Ok(Progress::Continue(State::CommentStartDash)));
                assert_eq!(snapshot(&run), expected); // No append/error/output/take yet.

                if next.is_none() && !finished {
                    for _ in 0..3 {
                        run.context_mut().input.push([]).unwrap();
                        assert_eq!(run.step(), Ok(Progress::NeedMoreInput));
                        assert_eq!(snapshot(&run), expected);
                    }
                    run.context_mut().input.finish();
                }
                let position = Position { line: 2, column: 6 };
                expected.current = Some((
                    match next {
                        None => Input::Eof,
                        Some(0x0d) => Input::CodePoint('\n'.into()),
                        Some(value) => Input::CodePoint(CodePoint::new(value).unwrap()),
                    },
                    position,
                ));
                match next {
                    None => {
                        expected.errors.push(error("eof-in-comment", 2, 6));
                        let comment = expected.storage.comment.take().unwrap();
                        expected
                            .tokens
                            .extend([Token::Comment { data: comment.data }, Token::Eof]);
                        // EOF does not switch back to Data or append the hyphen.
                        assert_eq!(run.step(), Ok(Progress::Finished));
                        assert_eq!(snapshot(&run), expected);
                        terminal(&mut run, Ok(Progress::Finished));
                    }
                    Some(0x3e) => {
                        expected.state = State::Data;
                        expected
                            .errors
                            .push(error("abrupt-closing-of-empty-comment", 2, 6));
                        let comment = expected.storage.comment.take().unwrap();
                        expected.tokens.push(Token::Comment { data: comment.data });
                        assert_eq!(run.step(), Ok(Progress::Continue(State::Data)));
                        assert_eq!(snapshot(&run), expected);
                        for (point, line, column) in [('z', 2, 7), ('\n', 2, 8), ('\0', 3, 1)] {
                            expected.current =
                                Some((Input::CodePoint(point.into()), Position { line, column }));
                            expected.tokens.push(Token::Character(point.into()));
                            if point == '\0' {
                                expected.errors.push(error(
                                    "unexpected-null-character",
                                    line,
                                    column,
                                ));
                            }
                            assert_eq!(run.step(), Ok(Progress::Continue(State::Data)));
                            assert_eq!(snapshot(&run), expected);
                        }
                        if !finished {
                            for _ in 0..3 {
                                assert_eq!(run.step(), Ok(Progress::NeedMoreInput));
                                assert_eq!(snapshot(&run), expected);
                            }
                            run.context_mut().input.finish();
                        }
                        expected.current = Some((Input::Eof, Position { line: 3, column: 2 }));
                        expected.tokens.push(Token::Eof);
                        assert_eq!(run.step(), Ok(Progress::Finished));
                        assert_eq!(snapshot(&run), expected);
                        terminal(&mut run, Ok(Progress::Finished));
                    }
                    Some(value) => {
                        expected.state = if value == 0x2d {
                            State::CommentEnd
                        } else {
                            State::Comment
                        };
                        if value != 0x2d {
                            expected
                                .storage
                                .comment
                                .as_mut()
                                .unwrap()
                                .data
                                .push('-'.into());
                        }
                        let preprocessing = match value {
                            0xd800 => Some("surrogate-in-input-stream"),
                            0xfdd0 => Some("noncharacter-in-input-stream"),
                            0x0b => Some("control-character-in-input-stream"),
                            _ => None,
                        };
                        if let Some(code) = preprocessing {
                            expected.input_errors.push(error(code, 2, 6));
                        }
                        assert_eq!(run.step(), Ok(Progress::Continue(expected.state)));
                        assert_eq!(snapshot(&run), expected);
                        if value != 0x2d {
                            assert_eq!(
                                run.context_mut().input.reconsume(),
                                Err(CursorError::AlreadyReconsuming)
                            );
                        }
                        if value == 0x2d {
                            // Newly implemented End consumes the next 'z',
                            // restores "--" (not 'z'), then replays in Comment.
                            expected.state = State::Comment;
                            expected.current = Some((
                                Input::CodePoint('z'.into()),
                                Position { line: 2, column: 7 },
                            ));
                            expected
                                .storage
                                .comment
                                .as_mut()
                                .unwrap()
                                .data
                                .extend(points("--"));
                            assert_eq!(run.step(), Ok(Progress::Continue(State::Comment)));
                            assert_eq!(snapshot(&run), expected);
                            comment_replay(&mut run, &mut expected);
                        } else {
                            comment_replay(&mut run, &mut expected);
                        }
                        // Cursor-only tail probe; never dispatch after this probe.
                        assert_eq!(run.context().input.current(), expected.current);
                        assert_eq!(run.context().input.errors(), expected.input_errors);
                        assert_eq!(
                            run.context_mut().input.consume(),
                            Input::CodePoint(if value == 0x2d {
                                '\n'.into()
                            } else {
                                'z'.into()
                            })
                        );
                        assert_eq!(
                            run.context().input.position(),
                            if value == 0x0d {
                                Position { line: 3, column: 1 }
                            } else if value == 0x2d {
                                Position { line: 2, column: 8 }
                            } else {
                                Position { line: 2, column: 7 }
                            }
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn open_exhaustion_suspends_but_explicit_eof_reconsumes_without_emission_or_error() {
    for seeded in [false, true] {
        for prefix in [false, true] {
            let context = if seeded {
                seeded_context()
            } else {
                Context {
                    pending_comment: Some(PendingComment::default()),
                    ..Default::default()
                }
            };
            let mut run = if prefix {
                entry(context)
            } else {
                Execution::new(State::CommentStart, context)
            };
            let mut expected = snapshot(&run);
            for _ in 0..3 {
                run.context_mut().input.push([]).unwrap();
                assert_eq!(run.step(), Ok(Progress::NeedMoreInput));
                assert_eq!(snapshot(&run), expected);
            }
            run.context_mut().input.finish();
            expected.state = State::Comment;
            expected.current = Some((
                Input::Eof,
                if prefix {
                    Position { line: 2, column: 5 }
                } else {
                    Position { line: 1, column: 1 }
                },
            ));
            assert_eq!(run.step(), Ok(Progress::Continue(State::Comment)));
            assert_eq!(snapshot(&run), expected);
            assert_eq!(
                run.context_mut().input.reconsume(),
                Err(CursorError::AlreadyReconsuming)
            );
            comment_replay(&mut run, &mut expected);
            terminal(&mut run, Ok(Progress::Finished));
        }
    }
}

#[test]
fn source_consumption_probe_before_any_destination_attempt() {
    let mut eof_run = entry(seeded_context());
    eof_run.context_mut().input.finish();
    assert_eq!(eof_run.step(), Ok(Progress::Continue(State::Comment)));
    let before = snapshot(&eof_run);
    assert_eq!(eof_run.context_mut().input.consume(), Input::Eof);
    assert_eq!(snapshot(&eof_run), before);
    for text in ["-z", ">z", "\0z", "<z", "&z", "\r\nz"] {
        let mut run = entry(seeded_context());
        run.context_mut().input.push(points(text)).unwrap();
        assert!(matches!(run.step(), Ok(Progress::Continue(_))));
        // Probe only the cursor after the source, never reconstruct/resume a stub.
        if !text.starts_with(['-', '>']) {
            let current = run.context().input.current().unwrap();
            let diagnostics = run.context().input.errors().to_vec();
            assert_eq!(run.context_mut().input.consume(), current.0);
            assert_eq!(run.context().input.current(), Some(current));
            assert_eq!(run.context().input.errors(), diagnostics);
        }
        assert_eq!(
            run.context_mut().input.consume(),
            Input::CodePoint('z'.into())
        );
        let line = if text.starts_with('\r') { 3 } else { 2 };
        let column = if line == 3 { 1 } else { 6 };
        assert_eq!(run.context().input.position(), Position { line, column });
    }
}

#[test]
fn abrupt_close_takes_empty_or_seeded_comment_then_data_handles_its_own_input() {
    for seeded in [false, true] {
        let context = if seeded {
            seeded_context()
        } else {
            Context {
                pending_comment: Some(PendingComment::default()),
                ..Default::default()
            }
        };
        let mut run = entry(context);
        run.context_mut()
            .input
            .push(points(">\r\n\0&tail"))
            .unwrap();
        let mut expected = snapshot(&run);
        expected.state = State::Data;
        expected.current = Some((
            Input::CodePoint('>'.into()),
            Position { line: 2, column: 5 },
        ));
        let comment = expected.storage.comment.take().unwrap();
        expected.tokens.push(Token::Comment { data: comment.data });
        expected
            .errors
            .push(error("abrupt-closing-of-empty-comment", 2, 5));
        assert_eq!(run.step(), Ok(Progress::Continue(State::Data)));
        assert_eq!(snapshot(&run), expected);
        expected.current = Some((
            Input::CodePoint('\n'.into()),
            Position { line: 2, column: 6 },
        ));
        expected.tokens.push(Token::Character('\n'.into()));
        assert_eq!(run.step(), Ok(Progress::Continue(State::Data)));
        assert_eq!(snapshot(&run), expected);
        expected.current = Some((
            Input::CodePoint('\0'.into()),
            Position { line: 3, column: 1 },
        ));
        expected.tokens.push(Token::Character('\0'.into()));
        expected
            .errors
            .push(error("unexpected-null-character", 3, 1));
        assert_eq!(run.step(), Ok(Progress::Continue(State::Data)));
        assert_eq!(snapshot(&run), expected);
        expected.current = Some((
            Input::CodePoint('&'.into()),
            Position { line: 3, column: 2 },
        ));
        expected.state = State::CharacterReference;
        expected.storage.return_state = Some(State::Data);
        assert_eq!(
            run.step(),
            Ok(Progress::Continue(State::CharacterReference))
        );
        assert_eq!(snapshot(&run), expected);
        let blocked = Err(UnimplementedState {
            state: State::CharacterReference,
        });
        assert_eq!(run.step(), blocked);
        assert_eq!(snapshot(&run), expected);
        terminal(&mut run, blocked);
    }
}

#[test]
fn normal_data_comment_entry_still_blocks_at_markup_declaration_open() {
    let mut context = seeded_context();
    context.input.push(points("<!-->tail")).unwrap();
    context.input.finish();
    let mut run = Execution::new(State::Data, context);
    let mut expected = snapshot(&run);
    for (point, column, state) in [
        ('<', 1, State::TagOpen),
        ('!', 2, State::MarkupDeclarationOpen),
    ] {
        expected.state = state;
        expected.current = Some((Input::CodePoint(point.into()), Position { line: 1, column }));
        assert_eq!(run.step(), Ok(Progress::Continue(state)));
        assert_eq!(snapshot(&run), expected);
    }
    let blocked = Err(UnimplementedState {
        state: State::MarkupDeclarationOpen,
    });
    assert_eq!(run.step(), blocked);
    assert_eq!(snapshot(&run), expected);
    terminal(&mut run, blocked);
}

type Trace = Vec<(Result<Progress, UnimplementedState>, Snapshot)>;

fn drain(run: &mut Execution, trace: &mut Trace) -> bool {
    for _ in 0..1000 {
        let before = snapshot(run);
        let result = run.step();
        if result == Ok(Progress::NeedMoreInput) {
            for _ in 0..3 {
                assert_eq!(snapshot(run), before);
                assert_eq!(run.step(), result);
            }
            return false;
        }
        trace.push((result, snapshot(run)));
        if !matches!(result, Ok(Progress::Continue(_))) {
            terminal(run, result);
            return true;
        }
    }
    panic!("source steps failed to progress");
}

fn trace(chunks: &[&[CodePoint]], closed: bool) -> Trace {
    let mut run = entry(seeded_context());
    let mut trace = vec![];
    for chunk in chunks {
        run.context_mut().input.push(chunk.iter().copied()).unwrap();
        if !closed && drain(&mut run, &mut trace) {
            return trace; // No feeding, reconstruction, or resumption after a stub.
        }
    }
    run.context_mut().input.finish();
    assert!(drain(&mut run, &mut trace));
    trace
}

#[test]
fn every_two_chunk_split_and_single_point_chunks_match_closed_source_steps() {
    let mut inputs: Vec<_> = [
        "",
        "-",
        "--tail",
        "-->",
        "---",
        "--!>",
        "--\0-->",
        "--!-->",
        "--><!",
        "-->&",
        "--><?",
        "->",
        "->x\r\n\0É😀\r",
        "->\r\r\n",
        "->\0&tail",
        "-><!tail",
        "-x",
        "-\0",
        "-\r\nX",
        "-\r\r\n",
        "-\r",
        "-É😀",
        "-\u{b}\u{fdd0}",
        ">",
        ">z",
        ">\r\n\0É😀\r",
        ">\r\r\n",
        ">\0&tail",
        "><!tail",
        ">\u{b}\u{fdd0}",
        "x",
        "\0",
        "\r\nz",
        "\r\r\n",
        "<",
        "&amp;",
        "É😀",
    ]
    .into_iter()
    .map(points)
    .collect();
    for value in [0xd800, 0xdfff, 0x10ffff] {
        let point = CodePoint::new(value).unwrap();
        inputs.push(vec![point, '\r'.into(), '\n'.into(), '\0'.into()]);
        inputs.push(vec![
            '-'.into(),
            point,
            '\r'.into(),
            '\n'.into(),
            '\0'.into(),
        ]);
        inputs.push(vec![
            '-'.into(),
            '>'.into(),
            point,
            '\r'.into(),
            '\n'.into(),
            '\0'.into(),
        ]);
        inputs.push(vec![
            '>'.into(),
            point,
            '\r'.into(),
            '\n'.into(),
            '\0'.into(),
        ]);
    }
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
            "single points: {input:?}"
        );
    }
}
