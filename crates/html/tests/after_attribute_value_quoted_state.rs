//! Local evidence, not upstream conformance. Isolated entry is seeded because
//! Data reaches value parsing; Character reference remains terminal. No stub is resumed.
//! Pinned algorithm: HTML 404b3eb14a7a936dddefb2d78e224dbeb01d2358
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144402-L144432>

use neptune_html::tokenizer::{
    Attribute, CodePoint, Context, Execution, Input, ParseError, PendingComment, PendingEndTag,
    PendingStartTag, Position, Progress, State, TagKind, Token, UnimplementedState,
};

const SOURCE: State = State::AfterAttributeValueQuoted;

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

fn retained() -> Vec<Attribute> {
    vec![
        attribute("same", "original"),
        Attribute {
            name: points("other"),
            value: vec![CodePoint::new(0xd800).unwrap(), '\0'.into(), '😀'.into()],
        },
    ]
}

fn seeded(kind: TagKind, flag: bool) -> Context {
    Context {
        current_tag: Some(kind),
        pending_start_tag: Some(PendingStartTag {
            name: points("start"),
            self_closing: flag,
            attributes: retained(),
            discarded_current_attribute: Some(attribute("same", "discarded-start")),
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("end"),
            self_closing: flag,
            attributes: retained(),
            discarded_current_attribute: Some(attribute("same", "discarded-end")),
        }),
        pending_comment: Some(PendingComment {
            data: points("unrelated"),
        }),
        tokens: vec![Token::Character('p'.into())],
        errors: vec![error("prior", 1, 1)],
        last_start_tag: Some(points("title")),
        return_state: Some(State::Rcdata),
        temporary_buffer: vec![CodePoint::new(0xdfff).unwrap()],
        ..Context::default()
    }
}

// Position evidence: a supplementary scalar counts as one point; CRLF as one LF.
fn prefix(c: &mut Context) {
    c.input.push(points("😀\r\n")).unwrap();
    assert_eq!(c.input.consume(), Input::CodePoint('😀'.into()));
    assert_eq!(c.input.consume(), Input::CodePoint('\n'.into()));
    assert_eq!(c.input.consume(), Input::NeedMoreInput);
}

fn storage(c: &Context, kind: TagKind) -> (&[Attribute], &Option<Attribute>) {
    match kind {
        TagKind::Start => {
            let t = c.pending_start_tag.as_ref().unwrap();
            (&t.attributes, &t.discarded_current_attribute)
        }
        TagKind::End => {
            let t = c.pending_end_tag.as_ref().unwrap();
            (&t.attributes, &t.discarded_current_attribute)
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
    last: Option<Vec<CodePoint>>,
    return_state: Option<State>,
    buffer: Vec<CodePoint>,
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
        last: c.last_start_tag.clone(),
        return_state: c.return_state,
        buffer: c.temporary_buffer.clone(),
    }
}

fn preprocessing(value: u32) -> Option<&'static str> {
    match value {
        0xd800..=0xdfff => Some("surrogate-in-input-stream"),
        0xfdd0..=0xfdef | 0x10ffff => Some("noncharacter-in-input-stream"),
        0x01..=0x08 | 0x0b | 0x0e..=0x1f | 0x7f..=0x9f => Some("control-character-in-input-stream"),
        _ => None,
    }
}

