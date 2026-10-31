//! Integrated RAWTEXT end tag open adapter evidence, not upstream run counts.

use crate::support::*;
use neptune_html::tokenizer::State;
use serde_json::json;

#[test]
fn rawtext_end_tag_open_requires_complete_eof_and_non_alpha_recovery() {
    for (input, output, errors) in [
        ("&amp;</", "&amp;</", json!([])),
        (
            "&amp;</\0",
            "&amp;</\u{fffd}",
            json!([{"code":"unexpected-null-character", "line":1, "col":8}]),
        ),
        ("<x</", "<x</", json!([])),
        (
            "<x</\0",
            "<x</\u{fffd}",
            json!([{"code":"unexpected-null-character", "line":1, "col":5}]),
        ),
        ("</ XMP>", "</ XMP>", json!([])),
        ("</0>", "</0>", json!([])),
        ("</>", "</>", json!([])),
        ("<//>", "<//>", json!([])),
        ("</\r\n!", "</\n!", json!([])),
        ("</<!", "</<!", json!([])),
    ] {
        let case = json!({"description":"local complete RAWTEXT end tag open fallback",
            "input":input, "output":[["Character",output]], "lastStartTag":"xmp", "errors":errors});
        assert_eq!(
            run_with(&case, "RAWTEXT state", None, execute),
            Outcome::Passed
        );
        let mut wrong = case.clone();
        // A formerly matching prefix must now fail comparison, not remain blocked.
        wrong["output"] = json!([[
            "Character",
            if input.starts_with("&amp;") {
                "&amp;"
            } else {
                "<x"
            }
        ]]);
        assert!(matches!(
            run_with(&wrong, "RAWTEXT state", None, execute),
            Outcome::Failed(_)
        ));
        wrong = case;
        wrong["errors"] = json!([{"code":"eof-before-tag-name", "line":1, "col":3}]);
        assert!(matches!(
            run_with(&wrong, "RAWTEXT state", None, execute),
            Outcome::Failed(_)
        ));
    }
}

#[test]
fn rawtext_end_tag_open_reconsumption_preserves_exact_diagnostics_without_duplication() {
    let case = json!({"description":"local non-scalar and NUL fallback",
        "input":"</\\uD800\\u0000\r\n</\\uFDD0</\\u000B", "doubleEscaped":true,
        "output":[["Character","</\\uD800\\uFFFD\n</\\uFDD0</\\u000B"]],
        "errors":[{"code":"surrogate-in-input-stream", "line":1, "col":3},
            {"code":"unexpected-null-character", "line":1, "col":4},
            {"code":"noncharacter-in-input-stream", "line":2, "col":3},
            {"code":"control-character-in-input-stream", "line":2, "col":6}]});
    assert_eq!(
        run_with(&case, "RAWTEXT state", None, execute),
        Outcome::Passed
    );
    let mut wrong = case.clone();
    wrong["errors"].as_array_mut().unwrap().pop();
    assert!(matches!(
        run_with(&wrong, "RAWTEXT state", None, execute),
        Outcome::Failed(_)
    ));
    wrong = case.clone();
    wrong["errors"][2]["col"] = json!(4);
    assert!(matches!(
        run_with(&wrong, "RAWTEXT state", None, execute),
        Outcome::Failed(_)
    ));
    wrong = case;
    let duplicate = wrong["errors"][0].clone();
    wrong["errors"].as_array_mut().unwrap().push(duplicate);
    assert!(matches!(
        run_with(&wrong, "RAWTEXT state", None, execute),
        Outcome::Failed(_)
    ));
}

#[test]
fn rawtext_end_tag_open_alpha_remains_terminal_after_literal_recovery() {
    for (input, output) in [
        ("</0></STYLE>", "</0>"),
        ("</</a", "</"),
        ("</&</A>", "</&"),
    ] {
        let case = json!({"description":"local matching prefix at name-state stub",
            "input":input, "output":[["Character",output]], "lastStartTag":"style"});
        assert_eq!(
            run_with(&case, "RAWTEXT state", None, execute),
            Outcome::Blocked(Blocked::Unimplemented(State::RawtextEndTagName))
        );
    }
}
