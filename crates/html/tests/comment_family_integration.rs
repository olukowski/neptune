//! Final #56 integration evidence, not upstream conformance or Data-gateway support.
//! Independent literal outcomes from pinned WHATWG HTML 404b3eb14a7a936dddefb2d78e224dbeb01d2358,
//! source 144514–144732. All ten ordinary-comment entries require initialized data.

use neptune_html::tokenizer::{
    Attribute, CodePoint, Context, Execution, Input, ParseError, PendingComment, PendingEndTag,
    PendingStartTag, Position, Progress, State, TagKind, Token, UnimplementedState,
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
fn literal_seed() -> Vec<CodePoint> {
    vec![
        '\0'.into(),
        '\r'.into(),
        CodePoint::new(0xd800).unwrap(),
        '<'.into(),
        '!'.into(),
    ]
}
fn context(initial: State, seeded: bool, selected: Option<TagKind>) -> Context {
    let prefix = match initial {
        State::CommentLessThanSign => "<",
        State::CommentLessThanSignBang
        | State::CommentLessThanSignBangDash
        | State::CommentLessThanSignBangDashDash => "<!",
        _ => "",
    };
    let mut data = if seeded { literal_seed() } else { vec![] };
    data.extend(points(prefix));
    let attribute = Attribute {
        name: points("k"),
        value: literal_seed(),
    };
    Context {
        tokens: vec![
            Token::Comment { data: vec![] },
            Token::Character('p'.into()),
            Token::Comment {
                data: points("prior"),
            },
        ],
        errors: vec![error("prior", 1, 1)],
        pending_comment: Some(PendingComment { data }),
        current_tag: selected,
        pending_start_tag: Some(PendingStartTag {
            name: points("start"),
            self_closing: true,
            attributes: vec![attribute.clone()],
            discarded_current_attribute: Some(attribute.clone()),
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("end"),
            self_closing: true,
            attributes: vec![attribute.clone()],
            discarded_current_attribute: Some(attribute),
        }),
        last_start_tag: Some(points("title")),
        return_state: Some(State::Rcdata),
        temporary_buffer: literal_seed(),
        ..Default::default()
    }
}
#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    state: State,
    current: Option<(Input, Position)>,
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
    input_errors: Vec<ParseError>,
    comment: Option<PendingComment>,
    selector: Option<TagKind>,
    start: Option<PendingStartTag>,
    end: Option<PendingEndTag>,
    last: Option<Vec<CodePoint>>,
    return_state: Option<State>,
    scratch: Vec<CodePoint>,
}
fn snapshot(run: &Execution) -> Snapshot {
    let c = run.context();
    Snapshot {
        state: run.state(),
        current: c.input.current(),
        tokens: c.tokens.clone(),
        errors: c.errors.clone(),
        input_errors: c.input.errors().to_vec(),
        comment: c.pending_comment.clone(),
        selector: c.current_tag,
        start: c.pending_start_tag.clone(),
        end: c.pending_end_tag.clone(),
        last: c.last_start_tag.clone(),
        return_state: c.return_state,
        scratch: c.temporary_buffer.clone(),
    }
}
type Trace = Vec<(Result<Progress, UnimplementedState>, Snapshot)>;
fn drain(run: &mut Execution, trace: &mut Trace, bound: usize) -> bool {
    for _ in 0..bound {
        let before = snapshot(run);
        let result = run.step();
        if result == Ok(Progress::NeedMoreInput) {
            assert_eq!(snapshot(run), before);
            for _ in 0..3 {
                run.context_mut().input.push([]).unwrap();
                assert_eq!(run.step(), result);
                assert_eq!(snapshot(run), before);
            }
            return false;
        }
        trace.push((result, snapshot(run)));
        if !matches!(result, Ok(Progress::Continue(_))) {
            let terminal = snapshot(run);
            for _ in 0..3 {
                assert_eq!(run.step(), result);
                assert_eq!(snapshot(run), terminal);
            }
            return true;
        }
    }
    panic!("comment family did not suspend/terminate within source-step bound");
}
struct Case {
    initial: State,
    input: Vec<CodePoint>,
    appended: Vec<CodePoint>,
    characters: Vec<CodePoint>,
    state: State,
    current: (Input, Position),
    errors: Vec<ParseError>,
    input_errors: Vec<ParseError>,
    blocked: bool,
}
fn outcome(run: &Execution, case: &Case, seeded: bool, selected: Option<TagKind>) {
    let mut expected = snapshot(&Execution::new(
        case.initial,
        context(case.initial, seeded, selected),
    ));
    expected.state = case.state;
    expected.current = Some(case.current);
    expected.errors.extend(case.errors.clone());
    expected.input_errors = case.input_errors.clone();
    let mut data = expected.comment.take().unwrap().data;
    data.extend(case.appended.clone());
    expected.tokens.push(Token::Comment { data });
    expected
        .tokens
        .extend(case.characters.iter().copied().map(Token::Character));
    if case.blocked {
        if case.state == State::CharacterReference {
            expected.return_state = Some(State::Data);
        }
        if case.state == State::ProcessingInstructionOpen {
            expected.scratch.clear();
        }
    } else {
        expected.tokens.push(Token::Eof);
    }
    assert_eq!(
        snapshot(run),
        expected,
        "{:?}: {:?}",
        case.initial,
        case.input
    );
}
fn run(
    case: &Case,
    seeded: bool,
    selected: Option<TagKind>,
    chunks: Option<Vec<&[CodePoint]>>,
) -> Trace {
    let mut run = Execution::new(case.initial, context(case.initial, seeded, selected));
    let mut trace = vec![];
    let bound = case.input.len() * 8 + 24;
    if let Some(chunks) = chunks {
        for chunk in chunks {
            run.context_mut().input.push(chunk.iter().copied()).unwrap();
            if drain(&mut run, &mut trace, bound) {
                assert!(case.blocked);
                outcome(&run, case, seeded, selected);
                assert_eq!(
                    trace.last().unwrap().0,
                    Err(UnimplementedState { state: case.state })
                );
                return trace; // Never feed, reconstruct, probe or resume a blocked run.
            }
        }
    } else {
        run.context_mut()
            .input
            .push(case.input.iter().copied())
            .unwrap();
    }
    run.context_mut().input.finish();
    assert!(drain(&mut run, &mut trace, bound));
    outcome(&run, case, seeded, selected);
    assert_eq!(
        trace.last().unwrap().0,
        if case.blocked {
            Err(UnimplementedState { state: case.state })
        } else {
            Ok(Progress::Finished)
        }
    );
    trace
}
fn cases() -> Vec<Case> {
    use State::*;
    let mut cases = vec![];
    // Hand-derived appended data, final EOF state/column, and prior state errors.
    // Entry prefixes '<' / '<!' are initialized context, not appended twice.
    for (initial, input, appended, characters, state, column, errors) in [
        (
            CommentStart,
            "",
            "",
            "",
            Comment,
            1,
            vec![error("eof-in-comment", 1, 1)],
        ),
        (CommentStart, "-->", "", "", Data, 4, vec![]),
        (CommentStart, "a>&!\"'-->", "a>&!\"'", "", Data, 10, vec![]),
        (
            CommentStart,
            ">tail",
            "",
            "tail",
            Data,
            6,
            vec![error("abrupt-closing-of-empty-comment", 1, 1)],
        ),
        (CommentStart, "-x-->", "-x", "", Data, 6, vec![]),
        (CommentStart, "<!-->", "<!", "", Data, 6, vec![]),
        (
            CommentStartDash,
            "",
            "",
            "",
            CommentStartDash,
            1,
            vec![error("eof-in-comment", 1, 1)],
        ),
        (CommentStartDash, "->", "", "", Data, 3, vec![]),
        (
            CommentStartDash,
            ">",
            "",
            "",
            Data,
            2,
            vec![error("abrupt-closing-of-empty-comment", 1, 1)],
        ),
        (CommentStartDash, "x-->", "-x", "", Data, 5, vec![]),
        (
            Comment,
            "",
            "",
            "",
            Comment,
            1,
            vec![error("eof-in-comment", 1, 1)],
        ),
        (
            Comment,
            "--!>",
            "",
            "",
            Data,
            5,
            vec![error("incorrectly-closed-comment", 1, 4)],
        ),
        (
            Comment,
            "<!--x-->",
            "<!--x",
            "",
            Data,
            9,
            vec![error("nested-comment", 1, 5)],
        ),
        (
            Comment,
            "<!--",
            "<!",
            "",
            CommentEnd,
            5,
            vec![error("eof-in-comment", 1, 5)],
        ),
        (Comment, "<!-->", "<!", "", Data, 6, vec![]),
        (
            Comment,
            "<<!---->",
            "<<!--",
            "",
            Data,
            9,
            vec![error("nested-comment", 1, 6)],
        ),
        (
            CommentStart,
            "<!--a<!--b-->",
            "<!--a<!--b",
            "",
            Data,
            14,
            vec![
                error("nested-comment", 1, 5),
                error("nested-comment", 1, 10),
            ],
        ),
        (
            CommentStart,
            "--!--!>",
            "--!",
            "",
            Data,
            8,
            vec![error("incorrectly-closed-comment", 1, 7)],
        ),
        (
            CommentLessThanSign,
            "!---->",
            "!--",
            "",
            Data,
            7,
            vec![error("nested-comment", 1, 4)],
        ),
        (CommentLessThanSign, "<x-->", "<x", "", Data, 6, vec![]),
        (CommentLessThanSignBang, "-x-->", "-x", "", Data, 6, vec![]),
        (
            CommentLessThanSignBang,
            "--x-->",
            "--x",
            "",
            Data,
            7,
            vec![error("nested-comment", 1, 3)],
        ),
        (
            CommentLessThanSignBang,
            "",
            "",
            "",
            Comment,
            1,
            vec![error("eof-in-comment", 1, 1)],
        ),
        (
            CommentLessThanSignBangDash,
            "-",
            "",
            "",
            CommentEnd,
            2,
            vec![error("eof-in-comment", 1, 2)],
        ),
        (
            CommentLessThanSignBangDash,
            "-x-->",
            "--x",
            "",
            Data,
            6,
            vec![error("nested-comment", 1, 2)],
        ),
        (
            CommentLessThanSignBangDash,
            ">-->",
            "->",
            "",
            Data,
            5,
            vec![],
        ),
        (
            CommentLessThanSignBangDash,
            "",
            "",
            "",
            CommentEndDash,
            1,
            vec![error("eof-in-comment", 1, 1)],
        ),
        (
            CommentLessThanSignBangDashDash,
            ">",
            "",
            "",
            Data,
            2,
            vec![],
        ),
        (
            CommentLessThanSignBangDashDash,
            "",
            "",
            "",
            CommentEnd,
            1,
            vec![error("eof-in-comment", 1, 1)],
        ),
        (
            CommentLessThanSignBangDashDash,
            "x-->",
            "--x",
            "",
            Data,
            5,
            vec![error("nested-comment", 1, 1)],
        ),
        (
            CommentLessThanSignBangDashDash,
            "-!>",
            "-",
            "",
            Data,
            4,
            vec![
                error("nested-comment", 1, 1),
                error("incorrectly-closed-comment", 1, 3),
            ],
        ),
        (
            CommentLessThanSignBangDashDash,
            "!x-->",
            "--!x",
            "",
            Data,
            6,
            vec![error("nested-comment", 1, 1)],
        ),
        (
            CommentLessThanSignBangDashDash,
            "!",
            "",
            "",
            CommentEndBang,
            2,
            vec![error("nested-comment", 1, 1), error("eof-in-comment", 1, 2)],
        ),
        (
            CommentLessThanSignBangDashDash,
            "<!-->",
            "--<!",
            "",
            Data,
            6,
            vec![error("nested-comment", 1, 1)],
        ),
        (
            CommentEndDash,
            "",
            "",
            "",
            CommentEndDash,
            1,
            vec![error("eof-in-comment", 1, 1)],
        ),
        (CommentEndDash, "->", "", "", Data, 3, vec![]),
        (CommentEndDash, "x-->", "-x", "", Data, 5, vec![]),
        (CommentEndDash, "-->", "-", "", Data, 4, vec![]),
        (
            CommentEnd,
            "",
            "",
            "",
            CommentEnd,
            1,
            vec![error("eof-in-comment", 1, 1)],
        ),
        (CommentEnd, ">tail", "", "tail", Data, 6, vec![]),
        (CommentEnd, "!x-->", "--!x", "", Data, 6, vec![]),
        (CommentEnd, "-->", "--", "", Data, 4, vec![]),
        (
            CommentEndBang,
            "",
            "",
            "",
            CommentEndBang,
            1,
            vec![error("eof-in-comment", 1, 1)],
        ),
        (
            CommentEndBang,
            ">tail",
            "",
            "tail",
            Data,
            6,
            vec![error("incorrectly-closed-comment", 1, 1)],
        ),
        (CommentEndBang, "-->", "--!", "", Data, 4, vec![]),
        (CommentEndBang, "-x-->", "--!-x", "", Data, 6, vec![]),
        (
            CommentEndBang,
            "!",
            "--!!",
            "",
            Comment,
            2,
            vec![error("eof-in-comment", 1, 2)],
        ),
    ] {
        cases.push(Case {
            initial,
            input: points(input),
            appended: points(appended),
            characters: points(characters),
            state,
            current: (Input::Eof, Position { line: 1, column }),
            errors,
            input_errors: vec![],
            blocked: false,
        });
    }
    // Actual code points, not surrogate strings: source nested-comment and cursor
    // diagnostics coexist; Comment alone replaces NUL; Data later emits original NUL.
    let surrogate = CodePoint::new(0xdfff).unwrap();
    let noncharacter = CodePoint::new(0xfdd0).unwrap();
    let mut input = points("\r\n<!--");
    input.extend([surrogate, '\0'.into(), noncharacter]);
    input.extend(points("\r\n--!>\0"));
    let mut appended = points("\n<!--");
    appended.extend([surrogate, '\u{fffd}'.into(), noncharacter, '\n'.into()]);
    cases.push(Case {
        initial: CommentStart,
        input,
        appended,
        characters: points("\0"),
        state: Data,
        current: (Input::Eof, Position { line: 3, column: 6 }),
        errors: vec![
            error("nested-comment", 2, 5),
            error("unexpected-null-character", 2, 6),
            error("incorrectly-closed-comment", 3, 4),
            error("unexpected-null-character", 3, 5),
        ],
        input_errors: vec![
            error("surrogate-in-input-stream", 2, 5),
            error("noncharacter-in-input-stream", 2, 7),
        ],
        blocked: false,
    });
    for (text, appended, state, line, column, errors) in [
        (
            "a\r\n\r\r",
            "a\n\n\n",
            Comment,
            4,
            1,
            vec![error("eof-in-comment", 4, 1)],
        ),
        (
            "\0<!--\0",
            "�<!--�",
            Comment,
            1,
            7,
            vec![
                error("unexpected-null-character", 1, 1),
                error("nested-comment", 1, 6),
                error("unexpected-null-character", 1, 6),
                error("eof-in-comment", 1, 7),
            ],
        ),
    ] {
        cases.push(Case {
            initial: CommentStart,
            input: points(text),
            appended: points(appended),
            characters: vec![],
            state,
            current: (Input::Eof, Position { line, column }),
            errors,
            input_errors: vec![],
            blocked: false,
        });
    }
    cases.push(Case {
        initial: CommentStart,
        input: points("\u{b}<!--\u{b}-->"),
        appended: points("\u{b}<!--\u{b}"),
        characters: vec![],
        state: Data,
        current: (
            Input::Eof,
            Position {
                line: 1,
                column: 10,
            },
        ),
        errors: vec![error("nested-comment", 1, 6)],
        input_errors: vec![
            error("control-character-in-input-stream", 1, 1),
            error("control-character-in-input-stream", 1, 6),
        ],
        blocked: false,
    });
    for (input, state, point, column) in [
        ("<!-->p&", CharacterReference, '&', 7),
        ("<!-->p<!", MarkupDeclarationOpen, '!', 8),
        ("<!-->p<?", ProcessingInstructionOpen, '?', 8),
    ] {
        cases.push(Case {
            initial: CommentStart,
            input: points(input),
            appended: points("<!"),
            characters: points("p"),
            state,
            current: (Input::CodePoint(point.into()), Position { line: 1, column }),
            errors: vec![],
            input_errors: vec![],
            blocked: true,
        });
    }
    cases
}

