//! Branch evidence for pinned HTML "Tag name state" (#tag-name-state), and
//! the opening Tokenization creation/emission/appropriate-end-tag paragraphs:
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>
//! Focused local tests, not upstream conformance counts.

use neptune_html::tokenizer::{
    CodePoint, Context, Execution, Input, ParseError, PendingComment, PendingEndTag,
    PendingStartTag, Position, Progress, State, TagKind, Token, UnimplementedState,
};

fn points(text: &str) -> Vec<CodePoint> {
    text.chars().map(CodePoint::from).collect()
}

fn error(code: &'static str, column: usize) -> ParseError {
    ParseError {
        code,
        position: Position { line: 1, column },
    }
}

fn seeded(kind: TagKind) -> Context {
    Context {
        current_tag: Some(kind),
        pending_start_tag: Some(PendingStartTag {
            name: points("start"),
            attributes: vec![],
            self_closing: false,
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("end"),
            attributes: vec![],
            self_closing: false,
        }),
        pending_comment: Some(PendingComment {
            data: points("comment"),
        }),
        tokens: vec![Token::Character('p'.into())],
        errors: vec![error("prior-diagnostic", 1)],
        last_start_tag: Some(points("title")),
        return_state: Some(State::Rcdata),
        temporary_buffer: vec![CodePoint::new(0xd800).unwrap(), 'x'.into()],
        ..Context::default()
    }
}

