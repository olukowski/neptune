//! Partial upstream execution report, distinct from fixture inventory.
#[path = "support/attribute_emission.rs"]
mod attribute_emission;
#[path = "support/rawtext.rs"]
mod rawtext;
#[path = "support/rcdata.rs"]
mod rcdata;
#[path = "support/regression.rs"]
mod regression;
mod support;

use std::{collections::BTreeMap, fs, path::Path};

use neptune_html::tokenizer::{CodePoint, Input, State, Token, UnimplementedState};
use serde_json::json;
use support::*;

// Keep availability failures loud even before any run can be compared.
fn fixture_paths(directory: &Path) -> Vec<std::path::PathBuf> {
    let mut paths: Vec<_> = fs::read_dir(directory)
        .expect("missing fixtures: run git submodule update --init --recursive")
        .map(|entry| entry.expect("fixture directory entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "test")
        })
        .collect();
    paths.sort();
    assert!(!paths.is_empty(), "no upstream tokenizer fixtures found");
    paths
}

#[test]
fn pinned_upstream_runner() {
    let directory =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/html5lib-tests/tokenizer");
    let paths = fixture_paths(&directory);
    assert_eq!(paths.len(), 14, "pinned fixture file inventory");
    let baseline = regression::passing_baseline();
    let mut outcomes = BTreeMap::new();
    let (mut passed, mut failed, mut blocked, mut cases, mut attempted) = (0, 0, 0, 0, 0);
    let mut reasons = BTreeMap::<String, usize>::new();
    for path in paths {
        let document: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).expect("read fixture")).expect("fixture JSON");
        let (group, policy) = if document.get("xmlViolationTests").is_some() {
            ("xmlViolationTests", Some(Blocked::XmlCoercion))
        } else {
            (
                "tests",
                if path.file_name().unwrap() == "pendingSpecChanges.test" {
                    Some(Blocked::PendingSpecReview)
                } else {
                    None
                },
            )
        };
        for (case_index, case) in document[group]
            .as_array()
            .expect("fixture test array")
            .iter()
            .enumerate()
        {
            cases += 1;
            for (state_index, state) in initial_states(case).into_iter().enumerate() {
                let id = (
                    path.file_name().unwrap().to_str().unwrap().to_owned(),
                    group.to_owned(),
                    case_index,
                    case["description"]
                        .as_str()
                        .expect("fixture description")
                        .to_owned(),
                    state.clone(),
                    state_index,
                );
                let result = run_with(case, &state, policy, |run| {
                    attempted += 1;
                    execute(run)
                });
                match &result {
                    Outcome::Passed => passed += 1,
                    Outcome::Failed(message) => {
                        failed += 1;
                        eprintln!("{id:?}: {message}");
                    }
                    Outcome::Blocked(reason) => {
                        blocked += 1;
                        *reasons.entry(format!("{reason:?}")).or_default() += 1;
                    }
                }
                assert!(
                    outcomes.insert(id, result).is_none(),
                    "duplicate run identity"
                );
            }
        }
    }
    eprintln!(
        "Upstream: {cases} fixture cases; {} runs; {attempted} dispatch-attempted; {} executed (= passed + failed); {passed} passed; {failed} failed; {blocked} blocked.\nBlocked reasons: {reasons:#?}",
        passed + failed + blocked,
        passed + failed
    );
    assert_eq!(cases, 6827, "pinned fixture case inventory");
    assert!(
        attempted > 0,
        "must reach production dispatch, not just inventory"
    );
    assert_eq!(
        failed, 0,
        "every comparable mismatch fails, including newly admitted runs"
    );
    regression::check_previous_passes(&baseline, &outcomes)
        .unwrap_or_else(|message| panic!("{message}"));
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
#[should_panic(expected = "missing fixtures: run git submodule update --init --recursive")]
fn synthetic_missing_fixtures_fail_with_setup_instructions() {
    // An executable is a file, never a fixture directory. No fixture is moved.
    fixture_paths(&std::env::current_exe().unwrap().join("missing-fixtures"));
}

#[test]
fn synthetic_malformed_known_fixture_shapes_fail_loudly() {
    let good = json!({"description":"synthetic", "input":"", "output":[]});
    for (field, value) in [
        ("input", json!(false)),
        ("output", json!({})),
        ("doubleEscaped", json!("true")),
        ("lastStartTag", json!(false)),
        ("errors", json!({})),
        ("errors", json!([{"code":"x","line":0,"col":1}])),
        ("output", json!([["Character", 1]])),
        ("output", json!([["EndTag", "p", {}]])),
        ("output", json!([["StartTag", "p", [], false]])),
        ("output", json!([["StartTag", "p", {"id":false}]])),
        ("output", json!([["StartTag", "p", {}, "true"]])),
    ] {
        let mut malformed = good.clone();
        malformed[field] = value;
        assert!(
            std::panic::catch_unwind(|| run_with(&malformed, "Data state", None, |_| {
                Completion::Complete {
                    tokens: vec![Token::Eof],
                    errors: vec![],
                }
            }))
            .is_err(),
            "{malformed}"
        );
    }
    for states in [json!([]), json!({}), json!([1])] {
        assert!(
            std::panic::catch_unwind(|| initial_states(&json!({"initialStates": states}))).is_err()
        );
    }
}

