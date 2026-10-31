//! RCDATA-specific production adapter regressions, not conformance runs.

use crate::support::*;
use neptune_html::tokenizer::State;
use serde_json::json;

#[test]
fn rcdata_runner_uses_existing_explicit_mapping_and_keeps_prefixes_blocked() {
    let case = json!({"description":"local RCDATA mapping", "input":"\u{0000}\r\n\u{feff}é😀",
        "output":[["Character","\u{fffd}\n\u{feff}é😀"]],
        "initialStates":["RCDATA state","RCDATA state"], "lastStartTag":"title",
        "errors":[{"code":"unexpected-null-character","line":1,"col":1}]});
    for state in initial_states(&case) {
        assert_eq!(run_with(&case, &state, None, execute), Outcome::Passed);
    }
    // Default remains Data, whose literal NUL does not match RCDATA output.
    assert!(matches!(
        run_with(&case, &initial_states(&json!({}))[0], None, execute),
        Outcome::Failed(_)
    ));
    let escaped = json!({"description":"local actual non-scalars through adapter",
        "input":"\\u0000\r\n\\uD800\\uDFFF\\uFDD0\\u000B", "doubleEscaped":true,
        "output":[["Character","\\uFFFD\n\\uD800\\uDFFF\\uFDD0\\u000B"]],
        "errors":[{"code":"unexpected-null-character","line":1,"col":1},
            {"code":"surrogate-in-input-stream","line":2,"col":1},
            {"code":"surrogate-in-input-stream","line":2,"col":2},
            {"code":"noncharacter-in-input-stream","line":2,"col":3},
            {"code":"control-character-in-input-stream","line":2,"col":4}]});
    assert_eq!(
        run_with(&escaped, "RCDATA state", None, execute),
        Outcome::Passed
    );
    let empty = json!({"description":"local empty RCDATA", "input":"", "output":[]});
    assert_eq!(
        run_with(&empty, "RCDATA state", None, execute),
        Outcome::Passed
    );
    for (input, target) in [
        ("p&", State::CharacterReference),
        ("p</p>", State::RcdataEndTagName),
    ] {
        let prefix = json!({"description":"local matching prefix is not completion", "input":input,
            "output":[["Character","p"]], "lastStartTag":"p"});
        assert_eq!(
            run_with(&prefix, "RCDATA state", None, execute),
            Outcome::Blocked(Blocked::Unimplemented(target))
        );
    }
}
