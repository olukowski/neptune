//! Complete explicit ScriptData entry after PR #45 gateway integration.
//! Pinned HTML 404b3eb14a7a936dddefb2d78e224dbeb01d2358, source
//! 143338–143359, 143654–144142; branch/fixture audit in the gateway state doc.
//! These local checks are not additional upstream conformance runs.

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

fn characters(text: &str) -> Vec<Token> {
    points(text).into_iter().map(Token::Character).collect()
}

fn end_tag(attributes: Vec<Attribute>, self_closing: bool) -> Token {
    Token::EndTag {
        name: points("script"),
        attributes,
        self_closing,
    }
}

fn attribute(name: &str, value: &str) -> Attribute {
    Attribute {
        name: points(name),
        value: points(value),
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
            assert_eq!(run.step(), result);
            assert_eq!(snapshot(run), trace.last().unwrap().1);
            return true;
        }
    }
    panic!("integrated source-step bound");
}

fn closed(input: &[CodePoint], last: Option<&str>) -> Execution {
    let mut context = seeded_context();
    context.last_start_tag = last.map(points);
    context.input.push(input.iter().copied()).unwrap();
    context.input.finish();
    Execution::new(State::ScriptData, context)
}

#[test]
fn escaped_close_preserves_each_creation_name_spelling_duplicate_identity_and_emission() {
    let input = "<!--</ScRiPt A='first' a='discard' B=last />\0";
    let mut run = closed(&points(input), Some("script"));
    let mut expected = snapshot(&run);
    for (state, point, column, output) in [
        (State::ScriptDataLessThanSign, '<', 1, ""),
        (State::ScriptDataEscapeStart, '!', 2, "<!"),
        (State::ScriptDataEscapeStartDash, '-', 3, "-"),
        (State::ScriptDataEscapedDashDash, '-', 4, "-"),
        (State::ScriptDataEscapedLessThanSign, '<', 5, ""),
        (State::ScriptDataEscapedEndTagOpen, '/', 6, ""),
        (State::ScriptDataEscapedEndTagName, 'S', 7, ""),
    ] {
        expected.state = state;
        expected.current = Some((Input::CodePoint(point.into()), Position { line: 1, column }));
        expected.tokens.extend(characters(output));
        if column == 6 {
            expected.temporary_buffer.clear();
        }
        if column == 7 {
            expected.current_tag = Some(TagKind::End);
            expected.end = Some(PendingEndTag::default());
        }
        assert_eq!(run.step(), Ok(Progress::Continue(state)));
        assert_eq!(snapshot(&run), expected);
    }
    for (index, original) in "ScRiPt".chars().enumerate() {
        expected.current = Some((
            Input::CodePoint(original.into()),
            Position {
                line: 1,
                column: index + 7,
            },
        ));
        expected
            .end
            .as_mut()
            .unwrap()
            .name
            .push(original.to_ascii_lowercase().into());
        expected.temporary_buffer.push(original.into());
        assert_eq!(run.step(), Ok(Progress::Continue(expected.state)));
        assert_eq!(snapshot(&run), expected);
    }
    expected.state = State::BeforeAttributeName;
    expected.current = Some((
        Input::CodePoint(' '.into()),
        Position {
            line: 1,
            column: 13,
        },
    ));
    assert_eq!(run.step(), Ok(Progress::Continue(expected.state)));
    assert_eq!(snapshot(&run), expected);
    let mut trace = vec![];
    assert!(drain(&mut run, &mut trace));
    assert_eq!(trace.last().unwrap().0, Ok(Progress::Finished));
    let duplicate_value = trace
        .iter()
        .find(|(_, s)| {
            s.state == State::AfterAttributeValueQuoted
                && s.current
                    == Some((
                        Input::CodePoint('\''.into()),
                        Position {
                            line: 1,
                            column: 34,
                        },
                    ))
        })
        .unwrap();
    let tag = duplicate_value.1.end.as_ref().unwrap();
    assert_eq!(tag.attributes, [attribute("a", "first")]);
    assert_eq!(
        tag.discarded_current_attribute,
        Some(attribute("a", "discard"))
    );
    assert_eq!(duplicate_value.1.current_tag, Some(TagKind::End));
    assert!(trace.iter().any(|(_, s)| {
        s.state == State::AttributeName
            && s.current
                == Some((
                    Input::CodePoint('B'.into()),
                    Position {
                        line: 1,
                        column: 36,
                    },
                ))
            && s.end.as_ref().unwrap().attributes == [attribute("a", "first"), Attribute::default()]
            && s.end
                .as_ref()
                .unwrap()
                .discarded_current_attribute
                .is_none()
    }));
    expected.state = State::Data;
    expected.current = Some((
        Input::Eof,
        Position {
            line: 1,
            column: 46,
        },
    ));
    expected.current_tag = None;
    expected.end = None;
    expected.tokens.push(end_tag(
        vec![attribute("a", "first"), attribute("b", "last")],
        true,
    ));
    expected
        .tokens
        .extend([Token::Character('\0'.into()), Token::Eof]);
    expected.errors.extend([
        error("duplicate-attribute", 1, 25),
        error("end-tag-with-attributes", 1, 44),
        error("end-tag-with-trailing-solidus", 1, 44),
        error("unexpected-null-character", 1, 45),
    ]);
    assert_eq!(snapshot(&run), expected);
}