#[test]
fn all_ten_entries_and_full_comment_start_match_literal_outcomes_at_every_split() {
    let cases = cases();
    let mut entries = vec![];
    for case in &cases {
        if !entries.contains(&case.initial) {
            entries.push(case.initial);
        }
        for seeded in [false, true] {
            for selected in [None, Some(TagKind::Start), Some(TagKind::End)] {
                let expected = run(case, seeded, selected, None);
                for split in 0..=case.input.len() {
                    assert_eq!(
                        run(
                            case,
                            seeded,
                            selected,
                            Some(vec![&case.input[..split], &[], &case.input[split..]])
                        ),
                        expected,
                        "{:?} split {split}: {:?}",
                        case.initial,
                        case.input
                    );
                }
                assert_eq!(
                    run(case, seeded, selected, Some(case.input.chunks(1).collect())),
                    expected
                );
            }
        }
    }
    assert_eq!(entries.len(), 10);
}

#[test]
fn explicit_start_nested_opener_has_independent_source_snapshots_and_once_only_preprocessing() {
    use State::*;
    let surrogate = CodePoint::new(0xdfff).unwrap();
    let noncharacter = CodePoint::new(0xfdd0).unwrap();
    let mut input = points("\r\n<!--");
    input.extend([surrogate, '\0'.into(), noncharacter]);
    input.extend(points("\r\n--!>\0"));
    let mut run = Execution::new(
        CommentStart,
        context(CommentStart, true, Some(TagKind::End)),
    );
    run.context_mut().input.push(input).unwrap();
    run.context_mut().input.finish();
    let mut expected = snapshot(&run);
    for (index, (state, item, line, column, appended, code, preprocessing, emit)) in [
        (
            Comment,
            Input::CodePoint('\n'.into()),
            1,
            1,
            vec![],
            None,
            None,
            "",
        ),
        (
            Comment,
            Input::CodePoint('\n'.into()),
            1,
            1,
            points("\n"),
            None,
            None,
            "",
        ),
        (
            CommentLessThanSign,
            Input::CodePoint('<'.into()),
            2,
            1,
            points("<"),
            None,
            None,
            "",
        ),
        (
            CommentLessThanSignBang,
            Input::CodePoint('!'.into()),
            2,
            2,
            points("!"),
            None,
            None,
            "",
        ),
        (
            CommentLessThanSignBangDash,
            Input::CodePoint('-'.into()),
            2,
            3,
            vec![],
            None,
            None,
            "",
        ),
        (
            CommentLessThanSignBangDashDash,
            Input::CodePoint('-'.into()),
            2,
            4,
            vec![],
            None,
            None,
            "",
        ),
        (
            CommentEnd,
            Input::CodePoint(surrogate),
            2,
            5,
            vec![],
            Some("nested-comment"),
            Some("surrogate-in-input-stream"),
            "",
        ),
        (
            Comment,
            Input::CodePoint(surrogate),
            2,
            5,
            points("--"),
            None,
            None,
            "",
        ),
        (
            Comment,
            Input::CodePoint(surrogate),
            2,
            5,
            vec![surrogate],
            None,
            None,
            "",
        ),
        (
            Comment,
            Input::CodePoint('\0'.into()),
            2,
            6,
            points("�"),
            Some("unexpected-null-character"),
            None,
            "",
        ),
        (
            Comment,
            Input::CodePoint(noncharacter),
            2,
            7,
            vec![noncharacter],
            None,
            Some("noncharacter-in-input-stream"),
            "",
        ),
        (
            Comment,
            Input::CodePoint('\n'.into()),
            2,
            8,
            points("\n"),
            None,
            None,
            "",
        ),
        (
            CommentEndDash,
            Input::CodePoint('-'.into()),
            3,
            1,
            vec![],
            None,
            None,
            "",
        ),
        (
            CommentEnd,
            Input::CodePoint('-'.into()),
            3,
            2,
            vec![],
            None,
            None,
            "",
        ),
        (
            CommentEndBang,
            Input::CodePoint('!'.into()),
            3,
            3,
            vec![],
            None,
            None,
            "",
        ),
        (
            Data,
            Input::CodePoint('>'.into()),
            3,
            4,
            vec![],
            Some("incorrectly-closed-comment"),
            None,
            "comment",
        ),
        (
            Data,
            Input::CodePoint('\0'.into()),
            3,
            5,
            vec![],
            Some("unexpected-null-character"),
            None,
            "nul",
        ),
        (Data, Input::Eof, 3, 6, vec![], None, None, "eof"),
    ]
    .into_iter()
    .enumerate()
    {
        expected.state = state;
        expected.current = Some((item, Position { line, column }));
        if !appended.is_empty() {
            expected.comment.as_mut().unwrap().data.extend(appended);
        }
        if let Some(code) = code {
            expected.errors.push(error(code, line, column));
        }
        if let Some(code) = preprocessing {
            expected.input_errors.push(error(code, line, column));
        }
        match emit {
            "comment" => {
                let data = expected.comment.take().unwrap().data;
                expected.tokens.push(Token::Comment { data });
            }
            "nul" => expected.tokens.push(Token::Character('\0'.into())),
            "eof" => expected.tokens.push(Token::Eof),
            "" => {}
            _ => unreachable!(),
        }
        assert_eq!(
            run.step(),
            Ok(if item == Input::Eof {
                Progress::Finished
            } else {
                Progress::Continue(state)
            })
        );
        assert_eq!(snapshot(&run), expected, "source step {index}");
        if matches!(index, 0 | 6 | 7) {
            assert_eq!(
                run.context_mut().input.reconsume(),
                Err(neptune_html::tokenizer::CursorError::AlreadyReconsuming)
            );
        }
    }
    for _ in 0..3 {
        assert_eq!(run.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(&run), expected);
    }
}

