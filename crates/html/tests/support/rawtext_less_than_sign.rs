//! RAWTEXT less-than sign adapter evidence; synthetic, not upstream counts.

use crate::support::*;
use neptune_html::tokenizer::State;
use serde_json::json;

#[test]
fn literal_recovery_including_eof_compares_complete_output_and_exact_diagnostics() {
    for input in ["<", "<<", "<!", "<A", "<&amp;", "<\r\n<\0"] {
        let normalized = input.replace("\r\n", "\n").replace('\0', "\u{fffd}");
        let case = json!({
            "description": "literal less-than recovery",
            "input": input, "output": [["Character", normalized]],
            "errors": if input.contains('\0') {
                json!([{"code":"unexpected-null-character", "line":2, "col":2}])
            } else { json!([]) }
        });
        assert_eq!(
            run_with(&case, "RAWTEXT state", None, execute),
            Outcome::Passed
        );
        let mut wrong = case;
        wrong["output"] = json!([["Character", ""]]);
        assert!(matches!(
            run_with(&wrong, "RAWTEXT state", None, execute),
            Outcome::Failed(_)
        ));
    }
    let case = json!({
        "description": "reconsumption does not duplicate preprocessing diagnostics",
        "doubleEscaped": true, "input": "<\\uD800<\\uFFFF<\\u000B",
        "output": [["Character", "<\\uD800<\\uFFFF<\\u000B"]],
        "errors": [
            {"code":"surrogate-in-input-stream", "line":1, "col":2},
            {"code":"noncharacter-in-input-stream", "line":1, "col":4},
            {"code":"control-character-in-input-stream", "line":1, "col":6}
        ]
    });
    assert_eq!(
        run_with(&case, "RAWTEXT state", None, execute),
        Outcome::Passed
    );
}

#[test]
fn even_a_matching_recovery_prefix_cannot_pass_at_end_tag_name_stub() {
    for input in ["<x</style>", "<x</A"] {
        let case = json!({
            "description":"later destination remains terminal",
            "input":input, "output":[["Character", "<x"]], "lastStartTag":"style"
        });
        assert_eq!(
            run_with(&case, "RAWTEXT state", None, execute),
            Outcome::Blocked(Blocked::Unimplemented(State::RawtextEndTagName))
        );
    }
}
