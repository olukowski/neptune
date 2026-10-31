//! RAWTEXT-local production-adapter evidence, not upstream conformance cases.

use crate::support::*;
use neptune_html::tokenizer::State;
use serde_json::json;

#[test]
fn rawtext_mapping_is_explicit_and_independent_while_data_remains_the_default() {
    let case = json!({
        "description": "independent RAWTEXT runs leave ampersands literal",
        "input": "&amp;>/style>", "output": [["Character", "&amp;>/style>"]],
        "initialStates": ["RAWTEXT state", "Data state", "RAWTEXT state"],
        "lastStartTag": "style"
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
        ]
    );
    let mut default = case;
    default.as_object_mut().unwrap().remove("initialStates");
    assert_eq!(initial_states(&default), ["Data state"]);
    assert_eq!(
        run_with(&default, &initial_states(&default)[0], None, execute),
        Outcome::Blocked(Blocked::Unimplemented(State::CharacterReference))
    );
    let style = json!({
        "description": "no tree-builder mode selection on style",
        "input": "<style>&amp;", "output": [["StartTag", "style", {}], ["Character", "&amp;"]]
    });
    assert_eq!(
        run_with(&style, "Data state", None, execute),
        Outcome::Blocked(Blocked::Unimplemented(State::CharacterReference))
    );
}

#[test]
fn rawtext_adapter_compares_complete_output_and_exact_preprocessing_and_null_diagnostics() {
    let case = json!({
        "description": "replacement, CRLF and actual non-scalar points through adapter",
        "doubleEscaped": true, "input": "\\u0000\r\n\\uD800\\uDFFF&",
        "output": [["Character", "\\uFFFD\n\\uD800\\uDFFF&"]],
        "errors": [
            {"code": "unexpected-null-character", "line": 1, "col": 1},
            {"code": "surrogate-in-input-stream", "line": 2, "col": 1},
            {"code": "surrogate-in-input-stream", "line": 2, "col": 2}
        ]
    });
    assert_eq!(
        run_with(&case, "RAWTEXT state", None, execute),
        Outcome::Passed
    );
    let mut wrong = case.clone();
    wrong["output"] = json!([["Character", "\\u0000\n\\uD800\\uDFFF&"]]);
    assert!(matches!(
        run_with(&wrong, "RAWTEXT state", None, execute),
        Outcome::Failed(_)
    ));
    wrong = case;
    wrong["errors"][1]["col"] = json!(2);
    assert!(matches!(
        run_with(&wrong, "RAWTEXT state", None, execute),
        Outcome::Failed(_)
    ));
    let empty = json!({"description": "empty RAWTEXT", "input": "", "output": []});
    assert_eq!(
        run_with(&empty, "RAWTEXT state", None, execute),
        Outcome::Passed
    );
}

#[test]
fn rawtext_matching_prefix_before_end_tag_name_is_terminally_blocked_not_passed() {
    for input in ["&amp;</style>", "&amp;</A"] {
        let case = json!({
            "description": "matching output prefix cannot pass at a stub",
            "input": input, "output": [["Character", "&amp;"]], "lastStartTag": "style"
        });
        assert_eq!(
            run_with(&case, "RAWTEXT state", None, execute),
            Outcome::Blocked(Blocked::Unimplemented(State::RawtextEndTagName))
        );
    }
}
