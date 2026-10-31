//! Branch evidence for pinned HTML "Attribute name state", including departure
//! comparison and the removed-but-current note. Local tests, not upstream runs.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>

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

fn attribute(name: &str, value: &str) -> Attribute {
    Attribute {
        name: points(name),
        value: points(value),
    }
}

fn seeded(kind: TagKind) -> Context {
    let attributes = vec![
        attribute("prior", "retained"),
        attribute("other", "predecessor"),
        Attribute {
            name: vec![],
            value: vec![CodePoint::new(0xd800).unwrap(), 'v'.into()],
        },
    ];
    let mut context = Context {
        current_tag: Some(kind),
        pending_start_tag: Some(PendingStartTag {
            name: points("start"),
            self_closing: true,
            attributes: attributes.clone(),
            ..PendingStartTag::default()
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("end"),
            self_closing: true,
            attributes,
            ..PendingEndTag::default()
        }),
        pending_comment: Some(PendingComment {
            data: points("comment"),
        }),
        tokens: vec![Token::Character('p'.into())],
        errors: vec![error("prior-diagnostic", 1, 1)],
        last_start_tag: Some(points("title")),
        return_state: Some(State::Rcdata),
        temporary_buffer: points("scratch"),
        ..Context::default()
    };
    // Unrelated tag storage can itself retain a detached current attribute.
    match kind {
        TagKind::Start => {
            context
                .pending_end_tag
                .as_mut()
                .unwrap()
                .discarded_current_attribute = Some(attribute("unrelated", "discarded"))
        }
        TagKind::End => {
            context
                .pending_start_tag
                .as_mut()
                .unwrap()
                .discarded_current_attribute = Some(attribute("unrelated", "discarded"))
        }
    }
    context
}