#[test]
fn both_quoted_value_sources_reach_emission_or_eof_with_exact_retained_and_detached_values() {
    for (source, quote) in [
        (State::AttributeValueSingleQuoted, '\''),
        (State::AttributeValueDoubleQuoted, '"'),
    ] {
        for kind in [TagKind::Start, TagKind::End] {
            for detached in [false, true] {
                for ending in [">", "/>", ""] {
                    let mut c = seeded(kind, false);
                    if !detached {
                        match kind {
                            TagKind::Start => {
                                c.pending_start_tag
                                    .as_mut()
                                    .unwrap()
                                    .discarded_current_attribute = None
                            }
                            TagKind::End => {
                                c.pending_end_tag
                                    .as_mut()
                                    .unwrap()
                                    .discarded_current_attribute = None
                            }
                        }
                    }
                    c.input.push(points(&format!("v{quote}{ending}"))).unwrap();
                    c.input.finish();
                    let mut e = Execution::new(source, c);
                    let mut expected = snapshot(&e);
                    let (name, attributes) = {
                        let (name, attributes, discarded) = match kind {
                            TagKind::Start => {
                                let t = expected.start.as_mut().unwrap();
                                (
                                    &t.name,
                                    &mut t.attributes,
                                    &mut t.discarded_current_attribute,
                                )
                            }
                            TagKind::End => {
                                let t = expected.end.as_mut().unwrap();
                                (
                                    &t.name,
                                    &mut t.attributes,
                                    &mut t.discarded_current_attribute,
                                )
                            }
                        };
                        discarded
                            .as_mut()
                            .unwrap_or_else(|| attributes.last_mut().unwrap())
                            .value
                            .push('v'.into());
                        (name.clone(), attributes.clone())
                    };
                    expected.current = Some((
                        Input::CodePoint('v'.into()),
                        Position { line: 1, column: 1 },
                    ));
                    assert_eq!(e.step(), Ok(Progress::Continue(source)));
                    assert_eq!(snapshot(&e), expected);
                    expected.state = SOURCE;
                    expected.current = Some((
                        Input::CodePoint(quote.into()),
                        Position { line: 1, column: 2 },
                    ));
                    assert_eq!(e.step(), Ok(Progress::Continue(SOURCE)));
                    assert_eq!(snapshot(&e), expected);
                    if ending == "/>" {
                        expected.state = State::SelfClosingStartTag;
                        expected.current = Some((
                            Input::CodePoint('/'.into()),
                            Position { line: 1, column: 3 },
                        ));
                        assert_eq!(e.step(), Ok(Progress::Continue(State::SelfClosingStartTag)));
                        assert_eq!(snapshot(&e), expected);
                    }
                    expected.kind = None;
                    match kind {
                        TagKind::Start => expected.start = None,
                        TagKind::End => expected.end = None,
                    }
                    if ending.is_empty() {
                        expected.current = Some((Input::Eof, Position { line: 1, column: 3 }));
                        expected.errors.push(error("eof-in-tag", 1, 3));
                    } else {
                        let column = if ending == "/>" { 4 } else { 3 };
                        expected.state = State::Data;
                        expected.current =
                            Some((Input::CodePoint('>'.into()), Position { line: 1, column }));
                        let self_closing = ending == "/>";
                        expected.tokens.push(match kind {
                            TagKind::Start => {
                                expected.last = Some(name.clone());
                                Token::StartTag {
                                    name,
                                    attributes,
                                    self_closing,
                                }
                            }
                            TagKind::End => {
                                expected
                                    .errors
                                    .push(error("end-tag-with-attributes", 1, column));
                                if self_closing {
                                    expected.errors.push(error(
                                        "end-tag-with-trailing-solidus",
                                        1,
                                        column,
                                    ));
                                }
                                Token::EndTag {
                                    name,
                                    attributes,
                                    self_closing,
                                }
                            }
                        });
                        assert_eq!(e.step(), Ok(Progress::Continue(State::Data)));
                        assert_eq!(snapshot(&e), expected);
                        expected.current = Some((
                            Input::Eof,
                            Position {
                                line: 1,
                                column: column + 1,
                            },
                        ));
                    }
                    expected.tokens.push(Token::Eof);
                    assert_eq!(e.step(), Ok(Progress::Finished));
                    assert_eq!(snapshot(&e), expected);
                    assert_eq!(e.step(), Ok(Progress::Finished));
                    assert_eq!(snapshot(&e), expected);
                }
            }
        }
    }
}