#[test]
fn escaped_close_requires_exact_emitted_history_and_mismatches_remain_literal() {
    for last in [None, Some("SCRIPT"), Some("script")] {
        let input = "<!--</ScRiPt>X";
        let mut run = closed(&points(input), last);
        let mut expected = snapshot(&run);
        let mut trace = vec![];
        assert!(drain(&mut run, &mut trace));
        expected.current = Some((
            Input::Eof,
            Position {
                line: 1,
                column: 15,
            },
        ));
        expected.temporary_buffer = points("ScRiPt");
        if last == Some("script") {
            expected.state = State::Data;
            expected.current_tag = None;
            expected.end = None;
            expected.tokens.extend(characters("<!--"));
            expected.tokens.push(end_tag(vec![], false));
            expected.tokens.extend(characters("X"));
        } else {
            expected.state = State::ScriptDataEscaped;
            expected.current_tag = Some(TagKind::End);
            expected.end = Some(PendingEndTag {
                name: points("script"),
                ..Default::default()
            });
            expected.tokens.extend(characters(input));
            expected
                .errors
                .push(error("eof-in-script-html-comment-like-text", 1, 15));
        }
        expected.tokens.push(Token::Eof);
        assert_eq!(trace.last().unwrap().0, Ok(Progress::Finished));
        assert_eq!(snapshot(&run), expected, "{last:?}");
    }
}

#[test]
fn double_escape_nesting_case_mismatches_and_both_return_modes_have_exact_output() {
    for (input, literal_prefix, literal_suffix) in [
        (
            "<!--<ScRiPt><script></ScRiPt></sCrIpT>X",
            "<!--<ScRiPt><script></ScRiPt>",
            "X",
        ),
        ("<!--<scrip></ScRiPt>X", "<!--<scrip>", "X"),
        ("<!--<script</ScRiPt>X", "<!--<script", "X"),
        (
            "<!--<script></scrip></script></ScRiPt>",
            "<!--<script></scrip></script>",
            "",
        ),
        ("<!--<script>--></ScRiPt>", "<!--<script>-->", ""),
    ] {
        let mut run = closed(&points(input), Some("script"));
        let mut trace = vec![];
        assert!(drain(&mut run, &mut trace));
        let mut expected = characters("p");
        expected.extend(characters(literal_prefix));
        expected.push(end_tag(vec![], false));
        expected.extend(characters(literal_suffix));
        expected.push(Token::Eof);
        assert_eq!(run.context().tokens, expected, "{input}");
        assert_eq!(run.context().errors, [error("prior-diagnostic", 1, 1)]);
        assert!(run.context().input.errors().is_empty());
        assert_eq!(run.state(), State::Data);
        assert_eq!(
            run.context().temporary_buffer,
            points(if input.contains("</sCrIpT>X") {
                "sCrIpT"
            } else {
                "ScRiPt"
            })
        );
        assert!(run.context().pending_end_tag.is_none());
        assert!(run.context().current_tag.is_none());
    }
    for delimiter in ["\t", "\n", "\u{c}", " ", "/", ">", "\r\n"] {
        let input = format!("<!--<ScRiPt{delimiter}x</sCrIpT{delimiter}</ScRiPt>");
        let normalized = if delimiter == "\r\n" { "\n" } else { delimiter };
        let literal = format!("p<!--<ScRiPt{normalized}x</sCrIpT{normalized}");
        let mut run = closed(&points(&input), Some("script"));
        let mut trace = vec![];
        assert!(drain(&mut run, &mut trace));
        let mut expected = characters(&literal);
        expected.extend([end_tag(vec![], false), Token::Eof]);
        assert_eq!(run.context().tokens, expected, "{delimiter:?}");
        assert_eq!(run.context().errors, [error("prior-diagnostic", 1, 1)]);
        assert!(run.context().input.errors().is_empty());
        assert!(
            trace.iter().any(
                |(result, _)| *result == Ok(Progress::Continue(State::ScriptDataDoubleEscaped))
            )
        );
        assert!(
            trace
                .iter()
                .any(|(result, _)| *result == Ok(Progress::Continue(State::ScriptDataEscaped)))
        );
        assert_eq!(run.state(), State::Data);
    }
}

