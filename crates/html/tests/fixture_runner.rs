//! Partial upstream execution report, distinct from fixture inventory.
mod support;

use std::{collections::BTreeMap, fs, path::Path};

use neptune_html::tokenizer::{CodePoint, Input, State, Token, UnimplementedState};
use serde_json::json;
use support::*;

#[test]
fn pinned_upstream_runner_reports_bogus_comment_coverage() {
    let directory =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/html5lib-tests/tokenizer");
    let mut paths: Vec<_> = fs::read_dir(directory)
        .expect("missing fixtures: run git submodule update --init --recursive")
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "test")
        })
        .collect();
    paths.sort();
    assert_eq!(paths.len(), 14);
    let (mut passed, mut failed, mut blocked, mut cases, mut attempted) = (0, 0, 0, 0, 0);
    let mut reasons = BTreeMap::<String, usize>::new();
    let mut passed_by_initial = BTreeMap::<String, usize>::new();
    let mut retained_data_passes = 0;
    let mut retained_pre_comment_passes = 0;
    let mut comment_passes = BTreeMap::new();
    // Explicit newly comparable upstream identities, not inferred from dispatch.
    const BOGUS_COMMENT_PASSES: &[(&str, &str)] = &[
        ("test2.test", "Illegal end tag name"),
        ("test3.test", r"</\u0000"),
        ("test3.test", r"</\u0009"),
        ("test3.test", r"</\u000A"),
        ("test3.test", r"</\u000B"),
        ("test3.test", r"</\u000C"),
        ("test3.test", "</ "),
        ("test3.test", r"</ \u0000"),
        ("test3.test", "</!"),
        ("test3.test", "</\""),
        ("test3.test", "</&"),
        ("test3.test", "</'"),
        ("test3.test", "</-"),
        ("test3.test", "<//"),
        ("test3.test", "</0"),
        ("test3.test", "</1"),
        ("test3.test", "</9"),
        ("test3.test", "</<"),
        ("test3.test", "</="),
        ("test3.test", "</?"),
        ("test3.test", "</@"),
        ("test3.test", "</["),
        ("test3.test", "</`"),
        ("test3.test", "</{"),
        ("test3.test", r"</\uDBC0\uDC00"),
    ];
    let mut newly_comparable_tag_passes = 0;
    let mut retained_tag_open_passes = 0;
    let mut former_tag_open = BTreeMap::<String, usize>::new();
    // Explicit case identities from the previous five End-tag-open blocks.
    // Independent of production dispatch; fixtures and policies are unchanged.
    const FORMER_END_TAG_OPEN: &[(&str, &str)] = &[
        ("test1.test", "Empty end tag"),
        ("test2.test", "Unescaped </"),
        ("test2.test", "Empty end tag with following characters"),
        ("test3.test", "</"),
        ("test3.test", "</>"),
    ];
    // Explicit previous Tag-name-blocked cases: only EOF-in-name can finish.
    const FORMER_TAG_NAME: &[(&str, &str, Option<State>)] = &[
        (
            "test4.test",
            "CR EOF in tag name",
            Some(State::BeforeAttributeName),
        ),
        (
            "test4.test",
            "Slash EOF in tag name",
            Some(State::SelfClosingStartTag),
        ),
        ("test4.test", "EOF in tag name state ", None),
        (
            "test4.test",
            "EOF in before attribute name state",
            Some(State::BeforeAttributeName),
        ),
        (
            "test4.test",
            "EOF in attribute name state",
            Some(State::BeforeAttributeName),
        ),
        (
            "test4.test",
            "EOF in after attribute name state",
            Some(State::BeforeAttributeName),
        ),
        (
            "test4.test",
            "EOF in before attribute value state",
            Some(State::BeforeAttributeName),
        ),
        (
            "test4.test",
            "EOF in attribute value (double quoted) state",
            Some(State::BeforeAttributeName),
        ),
        (
            "test4.test",
            "EOF in attribute value (single quoted) state",
            Some(State::BeforeAttributeName),
        ),
        (
            "test4.test",
            "EOF in attribute value (unquoted) state",
            Some(State::BeforeAttributeName),
        ),
        (
            "test4.test",
            "EOF in after attribute value state",
            Some(State::BeforeAttributeName),
        ),
    ];
    let mut former_tag_name_seen = 0;
    let mut newly_passed = BTreeMap::new();
    for path in paths {
        let document: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let (tests, policy) = if let Some(tests) = document.get("xmlViolationTests") {
            (tests, Some(Blocked::XmlCoercion))
        } else {
            (
                &document["tests"],
                if path.file_name().unwrap() == "pendingSpecChanges.test" {
                    Some(Blocked::PendingSpecReview)
                } else {
                    None
                },
            )
        };
        for case in tests.as_array().unwrap() {
            cases += 1;
            for state in initial_states(case) {
                let result = run_with(case, &state, policy, |run| {
                    attempted += 1;
                    execute(run)
                });
                // Independently identify the previous Data baseline: complete Data
                // runs never reached '&' or '<'; formerly Tag-open-blocked runs
                // reached '<' before any '&'. Do not infer support from dispatch.
                let input = decode(
                    case["input"].as_str().unwrap(),
                    case.get("doubleEscaped")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                );
                let first_transition = input.iter().find(|p| matches!(p.value(), 0x26 | 0x3c));
                let adapter_supported = policy.is_none()
                    && case["output"].as_array().unwrap().iter().all(|token| {
                        decode(
                            token[0].as_str().unwrap(),
                            case.get("doubleEscaped")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false),
                        ) == decode("Character", false)
                    });
                if state == "Data state" && adapter_supported {
                    let identity = (
                        path.file_name().unwrap().to_str().unwrap(),
                        case["description"].as_str().unwrap(),
                    );
                    if FORMER_END_TAG_OPEN.contains(&identity) {
                        assert_eq!(result, Outcome::Passed, "{identity:?}");
                        *newly_passed
                            .entry((identity.0.to_owned(), identity.1.to_owned()))
                            .or_insert(0) += 1;
                    } else if let Some((_, _, destination)) = FORMER_TAG_NAME
                        .iter()
                        .find(|(file, description, _)| (*file, *description) == identity)
                    {
                        let expected = destination
                            .map(|state| Outcome::Blocked(Blocked::Unimplemented(state)))
                            .unwrap_or(Outcome::Passed);
                        assert_eq!(result, expected, "{identity:?}");
                        former_tag_name_seen += 1;
                    } else if first_transition.is_some_and(|point| point.value() == 0x3c) {
                        retained_tag_open_passes += usize::from(result == Outcome::Passed);
                    }
                    if first_transition.is_none() {
                        retained_data_passes += usize::from(result == Outcome::Passed);
                    } else if first_transition.is_some_and(|point| point.value() == 0x3c) {
                        let label = match &result {
                            Outcome::Passed => "Passed".into(),
                            Outcome::Blocked(reason) => format!("{reason:?}"),
                            Outcome::Failed(_) => "Failed".into(),
                        };
                        *former_tag_open.entry(label).or_default() += 1;
                    }
                }
                match result {
                    Outcome::Passed => {
                        if case["output"].as_array().unwrap().iter().any(|token| {
                            decode(
                                token[0].as_str().unwrap(),
                                case.get("doubleEscaped")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false),
                            ) == decode("Comment", false)
                        }) {
                            assert_eq!(state, "Data state");
                            *comment_passes
                                .entry((
                                    path.file_name().unwrap().to_str().unwrap().to_owned(),
                                    case["description"].as_str().unwrap().to_owned(),
                                ))
                                .or_insert(0) += 1;
                        } else {
                            retained_pre_comment_passes += 1;
                        }
                        if case["output"].as_array().unwrap().iter().any(|token| {
                            let kind = decode(
                                token[0].as_str().unwrap(),
                                case.get("doubleEscaped")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false),
                            );
                            kind == decode("StartTag", false) || kind == decode("EndTag", false)
                        }) {
                            newly_comparable_tag_passes += 1;
                        }
                        passed += 1;
                        *passed_by_initial.entry(state.clone()).or_default() += 1;
                    }
                    Outcome::Failed(message) => {
                        failed += 1;
                        eprintln!(
                            "{} / {} / {state}: {message}",
                            path.display(),
                            case["description"]
                        );
                    }
                    Outcome::Blocked(reason) => {
                        blocked += 1;
                        *reasons.entry(format!("{reason:?}")).or_default() += 1;
                    }
                }
            }
        }
    }
    eprintln!(
        "Upstream: {cases} fixture cases; {} runs; {attempted} dispatch-attempted; {} executed (= passed + failed); {passed} passed; {failed} failed; {blocked} blocked.\nBlocked reasons: {reasons:#?}",
        passed + failed + blocked,
        passed + failed
    );
    assert_eq!(cases, 6827);
    assert!(
        attempted > 0,
        "must reach production dispatch, not just inventory"
    );
    assert_eq!((passed + failed + blocked, attempted), (7053, 5818));
    // Review deliberately when changing support; never silently widen blocking.
    assert_eq!(
        reasons,
        BTreeMap::from([
            ("PendingSpecReview".into(), 1),
            ("TokenVariants".into(), 1230),
            ("Unimplemented(BeforeAttributeName)".into(), 24),
            ("Unimplemented(CdataSection)".into(), 56),
            ("Unimplemented(CharacterReference)".into(), 4659),
            ("Unimplemented(MarkupDeclarationOpen)".into(), 240),
            ("Unimplemented(ProcessingInstructionOpen)".into(), 52),
            ("Unimplemented(Rawtext)".into(), 71),
            ("Unimplemented(Rcdata)".into(), 74),
            ("Unimplemented(ScriptData)".into(), 89),
            ("Unimplemented(SelfClosingStartTag)".into(), 6),
            ("XmlCoercion".into(), 4),
        ])
    );
    assert_eq!(
        (passed, failed, blocked),
        (547, 0, 6506),
        "Data through Tag name plus Bogus comment baseline"
    );
    assert_eq!(
        passed_by_initial,
        BTreeMap::from([("Data state".into(), 495), ("PLAINTEXT state".into(), 52)]),
        "retain prior coverage; 25 comment-output Data-initial runs newly compare"
    );
    assert_eq!(retained_pre_comment_passes, 522);
    assert_eq!(
        comment_passes,
        BOGUS_COMMENT_PASSES
            .iter()
            .map(|(file, description)| ((file.to_string(), description.to_string()), 1))
            .collect()
    );
    assert_eq!(retained_data_passes, 385);
    assert_eq!(retained_tag_open_passes, 24);
    assert_eq!(former_tag_name_seen, 11);
    assert_eq!(newly_comparable_tag_passes, 55);
    assert_eq!(
        newly_passed,
        FORMER_END_TAG_OPEN
            .iter()
            .map(|(file, description)| ((file.to_string(), description.to_string()), 1))
            .collect()
    );
    assert_eq!(
        former_tag_open,
        BTreeMap::from([
            ("Passed".into(), 30),
            ("Unimplemented(BeforeAttributeName)".into(), 9),
            ("Unimplemented(CharacterReference)".into(), 1),
            ("Unimplemented(ProcessingInstructionOpen)".into(), 25),
            ("Unimplemented(SelfClosingStartTag)".into(), 1),
        ]),
        "account for all 66 formerly Tag-open-blocked runs"
    );
    assert!(
        !reasons.contains_key("FixtureCapability"),
        "review new fixture fields"
    );
    assert!(
        !reasons.contains_key("InitialState"),
        "review new initial states"
    );
}