#[test]
fn full_start_reaches_and_repeatedly_suspends_in_every_family_state_without_taking_comment() {
    use State::*;
    for (input, appended, suspended, final_state, column) in [
        ("", "", CommentStart, Comment, 1),
        ("-", "", CommentStartDash, CommentStartDash, 2),
        ("x", "x", Comment, Comment, 2),
        ("<", "<", CommentLessThanSign, Comment, 2),
        ("<!", "<!", CommentLessThanSignBang, Comment, 3),
        ("<!-", "<!", CommentLessThanSignBangDash, CommentEndDash, 4),
        ("<!--", "<!", CommentLessThanSignBangDashDash, CommentEnd, 5),
        ("x-", "x", CommentEndDash, CommentEndDash, 3),
        ("--", "", CommentEnd, CommentEnd, 3),
        ("--!", "", CommentEndBang, CommentEndBang, 4),
    ] {
        for seeded in [false, true] {
            let mut run = Execution::new(
                CommentStart,
                context(CommentStart, seeded, Some(TagKind::End)),
            );
            let mut expected = snapshot(&run);
            run.context_mut().input.push(points(input)).unwrap();
            let mut trace = vec![];
            assert!(!drain(&mut run, &mut trace, 64));
            expected.state = suspended;
            expected.current = input.chars().last().map(|c| {
                (
                    Input::CodePoint(c.into()),
                    Position {
                        line: 1,
                        column: column - 1,
                    },
                )
            });
            expected
                .comment
                .as_mut()
                .unwrap()
                .data
                .extend(points(appended));
            assert_eq!(snapshot(&run), expected);
            run.context_mut().input.finish();
            assert!(drain(&mut run, &mut trace, 64));
            expected.state = final_state;
            expected.current = Some((Input::Eof, Position { line: 1, column }));
            expected.errors.push(error("eof-in-comment", 1, column));
            let data = expected.comment.take().unwrap().data;
            expected
                .tokens
                .extend([Token::Comment { data }, Token::Eof]);
            assert_eq!(snapshot(&run), expected);
        }
    }
}
