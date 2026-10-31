//! Local adapter/support regressions, not newly executable conformance cases.
use crate::support::*;
use neptune_html::tokenizer::{
    Attribute, CodePoint, PendingEndTag, PendingStartTag, State, TagKind, Token,
};
use serde_json::json;

fn attribute(name: &str, value: &str) -> Attribute {
    Attribute {
        name: decode(name, false),
        value: decode(value, false),
    }
}

#[test]
fn attribute_object_comparison_is_order_independent_but_exact_and_never_hides_duplicates() {
    let case = json!({"description":"synthetic attributes", "input":"",
        "output":[["StartTag","p",{"z":"", "id":"first", "é":"字", "":""},true]]});
    let attributes = vec![
        attribute("é", "字"),
        attribute("z", ""),
        attribute("id", "first"),
        attribute("", ""),
    ];
    let compare = |attributes| {
        run_with(&case, "Data state", None, |_| Completion::Complete {
            tokens: vec![
                Token::StartTag {
                    name: decode("p", false),
                    attributes,
                    self_closing: true,
                },
                Token::Eof,
            ],
            errors: vec![],
        })
    };
    assert_eq!(compare(attributes.clone()), Outcome::Passed);
    let mut reversed = attributes.clone();
    reversed.reverse();
    assert_eq!(compare(reversed), Outcome::Passed);
    let mut missing = attributes.clone();
    missing.pop();
    let mut extra = attributes.clone();
    extra.push(attribute("extra", ""));
    let mut name = attributes.clone();
    name[2].name = decode("ID", false); // No adapter case folding.
    let mut value = attributes.clone();
    value[2].value = decode("different", false);
    let mut duplicate = attributes.clone();
    duplicate.push(attribute("id", "first")); // Even equal duplicates must fail.
    let mut different_duplicate = attributes.clone();
    different_duplicate.push(attribute("id", "second"));
    for wrong in [missing, extra, name, value, duplicate, different_duplicate] {
        assert!(matches!(compare(wrong), Outcome::Failed(_)));
    }
    let tokens = vec![Token::StartTag {
        name: decode("p", false),
        attributes,
        self_closing: true,
    }];
    for wrong in [
        tokens.clone(),
        [tokens.clone(), vec![Token::Eof, Token::Eof]].concat(),
        [vec![Token::Eof], tokens].concat(),
    ] {
        assert!(matches!(
            run_with(&case, "Data state", None, |_| Completion::Complete {
                tokens: wrong,
                errors: vec![],
            }),
            Outcome::Failed(_)
        ));
    }
}

#[test]
fn initialized_start_emission_compares_double_escaped_attribute_names_and_values_literally() {
    let case = json!({"description":"initialized support, not an attribute-state path", "input":">",
        "doubleEscaped":true,
        "output":[["\\u0053tartTag","p",{"\\u0061\\uD800":"\\uD800\\uDFFF字", "é":"\\u0000\\uFFFF😀"}]]});
    let attributes = vec![
        Attribute {
            name: vec!['é'.into()],
            value: vec!['\0'.into(), CodePoint::new(0xffff).unwrap(), '😀'.into()],
        },
        Attribute {
            name: vec!['a'.into(), CodePoint::new(0xd800).unwrap()],
            value: vec![
                CodePoint::new(0xd800).unwrap(),
                CodePoint::new(0xdfff).unwrap(),
                '字'.into(),
            ],
        },
    ];
    let compare = |case: &serde_json::Value| {
        run_with(case, "Data state", None, |mut run| {
            run.initial = State::TagName;
            run.context.current_tag = Some(TagKind::Start);
            run.context.pending_start_tag = Some(PendingStartTag {
                name: decode("p", false),
                attributes: attributes.clone(),
                // Already detached duplicates are never emitted or compared.
                discarded_current_attribute: Some(attribute("é", "wrong")),
                ..PendingStartTag::default()
            });
            execute(run)
        })
    };
    assert_eq!(compare(&case), Outcome::Passed);
    let mut wrong = case.clone();
    wrong["output"] = json!([["StartTag","p",{"a\\uD800":"\\uFFFD字", "é":"\\u0000\\uFFFF😀"}]]);
    assert!(matches!(compare(&wrong), Outcome::Failed(_)));
    let literal = json!({"description":"not double escaped", "input":"",
        "output":[["StartTag","p",{"\\u0061":"\\u0041"}]]});
    assert_eq!(
        run_with(&literal, "Data state", None, |_| Completion::Complete {
            tokens: vec![
                Token::StartTag {
                    name: decode("p", false),
                    attributes: vec![attribute("\\u0061", "\\u0041")],
                    self_closing: false
                },
                Token::Eof
            ],
            errors: vec![],
        }),
        Outcome::Passed
    );
}

