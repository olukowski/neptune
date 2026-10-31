# Attribute value (unquoted) evidence

## Scope and immutable source

Only `State::AttributeValueUnquoted` is added, in
`crates/html/src/tokenizer/attribute_value_unquoted.rs`; local branch tests are in
`crates/html/tests/attribute_value_unquoted_state.rs`. Dispatch/module registration
and the independent support inventory are the only shared code edits. Shared
storage, detached-current identity, emission, and fixture comparison come from
prerequisite PR #15. Historically PR #20 integrated onto main
`81eecf0955187a6b9be74a1a1d68afaa4b7ba399` (merged PRs #16/#17/#18/#19),
preserving both quoted value states, After attribute value (quoted), and this
unquoted state, and adding only After attribute name.

Normative evidence is the [pinned HTML Standard source, lines 144356–144404](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144356-L144404)
at `404b3eb14a7a936dddefb2d78e224dbeb01d2358`, not the living algorithm or another
engine. Shared behavior follows the opening [Tokenization rules](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143174-L143210)
(unfinished-tag abandonment, emission, independent end-tag errors) and the
[removed-but-still-current attribute note](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144218-L144229).
The pinned fixture format allows six initial states, none of them attribute value
states; fixture input is preprocessed as specified, not split into scalar lines.
Neither pinned revision nor upstream fixtures nor existing pass baselines change.

## Branch-to-code/test map

All branch names below refer to the state-local `step` match. All test names refer
to the focused state test file.

| Consumed input | Code/effects | Focused evidence |
| --- | --- | --- |
| TAB, LF, FF, SPACE | `0x09 \| 0x0a \| 0x0c \| 0x20`: switch to Before attribute name; no append or reconsumption. Cursor normalizes CR/CRLF to LF. | `whitespace_terminates_without_append_error_or_reconsumption` checks every separator, CR/CRLF, exact consumption/positions and all preserved context; `whitespace_recovery_completes_or_abandons_unfinished_values` runs real destination recovery. |
| `&` | `0x26`: set return state to Attribute value (unquoted), switch to Character reference. Do not append, diagnose or reconsume. | `ampersand_sets_return_state_then_blocks_terminally_without_consuming_tail` checks replacement of a prior return state, exact source snapshot, open/closed input, unconsumed tail and repeated typed terminal block. |
| `>` | `0x3e`: shared `tag::emit_tag`, switch to Data. | `greater_than_emits_only_retained_attributes_with_shared_end_tag_errors_and_bookkeeping` checks ordered retained attributes, detached exclusion (including detached-only storage), both flag values, unrelated storage, last-start-tag rules, normalized delimiter position, exact/coexisting end-tag errors, real Data continuation and one terminal EOF. |
| NUL | `0`: `unexpected-null-character`, append U+FFFD through `tag::current_attribute_mut`. | `all_appending_ascii_and_non_scalars_preserve_current_identity_and_exact_errors` checks replacement, exact diagnostic, and detached-current identity; `normalized_positions_and_reconsumed_non_scalars_are_diagnosed_only_once` checks positions alongside malformed punctuation. |
| `"`, `'`, `<`, `=`, backtick | `0x22 \| 0x27 \| 0x3c \| 0x3d \| 0x60`: `unexpected-character-in-unquoted-attribute-value`, then literal append through `current_attribute_mut`. | The same two tests cover every forbidden point and exact error multiplicity/positions. No punctuation is dropped, replaced or reconsumed. |
| EOF | `Input::Eof`: `eof-in-tag`, shared `take_tag` abandonment, one terminal EOF. No tag or emission-only errors. | `open_exhaustion_is_effect_free_then_explicit_eof_abandons_only_selected_tag` checks empty/nonempty input, retained/detached current, both flagged tag kinds, selected-only abandonment, unrelated storage and last-start-tag preservation, exact EOF position and repeated termination. |
| Anything else | `_`: literal append through `current_attribute_mut`; stay in this state. | `all_appending_ascii_and_non_scalars_preserve_current_identity_and_exact_errors` enumerates all remaining ASCII and representative Unicode/non-scalars, including literal solidus, case-preserved uppercase, VT/NBSP and non-ASCII whitespace. It checks exact one-point consumption and preprocessing diagnostics. |

Every matrix applies to both tag kinds and retained versus detached current
attributes. Snapshot assertions include the selector, both pending tags, retained
order/values/names, detached storage, pending comment, prior tokens/errors,
preprocessing diagnostics/current position, last start tag, return state and
scratch buffer. A detached duplicate is never replaced by its retained predecessor.
Value operations always use the prerequisite helper, and emission never includes
the detached slot.

`normalized_positions_and_reconsumed_non_scalars_are_diagnosed_only_once` seeds a
valid cursor reconsumption before first dispatch: actual lone surrogates,
noncharacters and a control survive literally, preprocessing errors occur once,
and NUL/forbidden punctuation retain exact line/column diagnostics after CRLF.
It does not reconstruct a blocked execution or simulate an adjacent state.

`NeedMoreInput` is an effect-free suspension, not EOF or state departure. The
EOF test checks repeated suspension and empty pushes before explicit finish.
`all_two_chunk_splits_and_single_point_chunks_match_complete_source_step_traces`
compares every production source-step snapshot and terminal result with closed
input for all two-chunk splits and single-code-point chunks, including repeated
empty-input suspension, literal `/`, malformed values, NUL, non-scalars, CRLF,
consecutive/trailing CR, whitespace recovery, emission/Data continuation, EOF,
and later stubs. Tests stop at the first terminal stub and never resume it.
These are local state-specific streaming checks, not upstream conformance runs.

## Reachability and remaining stubs

PR #21 adds Before attribute value onto main `2983fbe`, preserving the merged
states and connecting unquoted value parsing from Data. Character reference
remains a terminal `UnimplementedState` on dispatch after `&`.
Whitespace executes Before attribute name/Attribute name/After attribute name:
exact snapshots check emission, Data continuation, EOF abandonment, and detached
clearing on new attribute creation. Recovery requiring a value now executes
Before attribute value and unquoted value; EOF is abandoned only in the latter.
A stub never emits EOF or a recoverable HTML parse error; blocked executions
cannot be reconstructed/resumed.

Isolated value contexts inspect source behavior independently. Real
`<a x=value>` / `</a x=value>` Data tests now complete with exact values, EOF,
and end-tag diagnostics. Combined integration tests also distinguish literal
solidus in an unquoted value from a structural trailing solidus after whitespace.
There is no tree construction, self-closing acknowledgement, automatic mode
selection, full-conformance claim or browser-security claim.

## Upstream evidence

Existing case-level baseline files remain untouched. Historically this
value-state PR alone added no upstream passes because Data entry was blocked.
The combined PR #21 integration's new passes and independently reviewed
identities are recorded in
[Before attribute value evidence](tokenizer-before-attribute-value.md).
PR #20's [After attribute name evidence](tokenizer-after-attribute-name.md)
retains its boolean-attribute/attribute-free allowlist unchanged.
Synthetic seeded tests and fixture inventory are not additional conformance
executions. Exact checks and counts are reported in the PR, separately from local
tests. No full-conformance or browser-security claim is made.