#[test]
fn production_data_output_prefix_never_turns_a_destination_stub_into_a_pass() {
    // Local adapter regression tests, not additional upstream conformance cases.
    for (input, output, errors, destination) in [
        ("p&", "p", json!([]), State::CharacterReference),
        ("p<a ", "p", json!([]), State::BeforeAttributeName),
        ("p<!", "p", json!([]), State::MarkupDeclarationOpen),
        ("p</A/", "p", json!([]), State::SelfClosingStartTag),
        (
            "p</0>&",
            "p",
            json!([{"code":"invalid-first-character-of-tag-name", "line":1, "col":4}]),
            State::CharacterReference,
        ),
        ("p<?", "p", json!([]), State::ProcessingInstructionOpen),
        (
            "p<0&",
            "p<0",
            json!([{"code":"invalid-first-character-of-tag-name", "line":1, "col":3}]),
            State::CharacterReference,
        ),
        (
            "p\0&",
            "p\0",
            json!([{"code":"unexpected-null-character", "line":1, "col":2}]),
            State::CharacterReference,
        ),
    ] {
        let case = json!({
            "description": "local prefix would match if blocked execution were compared",
            "input": input, "output": [["Character", output]], "errors": errors
        });
        assert_eq!(
            run_with(&case, "Data state", None, execute),
            Outcome::Blocked(Blocked::Unimplemented(destination))
        );
    }
}

