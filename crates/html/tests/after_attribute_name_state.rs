//! Pinned After attribute name branches, not additional upstream conformance runs.
//! https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144232-L144270

use neptune_html::tokenizer::{
    Attribute, CodePoint, Context, Execution, Input, ParseError, PendingComment, PendingEndTag,
    PendingStartTag, Position, Progress, State, TagKind, Token, UnimplementedState,
};

fn points(s: &str) -> Vec<CodePoint> {
    s.chars().map(Into::into).collect()
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
// Seeded entry is isolated evidence, NOT a reachable end-to-end value path.
fn seeded(kind: TagKind, populated: bool, detached: bool) -> Context {
    let attributes = if populated {
        vec![attribute("prior", "retained"), attribute("other", "value")]
    } else {
        vec![]
    };
    let discarded = detached.then(|| attribute("prior", "discarded"));
    Context {
        current_tag: Some(kind),
        pending_start_tag: Some(PendingStartTag {
            name: points("start"),
            attributes: attributes.clone(),
            discarded_current_attribute: discarded.clone(),
            ..Default::default()
        }),
        pending_end_tag: Some(PendingEndTag {
            name: points("end"),
            attributes,
            discarded_current_attribute: discarded,
            ..Default::default()
        }),
        pending_comment: Some(PendingComment {
            data: points("comment"),
        }),
        tokens: vec![Token::Character('p'.into())],
        errors: vec![error("prior", 1, 1)],
        last_start_tag: Some(points("title")),
        return_state: Some(State::Rcdata),
        temporary_buffer: points("scratch"),
        ..Default::default()
    }
}
fn storage(c: &mut Context) -> (&mut Vec<Attribute>, &mut Option<Attribute>) {
    match c.current_tag.unwrap() {
        TagKind::Start => {
            let t = c.pending_start_tag.as_mut().unwrap();
            (&mut t.attributes, &mut t.discarded_current_attribute)
        }
        TagKind::End => {
            let t = c.pending_end_tag.as_mut().unwrap();
            (&mut t.attributes, &mut t.discarded_current_attribute)
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
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
fn execution(mut c: Context, input: Vec<CodePoint>, closed: bool) -> Execution {
    c.input.push(input).unwrap();
    if closed {
        c.input.finish();
    }
    Execution::new(State::AfterAttributeName, c)
}

#[test]
fn whitespace_solidus_equals_consume_without_touching_storage_even_without_current_attribute() {
    for kind in [TagKind::Start, TagKind::End] {
        for populated in [false, true] {
            for detached in [false, true] {
                for closed in [false, true] {
                    for (text, point, target) in [
                        ("\t", '\t', State::AfterAttributeName),
                        ("\n", '\n', State::AfterAttributeName),
                        ("\u{c}", '\u{c}', State::AfterAttributeName),
                        (" ", ' ', State::AfterAttributeName),
                        ("\r", '\n', State::AfterAttributeName),
                        ("\r\n", '\n', State::AfterAttributeName),
                        ("/", '/', State::SelfClosingStartTag),
                        ("=", '=', State::BeforeAttributeValue),
                    ] {
                        let mut e = execution(
                            seeded(kind, populated, detached),
                            points(&format!("{text}Z")),
                            closed,
                        );
                        let mut expected = snapshot(&e);
                        expected.state = target;
                        expected.current = Some((
                            Input::CodePoint(point.into()),
                            Position { line: 1, column: 1 },
                        ));
                        assert_eq!(e.step(), Ok(Progress::Continue(target)));
                        assert_eq!(snapshot(&e), expected);
                        // Source-step inspection: '=' has not consumed the value.
                        // This isolated matrix intentionally includes no current attribute.
                        assert_eq!(
                            e.context_mut().input.consume(),
                            Input::CodePoint('Z'.into())
                        );
                        assert_eq!(
                            e.context().input.position(),
                            Position {
                                line: if point == '\n' { 2 } else { 1 },
                                column: if point == '\n' { 1 } else { 2 }
                            }
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn anything_else_creates_empty_attribute_clears_detached_current_and_reconsumes_literally() {
    let values = (0..=0x7f)
        .filter(|v| !matches!(v, 0x09 | 0x0a | 0x0c | 0x0d | 0x20 | 0x2f | 0x3d | 0x3e))
        .chain([
            0x85, 0xa0, 0x1680, 0x2003, 0x2028, 0x3000, 0xc9, 0x130, 0x1f600, 0xd800, 0xdfff,
            0xfdd0, 0x10ffff,
        ]);
    for value in values {
        let point = CodePoint::new(value).unwrap();
        for kind in [TagKind::Start, TagKind::End] {
            for populated in [false, true] {
                for closed in [false, true] {
                    // With no retained entries, alternate absent-current and
                    // detached-only entry as well as the populated lifecycle.
                    let detached = closed;
                    let mut e = execution(
                        seeded(kind, populated, detached),
                        vec![point, 'Z'.into()],
                        closed,
                    );
                    let mut expected = snapshot(&e);
                    expected.state = State::AttributeName;
                    expected.current =
                        Some((Input::CodePoint(point), Position { line: 1, column: 1 }));
                    let (attrs, discarded) = match kind {
                        TagKind::Start => {
                            let t = expected.start.as_mut().unwrap();
                            (&mut t.attributes, &mut t.discarded_current_attribute)
                        }
                        TagKind::End => {
                            let t = expected.end.as_mut().unwrap();
                            (&mut t.attributes, &mut t.discarded_current_attribute)
                        }
                    };
                    attrs.push(Attribute::default());
                    *discarded = None;
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
                    assert_eq!(snapshot(&e), expected, "{kind:?} U+{value:04X}");
                    // Real destination processes the reconsumed point, not the predecessor.
                    assert_eq!(e.step(), Ok(Progress::Continue(State::AttributeName)));
                    let current = storage(e.context_mut()).0.last().unwrap().clone();
                    let appended = match value {
                        0 => 0xfffd,
                        0x41..=0x5a => value + 0x20,
                        _ => value,
                    };
                    assert_eq!(
                        current,
                        Attribute {
                            name: vec![CodePoint::new(appended).unwrap()],
                            value: vec![]
                        }
                    );
                    let mut errors = expected.errors;
                    match value {
                        0 => errors.push(error("unexpected-null-character", 1, 1)),
                        0x22 | 0x27 | 0x3c => {
                            errors.push(error("unexpected-character-in-attribute-name", 1, 1))
                        }
                        _ => {}
                    }
                    assert_eq!(e.context().errors, errors);
                    assert_eq!(e.context().input.errors(), expected.input_errors);
                    assert_eq!(
                        e.context_mut().input.consume(),
                        Input::CodePoint('Z'.into())
                    );
                }
            }
        }
    }
}

#[test]
fn emission_and_eof_use_selected_tag_helpers_preserving_unrelated_context() {
    for kind in [TagKind::Start, TagKind::End] {
        for populated in [false, true] {
            for detached in [false, true] {
                for self_closing in [false, true] {
                    for eof in [false, true] {
                        let mut c = seeded(kind, populated, detached);
                        match kind {
                            TagKind::Start => {
                                c.pending_start_tag.as_mut().unwrap().self_closing = self_closing
                            }
                            TagKind::End => {
                                c.pending_end_tag.as_mut().unwrap().self_closing = self_closing
                            }
                        }
                        // Normalized positions follow CRLF and a supplementary scalar.
                        c.input.push(points("\r\n😀")).unwrap();
                        assert_eq!(c.input.consume(), Input::CodePoint('\n'.into()));
                        assert_eq!(c.input.consume(), Input::CodePoint('😀'.into()));
                        let mut e = execution(c, points(if eof { "" } else { ">Z" }), true);
                        let mut expected = snapshot(&e);
                        let (name, attrs) = match kind {
                            TagKind::Start => {
                                let t = expected.start.take().unwrap();
                                (t.name, t.attributes)
                            }
                            TagKind::End => {
                                let t = expected.end.take().unwrap();
                                (t.name, t.attributes)
                            }
                        };
                        expected.kind = None;
                        expected.current = Some((
                            if eof {
                                Input::Eof
                            } else {
                                Input::CodePoint('>'.into())
                            },
                            Position { line: 2, column: 2 },
                        ));
                        let result = if eof {
                            expected.errors.push(error("eof-in-tag", 2, 2));
                            expected.tokens.push(Token::Eof);
                            Ok(Progress::Finished)
                        } else {
                            expected.state = State::Data;
                            match kind {
                                TagKind::Start => {
                                    expected.last = Some(name.clone());
                                    expected.tokens.push(Token::StartTag {
                                        name,
                                        attributes: attrs,
                                        self_closing,
                                    });
                                }
                                TagKind::End => {
                                    if populated {
                                        expected.errors.push(error(
                                            "end-tag-with-attributes",
                                            2,
                                            2,
                                        ));
                                    }
                                    if self_closing {
                                        expected.errors.push(error(
                                            "end-tag-with-trailing-solidus",
                                            2,
                                            2,
                                        ));
                                    }
                                    expected.tokens.push(Token::EndTag {
                                        name,
                                        attributes: attrs,
                                        self_closing,
                                    });
                                }
                            }
                            Ok(Progress::Continue(State::Data))
                        };
                        assert_eq!(e.step(), result);
                        assert_eq!(snapshot(&e), expected);
                        if eof {
                            assert_eq!(e.step(), result);
                            assert_eq!(snapshot(&e), expected);
                        } else {
                            assert_eq!(
                                e.context_mut().input.consume(),
                                Input::CodePoint('Z'.into())
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn open_input_suspends_without_finalizing_or_losing_detached_current() {
    for kind in [TagKind::Start, TagKind::End] {
        let mut e = execution(seeded(kind, true, true), vec![], false);
        for text in ["", " \t\r", "\n\u{c} ", ""] {
            e.context_mut().input.push(points(text)).unwrap();
            while e.step() == Ok(Progress::Continue(State::AfterAttributeName)) {}
            let expected = snapshot(&e);
            for _ in 0..3 {
                e.context_mut().input.push([]).unwrap();
                assert_eq!(e.step(), Ok(Progress::NeedMoreInput));
                assert_eq!(snapshot(&e), expected);
            }
        }
        e.context_mut().input.finish();
        assert_eq!(e.step(), Ok(Progress::Finished));
        assert_eq!(e.context().errors.last(), Some(&error("eof-in-tag", 2, 3)));
        assert_eq!(
            e.context().tokens,
            [Token::Character('p'.into()), Token::Eof]
        );
    }
}

fn drain(
    e: &mut Execution,
    trace: &mut Vec<(Result<Progress, UnimplementedState>, Snapshot)>,
    bound: usize,
) -> bool {
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
    panic!("did not terminate or suspend");
}

#[test]
fn real_paths_finish_boolean_attributes_duplicates_and_abandon_unfinished_values() {
    for (input, kind, attrs, codes, abandoned) in [
        ("<DIV >", TagKind::Start, vec![], vec![], false),
        (
            "</DIV />",
            TagKind::End,
            vec![],
            vec!["end-tag-with-trailing-solidus"],
            false,
        ),
        (
            "<DIV A B a>",
            TagKind::Start,
            vec![attribute("a", ""), attribute("b", "")],
            vec!["duplicate-attribute"],
            false,
        ),
        (
            "</DIV A B a>",
            TagKind::End,
            vec![attribute("a", ""), attribute("b", "")],
            vec!["duplicate-attribute", "end-tag-with-attributes"],
            false,
        ),
        (
            "<DIV A A =tail",
            TagKind::Start,
            vec![attribute("a", "")],
            vec!["duplicate-attribute", "eof-in-tag"],
            true,
        ),
        (
            "<DIV /x>",
            TagKind::Start,
            vec![attribute("x", "")],
            vec!["unexpected-solidus-in-tag"],
            false,
        ),
    ] {
        let mut c = Context::default();
        c.input.push(points(input)).unwrap();
        c.input.finish();
        let mut e = Execution::new(State::Data, c);
        let mut trace = vec![];
        assert!(drain(&mut e, &mut trace, input.len() * 4 + 10));
        assert_eq!(
            e.context()
                .errors
                .iter()
                .map(|e| e.code)
                .collect::<Vec<_>>(),
            codes
        );
        if abandoned {
            assert_eq!(e.step(), Ok(Progress::Finished));
            assert_eq!(e.context().tokens, [Token::Eof]);
            assert_eq!(
                e.context().errors,
                [
                    error("duplicate-attribute", 1, 9),
                    error("eof-in-tag", 1, 15),
                ]
            );
            assert_eq!(e.context().current_tag, None);
            assert!(e.context().pending_start_tag.is_none());
        } else {
            assert_eq!(
                e.context().tokens,
                [
                    match kind {
                        TagKind::Start => Token::StartTag {
                            name: points("div"),
                            attributes: attrs,
                            self_closing: false
                        },
                        TagKind::End => Token::EndTag {
                            name: points("div"),
                            attributes: attrs,
                            self_closing: input.contains('/') && input.contains(" />")
                        },
                    },
                    Token::Eof
                ]
            );
            assert_eq!(
                e.context().last_start_tag,
                (kind == TagKind::Start).then(|| points("div"))
            );
        }
    }
}

#[test]
fn all_two_chunk_splits_and_single_points_match_complete_source_step_traces() {
    let mut samples: Vec<_> = [
        "",
        " \t\r\n\u{c} ",
        "\r\r\n",
        "/>",
        "/x>",
        "=tail",
        "A B a>",
        "\0'<=tail",
        ">Z",
        "<DIV >",
        "</DIV />",
        "<DIV A B a>",
        "</DIV A B a/>",
        "<DIV A A =tail",
        "<DIV A\r\nB>",
        "<DIV /\0>",
        "<DIV A",
        "<DIV \r",
    ]
    .map(points)
    .into();
    for value in [0xd800, 0xdfff, 0xfdd0, 0x10ffff, 0x0b, 0xa0, 0xc9] {
        let mut input = points("\r\n");
        input.push(CodePoint::new(value).unwrap());
        input.extend(points(" A>"));
        samples.push(input);
    }
    for input in samples {
        for initial in [State::Data, State::AfterAttributeName] {
            for kind in [TagKind::Start, TagKind::End] {
                let run = |chunks: Vec<&[CodePoint]>, closed: bool| {
                    let c = if initial == State::Data {
                        Context::default()
                    } else {
                        seeded(kind, true, true)
                    };
                    let mut e = Execution::new(initial, c);
                    let mut trace = vec![];
                    if closed {
                        e.context_mut().input.push(input.iter().copied()).unwrap();
                        e.context_mut().input.finish();
                    } else {
                        for chunk in chunks {
                            e.context_mut().input.push(chunk.iter().copied()).unwrap();
                            if drain(&mut e, &mut trace, input.len() * 5 + 10) {
                                return trace;
                            }
                            e.context_mut().input.push([]).unwrap();
                            assert!(!drain(&mut e, &mut trace, 1));
                        }
                        e.context_mut().input.finish();
                    }
                    assert!(drain(&mut e, &mut trace, input.len() * 5 + 10));
                    trace
                };
                let expected = run(vec![], true);
                for split in 0..=input.len() {
                    assert_eq!(
                        run(vec![&input[..split], &input[split..]], false),
                        expected,
                        "{initial:?} {kind:?} split {split}: {input:?}"
                    );
                }
                assert_eq!(run(input.chunks(1).collect(), false), expected);
            }
        }
    }
}
