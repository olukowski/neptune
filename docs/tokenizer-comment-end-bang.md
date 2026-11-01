# Comment end bang state

## Pinned source and scope

Normative algorithm: WHATWG HTML source at
`404b3eb14a7a936dddefb2d78e224dbeb01d2358`,
[Comment end bang state, lines 144708–144732](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144708-L144732)
(`#comment-end-bang-state`), retrieved directly from that immutable revision.
The pin remains unchanged in `docs/sources.toml`.

Only `CommentEndBang` is implemented here, in
`crates/html/src/tokenizer/comment_end_bang.rs`. `comment.rs` remains the private
shared helper module introduced by prerequisite PR #46
(`47c53e8dcf3e69d920c4be290036ffb70af72cc9`, this branch's base).
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

Source effects are inspectable before the next destination attempt. On this base,
Comment end dash and Comment are terminal `UnimplementedState` destinations;
they are never silently skipped, reconstructed, or resumed. Data executes normally
after `>`, including its separate NUL behavior and later terminal stub paths.

## Focused evidence

`crates/html/tests/comment_end_bang_state.rs` contains seven local tests:

- Every ASCII value plus Unicode boundaries/scalars, actual lone surrogates,
  noncharacters and controls; exact source snapshots, literal seeded data,
  pending-comment take semantics, diagnostics/positions, consumed tails and
  scheduled reconsumption. Empty and seeded comments are checked with both
  unrelated selected-tag kinds, retained/detached attributes, token prefixes,
  prior diagnostics, scratch, last-start-tag and return state.
- Terminal destination checks inspect source effects first and preserve the
  complete snapshot across repeated stub attempts.
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
  Suspensions preserve snapshots; no trace feeds or resumes terminal execution.

These tests and the support inventory are not upstream conformance executions.
The upstream runner adds no newly passing identities: it has no isolated
CommentEndBang initial-state mapping, and normal comment entry remains blocked.
All prior passing identities are retained; fixtures, pins and pass baselines
are unchanged. Exact current upstream counts are reported with the PR/check run,
not treated as whole-tokenizer conformance.