#[test]
fn whitespace_solidus_and_every_other_point_have_exact_source_effects_and_consumption() {
    let values = (0..=0x7f).filter(|v| *v != 0x3e).chain([
        0x85, 0xa0, 0xc9, 0x2003, 0x1f600, 0xd800, 0xdfff, 0xfdd0, 0x10ffff,
    ]);
    for value in values {
        let point = CodePoint::new(value).unwrap();
        let normalized = if value == 0x0d { '\n'.into() } else { point };
        let whitespace = matches!(value, 0x09 | 0x0a | 0x0c | 0x0d | 0x20);
        let recovery = !whitespace && value != 0x2f;
        let target = if value == 0x2f {
            State::SelfClosingStartTag
        } else {
            State::BeforeAttributeName
        };
        for kind in [TagKind::Start, TagKind::End] {
            for flag in [false, true] {
                let mut c = seeded(kind, flag);
                prefix(&mut c);
                c.input.push([point, 'Z'.into()]).unwrap();
                c.input.finish();
                let mut e = Execution::new(SOURCE, c);
                let mut expected = snapshot(&e);
                assert_eq!(e.step(), Ok(Progress::Continue(target)));
                expected.state = target;
                expected.current = Some((
                    Input::CodePoint(normalized),
                    Position { line: 2, column: 1 },
                ));
                if recovery {
                    expected
                        .errors
                        .push(error("missing-whitespace-between-attributes", 2, 1));
                }
                if let Some(code) = preprocessing(value) {
                    expected.input_errors.push(error(code, 2, 1));
                }
                // Includes both tags' retained/detached attributes and all unrelated storage.
                assert_eq!(snapshot(&e), expected, "{kind:?}, U+{value:04X}");
                if recovery {
                    assert_eq!(
                        e.context_mut().input.consume(),
                        Input::CodePoint(normalized)
                    );
                    assert_eq!(snapshot(&e), expected); // no duplicate preprocessing
                }
                // No reconsumption for whitespace or solidus; recovery consumed above.
                assert_eq!(
                    e.context_mut().input.consume(),
                    Input::CodePoint('Z'.into())
                );
                let column = if normalized == '\n'.into() { 1 } else { 2 };
                let line = if normalized == '\n'.into() { 3 } else { 2 };
                assert_eq!(e.context().input.position(), Position { line, column });
                assert_eq!(e.context().input.errors(), expected.input_errors);
            }
        }
    }
}

#[test]
fn greater_than_uses_shared_emission_excludes_detached_duplicate_and_preserves_identity() {
    for kind in [TagKind::Start, TagKind::End] {
        for flag in [false, true] {
            for detached_only in [false, true] {
                let mut c = seeded(kind, flag);
                if detached_only {
                    match kind {
                        TagKind::Start => c.pending_start_tag.as_mut().unwrap().attributes.clear(),
                        TagKind::End => c.pending_end_tag.as_mut().unwrap().attributes.clear(),
                    }
                }
                prefix(&mut c);
                c.input.push(points(">\0x")).unwrap();
                c.input.finish();
                let attributes = storage(&c, kind).0.to_vec();
                let mut e = Execution::new(SOURCE, c);
                let mut expected = snapshot(&e);
                assert_eq!(e.step(), Ok(Progress::Continue(State::Data)));
                expected.state = State::Data;
                expected.current = Some((
                    Input::CodePoint('>'.into()),
                    Position { line: 2, column: 1 },
                ));
                expected.kind = None;
                match kind {
                    TagKind::Start => {
                        expected.start = None;
                        expected.last = Some(points("start"));
                        expected.tokens.push(Token::StartTag {
                            name: points("start"),
                            attributes,
                            self_closing: flag,
                        });
                    }
                    TagKind::End => {
                        expected.end = None;
                        if !attributes.is_empty() {
                            expected.errors.push(error("end-tag-with-attributes", 2, 1));
                        }
                        if flag {
                            expected
                                .errors
                                .push(error("end-tag-with-trailing-solidus", 2, 1));
                        }
                        expected.tokens.push(Token::EndTag {
                            name: points("end"),
                            attributes,
                            self_closing: flag,
                        });
                    }
                }
                assert_eq!(snapshot(&e), expected);
                // Delimiter consumed once. Data, not this source, diagnoses/emits NUL.
                assert_eq!(e.step(), Ok(Progress::Continue(State::Data)));
                assert_eq!(
                    e.context().tokens.last(),
                    Some(&Token::Character('\0'.into()))
                );
                assert_eq!(
                    e.context().errors.last(),
                    Some(&error("unexpected-null-character", 2, 2))
                );
                assert_eq!(e.step(), Ok(Progress::Continue(State::Data)));
                assert_eq!(
                    e.context().tokens.last(),
                    Some(&Token::Character('x'.into()))
                );
                assert_eq!(e.step(), Ok(Progress::Finished));
                let terminal = snapshot(&e);
                assert_eq!(e.step(), Ok(Progress::Finished));
                assert_eq!(snapshot(&e), terminal);
                assert_eq!(e.context().tokens.last(), Some(&Token::Eof));
            }
        }
    }
}

