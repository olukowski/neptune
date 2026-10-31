# Attribute value (single-quoted)

## Pinned evidence and scope

Prerequisite PR #15 (merged at `046565d7c033ed2bff4934b83f689284113bb1b4`)
supplies tag/attribute storage, shared emission/abandonment, and fixture support.
This change implements **only** `State::AttributeValueSingleQuoted` in
`crates/html/src/tokenizer/attribute_value_single_quoted.rs`, with narrow
module/dispatch and independent support-inventory registration. PR #20's
integration onto main `81eecf0955187a6b9be74a1a1d68afaa4b7ba399` (merged
PRs #16/#17/#18/#19) preserves both quoted value states, After attribute value
(quoted), and the unquoted value state, and adds only After attribute name.

Normative evidence is the immutable HTML Standard revision recorded in
[sources.toml](sources.toml):
[Attribute value (single-quoted), lines 144326–144353](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144326-L144353).
The opening [Tokenization abandonment note, lines 143187–143189](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143187-L143189)
explains that an unfinished tag is not emitted at EOF.
The [removed-but-current duplicate rule, lines 144218–144229](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144218-L144229)
requires value appends to use `tag::current_attribute_mut`, including for a
detached duplicate, rather than selecting the preceding retained attribute.

## Branch-to-code/test map

All tests below are in
`crates/html/tests/attribute_value_single_quoted_state.rs`. They are local state
checks, not upstream conformance runs.

| Pinned branch | Implementation | Focused evidence (test names) |
| --- | --- | --- |
| Apostrophe U+0027 | `0x27`: consume, switch to After attribute value (quoted); do not append or reconsume | `quote_and_ampersand_consume_without_appending_before_executable_or_stub_destinations` |
| Ampersand U+0026 | `0x26`: set return state to Attribute value (single-quoted), then switch to Character reference; do not append or reconsume | Same delimiter test; `initially_empty_value_resumes_after_suspension_and_initially_absent_return_state_is_set` covers an absent return state as well as replacement of an existing one |
| NUL U+0000 | `0`: report `unexpected-null-character` at current position and append U+FFFD through the shared current-attribute helper | `every_ascii_append_branch_and_non_scalars_preserve_exact_current_identity_and_context`; `normalized_positions_nul_and_preprocessing_diagnostics_are_not_duplicated` |
| EOF | `Input::Eof`: report `eof-in-tag`, abandon only the selected tag through `tag::take_tag`, emit one EOF, finish | `open_exhaustion_is_effect_free_and_only_explicit_eof_abandons_selected_tag` |
| Anything else | `_`: append the code point unchanged through the shared current-attribute helper; remain in this state | Exhaustive ASCII append test, including TAB/LF/FF/SPACE, double quote, both angle brackets, equals, slash, backtick, case distinctions, Unicode whitespace/scalars, and actual surrogates/noncharacters |
| Open-input suspension (execution contract) | `Input::NeedMoreInput`: no state effects, no implied EOF | Open-exhaustion test; initially-empty-value test; chunk traces |

Every branch consumes once; none schedules reconsumption. The cursor normalizes
CR/CRLF before dispatch. Tests inspect normalized positions, preprocessing errors
(control, surrogate, noncharacter), repeated NUL diagnostics/replacement, and
unconsumed tails. A separately seeded cursor reconsumption test checks original
positions and that preprocessing diagnostics are not duplicated; it does not
invent a predecessor-state implementation.

Both tag kinds and retained/detached-current attributes are exercised. Snapshots
compare selected identity, tag names/flags, retained attributes and values,
unrelated pending tag/comment storage, scratch buffer, prior tokens/diagnostics,
return state, and last emitted start tag. Existing non-scalar stored values are
not preprocessed again. EOF abandons the selected tag and detached duplicate
without emission or end-tag emission diagnostics; unrelated storage and
last-start-tag bookkeeping remain untouched. Existing shared emission tests
continue to cover detached-duplicate exclusion. This handler itself never emits
a tag; closing quotes can now reach emission through After attribute value (quoted).

`all_two_chunk_splits_and_single_point_chunks_match_closed_source_step_traces`
compares each source step for every two-chunk split and single-code-point chunks,
including empty chunks, repeated suspension, CRLF boundaries, consecutive/trailing
CR, EOF, NUL, non-scalars, whitespace/punctuation, and later terminal destinations.

## Entry and conformance limitations

Direct isolated entry requires a selected pending tag with a current attribute.
These seeded tests do **not** establish a reachable end-to-end path: real Data
entry still stops at Before attribute value, as the focused integration test
asserts. After attribute value (quoted) now executes: source snapshots still
verify the closing quote independently, then whitespace recovery executes After
attribute name. Exact snapshots check selected-tag emission, detached exclusion,
end-tag errors, Data continuation, suspension, and one terminal EOF. The
normalized-input test inspects missing-whitespace diagnostics and reconsumption
without repeating preprocessing.
Character reference remains a typed terminal `UnimplementedState` stub: it
consumes no tail, emits no EOF, and adds no parse error. Repeated blocked
dispatch, even after feeding input, returns the same terminal error; blocked
Execution is never reconstructed/resumed. Combined quoted-source emission and
EOF checks live in `after_attribute_value_quoted_state.rs`.

No upstream fixtures, pinned revisions, or exclusions change. This value state
adds no upstream passes: the runner's initial states cannot enter it before an
existing stub. PR #20's separate [After attribute name evidence](tokenizer-after-attribute-name.md)
records its reviewed boolean-attribute/attribute-free pass additions. The prior
case-level baseline remains enforced, with full token/diagnostic comparison for
every comparable run; a blocked prefix is never a pass. Exact executed/passed/
failed/blocked evidence and required check results belong in the PR report.
This is partial tokenizer support, not full conformance or a browser-security claim.
