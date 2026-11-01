# Comment end bang state

## Current CommentEnd integration (PR #55)

On base `08ddeae1103a97766b5055e61e70ce4907c2477a` (merged #47–54),
CommentEnd is implemented. EndBang is now reachable from core Comment through End. Its '-' loop reaches real EndDash/End; malformed closure, fallback and EOF keep their original source effects.
See [current integration evidence](tokenizer-comment-end.md#integration-onto-47-54)
for exact source/destination assertions, every-split cases and validation.
Normal Data comment entry remains blocked at MarkupDeclarationOpen. Production
sibling algorithms/shared helpers are unchanged. Earlier PR support/count reports
below are historical; their pinned source branch evidence remains normative.

## Historical Comment integration (PR #54)

On base `f2ca599065ec7fa70cc437f5c2ea6b9e2ee68883` (merged #47–53) plus
PR #54, Comment is implemented alongside the seven merged comment states.
CommentEndBang's fallback now executes real Comment after the independently
asserted `--!` append/reconsumption. Its hyphen destination executes EndDash and
then Comment on fallback; only a second hyphen reaches the CommentEnd stub.
CommentEndBang itself is still reachable only by isolated entry, not core Comment.

All affected state suites execute actual destinations and every-split streaming
paths; [core Comment integration evidence](tokenizer-comment.md#integration-onto-47-53)
records independently expected loops, closures, diagnostics, positions and genuine
remaining blocks. No production sibling algorithm or shared helper changes.
Normal Data entry still blocks at MarkupDeclarationOpen. No new upstream pass
identities or full-conformance claim. **The PR-local support boundaries/counts
below are historical; pinned branch evidence remains normative.**

## Pinned source and scope

Normative algorithm: WHATWG HTML source at
`404b3eb14a7a936dddefb2d78e224dbeb01d2358`,
[Comment end bang state, lines 144708–144732](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144708-L144732)
(`#comment-end-bang-state`), retrieved directly from that immutable revision.
The pin remains unchanged in `docs/sources.toml`.

Only `CommentEndBang` is implemented here, in
`crates/html/src/tokenizer/comment_end_bang.rs`. `comment.rs` remains the private
shared helper module introduced by prerequisite PR #46
(`47c53e8dcf3e69d920c4be290036ffb70af72cc9`, the original branch base).
After independent review found no findings at
`e99606aee74eac9e2cd0be4ccb6bac215588b404`, this branch was rebased onto
`9c0c66b4c0c66137a795cd0e2f4efb0216729d4a` (merged #47–50).
There is no new storage, token variant, dependency, or runner capability.

Direct isolated entry must supply `pending_comment: Some(PendingComment { data })`.
That data is the existing comment data, excluding the withheld `--!` suffix.
It is preserved literally, without preprocessing it again. This state does not
create a comment. Ordinary Data entry still blocks at Markup declaration open;
no new Data-entry comment coverage or full-conformance claim is implied.

## Branch mapping

| Consumed input | Source effects | Result |
| --- | --- | --- |
| `-` U+002D | Append exactly `--!`, not the consumed hyphen. No error or emission. | Continue in Comment end dash, without reconsumption. |
| `>` U+003E | Report `incorrectly-closed-comment` at the consumed position. Emit/take only the existing comment, without appending `--!` or `>`. | Continue in Data, without reconsumption. |
| EOF | Report `eof-in-comment` at EOF. Emit/take only the existing comment, then emit one EOF token. No suffix append. | Finished; no reconsumption or Data switch. |
| Anything else | Append exactly `--!`, not the current character. No state error or emission. | Reconsume in Comment. |

Open exhausted input returns `NeedMoreInput`, not the EOF branch. NUL takes the
anything-else branch: this state neither replaces it nor diagnoses
`unexpected-null-character`. Those effects belong to the destination Comment
state, which is not implemented by this PR. Cursor preprocessing still normalizes
CR/CRLF and reports controls, surrogates and noncharacters once, retaining the
original code point on reconsumption.

Source effects are inspectable before the next destination attempt. On the rebased
base, Comment end dash executes normally after `-`; its destinations Comment and
Comment end remain terminal `UnimplementedState` stubs. Comment end bang's own
anything-else branch still blocks at Comment. Blocked executions are never
silently skipped, reconstructed, or resumed. Data executes normally after `>`,
including its separate NUL behavior and later terminal stub paths.

## Focused evidence

`crates/html/tests/comment_end_bang_state.rs` contains nine local tests:

- Every ASCII value plus Unicode boundaries/scalars, actual lone surrogates,
  noncharacters and controls; exact source snapshots, literal seeded data,
  pending-comment take semantics, diagnostics/positions, consumed tails and
  scheduled reconsumption. Empty and seeded comments are checked with both
  unrelated selected-tag kinds, retained/detached attributes, token prefixes,
  prior diagnostics, scratch, last-start-tag and return state.
- Terminal destination checks inspect source effects first and preserve the
  complete snapshot across repeated attempts at genuine remaining stubs.
- After the original `-` source snapshot, real Comment end dash executes every
  ASCII/representative non-scalar branch at normalized-history positions on open
  and closed input. A second hyphen consumes into Comment end without appending;
  other points append only `-` and reconsume in Comment, without replacement,
  emission or state diagnostics. Separate probe-only executions verify exact
  replay, untouched tails, positions and single preprocessing-error multiplicity;
  they never dispatch after probing or reconstruct a blocked run.
- Comment end dash suspends after `-` on open exhaustion, then explicit EOF reports
  exactly `eof-in-comment` at the next normalized position and takes/emits the
  existing data plus `--!`, followed by one EOF token. No extra hyphen or repeated
  emission is introduced; unrelated context and token-prefix ordering survive.
- EOF emits exactly the comment then one EOF; no withheld suffix, reconsumption,
  or repeated output. Positions and preexisting preprocessing errors are checked
  after a consumed surrogate and split/suppressed CRLF.
- Repeated open exhaustion and empty chunks are effect-free, including a CRLF
  boundary; explicit finish alone admits EOF effects.
- `>` emits/takes the comment once, then real Data processes NUL, CRLF and an
  actual surrogate with exact tokens, error codes, multiplicities and positions.
- Real Data comment entry remains blocked at Markup declaration open without
  creating or emitting a comment.
- Every two-chunk split and single-code-point chunks match closed-input
  source-step traces for representative empty, malformed, NUL, Unicode,
  non-scalar, CRLF/consecutive/trailing CR, EOF and downstream-blocked inputs.
  Traces now include real Comment end dash for `-`, `--`, `->`, `-NUL`, split CRLF
  and surrogate inputs, ending only at EOF or genuine remaining stubs.
  Suspensions preserve snapshots; no trace feeds or resumes terminal execution.

The reviewed production algorithm is unchanged by the rebase. Additive conflict
resolutions preserve all merged state modules, dispatch registrations, inventory
entries/context and docs. No predecessor/neighbor tests needed changes: no
implemented predecessor currently transitions into CommentEndBang (CommentEnd is
still a stub), and all merged suites pass unchanged.

These tests and the support inventory are not upstream conformance executions.
The upstream runner adds no newly passing identities: it has no isolated
CommentEndBang initial-state mapping, and normal comment entry remains blocked.
All prior passing identities are retained; fixtures, pins and pass baselines
are unchanged. Exact current upstream counts are reported with the PR/check run,
not treated as whole-tokenizer conformance.

## Rebased validation

All CONTRIBUTING checks, explicit inventory reporting, `pinned_upstream_runner`
and `git diff --check` pass on the rebased integration. Workspace tests: **412
passed, 0 failed, 0 ignored**, including nine focused CommentEndBang tests.
The upstream runner remains unchanged: 6,827 cases; 7,053 runs; 6,190
dispatch-attempted; **1,148 executed; 1,148 passed; 0 failed; 5,905 blocked**.
The inventory separately reads 14 files / 6,827 cases and executes zero
conformance cases. No new upstream passing identities are added.
