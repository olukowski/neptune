//! Partial upstream execution report, distinct from fixture inventory.
mod support;

use std::{collections::BTreeMap, fs, path::Path};

use neptune_html::tokenizer::{CodePoint, Input, State, Token, UnimplementedState};
use serde_json::json;
use support::*;

#[test]
fn pinned_upstream_runner_reports_data_and_plaintext_coverage() {
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
                match result {
                    Outcome::Passed => {
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
    assert_eq!((passed + failed + blocked, attempted), (7053, 5423));
    // Review deliberately when changing support; never silently widen blocking.
    assert_eq!(
        reasons,
        BTreeMap::from([
            ("PendingSpecReview".into(), 1),
            ("TokenVariants".into(), 1625),
            ("Unimplemented(CdataSection)".into(), 56),
            ("Unimplemented(CharacterReference)".into(), 4658),
            ("Unimplemented(Rawtext)".into(), 61),
            ("Unimplemented(Rcdata)".into(), 63),
            ("Unimplemented(ScriptData)".into(), 82),
            ("Unimplemented(TagOpen)".into(), 66),
            ("XmlCoercion".into(), 4),
        ])
    );
    assert_eq!(
        (passed, failed, blocked),
        (437, 0, 6616),
        "Data and PLAINTEXT baseline"
    );
    assert_eq!(
        passed_by_initial,
        BTreeMap::from([("Data state".into(), 385), ("PLAINTEXT state".into(), 52)]),
        "retain Data coverage; only PLAINTEXT is newly comparable"
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
        ("p<", "p", json!([]), State::TagOpen),
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
            Outcome::Blocked(Blocked::Unimplemented(State::TagOpen)),
            Outcome::Passed,
            Outcome::Blocked(Blocked::Unimplemented(State::Rcdata)),
        ]
    );
    let mut default = case.clone();
    default.as_object_mut().unwrap().remove("initialStates");
    assert_eq!(initial_states(&default), ["Data state"]);
    assert_eq!(
        run_with(&default, &initial_states(&default)[0], None, execute),
        Outcome::Blocked(Blocked::Unimplemented(State::TagOpen))
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
        json!(["EndTag", "p"]),
        json!(["DOCTYPE", null, "", "id", false]),
        json!(["Comment", "x"]),
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
