//! Local adapter checks for explicit Script data entry, not upstream pass counts.

use neptune_html::tokenizer::State;
use serde_json::json;

use crate::support::{Blocked, Outcome, execute, initial_states, run_with};

#[test]
fn script_data_mapping_literals_replacement_and_non_scalars_compare_exactly() {
    for case in [
        json!({"description":"empty", "input":"", "output":[]}),
        json!({"description":"literal references", "input":"&amp;&#0;&bogus;",
            "output":[["Character","&amp;&#0;&bogus;"]]}),
        json!({"description":"replacement, normalization and non-scalars", "doubleEscaped":true,
            "input":"\\u0000\r\n\\uD800\\uDFFF\\uFDD0\\u000B",
            "output":[["Character","\\uFFFD\n\\uD800\\uDFFF\\uFDD0\\u000B"]],
            "errors":[{"code":"unexpected-null-character","line":1,"col":1},
                {"code":"surrogate-in-input-stream","line":2,"col":1},
                {"code":"surrogate-in-input-stream","line":2,"col":2},
                {"code":"noncharacter-in-input-stream","line":2,"col":3},
                {"code":"control-character-in-input-stream","line":2,"col":4}]}),
    ] {
        assert_eq!(
            run_with(&case, "Script data state", None, |run| {
                assert_eq!(run.initial, State::ScriptData);
                assert!(run.context.return_state.is_none());
                assert!(run.context.last_start_tag.is_none());
                assert!(run.context.current_tag.is_none());
                assert!(run.context.pending_start_tag.is_none());
                assert!(run.context.pending_end_tag.is_none());
                assert!(run.context.pending_comment.is_none());
                assert!(run.context.temporary_buffer.is_empty());
                execute(run)
            }),
            Outcome::Passed
        );
        let mut wrong = case.clone();
        wrong["output"] = json!([["Character", "wrong"]]);
        assert!(matches!(
            run_with(&wrong, "Script data state", None, execute),
            Outcome::Failed(_)
        ));
        if case.get("errors").is_some() {
            let mut missing_error = case;
            missing_error["errors"].as_array_mut().unwrap().pop();
            assert!(matches!(
                run_with(&missing_error, "Script data state", None, execute),
                Outcome::Failed(_)
            ));
        }
    }
}

#[test]
fn script_data_less_than_keeps_real_end_tag_and_escape_blocks_despite_exact_prefixes() {
    for (suffix, prefix, target) in [
        ("/script>", "&\u{fffd}", State::ScriptDataEndTagOpen),
        ("/", "&\u{fffd}", State::ScriptDataEndTagOpen),
        ("!", "&\u{fffd}<!", State::ScriptDataEscapeStart),
        ("!--tail", "&\u{fffd}<!", State::ScriptDataEscapeStart),
    ] {
        let case = json!({"description":"matching prefix must not pass",
            "input":format!("&\0<{suffix}"), "output":[["Character",prefix]],
            "lastStartTag":"script",
            "errors":[{"code":"unexpected-null-character","line":1,"col":2}]});
        assert_eq!(
            run_with(&case, "Script data state", None, execute),
            Outcome::Blocked(Blocked::Unimplemented(target))
        );
    }
}

#[test]
fn script_data_less_than_fallback_requires_complete_literal_output_and_exact_errors() {
    for (suffix, output) in [
        ("", "&\u{fffd}<"),
        ("\0", "&\u{fffd}<\u{fffd}"),
        ("\r\n", "&\u{fffd}<\n"),
        ("test-->", "&\u{fffd}<test-->"),
        ("<x", "&\u{fffd}<<x"),
        ("&", "&\u{fffd}<&"),
    ] {
        let mut case = json!({"description":"literal fallback must complete",
            "input":format!("&\0<{suffix}"), "output":[["Character",output]],
            "errors":[{"code":"unexpected-null-character","line":1,"col":2}]});
        if suffix == "\0" {
            case["errors"]
                .as_array_mut()
                .unwrap()
                .push(json!({"code":"unexpected-null-character","line":1,"col":4}));
        }
        assert_eq!(
            run_with(&case, "Script data state", None, execute),
            Outcome::Passed
        );
        let mut wrong = case.clone();
        wrong["output"] = json!([["Character", "&\u{fffd}"]]);
        assert!(matches!(
            run_with(&wrong, "Script data state", None, execute),
            Outcome::Failed(_)
        ));
        case["errors"].as_array_mut().unwrap().pop();
        assert!(matches!(
            run_with(&case, "Script data state", None, execute),
            Outcome::Failed(_)
        ));
    }
    let case = json!({"description":"reconsumed non-scalars diagnose only once", "doubleEscaped":true,
        "input":"<\\uD800\\uFDD0\\u000B", "output":[["Character","<\\uD800\\uFDD0\\u000B"]],
        "errors":[{"code":"surrogate-in-input-stream","line":1,"col":2},
            {"code":"noncharacter-in-input-stream","line":1,"col":3},
            {"code":"control-character-in-input-stream","line":1,"col":4}]});
    assert_eq!(
        run_with(&case, "Script data state", None, execute),
        Outcome::Passed
    );
    let mut doubled = case;
    doubled["errors"]
        .as_array_mut()
        .unwrap()
        .push(json!({"code":"surrogate-in-input-stream","line":1,"col":2}));
    assert!(matches!(
        run_with(&doubled, "Script data state", None, execute),
        Outcome::Failed(_)
    ));
}

#[test]
fn script_data_is_explicit_not_automatic_on_script_start_tag_and_data_remains_default() {
    let case = json!({"description":"independent entries", "input":"&",
        "output":[["Character","&"]],
        "initialStates":["Script data state", "Data state", "Script data state"]});
    let outcomes: Vec<_> = initial_states(&case)
        .iter()
        .map(|state| run_with(&case, state, None, execute))
        .collect();
    assert_eq!(
        outcomes,
        [
            Outcome::Passed,
            Outcome::Blocked(Blocked::Unimplemented(State::CharacterReference)),
            Outcome::Passed
        ]
    );
    let default = json!({"description":"no automatic tree-builder mode", "input":"<script>&",
        "output":[["StartTag","script",{}],["Character","&"]]});
    assert_eq!(initial_states(&default), ["Data state"]);
    assert_eq!(
        run_with(&default, &initial_states(&default)[0], None, |run| {
            assert_eq!(run.initial, State::Data);
            execute(run)
        }),
        Outcome::Blocked(Blocked::Unimplemented(State::CharacterReference))
    );
    let literal = json!({"description":"ampersand is not decoded", "input":"&amp;",
        "output":[["Character","&"]]});
    assert!(matches!(
        run_with(&literal, "Script data state", None, execute),
        Outcome::Failed(_)
    ));
}
