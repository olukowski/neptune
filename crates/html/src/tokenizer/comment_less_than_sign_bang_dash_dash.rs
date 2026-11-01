//! Comment less-than sign bang dash dash only: pinned HTML Standard source,
//! lines 144642–144657, #comment-less-than-sign-bang-dash-dash-state.
//! <https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144642-L144657>
//! Branch evidence: docs/tokenizer-comment-less-than-sign-bang-dash-dash.md.

use super::{Context, Input, ParseError, Progress, State};

pub(super) fn step(context: &mut Context) -> Progress {
    match context.input.consume() {
        Input::NeedMoreInput => return Progress::NeedMoreInput,
        // '>' and conceptual EOF reconsume without a nested-comment error.
        Input::CodePoint(point) if point.value() == 0x3e => {}
        Input::Eof => {}
        // Anything else: diagnose, but do not append, replace NUL, or emit.
        Input::CodePoint(_) => context.errors.push(ParseError {
            code: "nested-comment",
            position: context.input.position(),
        }),
    }
    context
        .input
        .reconsume()
        .expect("just consumed a code point or EOF");
    Progress::Continue(State::CommentEnd)
}