#[test]
fn production_data_output_prefix_never_turns_a_destination_stub_into_a_pass() {
    // Local adapter regression tests, not additional upstream conformance cases.
    for (input, output, errors, destination) in [
        ("p&", "p", json!([]), State::CharacterReference),
        ("p<a x =&", "p", json!([]), State::CharacterReference),
        ("p<!", "p", json!([]), State::MarkupDeclarationOpen),
        (
            "p</A/x =&",
            "p",
            json!([{"code":"unexpected-solidus-in-tag", "line":1, "col":6}]),
            State::CharacterReference,
        ),
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
fn production_after_attribute_name_completion_and_abandonment_compare_full_results() {
    // These formerly blocked inputs now require complete tokens and diagnostics.
    // Local adapter checks, not additional upstream conformance runs.
    for case in [
        json!({"description":"value-entry EOF", "input":"p<a x =",
            "output":[["Character","p"]],
            "errors":[{"code":"eof-in-tag","line":1,"col":8}]}),
        json!({"description":"end-tag value-entry EOF", "input":"p</A/x =",
            "output":[["Character","p"]],
            "errors":[{"code":"unexpected-solidus-in-tag","line":1,"col":6},
                {"code":"eof-in-tag","line":1,"col":9}]}),
        json!({"description":"whitespace-only unfinished tag", "input":"p<a ",
            "output":[["Character","p"]],
            "errors":[{"code":"eof-in-tag","line":1,"col":5}]}),
        json!({"description":"unfinished end-tag recovery", "input":"p</A/x",
            "output":[["Character","p"]],
            "errors":[{"code":"unexpected-solidus-in-tag","line":1,"col":6},
                {"code":"eof-in-tag","line":1,"col":7}]}),
        json!({"description":"emitted prefix and abandoned tag", "input":"<p><A ",
            "output":[["StartTag","p",{}]],
            "errors":[{"code":"eof-in-tag","line":1,"col":7}]}),
        json!({"description":"emitted prefix and boolean recovery", "input":"<p/><a/\0>",
            "output":[["StartTag","p",{},true],["StartTag","a",{"\u{fffd}":""}]],
            "errors":[{"code":"unexpected-solidus-in-tag","line":1,"col":8},
                {"code":"unexpected-null-character","line":1,"col":8}]}),
    ] {
        assert_eq!(
            run_with(&case, "Data state", None, execute),
            Outcome::Passed
        );
        let mut wrong_output = case.clone();
        wrong_output["output"] = json!([]);
        assert!(matches!(
            run_with(&wrong_output, "Data state", None, execute),
            Outcome::Failed(_)
        ));
        let mut missing_error = case;
        missing_error["errors"].as_array_mut().unwrap().pop();
        assert!(matches!(
            run_with(&missing_error, "Data state", None, execute),
            Outcome::Failed(_)
        ));
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
            Outcome::Blocked(Blocked::Unimplemented(State::RcdataLessThanSign)),
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
    // EOF after '=' now completes by abandonment in unquoted value.
    let mut prefix = json!({"description":"tag prefix then abandoned value", "input":"<p><A x =",
        "output":[["StartTag","p",{}]],
        "errors":[{"code":"eof-in-tag","line":1,"col":10}]});
    assert_eq!(
        run_with(&prefix, "Data state", None, execute),
        Outcome::Passed
    );
    prefix["errors"] = json!([]);
    assert!(matches!(
        run_with(&prefix, "Data state", None, execute),
        Outcome::Failed(_)
    ));
    // A matching prefix still cannot turn Character reference into a pass.
    prefix["input"] = json!("<p><A x =&");
    assert_eq!(
        run_with(&prefix, "Data state", None, execute),
        Outcome::Blocked(Blocked::Unimplemented(State::CharacterReference))
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
fn production_self_closing_comparison_checks_start_flags_and_end_tag_diagnostics() {
    // Local adapter tests, not additional upstream conformance runs.
    let case = json!({"description":"flagged start", "input":"<br/>",
        "output":[["StartTag","br",{},true]]});
    assert_eq!(
        run_with(&case, "Data state", None, execute),
        Outcome::Passed
    );
    let mut wrong = case.clone();
    wrong["output"] = json!([["StartTag", "br", {}]]);
    assert!(matches!(
        run_with(&wrong, "Data state", None, execute),
        Outcome::Failed(_)
    ));
    wrong = case;
    wrong["input"] = json!("<br>");
    assert!(matches!(
        run_with(&wrong, "Data state", None, execute),
        Outcome::Failed(_)
    ));
    let mut end = json!({"description":"flagged end", "input":"</br/>",
        "output":[["EndTag","br"]],
        "errors":[{"code":"end-tag-with-trailing-solidus","line":1,"col":6}]});
    assert_eq!(run_with(&end, "Data state", None, execute), Outcome::Passed);
    end["errors"] = json!([]);
    assert!(matches!(
        run_with(&end, "Data state", None, execute),
        Outcome::Failed(_)
    ));
    let mut prefix = json!({"description":"flag prefix then abandoned value", "input":"<p/><a/\0 =",
        "output":[["StartTag","p",{},true]],
        "errors":[{"code":"unexpected-solidus-in-tag","line":1,"col":8},
            {"code":"unexpected-null-character","line":1,"col":8},
            {"code":"eof-in-tag","line":1,"col":11}]});
    assert_eq!(
        run_with(&prefix, "Data state", None, execute),
        Outcome::Passed
    );
    prefix["errors"].as_array_mut().unwrap().pop();
    assert!(matches!(
        run_with(&prefix, "Data state", None, execute),
        Outcome::Failed(_)
    ));
    prefix["input"] = json!("<p/><a/\0 =&");
    assert_eq!(
        run_with(&prefix, "Data state", None, execute),
        Outcome::Blocked(Blocked::Unimplemented(State::CharacterReference))
    );
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
        json!(["DOCTYPE", null, "", "id", false]),
        json!(["ProcessingInstruction", "x"]),
    ] {
        let mut case = case.clone();
        // Unsupported output anywhere blocks the whole run, even after a
        // supported attribute-bearing token. Never compare a partial prefix.
        case["output"] = json!([["StartTag", "p", {"id":"x"}], token]);
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