#[test]
fn eof_ownership_remains_with_real_escaped_or_double_escaped_destination() {
    for (suffix, final_state) in [
        ("<", State::ScriptDataEscaped),
        ("</", State::ScriptDataEscaped),
        ("</ScRiPt", State::ScriptDataEscaped),
        ("<ScRiPt", State::ScriptDataEscaped),
        ("<ScRiPt>", State::ScriptDataDoubleEscaped),
        ("<ScRiPt>-", State::ScriptDataDoubleEscapedDash),
        ("<ScRiPt>--", State::ScriptDataDoubleEscapedDashDash),
        ("<ScRiPt><", State::ScriptDataDoubleEscaped),
        ("<ScRiPt></", State::ScriptDataDoubleEscaped),
        ("<ScRiPt></ScRiPt", State::ScriptDataDoubleEscaped),
        ("<ScRiPt></ScRiPt>", State::ScriptDataEscaped),
        ("<scrip>", State::ScriptDataEscaped),
        ("</>", State::ScriptDataEscaped),
    ] {
        let input = format!("<!--{suffix}");
        let mut run = closed(&points(&input), Some("script"));
        let mut trace = vec![];
        assert!(drain(&mut run, &mut trace));
        assert_eq!(trace.last().unwrap().0, Ok(Progress::Finished));
        let mut expected = characters(&format!("p{input}"));
        expected.push(Token::Eof);
        assert_eq!(run.context().tokens, expected);
        assert_eq!(
            run.context().errors,
            [
                error("prior-diagnostic", 1, 1),
                error("eof-in-script-html-comment-like-text", 1, input.len() + 1)
            ]
        );
        assert!(run.context().input.errors().is_empty());
        assert_eq!(
            run.context().input.current(),
            Some((
                Input::Eof,
                Position {
                    line: 1,
                    column: input.len() + 1
                }
            ))
        );
        assert_eq!(run.state(), final_state, "{input}");
    }
}

fn malformed_input() -> Vec<CodePoint> {
    let mut input = points("<!--<ScRiPt>\r\n\0");
    input.extend([0xd800, 0xfdd0, 0x0b].map(|v| CodePoint::new(v).unwrap()));
    input.extend(points("</sCrIpT>\0</ScRiPt>\0"));
    input
}