fn name(context: &Context, kind: TagKind) -> &Vec<CodePoint> {
    match kind {
        TagKind::Start => &context.pending_start_tag.as_ref().unwrap().name,
        TagKind::End => &context.pending_end_tag.as_ref().unwrap().name,
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Snapshot {
    state: State,
    current: Option<(Input, Position)>,
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
    input_errors: Vec<ParseError>,
    kind: Option<TagKind>,
    start: Option<PendingStartTag>,
    end: Option<PendingEndTag>,
    comment: Option<PendingComment>,
    last_start_tag: Option<Vec<CodePoint>>,
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
        kind: context.current_tag,
        start: context.pending_start_tag.clone(),
        end: context.pending_end_tag.clone(),
        comment: context.pending_comment.clone(),
        last_start_tag: context.last_start_tag.clone(),
        return_state: context.return_state,
        buffer: context.temporary_buffer.clone(),
    }
}

#[test]
fn tag_name_appends_every_ascii_letter_and_anything_else_once_with_ascii_only_folding() {
    let mut values: Vec<_> = (0x41..=0x5a).chain(0x61..=0x7a).collect();
    // ASCII boundaries, markup-looking punctuation, Unicode uppercase, actual
    // surrogates, noncharacters, and controls. VT/NBSP are NOT whitespace branches.
    values.extend([
        0x21, 0x22, 0x26, 0x27, 0x30, 0x39, 0x3c, 0x3d, 0x3f, 0x40, 0x5b, 0x60, 0x7b, 0xa0, 0xc9,
        0x130, 0x391, 0xdf, 0x301, 0x1f600, 0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x0b, 0x01, 0,
    ]);
    for kind in [TagKind::Start, TagKind::End] {
        for value in &values {
            let value = *value;
            for closed in [false, true] {
                let point = CodePoint::new(value).unwrap();
                let mut context = seeded(kind);
                let mut expected_name = name(&context, kind).clone();
                context.input.push([point, 'Z'.into()]).unwrap();
                if closed {
                    context.input.finish();
                }
                let mut execution = Execution::new(State::TagName, context);
                let mut expected = snapshot(&execution);
                assert_eq!(execution.step(), Ok(Progress::Continue(State::TagName)));
                expected.current = Some((Input::CodePoint(point), Position { line: 1, column: 1 }));
                expected_name.push(
                    CodePoint::new(match value {
                        0x41..=0x5a => value + 0x20,
                        0 => 0xfffd,
                        _ => value,
                    })
                    .unwrap(),
                );
                match kind {
                    TagKind::Start => expected.start.as_mut().unwrap().name = expected_name.clone(),
                    TagKind::End => expected.end.as_mut().unwrap().name = expected_name.clone(),
                }
                if value == 0 {
                    expected.errors.push(error("unexpected-null-character", 1));
                }
                let preprocessing = match value {
                    0xd800 | 0xdfff => Some("surrogate-in-input-stream"),
                    0xfdd0 | 0x10ffff => Some("noncharacter-in-input-stream"),
                    0x0b | 0x01 => Some("control-character-in-input-stream"),
                    _ => None,
                };
                if let Some(code) = preprocessing {
                    expected.input_errors.push(error(code, 1));
                }
                assert_eq!(snapshot(&execution), expected);
                assert_eq!(execution.step(), Ok(Progress::Continue(State::TagName)));
                expected_name.push('z'.into());
                assert_eq!(name(execution.context(), kind), &expected_name);
                assert_eq!(
                    execution.context().input.position(),
                    Position { line: 1, column: 2 }
                );
                // No reconsumption or duplicated preprocessing/state diagnostics.
                assert_eq!(execution.context().errors, expected.errors);
                assert_eq!(execution.context().input.errors(), expected.input_errors);
            }
        }
    }
}

#[test]
fn tag_name_whitespace_and_solidus_consume_without_emitting_before_destination_entry() {
    for kind in [TagKind::Start, TagKind::End] {
        for (text, consumed, target) in [
            ("\t", '\t', State::BeforeAttributeName),
            ("\n", '\n', State::BeforeAttributeName),
            ("\u{c}", '\u{c}', State::BeforeAttributeName),
            (" ", ' ', State::BeforeAttributeName),
            ("\r", '\n', State::BeforeAttributeName),
            ("\r\n", '\n', State::BeforeAttributeName),
            ("/", '/', State::SelfClosingStartTag),
        ] {
            for closed in [false, true] {
                let mut context = seeded(kind);
                context.input.push(points(&format!("{text}Z>"))).unwrap();
                if closed {
                    context.input.finish();
                }
                let mut execution = Execution::new(State::TagName, context);
                let mut expected = snapshot(&execution);
                assert_eq!(execution.step(), Ok(Progress::Continue(target)));
                expected.state = target;
                expected.current = Some((
                    Input::CodePoint(consumed.into()),
                    Position { line: 1, column: 1 },
                ));
                assert_eq!(snapshot(&execution), expected);
                // Source-local inspection; destination integration is covered in
                // before_attribute_name_state.rs and self_closing_start_tag_state.rs.
                assert_eq!(
                    execution.context_mut().input.consume(),
                    Input::CodePoint('Z'.into())
                );
            }
        }
    }
}

#[test]
fn tag_name_greater_than_emits_only_selected_tag_then_consumes_next_character_in_data() {
    for kind in [TagKind::Start, TagKind::End] {
        let mut context = seeded(kind);
        context.input.push(points(">x")).unwrap();
        context.input.finish();
        let emitted_name = name(&context, kind).clone();
        let mut execution = Execution::new(State::TagName, context);
        let mut expected = snapshot(&execution);
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
        expected.state = State::Data;
        expected.current = Some((
            Input::CodePoint('>'.into()),
            Position { line: 1, column: 1 },
        ));
        expected.kind = None;
        match kind {
            TagKind::Start => {
                expected.start = None;
                expected.last_start_tag = Some(emitted_name.clone());
                expected.tokens.push(Token::StartTag {
                    name: emitted_name,
                    self_closing: false,
                });
            }
            TagKind::End => {
                expected.end = None;
                expected.tokens.push(Token::EndTag {
                    name: emitted_name,
                    self_closing: false,
                });
            }
        }
        // Default attributes and flag cause neither end-tag emission diagnostic.
        // End tags do not alter last-start-tag context, whether matching or not.
        assert_eq!(snapshot(&execution), expected);
        assert_eq!(execution.step(), Ok(Progress::Continue(State::Data)));
        assert_eq!(
            execution.context().tokens.last(),
            Some(&Token::Character('x'.into()))
        );
        assert_eq!(
            execution.context().input.position(),
            Position { line: 1, column: 2 }
        );
        assert_eq!(execution.step(), Ok(Progress::Finished));
        assert_eq!(execution.context().tokens.last(), Some(&Token::Eof));
    }
}

#[test]
fn tag_name_need_more_input_preserves_pending_name_but_final_eof_discards_only_current_tag() {
    for kind in [TagKind::Start, TagKind::End] {
        let mut execution = Execution::new(State::TagName, seeded(kind));
        execution.context_mut().input.push(points("A\0")).unwrap();
        for _ in 0..2 {
            assert_eq!(execution.step(), Ok(Progress::Continue(State::TagName)));
        }
        let mut expected = snapshot(&execution);
        for _ in 0..2 {
            execution.context_mut().input.push([]).unwrap();
            assert_eq!(execution.step(), Ok(Progress::NeedMoreInput));
            assert_eq!(snapshot(&execution), expected);
        }
        execution.context_mut().input.finish();
        assert_eq!(execution.step(), Ok(Progress::Finished));
        expected.current = Some((Input::Eof, Position { line: 1, column: 3 }));
        expected.errors.push(error("eof-in-tag", 3));
        expected.tokens.push(Token::Eof);
        expected.kind = None;
        match kind {
            TagKind::Start => expected.start = None,
            TagKind::End => expected.end = None,
        }
        assert_eq!(snapshot(&execution), expected);
        for _ in 0..2 {
            assert_eq!(execution.step(), Ok(Progress::Finished));
            assert_eq!(snapshot(&execution), expected);
        }
    }
}

fn complete(input: &[CodePoint], context: Context) -> Execution {
    let mut execution = Execution::new(State::Data, context);
    execution
        .context_mut()
        .input
        .push(input.iter().copied())
        .unwrap();
    execution.context_mut().input.finish();
    let mut trace = vec![];
    assert!(drain(&mut execution, &mut trace, input.len() * 3 + 5));
    assert_eq!(execution.step(), Ok(Progress::Finished));
    execution
}

#[test]
fn tag_name_simple_tag_milestone_and_unicode_malformed_name_integration() {
    for (input, tag, text) in [
        ("<p>Hello</p>", "p", "Hello"),
        ("<DIV>world</DIV>", "div", "world"),
        ("<AÉΑİß😀>x</AÉΑİß😀>", "aÉΑİß😀", "x"),
        ("<A\"'<&=!?09>text</A\"'<&=!?09>", "a\"'<&=!?09", "text"),
    ] {
        let execution = complete(&points(input), Context::default());
        let mut expected = vec![Token::StartTag {
            name: points(tag),
            self_closing: false,
        }];
        expected.extend(points(text).into_iter().map(Token::Character));
        expected.extend([
            Token::EndTag {
                name: points(tag),
                self_closing: false,
            },
            Token::Eof,
        ]);
        assert_eq!(execution.context().tokens, expected);
        assert_eq!(execution.context().last_start_tag, Some(points(tag)));
        assert_eq!(execution.context().current_tag, None);
        assert_eq!(execution.context().pending_start_tag, None);
        assert_eq!(execution.context().pending_end_tag, None);
        assert!(execution.context().errors.is_empty());
        assert!(execution.context().input.errors().is_empty());
    }
    let execution = complete(&points("<A\0B></A\0B>"), Context::default());
    assert_eq!(
        execution.context().tokens,
        [
            Token::StartTag {
                name: points("a�b"),
                self_closing: false
            },
            Token::EndTag {
                name: points("a�b"),
                self_closing: false
            },
            Token::Eof
        ]
    );
    assert_eq!(
        execution.context().errors,
        [
            error("unexpected-null-character", 3),
            error("unexpected-null-character", 9)
        ]
    );
    for (input, last) in [
        ("</OTHER>", None),
        ("<P></other>", Some(points("p"))),
        ("<P><DIV></p>", Some(points("div"))),
        ("<P></p><unfinished", Some(points("p"))),
    ] {
        let execution = complete(&points(input), Context::default());
        assert_eq!(execution.context().last_start_tag, last);
    }
    for input in ["<a", "</A", "<A\0", "</A\0"] {
        let execution = complete(&points(input), Context::default());
        assert_eq!(execution.context().tokens, [Token::Eof]);
        assert_eq!(
            execution.context().errors.last(),
            Some(&error("eof-in-tag", input.chars().count() + 1))
        );
        assert_eq!(execution.context().last_start_tag, None);
    }
}

#[test]
fn tag_name_selector_is_set_only_by_tag_creation_and_preserved_by_prior_state_branches() {
    for initial in [
        State::Data,
        State::Plaintext,
        State::TagOpen,
        State::EndTagOpen,
    ] {
        for input in ["", "A", "z", "\0", "<", "&", "!", "?", "/", ">", "0"] {
            let mut context = seeded(TagKind::End);
            context.input.push(points(input)).unwrap();
            context.input.finish();
            let mut execution = Execution::new(initial, context);
            assert!(execution.step().is_ok());
            let expected = match (initial, input) {
                (State::TagOpen, "A" | "z") => TagKind::Start,
                _ => TagKind::End,
            };
            assert_eq!(execution.context().current_tag, Some(expected));
        }
    }
}

// Record every completed source step, excluding only suspension. Repeated
// suspension and terminal calls are checked for no observable context effects.
fn drain(
    execution: &mut Execution,
    trace: &mut Vec<(Result<Progress, UnimplementedState>, Snapshot)>,
    bound: usize,
) -> bool {
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
    panic!("Tag name integration failed to consume or terminate");
}

#[test]
fn tag_name_all_chunk_splits_preserve_steps_names_tokens_diagnostics_and_transitions() {
    let mut samples = vec![
        points("<p>Hello</p>"),
        points("<DIV>world</DIV>"),
        points("\r\n<AÉΑİ\0B>\r\n</AÉΑİ\0B>\r"),
        points("<A\u{b}B>"),
        points("<a"),
        points("</"),
        points("</A"),
        points("<a\0"),
        points("</A\0"),
        points("<A B>"),
        points("</A B>"),
        points("<A/>"),
        points("</A/>"),
        points("<A\r\nB>"),
        points("</A\r\r\nB>"),
        points("<A\r"),
        points("\r\n</"),
        points("<A>\r\n\0\r\r\nx\r"),
        points("<P></other>"),
        points("<A\"'<&=!?09>text</A\"'<&=!?09>"),
    ];
    let mut non_scalar = points("<A");
    non_scalar.extend([0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x01].map(|v| CodePoint::new(v).unwrap()));
    non_scalar.extend(points(">\r\n</A>"));
    samples.push(non_scalar);
    for input in samples {
        let run = |chunks: Vec<&[CodePoint]>, closed: bool| {
            let mut execution = Execution::new(State::Data, Context::default());
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
                    if drain(&mut execution, &mut trace, input.len() * 3 + 5) {
                        return trace;
                    }
                    execution.context_mut().input.push([]).unwrap();
                    assert!(!drain(&mut execution, &mut trace, 1));
                }
                execution.context_mut().input.finish();
            }
            assert!(drain(&mut execution, &mut trace, input.len() * 3 + 5));
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
        assert_eq!(run(input.chunks(1).collect(), false), expected);
    }
}
