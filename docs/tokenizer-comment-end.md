# Comment end state

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

## Pinned source and scope

The normative baseline is WHATWG HTML revision
`404b3eb14a7a936dddefb2d78e224dbeb01d2358`, recorded in
[sources.toml](sources.toml). The exact source was read before implementation:

- [Comment end state, lines 144681–144706](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144681-L144706)
  (`#comment-end-state`).
- [Preprocessing, current/next input and conceptual EOF, lines 142560–142607](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142607).
- [Tokenization, reconsumption and token emission, lines 143151–143198](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143151-L143198).

Only `CommentEnd` is implemented in
`crates/html/src/tokenizer/comment_end.rs`. It uses the private
`comment::data_mut` and `comment::emit` helpers from prerequisite PR #46
(`47c53e8dcf3e69d920c4be290036ffb70af72cc9`, verified as an ancestor).
`comment.rs` remains shared support, not the core Comment state algorithm.
No new storage, token variants, dependencies, adapter capabilities, fixtures or
pins are introduced.

Isolated entry supplies `pending_comment: Some(PendingComment { data })`.
The two withheld hyphens that led to this state are **not** implicitly part of
that data. Existing data is literal, including NUL, CR and lone surrogates.
The independent support inventory supplies initialized pending-comment context.

## Branch mapping

The focused tests live in `crates/html/tests/comment_end_state.rs`.

| Consumed input | Source effects | Focused evidence |
| --- | --- | --- |
| `>` U+003E | Emit/take the pending comment; continue in Data. Neither delimiter nor withheld hyphens are appended. | `greater_than_takes_only_comment_then_real_data_consumes_tail_and_eof`: empty/literal seeded data, both unrelated tag selectors, one comment, delimiter consumption, real Data NUL output/error, final EOF and terminal repeat. |
| `!` U+0021 | Continue in CommentEndBang; no append, emission, state diagnostic or reconsumption. | `hyphens_append_one_each_then_bang_consumes_without_appending_or_reconsuming`: inspect exact source snapshot, then real EndBang consumes the next `z`, appends `--!`, replays in Comment and finishes with exact EOF emission/error. |
| `-` U+002D | Append one hyphen; stay in CommentEnd. | Same hyphen/bang test checks each of three steps; emission and EOF tests independently check accumulated data. |
| EOF | Report `eof-in-comment` at the consumed EOF position, emit/take the comment, then emit EOF. No switch or reconsumption. | `open_exhaustion_suspends_until_explicit_eof_diagnoses_and_emits_comment_then_eof`: empty/seeded data, zero/one/three hyphens, exact token ordering and diagnostic multiplicity, no duplicate terminal effects. |
| Anything else | Append exactly two hyphens, not the current point; schedule reconsumption in Comment. | `anything_else_restores_only_two_hyphens_then_real_comment_processes_reconsumed_input`: every other ASCII value, non-ASCII whitespace/letters/combining marks/supplementary scalar, actual surrogates, noncharacters and controls. NUL is not replaced or diagnosed by this source state. |
| Open exhaustion (execution contract, not spec EOF) | NeedMoreInput; no token, error or context mutation. | Repeated empty pushes/suspension before explicit EOF, plus streaming traces. |

Snapshots compare state, current input/normalized position, pending comment,
ordered tokens, state and preprocessing diagnostics, selected tag and both
pending tags (including retained/discarded attributes and flags), last emitted
start tag, return state and scratch. Seeded NUL/CR/surrogate data is never
preprocessed again. Fallback verifies scheduled reconsumption and the original
point/position with no duplicate preprocessing errors; the following tail is
unconsumed. Original source snapshots are retained before real Comment's append,
NUL replacement/error or less-than transition. Separate cursor-only probes end
without further dispatch; blocked execution is never reconstructed or resumed.

`direct_entry_accepts_reconsumed_characters_and_conceptual_eof_at_normalized_positions`
supplies already scheduled current characters and EOF directly, without executing
an unsupported predecessor. Every branch is checked at line 2, column 1 after
raw CRLF normalization, including the exact `eof-in-comment` position.

`every_two_chunk_split_and_single_point_chunks_match_closed_source_traces`
compares every source-step snapshot for all two-chunk splits (including empty
chunks) and single-code-point chunks. Samples include repeated hyphens, close,
bang, literal/NUL/non-scalar fallback, EOF/open exhaustion, and real Data tails
with CRLF, consecutive/trailing CR, NUL, scalars/non-scalars, tags and later
terminal stubs. A terminal block ends the run permanently, even with unfed input.
These are local state/streaming checks, not upstream conformance executions.

## Integration onto #47–54

