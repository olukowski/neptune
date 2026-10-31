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
fn script_data_less_than_keeps_real_escape_blocks_despite_exact_prefixes() {
    for (suffix, prefix, target) in [
        (
            "/ XMP><!tail",
            "&\u{fffd}</ XMP><!",
            State::ScriptDataEscapeStart,
        ),
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
        ("/", "&\u{fffd}</"),
        ("/ XMP>", "&\u{fffd}</ XMP>"),
        ("/\0", "&\u{fffd}</\u{fffd}"),
        ("/\r\n", "&\u{fffd}</\n"),
        ("\0", "&\u{fffd}<\u{fffd}"),
        ("\r\n", "&\u{fffd}<\n"),
        ("test-->", "&\u{fffd}<test-->"),
        ("<x", "&\u{fffd}<<x"),
        ("&", "&\u{fffd}<&"),
    ] {
        let mut case = json!({"description":"literal fallback must complete",
            "input":format!("&\0<{suffix}"), "output":[["Character",output]],
            "errors":[{"code":"unexpected-null-character","line":1,"col":2}]});
        if matches!(suffix, "\0" | "/\0") {
            case["errors"]
                .as_array_mut()
                .unwrap()
                .push(json!({"code":"unexpected-null-character","line":1,"col":if suffix == "\0" { 4 } else { 5 }}));
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
    for case in [
        json!({"description":"less-than fallback non-scalars diagnose only once", "doubleEscaped":true,
            "input":"<\\uD800\\uFDD0\\u000B", "output":[["Character","<\\uD800\\uFDD0\\u000B"]],
            "errors":[{"code":"surrogate-in-input-stream","line":1,"col":2},
                {"code":"noncharacter-in-input-stream","line":1,"col":3},
                {"code":"control-character-in-input-stream","line":1,"col":4}]}),
        json!({"description":"end-tag-open fallback non-scalars diagnose only once", "doubleEscaped":true,
            "input":"</\\uD800\\uFDD0\\u000B", "output":[["Character","</\\uD800\\uFDD0\\u000B"]],
            "errors":[{"code":"surrogate-in-input-stream","line":1,"col":3},
                {"code":"noncharacter-in-input-stream","line":1,"col":4},
                {"code":"control-character-in-input-stream","line":1,"col":5}]}),
    ] {
        assert_eq!(
            run_with(&case, "Script data state", None, execute),
            Outcome::Passed
        );
        let first_error = case["errors"][0].clone();
        let mut doubled = case;
        doubled["errors"].as_array_mut().unwrap().push(first_error);
        assert!(matches!(
            run_with(&doubled, "Script data state", None, execute),
            Outcome::Failed(_)
        ));
    }
}

#[test]
fn integrated_script_names_require_complete_matching_literal_eof_and_diagnostics() {
    for case in [
        json!({"description":"formerly name-blocked exact adapter path now completes", "input":"&\0</script>",
            "lastStartTag":"script", "output":[["Character","&\u{fffd}"],["EndTag","script"]],
            "errors":[{"code":"unexpected-null-character","line":1,"col":2}]}),
        json!({"description":"appropriate mixed case and following Data NUL", "input":"&amp;</ScRiPt>\0",
            "lastStartTag":"script", "output":[["Character","&amp;"],["EndTag","script"],["Character","\0"]],
            "errors":[{"code":"unexpected-null-character","line":1,"col":15}]}),
        json!({"description":"mismatch then appropriate close", "input":"</other></ScRiPt>",
            "lastStartTag":"script", "output":[["Character","</other>"],["EndTag","script"]]}),
        json!({"description":"absent history keeps end tag literal", "input":"</ScRiPt>",
            "output":[["Character","</ScRiPt>"]]}),
        json!({"description":"name EOF is literal, not tag EOF", "input":"</ScRiPt",
            "lastStartTag":"script", "output":[["Character","</ScRiPt"]]}),
        json!({"description":"delimiter hands EOF to attribute destination", "input":"</script ",
            "lastStartTag":"script", "output":[], "errors":[{"code":"eof-in-tag","line":1,"col":10}]}),
        json!({"description":"end tag emission errors coexist", "input":"</ScRiPt a='x'/>",
            "lastStartTag":"script", "output":[["EndTag","script"]],
            "errors":[{"code":"end-tag-with-attributes","line":1,"col":16},
                {"code":"end-tag-with-trailing-solidus","line":1,"col":16}]}),
        json!({"description":"name fallback non-scalars diagnose only once", "doubleEscaped":true,
            "input":"</ScRiPt\\uD800\\uFDD0\\u000B>", "lastStartTag":"script",
            "output":[["Character","</ScRiPt\\uD800\\uFDD0\\u000B>"]],
            "errors":[{"code":"surrogate-in-input-stream","line":1,"col":9},
                {"code":"noncharacter-in-input-stream","line":1,"col":10},
                {"code":"control-character-in-input-stream","line":1,"col":11}]}),
    ] {
        assert_eq!(
            run_with(&case, "Script data state", None, execute),
            Outcome::Passed
        );
        let mut truncated = case.clone();
        truncated["output"] = json!([["Character", "&amp;"]]);
        assert!(matches!(
            run_with(&truncated, "Script data state", None, execute),
            Outcome::Failed(_)
        ));
        if let Some(errors) = case["errors"].as_array() {
            let mut missing = case.clone();
            missing["errors"].as_array_mut().unwrap().pop();
            assert!(matches!(
                run_with(&missing, "Script data state", None, execute),
                Outcome::Failed(_)
            ));
            let mut doubled = case.clone();
            doubled["errors"]
                .as_array_mut()
                .unwrap()
                .push(errors[0].clone());
            assert!(matches!(
                run_with(&doubled, "Script data state", None, execute),
                Outcome::Failed(_)
            ));
        }
    }
}

#[test]
fn matching_script_close_does_not_hide_later_data_or_attribute_stubs() {
    for (input, output, target) in [
        (
            "</script>&tail",
            json!([["EndTag", "script"]]),
            State::CharacterReference,
        ),
        ("</script a='&tail", json!([]), State::CharacterReference),
        (
            "</script><!tail",
            json!([["EndTag", "script"]]),
            State::MarkupDeclarationOpen,
        ),
        (
            "</script<!tail",
            json!([["Character", "</script<!"]]),
            State::ScriptDataEscapeStart,
        ),
    ] {
        let case = json!({"description":"complete-looking prefix remains blocked", "input":input,
            "lastStartTag":"script", "output":output});
        assert_eq!(
            run_with(&case, "Script data state", None, execute),
            Outcome::Blocked(Blocked::Unimplemented(target))
        );
    }
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
