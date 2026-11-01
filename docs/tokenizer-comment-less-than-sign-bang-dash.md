# Comment less-than sign bang dash state

## Pinned normative evidence

Implementation: `crates/html/src/tokenizer/comment_less_than_sign_bang_dash.rs`.
Focused evidence: `crates/html/tests/comment_less_than_sign_bang_dash_state.rs`.

Read the exact immutable WHATWG HTML source recorded in `sources.toml`, revision
`404b3eb14a7a936dddefb2d78e224dbeb01d2358`:

- [Assigned state, lines 144626–144640](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144626-L144640).
- [Preprocessing, current/next input and conceptual EOF, lines 142560–142606](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142606).
- [Tokenization and reconsumption, lines 143149–143173](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143149-L143173).

| Input | Source effects | Focused evidence |
| --- | --- | --- |
| U+002D `-` | Consume; switch to Comment less-than sign bang dash dash. Do not append or reconsume the hyphen. | Exhaustive ASCII test checks exact snapshot, terminal destination and immediate tail consumption. Unicode hyphen lookalikes take fallback. |
| Anything else, including conceptual EOF | Consume; schedule the same item for reconsumption in Comment end dash. | Exhaustive ASCII/non-scalar test checks exact item and position, scheduled reconsumption and untouched tail; explicit EOF test checks reconsumed EOF at the next normalized position. |
| Exhausted open input (execution contract, not a spec EOF branch) | Suspend with no state, cursor-current, storage, output or diagnostic change. | Repeated empty feeds/suspensions, explicit finish and resumed feeds. |

Neither specification branch appends, emits, takes or replaces the pending
comment. Neither reports a state parse error, including on NUL or EOF. The
initialized `PendingComment` is supplied by isolated entry; seeded NUL, CR,
surrogates and noncharacters remain literal, without preprocessing stored data.
Existing comment-token boundaries/order and unrelated selected/unselected tags,
retained/detached attributes, last start tag, return state, scratch and prior
errors are preserved. The state needs neither shared comment helper: comment
mutation/emission belongs to destination implementations. `tokenizer/comment.rs`
remains the private shared helper module, not a state algorithm.

All ASCII code points and representative Unicode scalars, actual surrogates,
noncharacters and controls have exact normalized source positions and cursor
preprocessing errors checked, independently from state errors. Reconsumption
adds no duplicate preprocessing diagnostics. Open and finished inputs, empty
and seeded comments, and absent/start/end tag selectors are exercised.

Streaming tests compare closed input against every two-chunk split (including
empty chunks) and single-code-point chunks, with and without preceding CRLF
history. Representative inputs include both branches, EOF, CRLF and consecutive
CR, NUL, malformed comment-like text, non-scalars and untouched suffixes. Only
one assigned-state source step executes, whose snapshot remains separate from
actual supported-destination effects. Fallback now executes CommentEndDash:
non-EOF appends exactly one hyphen and reconsumes the same point in Comment;
EOF reports `eof-in-comment`, takes/emits the comment, then emits EOF and finishes.
Chunk traces compare both snapshots and cursor reconsumption/remaining tails.
Cursor probes precede remaining stub attempts, and no implemented state executes
with a probed cursor. This is local source/cursor chunk evidence, not
whole-tokenizer conformance.

## Scope and integration boundary

The original state PR registered only CommentLessThanSignBangDash, with both
destinations blocked on its prerequisite base. After CommentEndDash integration
(PR #50), tests retain all source-effect snapshots before executing that real
destination. Exact destination snapshots check the original point/position,
preprocessing-error multiplicity, hyphen-only append, unchanged unrelated context,
and EOF diagnostic/token ordering/take semantics with terminal repeat safety.
Only CommentLessThanSignBangDashDash (hyphen branch) and Comment (non-EOF
fallback after CommentEndDash) remain terminal `UnimplementedState` boundaries.
No blocked execution is resumed or reconstructed. Later sibling merges may
legitimately move those boundaries and require review, not hidden mismatches.

Direct entry uses initialized pending-comment context, including in the independent
support inventory. This change implements no siblings or Markup declaration open.
Normal Data comment entry still blocks at MarkupDeclarationOpen after consuming
`<!`, before creating a comment or reaching this state; a focused test retains
that boundary. No new Data-entry coverage, adapter capability, token/storage type,
dependency, fixture/pin edit or upstream passing-baseline identity is introduced.
Existing baseline passes remain mandatory; inventory and local tests are not
upstream conformance executions. Current runner counts are reported in the PR,
not treated as full conformance or browser-security evidence.
