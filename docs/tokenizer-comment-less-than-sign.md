# Comment less-than sign state

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
CommentEnd is implemented. Fallback can now continue through Comment/End to closure, EndBang recovery or EOF. The '<!' hyphen path still blocks only at BangDashDash.
See [current integration evidence](tokenizer-comment-end.md#integration-onto-47-54)
for exact source/destination assertions, every-split cases and validation.
Normal Data comment entry remains blocked at MarkupDeclarationOpen. Production
sibling algorithms/shared helpers are unchanged. Earlier PR support/count reports
below are historical; their pinned source branch evidence remains normative.

## Historical Comment integration (PR #54)

On base `f2ca599065ec7fa70cc437f5c2ea6b9e2ee68883` (merged #47–53) plus
PR #54, Comment is implemented alongside the seven merged comment states.
Less-than-sign fallback now executes real Comment, including EOF emission/take
and literal/NUL handling. Its bang path executes Bang, BangDash, EndDash and
Comment as applicable. Source append/no-take snapshots remain independent.

All affected state suites execute actual destinations and every-split streaming
paths; [core Comment integration evidence](tokenizer-comment.md#integration-onto-47-53)
records independently expected loops, closures, diagnostics, positions and genuine
remaining blocks. No production sibling algorithm or shared helper changes.
Normal Data entry still blocks at MarkupDeclarationOpen. No new upstream pass
identities or full-conformance claim. **The PR-local support boundaries/counts
below are historical; pinned branch evidence remains normative.**

## Pinned source and scope

Normative source: WHATWG HTML revision
`404b3eb14a7a936dddefb2d78e224dbeb01d2358`,
[Comment less-than sign state, source lines 144590–144607](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144590-L144607)
(`#comment-less-than-sign-state`). The immutable raw source was inspected directly;
no sibling algorithm or another engine supplies this implementation.
[Sources and fixture pins](sources.toml) are unchanged.

Implementation: `crates/html/src/tokenizer/comment_less_than_sign.rs`.
Focused tests: `crates/html/tests/comment_less_than_sign_state.rs`.
`tokenizer/comment.rs` remains exclusively shared helper support from prerequisite
PR #46 (`47c53e8dcf3e69d920c4be290036ffb70af72cc9`), the original prerequisite base.
There are no new dependencies, storage, token variants, or adapter capabilities.

Direct isolated entry supplies an initialized `PendingComment`. Normal predecessor
context includes an already-consumed `<` already appended to that comment's data
(pinned source lines 144568–144570); this state must not append that prior point
again. The independent support inventory supplies this context. No selected tag
or temporary-buffer setup is required by the state.

## Branch evidence

| Pinned branch | Source effects | Focused evidence |
| --- | --- | --- |
| `!` (144597–144599) | Append original point via `comment::data_mut`; switch to `CommentLessThanSignBang`; no reconsumption. | `every_ascii_branch_and_unicode_fallback_have_exact_source_effects`; `repeated_less_than_and_bang_append_in_order_without_taking_comment` |
| `<` (144601–144602) | Append original point via the shared helper; remain in `CommentLessThanSign`. | Same tests, including repeated `<` and exact accumulation order. |
| Anything else (144604–144605) | No data mutation; schedule reconsumption in `Comment`. Includes conceptual EOF, NUL, hyphen, `>`, ampersand, Unicode and non-scalars. | Exhaustive ASCII plus representative Unicode/non-scalars; `open_exhaustion_is_effect_free_but_explicit_eof_reconsumes_in_comment`. |
| Open exhaustion (execution contract, not a spec EOF branch) | Return `NeedMoreInput`, preserving current input and all context. | Repeated empty/open suspension, resumed feeding in chunk tests, explicit finish distinguished from exhaustion. |

All branches preserve unrelated pending tags (including retained and detached
attributes), selector, prior tokens/diagnostics, last-start-tag bookkeeping,
return state and scratch. Seeded comment data includes literal NUL, CR and a lone
surrogate: none is preprocessed again, replaced, discarded or emitted. This state
never calls `comment::emit`, takes the pending comment, emits EOF, or reports a
state parse error, **even at EOF**. Existing shared-helper tests independently
cover emission/take semantics; no sibling is implemented to force emission here.

Tests assert exact snapshots before destination attempts and exact normalized
positions, starting after a simulated consumed `<` at line 2, column 1.
Preprocessing errors for actual surrogates, noncharacters and controls occur once
at initial consumption (line 2, column 2), and remain unchanged on cursor
reconsumption. NUL is neither appended nor diagnosed by this source. CR normalizes
to LF before fallback; existing pending CR remains literal. Append branches leave
the following character unconsumed; fallback reconsumes the exact point or EOF at
its original position. Cursor probes precede the remaining unsupported destination
attempts; no blocked execution is reconstructed or resumed.

`every_two_chunk_split_and_single_point_chunks_preserve_source_step_traces`
compares every two-chunk split (including empty chunks) and single-code-point
chunks with closed-input traces for empty input, repeated `<`, bang, malformed
suffixes, NUL, CR/CRLF/consecutive/trailing CR, Unicode, lone surrogates,
noncharacters and controls. With PR #53 integration, these traces execute Bang,
Bang dash and End dash where reachable, including `!-` EOF emission and `!-`
fallback with NUL/CRLF/control/noncharacter input. They stop at the first genuine
terminal stub or EOF; no later chunk is fed after termination. Repeated suspension,
completion and terminal errors are effect-free.

## Destinations and upstream limitations

On the original PR #47 base, both destinations were stubs. After PR #53 rebases
onto `76f91837fc17eb6de8ec313a1c5dcac906203b41`, `CommentLessThanSignBang`
executes: source append/no-take assertions remain independent, then `!tail`
consumes and reconsumes `t` in the still-unsupported `Comment`. The real hyphen
path executes Bang dash and End dash; End dash alone appends the withheld hyphen
on fallback or reports `eof-in-comment`, takes/emits the comment and emits EOF.
`Comment` and `CommentLessThanSignBangDashDash` remain genuine terminal stubs.
See [the integrated state-local evidence](tokenizer-comment-less-than-sign-bang.md#integration-on-main).
No sibling algorithm or gateway is changed.

Normal Data entry on `<!--x-->` still blocks at `MarkupDeclarationOpen` before
comment creation. The focused Data regression checks this exact boundary. This
PR adds **no Data-entry comment coverage and no newly comparable upstream runs**.
All 952 prior protected passing identities remain unchanged; there are no new
passing identities to add or independently review for this state alone.

The upstream report is unchanged: 6,827 fixture cases; 7,053 runs; 6,190
dispatch-attempted; **1,148 executed, 1,148 passed, 0 failed, 5,905 blocked**.
Blocked reasons: Character reference 4,692; Markup declaration open 242;
Processing instruction open 52; CDATA section 56; token variants 858;
pending specification review 1; XML coercion 4. Inventory and focused tests are
not additional upstream conformance executions. Passing this partial suite is
not full tokenizer conformance or browser security.
