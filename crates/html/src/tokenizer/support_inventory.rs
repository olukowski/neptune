//! Deliberate test-only support inventory, independent of production dispatch.

use super::*;

#[test]
fn explicit_support_inventory_matches_dispatch() {
    // Deliberate support inventory, independent of dispatch outcomes.
    const IMPLEMENTED: &[State] = &[
        State::Data,
        State::Rcdata,
        State::RcdataLessThanSign,
        State::RcdataEndTagOpen,
        State::RcdataEndTagName,
        State::Plaintext,
        State::Rawtext,
        State::RawtextLessThanSign,
        State::RawtextEndTagOpen,
        State::RawtextEndTagName,
        State::ScriptData,
        State::ScriptDataDoubleEscapedDashDash,
        State::ScriptDataLessThanSign,
        State::ScriptDataEndTagOpen,
        State::ScriptDataEndTagName,
        State::ScriptDataEscapeStart,
        State::ScriptDataEscapeStartDash,
        State::ScriptDataEscaped,
        State::ScriptDataEscapedDash,
        State::ScriptDataEscapedDashDash,
        State::ScriptDataDoubleEscapedLessThanSign,
        State::ScriptDataDoubleEscapeEnd,
        State::ScriptDataDoubleEscapedDash, // Direct entry needs no pending storage.
        State::TagOpen,
        State::EndTagOpen,
        State::TagName,
        State::BeforeAttributeName,
        State::AttributeName,
        State::AfterAttributeName,
        State::BeforeAttributeValue,
        State::AttributeValueDoubleQuoted,
        State::AttributeValueSingleQuoted,
        State::AttributeValueUnquoted,
        State::AfterAttributeValueQuoted,
        State::SelfClosingStartTag,
        State::BogusComment,
    ];
    assert_eq!(State::ALL.len(), 84);
    for &state in State::ALL {
        for finished in [false, true] {
            let mut context = Context::default();
            if state == State::ScriptDataDoubleEscapedLessThanSign {
                // Direct entry follows an already consumed AND emitted '<'.
                // No selected tag or initialized scratch string is required.
                context.input.push(['<'.into()]).unwrap();
                assert_eq!(context.input.consume(), Input::CodePoint('<'.into()));
                context.tokens.push(Token::Character('<'.into()));
            }
            if finished {
                context.input.finish();
            }
            if IMPLEMENTED.contains(&state) {
                // Supply invariants for isolated tag/comment entry at EOF.
                context.current_tag = Some(TagKind::Start);
                context.pending_start_tag = Some(PendingStartTag::default());
                if state == State::RcdataEndTagName {
                    context.current_tag = Some(TagKind::End);
                    context.pending_end_tag = Some(PendingEndTag::default());
                }
                if matches!(
                    state,
                    State::AttributeName
                        | State::BeforeAttributeValue
                        | State::AttributeValueDoubleQuoted
                        | State::AttributeValueSingleQuoted
                        | State::AttributeValueUnquoted
                ) {
                    context
                        .pending_start_tag
                        .as_mut()
                        .unwrap()
                        .attributes
                        .push(Attribute::default());
                }
                if matches!(
                    state,
                    State::RawtextEndTagName | State::ScriptDataEndTagName
                ) {
                    context.current_tag = Some(TagKind::End);
                    context.pending_end_tag = Some(PendingEndTag::default());
                }
                if state == State::ScriptDataDoubleEscapeEnd {
                    // Direct entry supplies the normalized prefix scratch buffer;
                    // the predecessor clears it on '/', with no tag requirement.
                    context.temporary_buffer = vec!['s'.into(), 'c'.into(), 'r'.into()];
                }
                context.pending_comment = Some(PendingComment::default());
                let mut execution = Execution::new(state, context);
                let result = execution.step();
                assert!(result.is_ok(), "{state:?}");
                if !finished {
                    assert_eq!(result, Ok(Progress::NeedMoreInput), "{state:?}");
                }
                continue;
            }
            let mut execution = Execution::new(state, context);
            let expected = Err(UnimplementedState { state });
            assert_eq!(execution.step(), expected);
            assert_eq!(
                execution.step_with(|_, _| panic!("must not resume")),
                expected
            );
            assert_eq!(execution.context().input.current(), None);
            assert!(execution.context().tokens.is_empty());
            assert!(execution.context().errors.is_empty());
            assert!(execution.context().current_tag.is_none());
            assert!(execution.context().pending_start_tag.is_none());
            assert!(execution.context().pending_end_tag.is_none());
            assert!(execution.context().pending_comment.is_none());
            assert!(execution.context().temporary_buffer.is_empty());
        }
    }
    assert_eq!(
        UnimplementedState {
            state: State::AttributeName
        }
        .to_string(),
        "unimplemented HTML tokenizer state: Attribute name state"
    );
}