#[test]
fn open_exhaustion_is_effect_free_and_eof_abandons_only_selected_tag() {
    for kind in [TagKind::Start, TagKind::End] {
        for flag in [false, true] {
            for with_prefix in [false, true] {
                let mut c = seeded(kind, flag);
                if with_prefix {
                    prefix(&mut c);
                }
                let mut e = Execution::new(SOURCE, c);
                let mut expected = snapshot(&e);
                for _ in 0..2 {
                    e.context_mut().input.push([]).unwrap();
                    assert_eq!(e.step(), Ok(Progress::NeedMoreInput));
                    assert_eq!(snapshot(&e), expected);
                }
                e.context_mut().input.finish();
                assert_eq!(e.step(), Ok(Progress::Finished));
                let line = if with_prefix { 2 } else { 1 };
                expected.current = Some((Input::Eof, Position { line, column: 1 }));
                expected.kind = None;
                match kind {
                    TagKind::Start => expected.start = None,
                    TagKind::End => expected.end = None,
                }
                expected.errors.push(error("eof-in-tag", line, 1));
                expected.tokens.push(Token::Eof);
                assert_eq!(snapshot(&e), expected); // no emission-only end-tag errors
                for _ in 0..2 {
                    assert_eq!(e.step(), Ok(Progress::Finished));
                    assert_eq!(snapshot(&e), expected);
                }
            }
        }
    }
}

#[test]
fn missing_whitespace_precedes_attribute_creation_and_name_errors_without_reprocessing() {
    for value in [
        0, 0x22, 0x27, 0x3c, 0x41, 0x26, 0x0b, 0xa0, 0xc9, 0xd800, 0xdfff, 0xfdd0, 0x10ffff,
    ] {
        for kind in [TagKind::Start, TagKind::End] {
            let point = CodePoint::new(value).unwrap();
            let mut c = seeded(kind, false);
            prefix(&mut c);
            c.input.push([point, '='.into()]).unwrap();
            c.input.finish();
            let old_start = c.pending_start_tag.clone();
            let old_end = c.pending_end_tag.clone();
            let mut e = Execution::new(SOURCE, c);
            assert_eq!(e.step(), Ok(Progress::Continue(State::BeforeAttributeName)));
            assert_eq!(
                e.context().errors,
                [
                    error("prior", 1, 1),
                    error("missing-whitespace-between-attributes", 2, 1)
                ]
            );
            assert_eq!(e.context().pending_start_tag, old_start);
            assert_eq!(e.context().pending_end_tag, old_end);
            let input_errors = preprocessing(value)
                .map(|code| error(code, 2, 1))
                .into_iter()
                .collect::<Vec<_>>();
            assert_eq!(e.context().input.errors(), input_errors);
            let current = e.context().input.current();
            assert_eq!(e.step(), Ok(Progress::Continue(State::AttributeName)));
            assert_eq!(e.context().input.current(), current);
            let (attrs, detached) = storage(e.context(), kind);
            assert_eq!(attrs, [retained(), vec![Attribute::default()]].concat());
            assert!(detached.is_none()); // starting a new attribute clears OLD detached current
            assert_eq!(e.context().errors.len(), 2);
            assert_eq!(e.step(), Ok(Progress::Continue(State::AttributeName)));
            assert_eq!(e.context().input.current(), current);
            let name = match value {
                0 => '\u{fffd}'.into(),
                0x41 => 'a'.into(),
                _ => point,
            };
            let expected_attr = Attribute {
                name: vec![name],
                value: vec![],
            };
            assert_eq!(
                storage(e.context(), kind).0,
                [retained(), vec![expected_attr]].concat()
            );
            let name_error = match value {
                0 => Some("unexpected-null-character"),
                0x22 | 0x27 | 0x3c => Some("unexpected-character-in-attribute-name"),
                _ => None,
            };
            let mut errors = vec![
                error("prior", 1, 1),
                error("missing-whitespace-between-attributes", 2, 1),
            ];
            if let Some(code) = name_error {
                errors.push(error(code, 2, 1));
            }
            assert_eq!(e.context().errors, errors);
            assert_eq!(e.context().input.errors(), input_errors);
            assert_eq!(
                e.step(),
                Ok(Progress::Continue(State::BeforeAttributeValue))
            );
            assert_eq!(
                e.context().input.position(),
                Position { line: 2, column: 2 }
            );
            assert_eq!(e.context().errors, errors);
            assert_eq!(e.context().input.errors(), input_errors);
            assert_eq!(e.context().tokens, [Token::Character('p'.into())]);
            match kind {
                TagKind::Start => assert_eq!(e.context().pending_end_tag, old_end),
                TagKind::End => assert_eq!(e.context().pending_start_tag, old_start),
            }
            assert_before_value_eof(&mut e, kind);
        }
    }
}

