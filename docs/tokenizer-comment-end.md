# Comment end state

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
| `!` U+0021 | Continue in CommentEndBang; no append, emission, state diagnostic or reconsumption. | `hyphens_append_one_each_then_bang_consumes_without_appending_or_reconsuming`: inspect exact source snapshot, then terminal destination block; raw cursor inspection finds the untouched tail, not `!`. |
| `-` U+002D | Append one hyphen; stay in CommentEnd. | Same hyphen/bang test checks each of three steps; emission and EOF tests independently check accumulated data. |
| EOF | Report `eof-in-comment` at the consumed EOF position, emit/take the comment, then emit EOF. No switch or reconsumption. | `open_exhaustion_suspends_until_explicit_eof_diagnoses_and_emits_comment_then_eof`: empty/seeded data, zero/one/three hyphens, exact token ordering and diagnostic multiplicity, no duplicate terminal effects. |
| Anything else | Append exactly two hyphens, not the current point; schedule reconsumption in Comment. | `anything_else_restores_only_two_hyphens_and_reconsumes_without_comment_processing`: every other ASCII value, non-ASCII whitespace/letters/combining marks/supplementary scalar, actual surrogates, noncharacters and controls. NUL is not replaced or diagnosed by this source state. |
| Open exhaustion (execution contract, not spec EOF) | NeedMoreInput; no token, error or context mutation. | Repeated empty pushes/suspension before explicit EOF, plus streaming traces. |

Snapshots compare state, current input/normalized position, pending comment,
ordered tokens, state and preprocessing diagnostics, selected tag and both
pending tags (including retained/discarded attributes and flags), last emitted
start tag, return state and scratch. Seeded NUL/CR/surrogate data is never
preprocessed again. Fallback verifies scheduled reconsumption and the original
point/position with no duplicate preprocessing errors; the following tail is
unconsumed. Only the cursor is inspected after blocking, never a reconstructed
or resumed tokenizer execution.

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

## Entry limitations and upstream evidence

`Comment` and `CommentEndBang` remain explicit terminal `UnimplementedState`
destinations. Tests inspect the completed CommentEnd source effects before the
next destination attempt. They do not implement the destination's NUL handling,
recovery, appends or emission. After sibling implementations merge, destination
integration expectations must be reviewed and the full checks rerun; the source
branch contract above remains the baseline.

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

Validation: formatting, Clippy with warnings denied, workspace tests (388 passed,
0 failed/ignored, including seven focused tests), rustdoc with warnings denied,
and the separate pinned upstream report all pass. No fixture or source pin is
changed. This is partial standalone tokenizer support, not tree construction or
a browser-security claim.