#[test]
fn production_plaintext_mapping_multiple_runs_and_data_default_remain_explicit() {
    // Local adapter regressions, not additional upstream conformance cases.
    let case = json!({
        "description": "same literal output, independent initial-state runs",
        "input": "<plaintext>&amp;", "output": [["Character", "<plaintext>&amp;"]],
        "initialStates": ["PLAINTEXT state", "Data state", "PLAINTEXT state", "RCDATA state"],
        "lastStartTag": "plaintext"
    });
    let outcomes: Vec<_> = initial_states(&case)
        .iter()
        .map(|state| run_with(&case, state, None, execute))
        .collect();
    assert_eq!(
        outcomes,
        [
            Outcome::Passed,
            Outcome::Blocked(Blocked::Unimplemented(State::CharacterReference)),
            Outcome::Passed,
            Outcome::Blocked(Blocked::Unimplemented(State::Rcdata)),
        ]
    );
    let mut default = case.clone();
    default.as_object_mut().unwrap().remove("initialStates");
    assert_eq!(initial_states(&default), ["Data state"]);
    assert_eq!(
        run_with(&default, &initial_states(&default)[0], None, execute),
        Outcome::Blocked(Blocked::Unimplemented(State::CharacterReference))
    );

    let null = json!({
        "description": "PLAINTEXT replacement and preprocessing through production adapter",
        "input": "\\u0000\r\n\\uD800&<", "doubleEscaped": true,
        "output": [["Character", "\\uFFFD\n\\uD800&<"]],
        "errors": [
            {"code": "unexpected-null-character", "line": 1, "col": 1},
            {"code": "surrogate-in-input-stream", "line": 2, "col": 1}
        ]
    });
    assert_eq!(
        run_with(&null, "PLAINTEXT state", None, execute),
        Outcome::Passed
    );
    let empty = json!({"description": "PLAINTEXT empty", "input": "", "output": []});
    assert_eq!(
        run_with(&empty, "PLAINTEXT state", None, execute),
        Outcome::Passed
    );
}

