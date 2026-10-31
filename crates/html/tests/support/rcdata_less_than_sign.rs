//! Local production-adapter checks for RCDATA less-than sign, not upstream runs.

use crate::support::*;
use neptune_html::tokenizer::State;
use serde_json::json;

#[test]
fn rcdata_less_than_sign_adapter_requires_complete_literal_recovery_and_exact_errors() {
    for input in [
        "<",
        "p<",
        "<<",
        "<!",
        "<title>",
        "<!--x--><?x>",
        "p</",
        "</0>",
        "</>",
    ] {
        let case = json!({"description":"local literal less-than recovery",
            "input":input, "output":[["Character",input]]});
        assert_eq!(
            run_with(&case, "RCDATA state", None, execute),
            Outcome::Passed
        );
        let mut wrong = case;
        wrong["output"] = json!([["Character", "wrong"]]);
        assert!(matches!(
            run_with(&wrong, "RCDATA state", None, execute),
            Outcome::Failed(_)
        ));
    }
    let escaped = json!({"description":"local reconsumption diagnostic multiplicity",
        "input":"<\\u0000\r\n<\\uD800<\\uFDD0<\\u000B", "doubleEscaped":true,
        "output":[["Character","<\\uFFFD\n<\\uD800<\\uFDD0<\\u000B"]],
        "errors":[{"code":"unexpected-null-character","line":1,"col":2},
            {"code":"surrogate-in-input-stream","line":2,"col":2},
            {"code":"noncharacter-in-input-stream","line":2,"col":4},
            {"code":"control-character-in-input-stream","line":2,"col":6}]});
    assert_eq!(
        run_with(&escaped, "RCDATA state", None, execute),
        Outcome::Passed
    );
    let mut wrong = escaped;
    wrong["errors"].as_array_mut().unwrap().pop();
    assert!(matches!(
        run_with(&wrong, "RCDATA state", None, execute),
        Outcome::Failed(_)
    ));
}

#[test]
fn rcdata_less_than_sign_adapter_preserves_terminal_blocks_after_literal_prefixes() {
    for (input, output) in [
        (
            "p</title>&",
            json!([["Character", "p"], ["EndTag", "title"]]),
        ),
        (
            "p<!</title>&",
            json!([["Character", "p<!"], ["EndTag", "title"]]),
        ),
        ("p<&", json!([["Character", "p<"]])),
        ("p</Other&", json!([["Character", "p</Other"]])),
    ] {
        let case = json!({"description":"local blocked prefix is not completion",
            "input":input,"output":output,"lastStartTag":"title"});
        assert_eq!(
            run_with(&case, "RCDATA state", None, execute),
            Outcome::Blocked(Blocked::Unimplemented(State::CharacterReference))
        );
    }
}