fn storage(context: &mut Context) -> (&mut Vec<Attribute>, &mut Option<Attribute>) {
    match context.current_tag.unwrap() {
        TagKind::Start => {
            let tag = context.pending_start_tag.as_mut().unwrap();
            (&mut tag.attributes, &mut tag.discarded_current_attribute)
        }
        TagKind::End => {
            let tag = context.pending_end_tag.as_mut().unwrap();
            (&mut tag.attributes, &mut tag.discarded_current_attribute)
        }
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

impl Snapshot {
    fn storage(&mut self, kind: TagKind) -> (&mut Vec<Attribute>, &mut Option<Attribute>) {
        match kind {
            TagKind::Start => {
                let tag = self.start.as_mut().unwrap();
                (&mut tag.attributes, &mut tag.discarded_current_attribute)
            }
            TagKind::End => {
                let tag = self.end.as_mut().unwrap();
                (&mut tag.attributes, &mut tag.discarded_current_attribute)
            }
        }
    }
}

fn snapshot(execution: &Execution) -> Snapshot {
    let c = execution.context();
    Snapshot {
        state: execution.state(),
        current: c.input.current(),
        tokens: c.tokens.clone(),
        errors: c.errors.clone(),
        input_errors: c.input.errors().to_vec(),
        kind: c.current_tag,
        start: c.pending_start_tag.clone(),
        end: c.pending_end_tag.clone(),
        comment: c.pending_comment.clone(),
        last_start_tag: c.last_start_tag.clone(),
        return_state: c.return_state,
        buffer: c.temporary_buffer.clone(),
    }
}

fn assert_blocked(e: &mut Execution, target: State, expected: &Snapshot) {
    for _ in 0..2 {
        assert_eq!(e.step(), Err(UnimplementedState { state: target }));
        assert_eq!(&snapshot(e), expected);
    }
}

#[test]
fn appends_each_ascii_branch_and_literal_non_ascii_with_exact_diagnostics_and_context() {
    let values = (0..=0x7f)
        .filter(|v| !matches!(v, 0x09 | 0x0a | 0x0c | 0x0d | 0x20 | 0x2f | 0x3d | 0x3e))
        .chain([
            0x85, 0xa0, 0x1680, 0x2003, 0x2028, 0x3000, 0xc9, 0x130, 0x212a, 0x1f600, 0xd800,
            0xdfff, 0xfdd0, 0x10ffff,
        ]);
    for value in values {
        let point = CodePoint::new(value).unwrap();
        for kind in [TagKind::Start, TagKind::End] {
            for closed in [false, true] {
                let mut context = seeded(kind);
                context.input.push([point, 'Z'.into()]).unwrap();
                if closed {
                    context.input.finish();
                }
                let mut e = Execution::new(State::AttributeName, context);
                let mut expected = snapshot(&e);
                expected.current = Some((Input::CodePoint(point), Position { line: 1, column: 1 }));
                let appended = match value {
                    0 => '\u{fffd}'.into(),
                    0x41..=0x5a => CodePoint::new(value + 0x20).unwrap(),
                    _ => point,
                };
                expected
                    .storage(kind)
                    .0
                    .last_mut()
                    .unwrap()
                    .name
                    .push(appended);
                let diagnostic = match value {
                    0 => Some("unexpected-null-character"),
                    0x22 | 0x27 | 0x3c => Some("unexpected-character-in-attribute-name"),
                    _ => None,
                };
                if let Some(code) = diagnostic {
                    expected.errors.push(error(code, 1, 1));
                }
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
                assert_eq!(e.step(), Ok(Progress::Continue(State::AttributeName)));
                assert_eq!(snapshot(&e), expected, "{kind:?}, {value:x}");
                // Source-local cursor inspection: exactly one consumed, no reconsumption.
                assert_eq!(
                    e.context_mut().input.consume(),
                    Input::CodePoint('Z'.into())
                );
                assert_eq!(
                    e.context().input.position(),
                    Position { line: 1, column: 2 }
                );
            }
        }
    }
}

#[test]
fn every_departure_checks_duplicates_before_terminal_destination_and_preserves_current_identity() {
    for kind in [TagKind::Start, TagKind::End] {
        for (text, consumed, target, reconsume) in [
            (
                "\tZ",
                Input::CodePoint('\t'.into()),
                State::AfterAttributeName,
                true,
            ),
            (
                "\nZ",
                Input::CodePoint('\n'.into()),
                State::AfterAttributeName,
                true,
            ),
            (
                "\u{c}Z",
                Input::CodePoint('\u{c}'.into()),
                State::AfterAttributeName,
                true,
            ),
            (
                " Z",
                Input::CodePoint(' '.into()),
                State::AfterAttributeName,
                true,
            ),
            (
                "\rZ",
                Input::CodePoint('\n'.into()),
                State::AfterAttributeName,
                true,
            ),
            (
                "\r\nZ",
                Input::CodePoint('\n'.into()),
                State::AfterAttributeName,
                true,
            ),
            (
                "/Z",
                Input::CodePoint('/'.into()),
                State::AfterAttributeName,
                true,
            ),
            (
                ">Z",
                Input::CodePoint('>'.into()),
                State::AfterAttributeName,
                true,
            ),
            (
                "=Z",
                Input::CodePoint('='.into()),
                State::BeforeAttributeValue,
                false,
            ),
            ("", Input::Eof, State::AfterAttributeName, true),
        ] {
            for duplicate in [false, true] {
                let mut c = seeded(kind);
                storage(&mut c).0.last_mut().unwrap().name =
                    points(if duplicate { "prior" } else { "different" });
                c.input.push(points(text)).unwrap();
                c.input.finish();
                let mut e = Execution::new(State::AttributeName, c);
                let mut expected = snapshot(&e);
                expected.state = target;
                expected.current = Some((consumed, Position { line: 1, column: 1 }));
                if duplicate {
                    let (attributes, discarded) = expected.storage(kind);
                    *discarded = attributes.pop();
                    expected.errors.push(error("duplicate-attribute", 1, 1));
                }
                assert_eq!(e.step(), Ok(Progress::Continue(target)));
                assert_eq!(snapshot(&e), expected, "{kind:?}, {text:?}, {duplicate}");
                // No fake EOF, no eof-in-tag/end-tag-with-attributes, no output.
                assert_blocked(&mut e, target, &expected);
                assert_eq!(
                    e.context_mut().input.consume(),
                    if reconsume {
                        consumed
                    } else {
                        Input::CodePoint('Z'.into())
                    }
                );
                if reconsume && consumed != Input::Eof {
                    assert_eq!(
                        e.context_mut().input.consume(),
                        Input::CodePoint('Z'.into())
                    );
                }
                let inspected = snapshot(&e);
                assert_blocked(&mut e, target, &inspected); // cannot resume a terminal block
            }
        }
    }
}

#[test]
fn suspension_never_finishes_a_name_or_removes_a_prefix_duplicate_but_final_eof_does() {
    for kind in [TagKind::Start, TagKind::End] {
        for extend in [false, true] {
            let mut c = seeded(kind);
            storage(&mut c).0[0].name = points("a");
            c.input.push(points("A")).unwrap();
            let mut e = Execution::new(State::AttributeName, c);
            assert_eq!(e.step(), Ok(Progress::Continue(State::AttributeName)));
            let unchanged = snapshot(&e);
            for _ in 0..3 {
                e.context_mut().input.push([]).unwrap();
                assert_eq!(e.step(), Ok(Progress::NeedMoreInput));
                assert_eq!(snapshot(&e), unchanged);
            }
            if extend {
                e.context_mut().input.push(points("B")).unwrap();
                assert_eq!(e.step(), Ok(Progress::Continue(State::AttributeName)));
                assert_eq!(
                    storage(e.context_mut()).0.last().unwrap().name,
                    points("ab")
                );
                assert_eq!(e.step(), Ok(Progress::NeedMoreInput));
            }
            e.context_mut().input.finish();
            assert_eq!(e.step(), Ok(Progress::Continue(State::AfterAttributeName)));
            let (attributes, discarded) = storage(e.context_mut());
            assert_eq!(attributes.len(), if extend { 3 } else { 2 });
            assert_eq!(discarded.is_some(), !extend);
            assert_eq!(e.context().errors.len(), if extend { 1 } else { 2 });
            if !extend {
                assert_eq!(
                    e.context().errors.last(),
                    Some(&error("duplicate-attribute", 1, 2))
                );
            }
            let expected = snapshot(&e);
            assert_blocked(&mut e, State::AfterAttributeName, &expected);
            assert_eq!(e.context_mut().input.consume(), Input::Eof);
        }
    }
}

#[test]
fn completed_names_compare_exactly_after_ascii_folding_replacement_and_diagnostic_appends() {
    for kind in [TagKind::Start, TagKind::End] {
        for (prior, input, duplicate) in [
            ("abc", "ABC", true),
            ("ABC", "abc", false), // seeded storage is not renormalized
            ("é", "É", false),
            ("É", "É", true),
            ("k", "K", false),
            ("i", "İ", false),
            ("é", "e\u{301}", false), // no Unicode normalization
            ("", "", true),
            ("a�\"'<", "A\0\"'<", true),
        ] {
            let mut c = seeded(kind);
            storage(&mut c).0[0].name = points(prior);
            // Seed a normalized location before isolated entry, no fake state algorithm.
            c.input.push(points("\r\n😀")).unwrap();
            assert_eq!(c.input.consume(), Input::CodePoint('\n'.into()));
            assert_eq!(c.input.consume(), Input::CodePoint('😀'.into()));
            c.input.push(points(&format!("{input}="))).unwrap();
            c.input.finish();
            let mut e = Execution::new(State::AttributeName, c);
            for _ in input.chars() {
                assert_eq!(e.step(), Ok(Progress::Continue(State::AttributeName)));
            }
            let mut expected_errors = vec![error("prior-diagnostic", 1, 1)];
            if input.contains('\0') {
                expected_errors.extend([
                    error("unexpected-null-character", 2, 3),
                    error("unexpected-character-in-attribute-name", 2, 4),
                    error("unexpected-character-in-attribute-name", 2, 5),
                    error("unexpected-character-in-attribute-name", 2, 6),
                ]);
            }
            assert_eq!(e.context().errors, expected_errors); // no duplicate before departure
            assert_eq!(
                e.step(),
                Ok(Progress::Continue(State::BeforeAttributeValue))
            );
            if duplicate {
                expected_errors.push(error("duplicate-attribute", 2, input.chars().count() + 2));
            }
            assert_eq!(e.context().errors, expected_errors);
            let (attributes, discarded) = storage(e.context_mut());
            assert_eq!(attributes[0].name, points(prior));
            assert_eq!(attributes[0].value, points("retained"));
            assert_eq!(attributes[1], attribute("other", "predecessor"));
            assert_eq!(discarded.is_some(), duplicate);
            let current = discarded
                .as_ref()
                .unwrap_or_else(|| attributes.last().unwrap());
            assert_eq!(current.value, [CodePoint::new(0xd800).unwrap(), 'v'.into()]);
            let terminal = snapshot(&e);
            assert_blocked(&mut e, State::BeforeAttributeValue, &terminal);
        }
    }
}

#[test]
fn real_tag_name_before_attribute_name_entry_accumulates_names_but_never_emits_attributes() {
    for (input, kind, name, target, errors) in [
        (
            "<DIV AbC>",
            TagKind::Start,
            "abc",
            State::AfterAttributeName,
            vec![],
        ),
        (
            "</DIV AbC",
            TagKind::End,
            "abc",
            State::AfterAttributeName,
            vec![],
        ),
        (
            "<a =X=tail",
            TagKind::Start,
            "=x",
            State::BeforeAttributeValue,
            vec![error("unexpected-equals-sign-before-attribute-name", 1, 4)],
        ),
        (
            "😀\r\n<a A\0\"'<=tail",
            TagKind::Start,
            "a�\"'<",
            State::BeforeAttributeValue,
            vec![
                error("unexpected-null-character", 2, 5),
                error("unexpected-character-in-attribute-name", 2, 6),
                error("unexpected-character-in-attribute-name", 2, 7),
                error("unexpected-character-in-attribute-name", 2, 8),
            ],
        ),
        (
            "</a É\u{a0}/>",
            TagKind::End,
            "É\u{a0}",
            State::AfterAttributeName,
            vec![],
        ),
    ] {
        let mut c = Context::default();
        c.input.push(points(input)).unwrap();
        c.input.finish();
        let mut e = Execution::new(State::Data, c);
        let mut saw = vec![];
        for _ in 0..input.chars().count() * 3 + 5 {
            saw.push(e.state());
            if e.step().is_err() {
                break;
            }
        }
        assert_eq!(e.step(), Err(UnimplementedState { state: target }));
        assert!(saw.contains(&State::TagName));
        assert!(saw.contains(&State::BeforeAttributeName));
        assert!(saw.contains(&State::AttributeName));
        assert_eq!(e.context().current_tag, Some(kind));
        assert_eq!(e.context().errors, errors);
        assert!(e.context().input.errors().is_empty());
        let (attributes, discarded) = storage(e.context_mut());
        assert_eq!(attributes, &[attribute(name, "")]);
        assert!(discarded.is_none());
        assert!(e.context().last_start_tag.is_none());
        assert!(!e.context().tokens.iter().any(|token| matches!(
            token,
            Token::StartTag { .. } | Token::EndTag { .. } | Token::Eof
        )));
    }
}

#[test]
fn non_scalar_names_survive_before_name_reconsumption_and_exact_duplicate_comparison() {
    for kind in [TagKind::Start, TagKind::End] {
        let prefix = if kind == TagKind::Start {
            "<A "
        } else {
            "</A "
        };
        let column = prefix.chars().count() + 1;
        let raw = [0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x0b];
        let name: Vec<_> = raw.map(|v| CodePoint::new(v).unwrap()).into();
        let mut c = Context::default();
        c.input
            .push(
                points(prefix)
                    .into_iter()
                    .chain(name.clone())
                    .chain(points("=")),
            )
            .unwrap();
        c.input.finish();
        let mut e = Execution::new(State::Data, c);
        assert!(drain(&mut e, &mut vec![], 40));
        assert_eq!(
            e.step(),
            Err(UnimplementedState {
                state: State::BeforeAttributeValue
            })
        );
        assert_eq!(
            storage(e.context_mut()).0,
            &[Attribute {
                name: name.clone(),
                value: vec![]
            }]
        );
        assert!(e.context().errors.is_empty());
        let errors = vec![
            error("surrogate-in-input-stream", 1, column),
            error("surrogate-in-input-stream", 1, column + 1),
            error("noncharacter-in-input-stream", 1, column + 2),
            error("noncharacter-in-input-stream", 1, column + 3),
            error("control-character-in-input-stream", 1, column + 4),
        ];
        assert_eq!(e.context().input.errors(), errors); // first point was reconsumed only once

        // Downstream stubs prevent a real second attribute; seed its prior name.
        // Do NOT reconstruct the blocked execution or invent an After-name handler.
        for duplicate in [false, true] {
            let mut c = seeded(kind);
            storage(&mut c).0[0].name = name.clone();
            let mut current_name = name.clone();
            if !duplicate {
                current_name[0] = CodePoint::new(0xd801).unwrap();
            }
            c.input
                .push(current_name.iter().copied().chain(points("=")))
                .unwrap();
            c.input.finish();
            let mut e = Execution::new(State::AttributeName, c);
            assert!(drain(&mut e, &mut vec![], 20));
            let (attributes, discarded) = storage(e.context_mut());
            assert_eq!(attributes[0].name, name);
            assert_eq!(discarded.is_some(), duplicate);
            let current = discarded
                .as_ref()
                .unwrap_or_else(|| attributes.last().unwrap());
            assert_eq!(current.name, current_name);
            assert_eq!(e.context().errors.len(), if duplicate { 2 } else { 1 });
            if duplicate {
                assert_eq!(
                    e.context().errors.last(),
                    Some(&error("duplicate-attribute", 1, 6))
                );
            }
        }
    }
}

// Real execution only, stopping at the FIRST terminal stub. Suspension is not
// a completed source step; empty chunks and repeated suspension are effect-free.
type Trace = Vec<(Result<Progress, UnimplementedState>, Snapshot)>;
fn drain(e: &mut Execution, trace: &mut Trace, bound: usize) -> bool {
    for _ in 0..bound {
        let before = snapshot(e);
        let result = e.step();
        if result == Ok(Progress::NeedMoreInput) {
            assert_eq!(snapshot(e), before);
            assert_eq!(e.step(), result);
            assert_eq!(snapshot(e), before);
            return false;
        }
        trace.push((result, snapshot(e)));
        if !matches!(result, Ok(Progress::Continue(_))) {
            let terminal = snapshot(e);
            assert_eq!(e.step(), result);
            assert_eq!(snapshot(e), terminal);
            return true;
        }
    }
    panic!("Attribute name integration did not suspend or terminate");
}

#[test]
fn every_chunk_split_matches_closed_source_steps_at_name_boundaries_crlf_errors_and_stubs() {
    let mut samples: Vec<_> = [
        "",
        "prior",
        "PRIOR=tail",
        "priorX",
        "other ",
        "prior\r\n=tail",
        "\0\"'<éK",
        "<a abc>",
        "</a ABC",
        "<a a=tail",
        "<a =X>",
        "<a prior\r\n=tail",
        "😀\r\n</A\tB\0\"'<>",
        "<a\r\nÉ\u{a0}/>",
        "<a x\r",
        "<a\r\r\nN>",
        "<a/\0>",
        "<p>ok</p>",
    ]
    .map(points)
    .into();
    for value in [0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x0b] {
        let mut input = points("<A \r\n");
        input.push(CodePoint::new(value).unwrap());
        input.extend(points("Z=tail"));
        samples.push(input);
    }
    for input in samples {
        for initial in [State::Data, State::AttributeName] {
            for kind in [TagKind::Start, TagKind::End] {
                let run = |chunks: Vec<&[CodePoint]>, closed: bool| {
                    let c = if initial == State::Data {
                        Context::default()
                    } else {
                        seeded(kind)
                    };
                    let mut e = Execution::new(initial, c);
                    let mut trace = vec![];
                    if closed {
                        e.context_mut().input.push(input.iter().copied()).unwrap();
                        e.context_mut().input.finish();
                    } else {
                        for chunk in chunks {
                            e.context_mut().input.push(chunk.iter().copied()).unwrap();
                            if drain(&mut e, &mut trace, input.len() * 3 + 5) {
                                return trace;
                            }
                            e.context_mut().input.push([]).unwrap();
                            assert!(!drain(&mut e, &mut trace, 1));
                        }
                        e.context_mut().input.finish();
                    }
                    assert!(drain(&mut e, &mut trace, input.len() * 3 + 5));
                    trace
                };
                let expected = run(vec![], true);
                for split in 0..=input.len() {
                    assert_eq!(
                        run(vec![&input[..split], &input[split..]], false),
                        expected,
                        "{initial:?}, {kind:?}, split {split} of {input:?}"
                    );
                }
                assert_eq!(run(input.chunks(1).collect(), false), expected);
            }
        }
    }
}
