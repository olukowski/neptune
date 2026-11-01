# Comment start state

## Pinned source and scope

Implements only **Comment start state** (`#comment-start-state`) from the
[immutable HTML source, lines 144514–144532](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144514-L144532),
revision `404b3eb14a7a936dddefb2d78e224dbeb01d2358` recorded in
[`sources.toml`](sources.toml). The source was read directly at that revision.
The prerequisite shared comment helpers are present in base
`47c53e8dcf3e69d920c4be290036ffb70af72cc9` (PR #46).
After independent review with no findings, this PR was rebased onto
`6d99604d8e374bb15bc9b7ad4b6cd3f6166b4ed4` (including PRs #47 and #48).
Additive module/dispatch/inventory/docs conflicts were resolved preserving both
merged states. CommentStart's reviewed algorithm and tests are unchanged;
its Comment and CommentStartDash destinations still block as before.

Algorithm: `crates/html/src/tokenizer/comment_start.rs`.
Focused evidence: `crates/html/tests/comment_start_state.rs`.
`tokenizer/comment.rs` remains the private shared helper module, unchanged.
No sibling state, Markup declaration open, storage/token variant, dependency,
fixture adapter, upstream fixture/pin, or passing baseline is added or changed.
Direct entry requires an initialized `pending_comment`; the independent support
inventory already supplies this context, including for EOF.

## Branch mapping

| Consumed item | Source effects | Focused evidence |
| --- | --- | --- |
| U+002D `-` | Switch to Comment start dash; no append, emission, diagnostic, or reconsumption. | Exhaustive ASCII source snapshots; cursor-only tail probe; streaming `-` / `--tail`. |
| U+003E `>` | Report `abrupt-closing-of-empty-comment` at the consumed delimiter; switch to Data and emit/take the current comment unchanged. No reconsumption. | Empty/seeded emission; exhaustive source snapshots; exact subsequent Data output, diagnostics, EOF and blocks. |
| Anything else, including conceptual EOF | Reconsume in Comment; do not append, replace NUL, emit, take the comment, or report a state error. | Exhaustive ASCII and representative Unicode/non-scalars; explicit EOF/open-exhaustion distinction; reconsumption flag and cursor-only probes. |
| Open input exhausted | Suspend without changing current item, pending data, tokens, or diagnostics. Not EOF. | Repeated suspension at fresh and positioned isolated entry; every streaming drain. |

Source effects are checked before any destination attempt. Comment and Comment
start dash remain terminal `UnimplementedState` destinations in this PR. In
particular, EOF reconsumes into the Comment stub: this PR neither emits EOF nor
reports `eof-in-comment` on that path. A block is not a recoverable HTML error or
a supported pause. Tests never reconstruct or resume blocked execution.
Cursor-only consumption probes stop before attempting a destination; they are
not substitutes for destination algorithms.

## Coverage and limitations

The six focused tests cover all ASCII points, ASCII boundary lookalikes,
non-ASCII letters and supplementary scalars, actual lone surrogates, noncharacters,
controls, NUL and CR/CRLF normalization. Source snapshots assert exact tokens,
state/preprocessing diagnostics and normalized positions, pending-comment take
versus preservation, prior token boundaries/order, unrelated retained and detached
tag attributes, selector, scratch, return state and last-start-tag context.
Seeded comment data contains literal NUL, CR, markup punctuation, a surrogate and
a noncharacter; emission preserves it without reprocessing. Empty data is emitted
as its own Comment token. The abrupt-close diagnostic is unconditional, even for
seeded data supplied by isolated entry.

Representative inputs are checked at every two-chunk split (including empty
chunks), and as single-code-point chunks, against closed-input source-step
traces. Cases include all three branches, EOF, malformed input, CRLF and
consecutive/trailing CR, NUL, non-scalars, Data continuation, and later terminal
blocks. Repeated terminal calls must be effect-free.

A separate real Data-entry regression proves `<!-->tail` still stops at Markup
declaration open before ordinary comment creation or emission. There is **no new
Data-entry comment coverage**, automatic tree-builder behavior, full tokenizer
conformance, or browser-security claim.

## Validation report

All CONTRIBUTING checks passed: formatting, warning-denied Clippy, locked
workspace tests (397 test functions passed, zero failed), and warning-denied
documentation. All six focused tests passed.
The separately invoked `pinned_upstream_runner` reports:

- Fixture cases: 6,827; case/initial-state runs: 7,053.
- Dispatch-attempted: 6,190.
- Executed: **1,148**; passed: **1,148**; failed: **0**; blocked: **5,905**.

Existing case-level pass-baseline regressions pass unchanged. No new upstream
identities become comparable: the upstream format does not select Comment start
as an initial state, and ordinary Data comment entry remains blocked. Inventory
and the six local tests are not additional upstream conformance cases. These
counts describe this PR after rebasing onto the base above, not later sibling
integration.