#[test]
fn synthetic_executor_validates_defaults_multiple_runs_context_and_double_escaping() {
    let case = json!({
        "description": "synthetic, not conformance",
        "input": "x\\uD800\\uDFFF\r\ny",
        "doubleEscaped": true,
        "lastStartTag": "title",
        "initialStates": ["Data state", "RCDATA state"],
        "output": [["\\u0043haracter", "\\uD800"], ["Character", "\\uDFFF"]]
    });
    let states = initial_states(&case);
    assert_eq!(states, ["Data state", "RCDATA state"]);
    for (state, expected) in states.iter().zip([State::Data, State::Rcdata]) {
        assert_eq!(
            run_with(&case, state, None, |mut run| {
                assert_eq!(run.initial, expected);
                assert_eq!(run.context.last_start_tag, Some(decode("title", false)));
                let mut points = Vec::new();
                loop {
                    match run.context.input.consume() {
                        Input::CodePoint(point) => points.push(point.value()),
                        Input::Eof => break,
                        Input::NeedMoreInput => panic!("fixture is closed"),
                    }
                }
                assert_eq!(points, [0x78, 0xd800, 0xdfff, 0x0a, 0x79]);
                assert_eq!(run.context.input.errors().len(), 2);
                // Deliberately arbitrary output from an independent test executor,
                // not a production state handler or an echo of expected tokens.
                Completion::Complete {
                    tokens: vec![
                        Token::Character(CodePoint::new(0xd800).unwrap()),
                        Token::Character(CodePoint::new(0xdfff).unwrap()),
                        Token::Eof,
                    ],
                    errors: vec![],
                }
            }),
            Outcome::Passed
        );
    }
    assert_eq!(initial_states(&json!({})), ["Data state"]);
    assert_eq!(
        decode("\\u0041\\uD800\\uZZZZ\\u12", true),
        ['A'.into(), CodePoint::new(0xd800).unwrap()]
            .into_iter()
            .chain("\\uZZZZ\\u12".chars().map(CodePoint::from))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        decode("\\u0041", false),
        "\\u0041".chars().map(CodePoint::from).collect::<Vec<_>>()
    );
}

