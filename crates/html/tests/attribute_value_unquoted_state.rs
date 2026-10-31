//! Pinned Attribute value (unquoted) branch evidence; local, seeded contexts are
//! source-step contexts; Data paths are tested too. Terminal stubs are never resumed.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144356-L144404>

use neptune_html::tokenizer::{
    Attribute, CodePoint, Context, Execution, Input, ParseError, PendingComment, PendingEndTag,
    PendingStartTag, Position, Progress, State, TagKind, Token, UnimplementedState,
};

fn points(text: &str) -> Vec<CodePoint> {
    text.chars().map(Into::into).collect()
}

fn attribute(name: &str, value: &str) -> Attribute {
    Attribute {
        name: points(name),
        value: points(value),
    }
}

fn error(code: &'static str, line: usize, column: usize) -> ParseError {
    ParseError {
        code,
        position: Position { line, column },
    }
}

fn seeded(kind: TagKind, detached: bool) -> Context {
    let attributes = vec![attribute("prior", "retained"), attribute("current", "seed")];
    let discarded = detached.then(|| attribute("prior", "duplicate"));
    Context {
        current_tag: Some(kind),
        pending_start_tag: Some(PendingStartTag {
            name: points("start"),
            attributes: attributes.clone(),
            discarded_current_attribute: discarded.clone(),
            self_closing: false,
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("end"),
            attributes,
            discarded_current_attribute: discarded,
            self_closing: false,
        }),
        pending_comment: Some(PendingComment {
            data: points("comment"),
        }),
        tokens: vec![Token::Character('p'.into())],
        errors: vec![error("prior-diagnostic", 1, 1)],
        last_start_tag: Some(points("previous")),
        return_state: Some(State::Rcdata),
        temporary_buffer: points("scratch"),
        ..Context::default()
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

    fn current_attribute(&mut self, kind: TagKind) -> &mut Attribute {
        let (retained, discarded) = self.storage(kind);
        if let Some(attribute) = discarded {
            attribute
        } else {
            retained.last_mut().unwrap()
        }
    }

    fn take_selected(&mut self, kind: TagKind) -> Token {
        self.kind = None;
        match kind {
            TagKind::Start => {
                let tag = self.start.take().unwrap();
                Token::StartTag {
                    name: tag.name,
                    attributes: tag.attributes,
                    self_closing: tag.self_closing,
                }
            }
            TagKind::End => {
                let tag = self.end.take().unwrap();
                Token::EndTag {
                    name: tag.name,
                    attributes: tag.attributes,
                    self_closing: tag.self_closing,
                }
            }
        }
    }
}

fn snapshot(e: &Execution) -> Snapshot {
    let c = e.context();
    Snapshot {
        state: e.state(),
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

fn assert_terminal(e: &mut Execution, result: Result<Progress, UnimplementedState>) {
    let before = snapshot(e);
    for _ in 0..2 {
        assert_eq!(e.step(), result);
        assert_eq!(snapshot(e), before);
    }
}

#[test]
fn all_appending_ascii_and_non_scalars_preserve_current_identity_and_exact_errors() {
    let values = (0..=0x7f)
        .filter(|v| !matches!(v, 0x09 | 0x0a | 0x0c | 0x0d | 0x20 | 0x26 | 0x3e))
        .chain([
            0x85, 0xa0, 0x1680, 0x2003, 0x2028, 0x3000, 0xc9, 0x130, 0x212a, 0x1f600, 0xd800,
            0xdfff, 0xfdd0, 0x10ffff,
        ]);
    for value in values {
        let point = CodePoint::new(value).unwrap();
        for kind in [TagKind::Start, TagKind::End] {
            for detached in [false, true] {
                for closed in [false, true] {
                    let mut c = seeded(kind, detached);
                    c.input.push([point, 'Z'.into()]).unwrap();
                    if closed {
                        c.input.finish();
                    }
                    let mut e = Execution::new(State::AttributeValueUnquoted, c);
                    let mut expected = snapshot(&e);
                    expected.current =
                        Some((Input::CodePoint(point), Position { line: 1, column: 1 }));
                    expected.current_attribute(kind).value.push(if value == 0 {
                        '\u{fffd}'.into()
                    } else {
                        point
                    });
                    let diagnostic = match value {
                        0 => Some("unexpected-null-character"),
                        0x22 | 0x27 | 0x3c | 0x3d | 0x60 => {
                            Some("unexpected-character-in-unquoted-attribute-value")
                        }
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
                    assert_eq!(
                        e.step(),
                        Ok(Progress::Continue(State::AttributeValueUnquoted))
                    );
                    assert_eq!(
                        snapshot(&e),
                        expected,
                        "{kind:?}, detached={detached}, U+{value:04X}"
                    );
                    // Exactly one consumption, never reconsume the appended point.
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
}

#[test]
fn whitespace_terminates_without_append_error_or_reconsumption() {
    for kind in [TagKind::Start, TagKind::End] {
        for detached in [false, true] {
            for (raw, normalized, position) in [
                ("\tZ", '\t', Position { line: 1, column: 2 }),
                ("\nZ", '\n', Position { line: 2, column: 1 }),
                ("\u{c}Z", '\u{c}', Position { line: 1, column: 2 }),
                (" Z", ' ', Position { line: 1, column: 2 }),
                ("\rZ", '\n', Position { line: 2, column: 1 }),
                ("\r\nZ", '\n', Position { line: 2, column: 1 }),
            ] {
                let mut c = seeded(kind, detached);
                c.input.push(points(raw)).unwrap();
                let mut e = Execution::new(State::AttributeValueUnquoted, c);
                let mut expected = snapshot(&e);
                expected.state = State::BeforeAttributeName;
                expected.current = Some((
                    Input::CodePoint(normalized.into()),
                    Position { line: 1, column: 1 },
                ));
                assert_eq!(e.step(), Ok(Progress::Continue(State::BeforeAttributeName)));
                assert_eq!(snapshot(&e), expected);
                assert_eq!(
                    e.context_mut().input.consume(),
                    Input::CodePoint('Z'.into())
                );
                assert_eq!(e.context().input.position(), position);
            }
        }
    }
}

#[test]
fn ampersand_sets_return_state_then_blocks_terminally_without_consuming_tail() {
    for kind in [TagKind::Start, TagKind::End] {
        for detached in [false, true] {
            for closed in [false, true] {
                let mut c = seeded(kind, detached);
                c.input.push(points("&name;>")).unwrap();
                if closed {
                    c.input.finish();
                }
                let mut e = Execution::new(State::AttributeValueUnquoted, c);
                let mut expected = snapshot(&e);
                expected.state = State::CharacterReference;
                expected.return_state = Some(State::AttributeValueUnquoted);
                expected.current = Some((
                    Input::CodePoint('&'.into()),
                    Position { line: 1, column: 1 },
                ));
                assert_eq!(e.step(), Ok(Progress::Continue(State::CharacterReference)));
                assert_eq!(snapshot(&e), expected);
                assert_terminal(
                    &mut e,
                    Err(UnimplementedState {
                        state: State::CharacterReference,
                    }),
                );
                // Inspect only, never reconstruct or resume the blocked execution.
                assert_eq!(
                    e.context_mut().input.consume(),
                    Input::CodePoint('n'.into())
                );
                assert_eq!(
                    e.context().input.position(),
                    Position { line: 1, column: 2 }
                );
                assert_terminal(
                    &mut e,
                    Err(UnimplementedState {
                        state: State::CharacterReference,
                    }),
                );
            }
        }
    }
}

#[test]
fn greater_than_emits_only_retained_attributes_with_shared_end_tag_errors_and_bookkeeping() {
    for kind in [TagKind::Start, TagKind::End] {
        for detached in [false, true] {
            for flag in [false, true] {
                for detached_only in [false, true] {
                    if detached_only && !detached {
                        continue;
                    } // valid current attribute required
                    let mut c = seeded(kind, detached);
                    match kind {
                        TagKind::Start => {
                            let tag = c.pending_start_tag.as_mut().unwrap();
                            tag.self_closing = flag;
                            if detached_only {
                                tag.attributes.clear();
                            }
                        }
                        TagKind::End => {
                            let tag = c.pending_end_tag.as_mut().unwrap();
                            tag.self_closing = flag;
                            if detached_only {
                                tag.attributes.clear();
                            }
                        }
                    }
                    // Existing input and attribute values are not preprocessed again.
                    c.input.push(points("x\r\nX>Z")).unwrap();
                    assert_eq!(c.input.consume(), Input::CodePoint('x'.into()));
                    assert_eq!(c.input.consume(), Input::CodePoint('\n'.into()));
                    c.input.finish();
                    let mut e = Execution::new(State::AttributeValueUnquoted, c);
                    assert_eq!(
                        e.step(),
                        Ok(Progress::Continue(State::AttributeValueUnquoted))
                    );
                    let mut expected = snapshot(&e);
                    let token = expected.take_selected(kind);
                    if let Token::StartTag { name, .. } = &token {
                        expected.last_start_tag = Some(name.clone());
                    }
                    if kind == TagKind::End {
                        if !detached_only {
                            expected.errors.push(error("end-tag-with-attributes", 2, 2));
                        }
                        if flag {
                            expected
                                .errors
                                .push(error("end-tag-with-trailing-solidus", 2, 2));
                        }
                    }
                    expected.tokens.push(token);
                    expected.state = State::Data;
                    expected.current = Some((
                        Input::CodePoint('>'.into()),
                        Position { line: 2, column: 2 },
                    ));
                    assert_eq!(e.step(), Ok(Progress::Continue(State::Data)));
                    assert_eq!(snapshot(&e), expected);
                    // The delimiter was consumed; real Data handles the next point.
                    assert_eq!(e.step(), Ok(Progress::Continue(State::Data)));
                    expected.tokens.push(Token::Character('Z'.into()));
                    expected.current = Some((
                        Input::CodePoint('Z'.into()),
                        Position { line: 2, column: 3 },
                    ));
                    assert_eq!(snapshot(&e), expected);
                    assert_eq!(e.step(), Ok(Progress::Finished));
                    expected.tokens.push(Token::Eof);
                    expected.current = Some((Input::Eof, Position { line: 2, column: 4 }));
                    assert_eq!(snapshot(&e), expected);
                    assert_terminal(&mut e, Ok(Progress::Finished));
                }
            }
        }
    }
}

#[test]
fn open_exhaustion_is_effect_free_then_explicit_eof_abandons_only_selected_tag() {
    for kind in [TagKind::Start, TagKind::End] {
        for detached in [false, true] {
            for text in ["", "A\0\"'<=`/😀"] {
                let mut c = seeded(kind, detached);
                // A flag must not produce emission errors on abandonment.
                c.pending_start_tag.as_mut().unwrap().self_closing = true;
                c.pending_end_tag.as_mut().unwrap().self_closing = true;
                c.input.push(points(text)).unwrap();
                let mut e = Execution::new(State::AttributeValueUnquoted, c);
                for _ in text.chars() {
                    assert_eq!(
                        e.step(),
                        Ok(Progress::Continue(State::AttributeValueUnquoted))
                    );
                }
                let before = snapshot(&e);
                for _ in 0..2 {
                    assert_eq!(e.step(), Ok(Progress::NeedMoreInput));
                    assert_eq!(snapshot(&e), before);
                }
                e.context_mut().input.push([]).unwrap();
                assert_eq!(e.step(), Ok(Progress::NeedMoreInput));
                assert_eq!(snapshot(&e), before);
                e.context_mut().input.finish();
                let mut expected = before;
                expected.take_selected(kind); // abandoned, not emitted
                expected.tokens.push(Token::Eof);
                expected.current = Some((
                    Input::Eof,
                    Position {
                        line: 1,
                        column: text.chars().count() + 1,
                    },
                ));
                expected
                    .errors
                    .push(error("eof-in-tag", 1, text.chars().count() + 1));
                assert_eq!(e.step(), Ok(Progress::Finished));
                assert_eq!(snapshot(&e), expected);
                assert_terminal(&mut e, Ok(Progress::Finished));
            }
        }
    }
}

#[test]
fn normalized_positions_and_reconsumed_non_scalars_are_diagnosed_only_once() {
    for kind in [TagKind::Start, TagKind::End] {
        for detached in [false, true] {
            let mut c = seeded(kind, detached);
            let raw = [0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x0b];
            c.input
                .push(
                    points("x\r\n")
                        .into_iter()
                        .chain(raw.map(|v| CodePoint::new(v).unwrap()))
                        .chain(points("\0\"'<=`>")),
                )
                .unwrap();
            c.input.consume();
            c.input.consume();
            assert_eq!(
                c.input.consume(),
                Input::CodePoint(CodePoint::new(0xd800).unwrap())
            );
            c.input.reconsume().unwrap(); // a valid pre-dispatch cursor context, not a resumed stub
            let mut e = Execution::new(State::AttributeValueUnquoted, c);
            for _ in 0..11 {
                assert_eq!(
                    e.step(),
                    Ok(Progress::Continue(State::AttributeValueUnquoted))
                );
            }
            let mut expected_value = points(if detached { "duplicate" } else { "seed" });
            expected_value.extend(raw.map(|v| CodePoint::new(v).unwrap()));
            expected_value.extend(points("\u{fffd}\"'<=`"));
            assert_eq!(snapshot(&e).current_attribute(kind).value, expected_value);
            assert_eq!(
                e.context().input.errors(),
                [
                    error("surrogate-in-input-stream", 2, 1),
                    error("surrogate-in-input-stream", 2, 2),
                    error("noncharacter-in-input-stream", 2, 3),
                    error("noncharacter-in-input-stream", 2, 4),
                    error("control-character-in-input-stream", 2, 5),
                ]
            );
            let mut errors = vec![
                error("prior-diagnostic", 1, 1),
                error("unexpected-null-character", 2, 6),
            ];
            errors.extend((7..=11).map(|column| {
                error(
                    "unexpected-character-in-unquoted-attribute-value",
                    2,
                    column,
                )
            }));
            assert_eq!(e.context().errors, errors);
            assert_eq!(e.step(), Ok(Progress::Continue(State::Data)));
            if kind == TagKind::End {
                errors.push(error("end-tag-with-attributes", 2, 12));
            }
            assert_eq!(e.context().errors, errors);
        }
    }
}

// Traces include every production source step and the first terminal result, not
// merely matching output prefixes. NeedMoreInput is effect-free and resumable.
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
            assert_terminal(e, result);
            return true;
        }
    }
    panic!("unquoted trace did not suspend or terminate");
}

#[test]
fn whitespace_recovery_completes_or_abandons_unfinished_values() {
    for kind in [TagKind::Start, TagKind::End] {
        for detached in [false, true] {
            for text in [" /tail", " >tail", " ", " next=tail"] {
                let mut c = seeded(kind, detached);
                c.input.push(points(text)).unwrap();
                c.input.finish();
                let mut e = Execution::new(State::AttributeValueUnquoted, c);
                let mut expected = snapshot(&e);
                // Source whitespace terminates the value, without mutation.
                expected.state = State::BeforeAttributeName;
                expected.current = Some((
                    Input::CodePoint(' '.into()),
                    Position { line: 1, column: 1 },
                ));
                assert_eq!(e.step(), Ok(Progress::Continue(expected.state)));
                assert_eq!(snapshot(&e), expected);
                assert!(drain(&mut e, &mut vec![], 30));
                if text == " next=tail" {
                    expected.state = State::AttributeValueUnquoted;
                    expected.current = Some((
                        Input::Eof,
                        Position {
                            line: 1,
                            column: 11,
                        },
                    ));
                    expected.take_selected(kind);
                    expected.errors.push(error("eof-in-tag", 1, 11));
                    expected.tokens.push(Token::Eof);
                    assert_eq!(snapshot(&e), expected);
                    assert_terminal(&mut e, Ok(Progress::Finished));
                } else {
                    let eof = Position {
                        line: 1,
                        column: text.chars().count() + 1,
                    };
                    expected.current = Some((Input::Eof, eof));
                    if text == " /tail" {
                        expected
                            .errors
                            .push(error("unexpected-solidus-in-tag", 1, 3));
                        let (attributes, discarded) = expected.storage(kind);
                        attributes.push(attribute("tail", ""));
                        *discarded = None;
                    }
                    let token = expected.take_selected(kind);
                    if text == " >tail" {
                        expected.state = State::Data;
                        if kind == TagKind::Start {
                            expected.last_start_tag = Some(points("start"));
                        } else {
                            expected.errors.push(error("end-tag-with-attributes", 1, 2));
                        }
                        expected.tokens.push(token);
                        expected
                            .tokens
                            .extend("tail".chars().map(|c| Token::Character(c.into())));
                    } else {
                        expected.state = State::AfterAttributeName;
                        expected.errors.push(ParseError {
                            code: "eof-in-tag",
                            position: eof,
                        });
                    }
                    expected.tokens.push(Token::Eof);
                    assert_eq!(
                        snapshot(&e),
                        expected,
                        "{kind:?} detached={detached} {text:?}"
                    );
                    assert_terminal(&mut e, Ok(Progress::Finished));
                }
            }
        }
    }
    // Data now reaches unquoted parsing and emits complete retained values.
    for (text, kind) in [
        ("<a x=value>", TagKind::Start),
        ("</a x=value>", TagKind::End),
    ] {
        let mut c = Context::default();
        c.input.push(points(text)).unwrap();
        c.input.finish();
        let mut e = Execution::new(State::Data, c);
        assert!(drain(&mut e, &mut vec![], 40));
        assert_terminal(&mut e, Ok(Progress::Finished));
        let token = match kind {
            TagKind::Start => Token::StartTag {
                name: points("a"),
                attributes: vec![attribute("x", "value")],
                self_closing: false,
            },
            TagKind::End => Token::EndTag {
                name: points("a"),
                attributes: vec![attribute("x", "value")],
                self_closing: false,
            },
        };
        assert_eq!(e.context().tokens, [token, Token::Eof]);
        assert_eq!(
            e.context().errors,
            if kind == TagKind::End {
                vec![error("end-tag-with-attributes", 1, 12)]
            } else {
                vec![]
            }
        );
        assert_eq!(e.context().current_tag, None);
    }
}

#[test]
fn all_two_chunk_splits_and_single_point_chunks_match_complete_source_step_traces() {
    let mut samples: Vec<_> = [
        "",
        "AZ/é😀",
        "\0\"'<=`/>",
        "value/>",
        "value&name;>",
        "&",
        "value \t\r\nnext=x>",
        "value\r\n>",
        "value\r\r\n/>",
        "value\r",
        "value>",
        "value>x\0&",
        "value>\r\n</A>",
        " /",
        " /tail",
        " >tail",
        " ",
        " \r\n",
        "<a x=value>",
        "</a x=value>",
    ]
    .map(points)
    .into();
    samples.push(
        [
            points("AZ"),
            [0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x0b]
                .map(|v| CodePoint::new(v).unwrap())
                .to_vec(),
            points("\0\"'<=`/>\r\n"),
        ]
        .concat(),
    );
    for input in samples {
        for initial in [State::AttributeValueUnquoted, State::Data] {
            for kind in [TagKind::Start, TagKind::End] {
                for detached in [false, true] {
                    let run = |chunks: Vec<&[CodePoint]>, closed: bool| {
                        let c = if initial == State::Data {
                            Context::default()
                        } else {
                            seeded(kind, detached)
                        };
                        let mut e = Execution::new(initial, c);
                        let mut trace = vec![];
                        if closed {
                            e.context_mut().input.push(input.iter().copied()).unwrap();
                            e.context_mut().input.finish();
                        } else {
                            for chunk in chunks {
                                e.context_mut().input.push(chunk.iter().copied()).unwrap();
                                if drain(&mut e, &mut trace, input.len() * 3 + 10) {
                                    return trace;
                                }
                                e.context_mut().input.push([]).unwrap();
                                assert!(!drain(&mut e, &mut trace, 1));
                            }
                            e.context_mut().input.finish();
                        }
                        assert!(drain(&mut e, &mut trace, input.len() * 3 + 10));
                        trace
                    };
                    let expected = run(vec![], true);
                    for split in 0..=input.len() {
                        assert_eq!(
                            run(vec![&input[..split], &input[split..]], false),
                            expected,
                            "{initial:?}, {kind:?}, detached={detached}, split {split} of {input:?}"
                        );
                    }
                    assert_eq!(run(input.chunks(1).collect(), false), expected);
                }
            }
        }
    }
}