#[test]
fn decoded_attribute_key_collisions_block_explicitly_before_execution() {
    // The fixture object format cannot represent duplicate decoded keys
    // unambiguously. Neither silently keep one nor pretend it was an HTML duplicate.
    let case = json!({"description":"ambiguous decoded object", "input":"", "doubleEscaped":true,
        "output":[["StartTag","p",{"a":"first", "\\u0061":"second"}]]});
    assert_eq!(
        run_with(&case, "Data state", None, |_| panic!("ambiguous fixture")),
        Outcome::Blocked(Blocked::FixtureCapability)
    );
}

#[test]
fn end_tag_name_only_projection_keeps_both_emission_diagnostics_exact() {
    let case = json!({"description":"initialized end-tag support", "input":">",
        "output":[["EndTag","p"]], "errors":[
            {"code":"end-tag-with-attributes","line":1,"col":1},
            {"code":"end-tag-with-trailing-solidus","line":1,"col":1}]});
    let completion = |mut run: Run| {
        run.initial = State::SelfClosingStartTag;
        run.context.current_tag = Some(TagKind::End);
        run.context.pending_end_tag = Some(PendingEndTag {
            name: decode("p", false),
            attributes: vec![attribute("id", "first"), attribute("class", "second")],
            discarded_current_attribute: Some(attribute("id", "discarded")),
            ..PendingEndTag::default()
        });
        execute(run)
    };
    assert_eq!(
        run_with(&case, "Data state", None, completion),
        Outcome::Passed
    );
    // Production fields project away only because upstream EndTag is name-only;
    // required errors, including exact location/multiplicity, never project away.
    for errors in [
        vec![Diagnostic::from(&diagnostic(
            "end-tag-with-attributes",
            1,
            1,
        ))],
        vec![Diagnostic::from(&diagnostic(
            "end-tag-with-trailing-solidus",
            1,
            1,
        ))],
        vec![
            Diagnostic::from(&diagnostic("end-tag-with-attributes", 1, 2)),
            Diagnostic::from(&diagnostic("end-tag-with-trailing-solidus", 1, 1)),
        ],
        vec![
            Diagnostic::from(&diagnostic("end-tag-with-attributes", 1, 1)),
            Diagnostic::from(&diagnostic("end-tag-with-trailing-solidus", 2, 1)),
        ],
        vec![
            Diagnostic::from(&diagnostic("end-tag-with-attributes", 1, 1)),
            Diagnostic::from(&diagnostic("end-tag-with-attributes", 1, 1)),
            Diagnostic::from(&diagnostic("end-tag-with-trailing-solidus", 1, 1)),
        ],
        vec![],
    ] {
        assert!(matches!(
            run_with(&case, "Data state", None, |run| {
                let Completion::Complete { tokens, .. } = completion(run) else {
                    panic!("local support must finish")
                };
                Completion::Complete { tokens, errors }
            }),
            Outcome::Failed(_)
        ));
    }
    let mut reversed = case.clone();
    reversed["errors"].as_array_mut().unwrap().reverse();
    assert_eq!(
        run_with(&reversed, "Data state", None, completion),
        Outcome::Passed
    );
}

#[test]
fn expected_attributes_do_not_hide_real_destination_stubs() {
    let boolean = json!({"description":"reachable boolean attribute", "input":"<p id>",
        "output":[["StartTag","p",{"id":""}]]});
    assert_eq!(
        run_with(&boolean, "Data state", None, execute),
        Outcome::Passed
    );
    let mut wrong = boolean;
    wrong["output"] = json!([["StartTag","p",{"id":"x"}]]);
    assert!(matches!(
        run_with(&wrong, "Data state", None, execute),
        Outcome::Failed(_)
    ));
    // These exact former stub inputs now require real complete values.
    for input in ["<p id =x>", "<p id=x>", "<p id='x'>", "<p id=\"x\">"] {
        let case = json!({"description":"reachable attribute value", "input":input,
            "output":[["StartTag","p",{"id":"x"}]]});
        assert_eq!(
            run_with(&case, "Data state", None, execute),
            Outcome::Passed
        );
        let mut wrong = case;
        wrong["output"] = json!([["StartTag","p",{"id":""}]]);
        assert!(matches!(
            run_with(&wrong, "Data state", None, execute),
            Outcome::Failed(_)
        ));
    }
    for (input, destination) in [
        ("<p id =&>", State::CharacterReference),
        ("<p id=x&>", State::CharacterReference),
    ] {
        let case = json!({"description":"attribute adapter is not a state implementation", "input":input,
            "output":[["StartTag","p",{"id":"x"}]]});
        assert_eq!(
            run_with(&case, "Data state", None, execute),
            Outcome::Blocked(Blocked::Unimplemented(destination))
        );
    }
}