// EOF is reconsumed by Before value; only unquoted value abandons the tag.
fn assert_before_value_eof(e: &mut Execution, kind: TagKind) {
    let mut expected = snapshot(e);
    assert_eq!(
        e.step(),
        Ok(Progress::Continue(State::AttributeValueUnquoted))
    );
    let position = e.context().input.position();
    expected.state = State::AttributeValueUnquoted;
    expected.current = Some((Input::Eof, position));
    assert_eq!(snapshot(e), expected);
    expected.kind = None;
    match kind {
        TagKind::Start => expected.start = None,
        TagKind::End => expected.end = None,
    }
    expected.errors.push(ParseError {
        code: "eof-in-tag",
        position,
    });
    expected.tokens.push(Token::Eof);
    for _ in 0..3 {
        assert_eq!(e.step(), Ok(Progress::Finished));
        assert_eq!(snapshot(e), expected);
    }
}

fn assert_terminal_stub(e: &mut Execution, state: State) {
    let before = snapshot(e);
    let blocked = Err(UnimplementedState { state });
    assert_eq!(e.step(), blocked);
    assert_eq!(snapshot(e), before);
    // Closing input does not clear a terminal block.
    e.context_mut().input.finish();
    for _ in 0..2 {
        assert_eq!(e.step(), blocked);
        assert_eq!(snapshot(e), before);
    }
}

#[test]
fn later_name_input_diagnostics_are_not_source_errors() {
    for kind in [TagKind::Start, TagKind::End] {
        let mut c = seeded(kind, false);
        prefix(&mut c);
        c.input
            .push([
                'A'.into(),
                '\0'.into(),
                CodePoint::new(0xd800).unwrap(),
                '='.into(),
            ])
            .unwrap();
        c.input.finish();
        let mut e = Execution::new(SOURCE, c);
        assert_eq!(e.step(), Ok(Progress::Continue(State::BeforeAttributeName)));
        let source_errors = vec![
            error("prior", 1, 1),
            error("missing-whitespace-between-attributes", 2, 1),
        ];
        assert_eq!(e.context().errors, source_errors);
        assert!(e.context().input.errors().is_empty());
        assert_eq!(e.step(), Ok(Progress::Continue(State::AttributeName)));
        assert_eq!(e.step(), Ok(Progress::Continue(State::AttributeName))); // reconsumed A
        assert_eq!(e.context().errors, source_errors);
        assert!(e.context().input.errors().is_empty());
        assert_eq!(e.step(), Ok(Progress::Continue(State::AttributeName))); // next NUL
        let errors = [
            source_errors,
            vec![error("unexpected-null-character", 2, 2)],
        ]
        .concat();
        assert_eq!(e.context().errors, errors);
        assert!(e.context().input.errors().is_empty());
        assert_eq!(e.step(), Ok(Progress::Continue(State::AttributeName))); // next surrogate
        assert_eq!(e.context().errors, errors);
        assert_eq!(
            e.context().input.errors(),
            [error("surrogate-in-input-stream", 2, 3)]
        );
        assert_eq!(
            storage(e.context(), kind).0.last().unwrap().name,
            vec![
                'a'.into(),
                '\u{fffd}'.into(),
                CodePoint::new(0xd800).unwrap()
            ]
        );
        assert_eq!(
            e.step(),
            Ok(Progress::Continue(State::BeforeAttributeValue))
        );
        assert_eq!(e.context().errors, errors);
        assert_eq!(
            e.context().input.errors(),
            [error("surrogate-in-input-stream", 2, 3)]
        );
        assert_before_value_eof(&mut e, kind);
    }
}