Initial head `69c09852e2e97d6865319f9176eaf2c50b276fad` received independent
review with no findings and full checks. At the coordinator's instruction, #55
rebases onto `08ddeae1103a97766b5055e61e70ce4907c2477a` (merged #47–54).
All additive modules, dispatch/inventory entries, initialized entry context and
source evidence are preserved. The CommentEnd algorithm is byte-for-byte unchanged
from that reviewed head. All merged production siblings and shared `comment.rs`
are unchanged from the new base. Only CommentEnd is newly registered (50 supported
states total, including Bogus comment and nine ordinary-comment states).

Semantic integration changes, derived directly from the same pinned
[comment-family source, lines 144514–144732](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144514-L144732):

- Core Comment's `-->` now closes into Data with a taken/emitted comment;
  `--!` reaches real EndBang rather than a stub. Extra hyphens append one each;
  fallback restores exactly two, before real Comment appends/replaces/transitions.
- EndDash, StartDash, Start and EndBang suites retain original source snapshots,
  then execute End fallback and real Comment replay rather than expecting a
  development block. Every updated destination snapshot checks exact current
  point/normalized position, data, diagnostics (preprocessing once), token order
  and all unrelated context. The cursor-only probe paths stay separate.
- All affected predecessor chunk suites now include new closure, malformed-ending,
  EndBang-loop, NUL and comment-prefix/later-stub samples at every two-chunk split
  and as single-code-point chunks. They run real destinations without resuming
  or reconstructing a blocked execution.
- The independently expected table in `comment_state.rs` now covers all nine
  isolated ordinary-comment entry states, including `-->`, `--->`, `--!>`,
  `--!-->`, `--!-x-->`, `--!--!>`, `--!x-->`, literal/replacement fallback,
  End/EndBang/EndDash EOF states and returns through repeated Bang/End loops.
  The changed formerly blocked cases have literal expected data/errors/positions,
  not snapshots blessed from execution. Every case is checked at every split and
  as single-code-point chunks, in addition to source-step trace comparison.
- `core_end_bang_loop_has_independent_source_snapshots_and_once_only_preprocessing`
  asserts each step of core `--NUL--!SURROGATE CRLF --!> CONTROL`: withheld
  delimiters, original/replayed positions, source-vs-destination NUL handling,
  single surrogate/control preprocessing errors, incorrect-close at 2:4, exact
  literal seeded/emitted data, Data tail, take semantics and terminal EOF order.
  The same independent outcome participates in every-split integrated tests.
- `real_core_end_and_bang_suspend_with_withheld_delimiters_until_explicit_eof`
  checks repeated empty-chunk suspension at real End/EndBang after `--`, `--!`
  and `--!--`; EOF emits only the independently expected data, not the withheld
  suffix, with one precise error and one EOF.
- Emitted core-comment prefixes followed by `&`, `<!`, or `<?` remain genuinely
  blocked at CharacterReference, MarkupDeclarationOpen or ProcessingInstructionOpen.
  Expected Data return-state assignment and TagOpen's scratch clear are checked,
  rather than falsely asserting those downstream source effects are unchanged.
  Nested-opener samples still stop in BangDashDash before speculative effects.

README, the authoritative opening support list and sibling evidence notices now
reflect actual reachability. Historical PR-local support/count reports remain
explicitly marked as historical; pinned source-branch evidence is retained.
No gateway, final sibling, adapter, fixture, pin or baseline change is made.

## Entry limitations and upstream evidence

`Comment` and `CommentEndBang` now execute real destination steps after the
independently asserted CommentEnd source effects. CommentLessThanSignBangDashDash
is the only remaining ordinary-comment family stub. MarkupDeclarationOpen and
other unsupported Data destinations still report terminal `UnimplementedState`;
no matching comment output prefix can hide a later block.

`normal_data_comment_entry_still_blocks_before_creation` checks that ordinary
`<!--x-->` input consumes `<!` and blocks at `MarkupDeclarationOpen`, with no
pending comment creation or output. No new ordinary Data-entry comment coverage
or whole-tokenizer conformance is claimed.

Before and after this implementation, running
`cargo test --workspace --locked pinned_upstream_runner -- --nocapture` reports:

- 6,827 fixture cases; 7,053 initial-state runs; 6,190 dispatch-attempted.
- **Executed 1,148; passed 1,148; failed 0; blocked 5,905.**
- Blocks: CharacterReference 4,692; MarkupDeclarationOpen 242;
  ProcessingInstructionOpen 52; CdataSection 56; TokenVariants 858;
  XmlCoercion 4; PendingSpecReview 1.

No new upstream initial-state run enters CommentEnd, so there are no new passing
identities to add. Every existing baseline file and identity remains unchanged;
the runner's pass-regression checks pass. Inventory is not counted as conformance.

Validation: formatting, Clippy with warnings denied, workspace tests (445 passed,
0 failed/ignored, including nine focused CommentEnd tests), rustdoc with warnings denied,
and the separate pinned upstream report all pass. No fixture or source pin is
changed. This is partial standalone tokenizer support, not tree construction or
a browser-security claim.
