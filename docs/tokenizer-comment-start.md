# Comment start state

## Current complete ordinary-comment family (PR #56)

All ten ordinary-comment states are implemented and reachable from explicit
CommentStart entry with initialized pending-comment context. Nested openers now
execute BangDashDash → real CommentEnd, including exact nested-comment diagnostics,
delimiter restoration/withholding, closure, EndBang loops and EOF take/emission.
See [final integration evidence](tokenizer-comment-less-than-sign-bang-dash-dash.md#final-integration-onto-47-55)
for independent outcomes across all ten entries, source snapshots and every-split
streaming checks. Normal Data `<!--` entry still blocks at MarkupDeclarationOpen.
No sibling production algorithm/shared helper, fixture, pin or pass baseline changed.
Earlier PR-local support/count reports below are historical, not current
reachability; their pinned algorithm-branch evidence remains normative.

## Historical CommentEnd integration (PR #55)

On base `08ddeae1103a97766b5055e61e70ce4907c2477a` (merged #47–54),
CommentEnd is implemented. StartDash's second hyphen now reaches real End, restoring withheld delimiters on fallback or closing on '>'; CommentStart's own source effects remain unchanged.
See [current integration evidence](tokenizer-comment-end.md#integration-onto-47-54)
for exact source/destination assertions, every-split cases and validation.
Normal Data comment entry remains blocked at MarkupDeclarationOpen. Production
sibling algorithms/shared helpers are unchanged. Earlier PR support/count reports
below are historical; their pinned source branch evidence remains normative.

## Historical Comment integration (PR #54)

On base `f2ca599065ec7fa70cc437f5c2ea6b9e2ee68883` (merged #47–53) plus
PR #54, Comment is implemented alongside the seven merged comment states.
CommentStart's fallback (including EOF) now executes real Comment. After its
hyphen, real CommentStartDash likewise replays fallback into Comment. The original
source snapshots remain independent; NUL replacement and EOF emission/take belong
to the actual destination, not CommentStart.

All affected state suites execute actual destinations and every-split streaming
paths; [core Comment integration evidence](tokenizer-comment.md#integration-onto-47-53)
records independently expected loops, closures, diagnostics, positions and genuine
remaining blocks. No production sibling algorithm or shared helper changes.
Normal Data entry still blocks at MarkupDeclarationOpen. No new upstream pass
identities or full-conformance claim. **The PR-local support boundaries/counts
below are historical; pinned branch evidence remains normative.**

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
merged states. At that reviewed head, CommentStart's algorithm and tests were
unchanged and its Comment and CommentStartDash destinations still blocked.
Subsequent CommentStartDash integration is documented below.

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

Source effects are checked before any destination attempt. Comment remains a
terminal `UnimplementedState` destination. Comment start dash now executes after
the source's hyphen branch (see integration below). In particular, direct EOF
in CommentStart still reconsumes into the Comment stub: it neither emits EOF nor
reports `eof-in-comment` on that path. A block is not a recoverable HTML error or
a supported pause. Tests never reconstruct or resume blocked execution.
Cursor-only consumption probes stop before attempting a destination; they are
not substitutes for destination algorithms.

## Coverage and limitations

The original six focused tests cover all ASCII points, ASCII boundary lookalikes,
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

## Original validation report

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

## CommentStartDash integration (#52)

Rebased CommentStartDash onto `fb249f0e4273ed1626ce2f28ddc9f9cd8218dc78`
(including #47–51). CommentStart's algorithm and all independently expected
source effects remain unchanged. Its exhaustive source test now executes
CommentStartDash after `-`: the next `z` is consumed, one hyphen appended, and
`z` reconsumed into the genuine Comment stub, with unchanged unrelated context.

The additional
`hyphen_entry_executes_all_real_start_dash_branches_after_exact_source_effects`
test asserts CommentStart's source snapshot first, then all four actual
CommentStartDash branches, using empty/seeded data and open/closed input:

- A second hyphen consumes without append and blocks at CommentEnd.
- `>` reports abrupt closing at line 2, column 6, takes/emits the exact
  comment (no deferred hyphen), and runs Data through literal tail, CRLF,
  NUL/error and EOF, preserving token order and unrelated context.
- Open exhaustion after the first hyphen repeatedly suspends unchanged. Only
  explicit EOF reports `eof-in-comment` at line 2, column 6 and takes/emits
  comment then EOF, remaining in CommentStartDash. This differs from direct
  CommentStart EOF, whose source-only assertions are retained.
- Fallback (ordinary input, NUL, CR, surrogate, noncharacter and control)
  appends only one hyphen, preserves preprocessing diagnostics exactly once,
  and reconsumes in the genuine Comment stub. Cursor-only probes check
  normalized positions and the untouched tail, never resuming a block.

Predecessor chunk traces now also include `->`, `->` plus Data/error/blocking
tails, hyphen-prefixed fallback/CRLF/NUL/non-scalars, and trailing/consecutive CR,
at every two-chunk split and as single-code-point chunks. No block is
reconstructed or resumed. Normal Data entry still blocks at MarkupDeclarationOpen.
The original validation counts above are historical; current integration checks
and conformance counts are reported with #52.