#[test]
fn equals_recovery_and_new_duplicate_keep_retained_values_untouched() {
    for kind in [TagKind::Start, TagKind::End] {
        for text in ["=x=", "same="] {
            let mut c = seeded(kind, false);
            c.input.push(points(text)).unwrap();
            c.input.finish();
            let mut e = Execution::new(SOURCE, c);
            assert_eq!(e.step(), Ok(Progress::Continue(State::BeforeAttributeName)));
            assert_eq!(
                e.context().errors.last(),
                Some(&error("missing-whitespace-between-attributes", 1, 1))
            );
            assert_eq!(e.step(), Ok(Progress::Continue(State::AttributeName)));
            assert!(storage(e.context(), kind).1.is_none());
            let steps = if text == "=x=" { 1 } else { 4 };
            for _ in 0..steps {
                assert_eq!(e.step(), Ok(Progress::Continue(State::AttributeName)));
            }
            assert_eq!(
                e.step(),
                Ok(Progress::Continue(State::BeforeAttributeValue))
            );
            let (attrs, detached) = storage(e.context(), kind);
            let mut errors = vec![
                error("prior", 1, 1),
                error("missing-whitespace-between-attributes", 1, 1),
            ];
            if text == "same=" {
                assert_eq!(attrs, retained());
                assert_eq!(detached, &Some(attribute("same", "")));
                errors.push(error("duplicate-attribute", 1, 5));
            } else {
                assert_eq!(attrs, [retained(), vec![attribute("=x", "")]].concat());
                assert!(detached.is_none());
                errors.push(error("unexpected-equals-sign-before-attribute-name", 1, 1));
            }
            assert_eq!(e.context().errors, errors);
            assert_before_value_eof(&mut e, kind);
        }
    }
}