#[test]
fn nul_and_non_scalars_cross_double_escaped_escaped_and_data_with_exact_diagnostics() {
    let mut run = closed(&malformed_input(), Some("script"));
    let mut expected = snapshot(&run);
    let mut trace = vec![];
    assert!(drain(&mut run, &mut trace));
    expected.state = State::Data;
    expected.current = Some((
        Input::Eof,
        Position {
            line: 2,
            column: 25,
        },
    ));
    expected.current_tag = None;
    expected.end = None;
    expected.temporary_buffer = points("ScRiPt");
    expected.tokens.extend(characters("<!--<ScRiPt>\n\u{fffd}"));
    expected
        .tokens
        .extend([0xd800, 0xfdd0, 0x0b].map(|v| Token::Character(CodePoint::new(v).unwrap())));
    expected.tokens.extend(characters("</sCrIpT>\u{fffd}"));
    expected.tokens.extend([
        end_tag(vec![], false),
        Token::Character('\0'.into()),
        Token::Eof,
    ]);
    expected.errors.extend([
        error("unexpected-null-character", 2, 1),
        error("unexpected-null-character", 2, 14),
        error("unexpected-null-character", 2, 24),
    ]);
    expected.input_errors.extend([
        error("surrogate-in-input-stream", 2, 2),
        error("noncharacter-in-input-stream", 2, 3),
        error("control-character-in-input-stream", 2, 4),
    ]);
    assert_eq!(snapshot(&run), expected);
}

#[test]
fn genuine_unrelated_blocks_after_script_close_remain_terminal_not_prefix_success() {
    for (input, target, prefix) in [
        (
            "<!--<script></script></script>&tail",
            State::CharacterReference,
            "<!--<script></script>",
        ),
        ("<!--</script><!tail", State::MarkupDeclarationOpen, "<!--"),
        ("<!--</script a='&tail", State::CharacterReference, "<!--"),
    ] {
        let mut run = closed(&points(input), Some("script"));
        let mut trace = vec![];
        assert!(drain(&mut run, &mut trace));
        let blocked = Err(UnimplementedState { state: target });
        assert_eq!(trace.last().unwrap().0, blocked);
        let before = snapshot(&run);
        let mut tokens = characters(&format!("p{prefix}"));
        if target != State::CharacterReference || !input.contains(" a=") {
            tokens.push(end_tag(vec![], false));
        }
        assert_eq!(run.context().tokens, tokens);
        assert_eq!(run.context().errors, [error("prior-diagnostic", 1, 1)]);
        for _ in 0..3 {
            assert_eq!(run.step(), blocked);
            assert_eq!(snapshot(&run), before);
        }
        assert!(!run.context().tokens.contains(&Token::Eof));
    }
}

#[test]
fn all_integrated_two_chunk_splits_and_single_point_streams_match_closed_source_steps() {
    let mut inputs: Vec<_> = [
        "<!--</ScRiPt A='first' a='discard' B=last />\0",
        "<!--</ScRiPt",
        "<!--</>",
        "<!--<ScRiPt",
        "<!--<ScRiPt><script></ScRiPt></sCrIpT>X",
        "<!--<script></scrip></script></ScRiPt>",
        "<!--<script>--></ScRiPt>",
        "<!--<script></script-->",
        "<!--<script><",
        "<!--<scrip></script>-->",
        "<!--<script</script>-->",
        "<!--<ScRiPt\r\nx</ScRiPt\r\n</ScRiPt>",
        "<!--</ScRiPt\r\n a='\0' a='discard'/>\0",
        "<!--<\r\n\0--><",
    ]
    .map(points)
    .into_iter()
    .collect();
    inputs.push(malformed_input());
    for input in inputs {
        let mut reference = closed(&input, Some("script"));
        let mut expected = vec![];
        assert!(drain(&mut reference, &mut expected));
        assert_eq!(expected.last().unwrap().0, Ok(Progress::Finished));
        let mut chunkings: Vec<_> = (0..=input.len())
            .map(|split| vec![split, input.len() - split])
            .collect();
        chunkings.push(vec![1; input.len()]);
        for chunks in chunkings {
            let mut run = Execution::new(State::ScriptData, seeded_context());
            let mut actual = vec![];
            let mut offset = 0;
            assert!(!drain(&mut run, &mut actual));
            for length in chunks {
                run.context_mut().input.push([]).unwrap();
                run.context_mut()
                    .input
                    .push(input[offset..offset + length].iter().copied())
                    .unwrap();
                offset += length;
                assert!(
                    !drain(&mut run, &mut actual),
                    "open input must not terminate"
                );
            }
            assert_eq!(offset, input.len());
            run.context_mut().input.finish();
            assert!(drain(&mut run, &mut actual));
            assert_eq!(actual, expected, "{input:?}");
        }
    }
}
