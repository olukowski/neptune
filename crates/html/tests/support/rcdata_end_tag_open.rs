//! Local integrated production-adapter checks, not upstream conformance runs.

use crate::support::*;
use serde_json::json;

#[test]
fn rcdata_end_tag_open_adapter_requires_complete_eof_and_non_alpha_fallback() {
    for input in ["p</", "</0>", "</>", "<//>", "</ XMP>", "</\r\n!"] {
        let normalized = input.replace("\r\n", "\n");
        let case = json!({"description":"local reachable end tag open fallback",
            "input":input, "output":[["Character",normalized]], "lastStartTag":"xmp"});
        assert_eq!(
            run_with(&case, "RCDATA state", None, execute),
            Outcome::Passed
        );
        let mut wrong = case.clone();
        wrong["output"] = json!([["Character", ""]]);
        assert!(matches!(
            run_with(&wrong, "RCDATA state", None, execute),
            Outcome::Failed(_)
        ));
        // No missing-name/EOF-before-name parse error belongs to this state.
        wrong = case;
        wrong["errors"] = json!([{"code":"eof-before-tag-name","line":1,"col":3}]);
        assert!(matches!(
            run_with(&wrong, "RCDATA state", None, execute),
            Outcome::Failed(_)
        ));
    }
}

#[test]
fn rcdata_end_tag_open_adapter_preserves_reconsumed_diagnostics_exactly_once() {
    let case = json!({"description":"local non-scalar and NUL fallback",
        "input":"</\\uD800\\u0000\r\n</\\uFDD0</\\u000B", "doubleEscaped":true,
        "output":[["Character","</\\uD800\\uFFFD\n</\\uFDD0</\\u000B"]],
        "errors":[{"code":"surrogate-in-input-stream","line":1,"col":3},
            {"code":"unexpected-null-character","line":1,"col":4},
            {"code":"noncharacter-in-input-stream","line":2,"col":3},
            {"code":"control-character-in-input-stream","line":2,"col":6}]});
    assert_eq!(
        run_with(&case, "RCDATA state", None, execute),
        Outcome::Passed
    );
    let mut wrong = case.clone();
    wrong["errors"].as_array_mut().unwrap().pop();
    assert!(matches!(
        run_with(&wrong, "RCDATA state", None, execute),
        Outcome::Failed(_)
    ));
    wrong = case;
    let duplicate = wrong["errors"][0].clone();
    wrong["errors"].as_array_mut().unwrap().push(duplicate);
    assert!(matches!(
        run_with(&wrong, "RCDATA state", None, execute),
        Outcome::Failed(_)
    ));
}

#[test]
fn rcdata_end_tag_open_adapter_now_requires_complete_alpha_paths() {
    for (input, output) in [
        (
            "p</TITLE>",
            json!([["Character", "p"], ["EndTag", "title"]]),
        ),
        ("<!x</foo", json!([["Character", "<!x</foo"]])),
        (
            "</0></title>",
            json!([["Character", "</0>"], ["EndTag", "title"]]),
        ),
    ] {
        let case = json!({"description":"local reachable name-state completion",
            "input":input, "output":output, "lastStartTag":"title"});
        assert_eq!(
            run_with(&case, "RCDATA state", None, execute),
            Outcome::Passed
        );
        let mut wrong = case;
        wrong["output"] = json!([]);
        assert!(matches!(
            run_with(&wrong, "RCDATA state", None, execute),
            Outcome::Failed(_)
        ));
    }
}