// Excludes suspension snapshots, but verifies suspension and repeated terminal
// steps are effect-free. Every consuming/reconsuming source step is compared.
fn drain(
    e: &mut Execution,
    trace: &mut Vec<(Result<Progress, UnimplementedState>, Snapshot)>,
) -> bool {
    for _ in 0..200 {
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
    panic!("execution did not consume or terminate");
}

#[test]
fn seeded_destination_paths_complete_or_block_without_accepting_prefixes() {
    for kind in [TagKind::Start, TagKind::End] {
        for (text, target) in [
            (" >", None),
            ("/ >", None),
            (" next>", None),
            (" same>", None),
            ("X=", None),
            ("=x=", None),
            (">&", Some(State::CharacterReference)),
            ("/><!", Some(State::MarkupDeclarationOpen)),
            ("/", None),
            ("/>", None),
            (">", None),
            ("\r\n", None),
        ] {
            let mut c = seeded(kind, false);
            c.input.push(points(text)).unwrap();
            c.input.finish();
            let mut e = Execution::new(SOURCE, c);
            let mut expected = snapshot(&e);
            assert!(drain(&mut e, &mut vec![]));
            if let Some(state) = target {
                assert_terminal_stub(&mut e, state);
                assert!(!e.context().tokens.contains(&Token::Eof));
            } else {
                let eof = if text == "\r\n" {
                    Position { line: 2, column: 1 }
                } else {
                    Position {
                        line: 1,
                        column: text.chars().count() + 1,
                    }
                };
                expected.current = Some((Input::Eof, eof));
                expected.kind = None;
                let value_eof = text == "X=" || text == "=x=";
                let abandoned = text == "/" || text == "\r\n" || value_eof;
                expected.state = if text == "/" {
                    State::SelfClosingStartTag
                } else if value_eof {
                    State::AttributeValueUnquoted
                } else if abandoned {
                    State::AfterAttributeName
                } else {
                    State::Data
                };
                if value_eof {
                    expected
                        .errors
                        .push(error("missing-whitespace-between-attributes", 1, 1));
                    if text == "=x=" {
                        expected.errors.push(error(
                            "unexpected-equals-sign-before-attribute-name",
                            1,
                            1,
                        ));
                    }
                }
                let mut attributes = retained();
                if text == " next>" {
                    attributes.push(attribute("next", ""));
                }
                if text == "/ >" {
                    expected
                        .errors
                        .push(error("unexpected-solidus-in-tag", 1, 2));
                }
                if text == " same>" {
                    expected.errors.push(error("duplicate-attribute", 1, 6));
                }
                let (name, token) = match kind {
                    TagKind::Start => {
                        let tag = expected.start.take().unwrap();
                        let name = tag.name;
                        let token = Token::StartTag {
                            name: name.clone(),
                            attributes,
                            self_closing: text == "/>",
                        };
                        (name, token)
                    }
                    TagKind::End => {
                        let tag = expected.end.take().unwrap();
                        let name = tag.name;
                        let token = Token::EndTag {
                            name: name.clone(),
                            attributes,
                            self_closing: text == "/>",
                        };
                        (name, token)
                    }
                };
                if abandoned {
                    expected.errors.push(ParseError {
                        code: "eof-in-tag",
                        position: eof,
                    });
                } else {
                    let column = text.chars().count();
                    if kind == TagKind::Start {
                        expected.last = Some(name);
                    } else {
                        expected
                            .errors
                            .push(error("end-tag-with-attributes", 1, column));
                        if text == "/>" {
                            expected
                                .errors
                                .push(error("end-tag-with-trailing-solidus", 1, column));
                        }
                    }
                    expected.tokens.push(token);
                }
                expected.tokens.push(Token::Eof);
                for _ in 0..3 {
                    assert_eq!(e.step(), Ok(Progress::Finished));
                    assert_eq!(snapshot(&e), expected, "{kind:?} {text:?}");
                }
            }
            if text == "/>" {
                assert!(matches!(
                    e.context().tokens[1],
                    Token::StartTag {
                        self_closing: true,
                        ..
                    } | Token::EndTag {
                        self_closing: true,
                        ..
                    }
                ));
                assert!(
                    !e.context()
                        .errors
                        .iter()
                        .any(|e| e.code == "missing-whitespace-between-attributes"
                            || e.code == "unexpected-solidus-in-tag")
                );
            }
        }
    }
    // Data entry now reaches the quoted states and compares complete tokens.
    for text in ["<a x='v'>", "</a x=\"v\">", "<p/><a x='v'>"] {
        let mut c = Context::default();
        c.input.push(points(text)).unwrap();
        c.input.finish();
        let mut e = Execution::new(State::Data, c);
        assert!(drain(&mut e, &mut vec![]));
        assert_eq!(e.step(), Ok(Progress::Finished));
        let token = if text.starts_with("</") {
            Token::EndTag {
                name: points("a"),
                attributes: vec![attribute("x", "v")],
                self_closing: false,
            }
        } else {
            Token::StartTag {
                name: points("a"),
                attributes: vec![attribute("x", "v")],
                self_closing: false,
            }
        };
        let mut tokens = vec![];
        if text.starts_with("<p/>") {
            tokens.push(Token::StartTag {
                name: points("p"),
                attributes: vec![],
                self_closing: true,
            });
        }
        tokens.extend([token, Token::Eof]);
        assert_eq!(e.context().tokens, tokens);
        assert_eq!(
            e.context().errors,
            if text.starts_with("</") {
                vec![error("end-tag-with-attributes", 1, 10)]
            } else {
                vec![]
            }
        );
    }
}

#[test]
fn all_two_chunk_splits_and_single_point_chunks_match_closed_source_step_traces() {
    let mut samples = [
        "",
        ">",
        ">\0😀\r\n",
        "/>",
        "/",
        " >",
        " next>",
        " same>",
        "\t\n\u{c} x=",
        "X=",
        "=x=",
        "same=",
        "\0=",
        "\"=",
        "'x=",
        "<=",
        "\u{b}=",
        "\u{a0}=",
        "\r\n>",
        "\r\r\n>",
        "\r",
        "/\0>",
        "/ >",
        ">&",
        "/><!",
        ">x<a z=",
        ">x<a/\r\n>",
    ]
    .map(points)
    .to_vec();
    for value in [0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x01] {
        samples.push(vec![CodePoint::new(value).unwrap(), '='.into()]);
        samples.push(vec![
            '>'.into(),
            CodePoint::new(value).unwrap(),
            '\r'.into(),
        ]);
    }
    for kind in [TagKind::Start, TagKind::End] {
        for input in &samples {
            let mut c = seeded(kind, false);
            c.input.push(input.iter().copied()).unwrap();
            c.input.finish();
            let mut closed = Execution::new(SOURCE, c);
            let mut expected = vec![];
            assert!(drain(&mut closed, &mut expected));
            let mut chunkings = (0..=input.len())
                .map(|split| vec![&input[..split], &input[split..]])
                .collect::<Vec<_>>();
            chunkings.push(input.chunks(1).collect());
            for chunks in chunkings {
                let mut e = Execution::new(SOURCE, seeded(kind, false));
                let mut trace = vec![];
                let mut terminal = false;
                for chunk in chunks {
                    if terminal {
                        break;
                    } // NEVER reconstruct/resume a blocked run
                    e.context_mut().input.push([]).unwrap();
                    e.context_mut().input.push(chunk.iter().copied()).unwrap();
                    terminal = drain(&mut e, &mut trace);
                }
                if !terminal {
                    e.context_mut().input.finish();
                    assert!(drain(&mut e, &mut trace));
                }
                assert_eq!(trace, expected, "{kind:?}, {input:?}");
            }
        }
    }
}