#[test]
fn synthetic_comparison_requires_exact_tokens_errors_locations_and_completion() {
    let case = json!({"description":"synthetic", "input":"", "output":[["Character", "ab"]],
        "errors":[{"code":"synthetic", "line":2,"col":3}]});
    let good_tokens = vec![
        Token::Character('a'.into()),
        Token::Character('b'.into()),
        Token::Eof,
    ];
    let good_errors = vec![Diagnostic::from(&diagnostic("synthetic", 2, 3))];
    let compare = |tokens, errors| {
        run_with(&case, "Data state", None, |_| Completion::Complete {
            tokens,
            errors,
        })
    };
    assert_eq!(
        compare(good_tokens.clone(), good_errors.clone()),
        Outcome::Passed
    );
    for (tokens, errors) in [
        (vec![Token::Eof], good_errors.clone()),
        (
            vec![
                Token::Character('b'.into()),
                Token::Character('a'.into()),
                Token::Eof,
            ],
            good_errors.clone(),
        ),
        (good_tokens[..2].to_vec(), good_errors.clone()),
        (vec![Token::Eof, Token::Eof], good_errors.clone()),
        (good_tokens.clone(), vec![]),
        (
            good_tokens.clone(),
            vec![Diagnostic::from(&diagnostic("wrong", 2, 3))],
        ),
        (
            good_tokens.clone(),
            vec![Diagnostic::from(&diagnostic("synthetic", 2, 4))],
        ),
        (
            good_tokens.clone(),
            vec![Diagnostic::from(&diagnostic("synthetic", 1, 3))],
        ),
        (
            good_tokens,
            [good_errors.clone(), good_errors.clone()].concat(),
        ),
    ] {
        assert!(matches!(compare(tokens, errors), Outcome::Failed(_)));
    }
    assert!(matches!(
        run_with(&case, "Data state", None, |_| Completion::Incomplete),
        Outcome::Failed(_)
    ));
    assert_eq!(
        run_with(&case, "Data state", None, |_| Completion::Blocked(
            UnimplementedState {
                state: State::TagOpen
            }
        )),
        Outcome::Blocked(Blocked::Unimplemented(State::TagOpen))
    );
    let empty = json!({"description":"synthetic empty", "input":"", "output":[]});
    assert_eq!(
        run_with(&empty, "Data state", None, |_| Completion::Complete {
            tokens: vec![Token::Eof],
            errors: vec![]
        }),
        Outcome::Passed
    );
}

#[test]
fn production_tag_comparison_checks_kind_name_order_and_double_escaped_code_points() {
    // Local adapter tests, not extra upstream conformance runs.
    let case = json!({"description":"simple tags", "input":"<DIV>xy</DIV>",
        "output":[["StartTag","div",{}],["Character","xy"],["EndTag","div"]]});
    assert_eq!(
        run_with(&case, "Data state", None, execute),
        Outcome::Passed
    );
    for output in [
        json!([
            ["StartTag", "DIV", {}],
            ["Character", "xy"],
            ["EndTag", "div"]
        ]),
        json!([["EndTag", "div"], ["Character", "xy"], ["EndTag", "div"]]),
        json!([
            ["StartTag", "div", {}],
            ["EndTag", "div"],
            ["Character", "xy"]
        ]),
        json!([["StartTag", "div", {}], ["Character", "xy"]]),
        json!([
            ["StartTag", "div", {}],
            ["Character", "xy"],
            ["EndTag", "DIV"]
        ]),
    ] {
        let mut wrong = case.clone();
        wrong["output"] = output;
        assert!(matches!(
            run_with(&wrong, "Data state", None, execute),
            Outcome::Failed(_)
        ));
    }
    let escaped = json!({"description":"tag code points, not UTF-16 pair folding",
        "doubleEscaped":true, "input":"<A\\uD800\\uDFFF>",
        "output":[["\\u0053tartTag","a\\uD800\\uDFFF",{}]],
        "errors":[{"code":"surrogate-in-input-stream","line":1,"col":3},
            {"code":"surrogate-in-input-stream","line":1,"col":4}]});
    assert_eq!(
        run_with(&escaped, "Data state", None, execute),
        Outcome::Passed
    );
    // A matching emitted tag prefix still cannot turn an attribute-state block
    // into a pass, and an EOF-abandoned tag must never compare as emitted output.
    let prefix = json!({"description":"tag prefix", "input":"<p><A ",
        "output":[["StartTag","p",{}]]});
    assert_eq!(
        run_with(&prefix, "Data state", None, execute),
        Outcome::Blocked(Blocked::Unimplemented(State::BeforeAttributeName))
    );
    let abandoned = json!({"description":"abandoned tag", "input":"<A",
        "output":[["StartTag","a",{}]],
        "errors":[{"code":"eof-in-tag","line":1,"col":3}]});
    assert!(matches!(
        run_with(&abandoned, "Data state", None, execute),
        Outcome::Failed(_)
    ));
}

