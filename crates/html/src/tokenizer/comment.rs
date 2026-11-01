//! Shared pending-comment data and emission support, not a tokenizer state.
//! Normative baseline: Tokenization (token emission) and Bogus comment state
//! (#bogus-comment-state) in HTML Standard source at
//! 404b3eb14a7a936dddefb2d78e224dbeb01d2358.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source>

use super::{CodePoint, Context, Token};

// The entering state establishes pending-comment storage. Isolated entry must
// likewise supply it. Neither helper creates storage or preprocesses its data.
pub(super) fn data_mut(context: &mut Context) -> &mut Vec<CodePoint> {
    &mut context
        .pending_comment
        .as_mut()
        .expect("comment operation requires a current comment")
        .data
}

// Take only the pending comment. State changes, diagnostics and EOF emission
// remain the responsibility of the calling state.
pub(super) fn emit(context: &mut Context) {
    let comment = context
        .pending_comment
        .take()
        .expect("comment operation requires a current comment");
    context.tokens.push(Token::Comment { data: comment.data });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::{
        Attribute, Input, ParseError, PendingComment, PendingEndTag, PendingStartTag, State,
        TagKind,
    };

    fn points(text: &str) -> Vec<CodePoint> {
        text.chars().map(CodePoint::from).collect()
    }

    fn seeded(kind: TagKind) -> Context {
        let attribute = Attribute {
            name: points("name"),
            value: points("value"),
        };
        let mut context = Context {
            tokens: vec![Token::Character('p'.into())],
            current_tag: Some(kind),
            pending_start_tag: Some(PendingStartTag {
                name: points("start"),
                self_closing: true,
                attributes: vec![attribute.clone()],
                discarded_current_attribute: Some(attribute.clone()),
            }),
            pending_end_tag: Some(PendingEndTag {
                name: points("end"),
                self_closing: true,
                attributes: vec![attribute.clone()],
                discarded_current_attribute: Some(attribute),
            }),
            last_start_tag: Some(points("title")),
            return_state: Some(State::Rcdata),
            temporary_buffer: points("scratch"),
            ..Context::default()
        };
        let surrogate = CodePoint::new(0xd800).unwrap();
        context.input.push([surrogate, 'x'.into()]).unwrap();
        assert_eq!(context.input.consume(), Input::CodePoint(surrogate));
        context.input.reconsume().unwrap();
        context.errors.push(ParseError {
            code: "prior",
            position: context.input.position(),
        });
        context
    }

    fn assert_unrelated_unchanged(context: &Context, before: &Context) {
        assert_eq!(context.current_tag, before.current_tag);
        assert_eq!(context.pending_start_tag, before.pending_start_tag);
        assert_eq!(context.pending_end_tag, before.pending_end_tag);
        assert_eq!(context.last_start_tag, before.last_start_tag);
        assert_eq!(context.return_state, before.return_state);
        assert_eq!(context.temporary_buffer, before.temporary_buffer);
        assert_eq!(context.errors, before.errors);
        assert_eq!(context.input.current(), before.input.current());
        assert_eq!(context.input.position(), before.input.position());
        assert_eq!(context.input.errors(), before.input.errors());
    }

    #[test]
    fn mutation_preserves_literal_data_without_other_effects() {
        for kind in [TagKind::Start, TagKind::End] {
            let before = seeded(kind);
            let mut context = seeded(kind);
            context.pending_comment = Some(PendingComment {
                data: points("seed\0\r"),
            });
            let appended = [0x3c, 0x26, 0, 0x0d, 0xd800, 0xdfff, 0xfdd0, 0x1f600]
                .map(|value| CodePoint::new(value).unwrap());
            data_mut(&mut context).extend(appended);
            data_mut(&mut context).push('!'.into());
            let mut expected = points("seed\0\r");
            expected.extend(appended);
            expected.push('!'.into());
            assert_eq!(
                context.pending_comment,
                Some(PendingComment { data: expected })
            );
            assert_eq!(context.tokens, before.tokens);
            assert_unrelated_unchanged(&context, &before);
            // No consumption or loss of the caller's scheduled reconsumption.
            assert_eq!(
                context.input.consume(),
                Input::CodePoint(CodePoint::new(0xd800).unwrap())
            );
            assert_eq!(context.input.errors(), before.input.errors());
            assert_eq!(context.input.consume(), Input::CodePoint('x'.into()));
        }
    }

    #[test]
    fn emission_takes_only_comment_and_preserves_token_order_without_eof() {
        for kind in [TagKind::Start, TagKind::End] {
            for data in [
                vec![],
                vec!['\0'.into(), '\r'.into(), CodePoint::new(0xdfff).unwrap()],
            ] {
                let before = seeded(kind);
                let mut context = seeded(kind);
                context.pending_comment = Some(PendingComment { data: data.clone() });
                emit(&mut context);
                assert!(context.pending_comment.is_none());
                assert_eq!(
                    context.tokens,
                    [Token::Character('p'.into()), Token::Comment { data }]
                );
                assert_unrelated_unchanged(&context, &before);
                // A later comment remains separate, including empty boundaries.
                context.pending_comment = Some(PendingComment::default());
                emit(&mut context);
                assert!(context.pending_comment.is_none());
                assert_eq!(context.tokens.len(), 3);
                assert_eq!(
                    context.tokens.last(),
                    Some(&Token::Comment { data: vec![] })
                );
                assert_unrelated_unchanged(&context, &before);
                assert_eq!(
                    context.input.consume(),
                    Input::CodePoint(CodePoint::new(0xd800).unwrap())
                );
                assert_eq!(context.input.errors(), before.input.errors());
                assert_eq!(context.input.consume(), Input::CodePoint('x'.into()));
            }
        }
    }
}