#[test]
fn production_comment_comparison_checks_data_kind_order_boundaries_and_double_escaping() {
    // Local adapter tests, not additional upstream conformance runs.
    let case = json!({"description":"comment recovery and EOF", "input":"</0>t</1",
        "output":[["Comment","0"],["Character","t"],["Comment","1"]],
        "errors":[{"code":"invalid-first-character-of-tag-name","line":1,"col":3},
            {"code":"invalid-first-character-of-tag-name","line":1,"col":8}]});
    assert_eq!(
        run_with(&case, "Data state", None, execute),
        Outcome::Passed
    );
    for output in [
        json!([["Comment", "wrong"], ["Character", "t"], ["Comment", "1"]]),
        json!([["Character", "0t"], ["Comment", "1"]]),
        json!([["Comment", "0"], ["Comment", "1"], ["Character", "t"]]),
        json!([["Comment", "01"], ["Character", "t"]]),
        json!([["Comment", "0"], ["Character", "t"]]),
    ] {
        let mut wrong = case.clone();
        wrong["output"] = output;
        assert!(matches!(
            run_with(&wrong, "Data state", None, execute),
            Outcome::Failed(_)
        ));
    }
    let escaped = json!({"description":"comment code points, not scalar strings",
        "doubleEscaped":true, "input":"</\\uD800\\uDFFF\\u0000>",
        "output":[["\\u0043omment","\\uD800\\uDFFF\\uFFFD"]],
        "errors":[{"code":"invalid-first-character-of-tag-name","line":1,"col":3},
            {"code":"surrogate-in-input-stream","line":1,"col":3},
            {"code":"surrogate-in-input-stream","line":1,"col":4},
            {"code":"unexpected-null-character","line":1,"col":5}]});
    assert_eq!(
        run_with(&escaped, "Data state", None, execute),
        Outcome::Passed
    );
    let empty = json!({"description":"synthetic empty comment boundaries", "input":"",
        "output":[["Comment",""],["Comment",""]]});
    let compare = |tokens| {
        run_with(&empty, "Data state", None, |_| Completion::Complete {
            tokens,
            errors: vec![],
        })
    };
    assert_eq!(
        compare(vec![
            Token::Comment { data: vec![] },
            Token::Comment { data: vec![] },
            Token::Eof
        ]),
        Outcome::Passed
    );
    assert!(matches!(
        compare(vec![Token::Comment { data: vec![] }, Token::Eof]),
        Outcome::Failed(_)
    ));
    // An exact comment prefix is still blocked if the next algorithm is missing.
    let prefix = json!({"description":"comment prefix then stub", "input":"</0><!",
        "output":[["Comment","0"]],
        "errors":[{"code":"invalid-first-character-of-tag-name","line":1,"col":3}]});
    assert_eq!(
        run_with(&prefix, "Data state", None, execute),
        Outcome::Blocked(Blocked::Unimplemented(State::MarkupDeclarationOpen))
    );
}

#[test]
#[should_panic(expected = "Comment token shape")]
fn comment_adapter_rejects_extra_fields_instead_of_dropping_them() {
    let case =
        json!({"description":"malformed known token", "input":"", "output":[["Comment","x",true]]});
    run_with(&case, "Data state", None, |_| {
        panic!("must reject before execution")
    });
}

#[test]
fn unsupported_capabilities_never_invoke_executor_or_compare_a_partial_token() {
    let case = json!({"description":"synthetic", "input":"", "output":[["StartTag","p",{},true]]});
    for reason in [Blocked::XmlCoercion, Blocked::PendingSpecReview] {
        assert_eq!(
            run_with(&case, "Data state", Some(reason), |_| panic!(
                "policy blocked"
            )),
            Outcome::Blocked(reason)
        );
    }
    for token in [
        json!(["StartTag", "p", {}, true]),
        json!(["StartTag", "p", {"id":"x"}]),
        json!(["DOCTYPE", null, "", "id", false]),
        json!(["ProcessingInstruction", "x"]),
    ] {
        let mut case = case.clone();
        case["output"] = json!([token]);
        assert_eq!(
            run_with(&case, "Data state", None, |_| panic!("unsupported fields")),
            Outcome::Blocked(Blocked::TokenVariants)
        );
    }
    assert_eq!(
        run_with(&case, "unknown state", None, |_| panic!("unknown state")),
        Outcome::Blocked(Blocked::InitialState)
    );
    let mut unknown = case;
    unknown["futureCapability"] = json!(true);
    assert_eq!(
        run_with(&unknown, "Data state", None, |_| panic!("unknown field")),
        Outcome::Blocked(Blocked::FixtureCapability)
    );
}
