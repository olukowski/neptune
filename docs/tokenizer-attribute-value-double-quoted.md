# Attribute value (double-quoted) state coverage

Normative baseline: immutable [HTML Standard source, lines
144296–144324](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144296-L144324)
at `404b3eb14a7a936dddefb2d78e224dbeb01d2358`,
**Attribute value (double-quoted) state**
(`#attribute-value-(double-quoted)-state`). EOF abandonment also follows the
[opening Tokenization note](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143187-L143189).
Sources and upstream fixtures remain pinned as recorded in [sources.toml](sources.toml).

This state's algorithm lives only in `attribute_value_double_quoted.rs`.
The following tests are in
`crates/html/tests/attribute_value_double_quoted_state.rs`:

| Consumed input | Code branch / source effects | Focused test |
| --- | --- | --- |
| `"` U+0022 | `0x22`: switch to After attribute value (quoted); do not append or reconsume. | `quote_and_ampersand_consume_without_appending_before_executable_or_stub_destinations` |
| `&` U+0026 | `0x26`: set return state to Attribute value (double-quoted), then switch to Character reference; do not append or reconsume. | Same test; absent, different, and already-equal return states |
| NUL U+0000 | `0`: report `unexpected-null-character`; append U+FFFD to the current value. | `nul_and_every_literal_ascii_branch_preserve_identity_and_unrelated_storage` |
| EOF | `Input::Eof`: report `eof-in-tag`; abandon only the selected tag via shared `tag::take_tag`; emit one terminal EOF. | `eof_abandons_only_selected_tag_without_emission_errors_or_last_start_tag_update` |
| Anything else | `_`: append the code point literally, including whitespace, apostrophe, angle brackets, uppercase, malformed markup-like text, and non-scalars. | Exhaustive ASCII / selected non-ASCII test above |

All branches consume once; none schedules reconsumption. Value mutation always
uses `tag::current_attribute_mut`, including detached duplicates with a retained
predecessor. Snapshots check exact selected-tag identity, names, flags, retained
and detached attributes, unrelated pending tags/comment, existing output/errors,
scratch buffer, return state, and last emitted start tag. EOF never emits the
unfinished tag or emission-only end-tag errors, even with retained attributes
and a preexisting self-closing flag. Existing shared emission tests independently
cover detached duplicate exclusion. This handler itself never emits a tag;
closing quotes can now reach emission through After attribute value (quoted).

Additional focused tests:

- `open_exhaustion_is_effect_free_before_and_after_values_and_can_finish_or_take_a_delimiter`:
  repeated NeedMoreInput/empty chunks preserve context and current input;
  later input or explicit finish resumes only a suspended execution. A closing
  quote now reaches effect-free suspension in After attribute value (quoted),
  then explicit EOF abandons only the selected tag with `eof-in-tag`.
- `normalized_locations_and_non_scalars_are_preserved_with_preprocessing_errors_once`:
  CRLF-normalized positions after a supplementary scalar, literal surrogates,
  noncharacters and controls, NUL replacement, and exact diagnostic multiplicity.
  Valid cursor reconsumption on seeded entry does not duplicate preprocessing.
- `real_data_entry_completes_abandons_or_blocks_only_at_character_reference`:
  real start/end tags emit complete values; EOF inside the value abandons the
  selected tag, while NUL replacement followed by ampersand preserves pending
  value/return state and terminal Character reference without consuming its tail.
- `all_two_chunk_splits_and_single_point_chunks_match_every_closed_source_step`:
  representative values, malformed markup, whitespace, CRLF splits,
  consecutive/trailing CR, NUL, non-scalars, delimiter/stub boundaries and EOF
  compare every source-step snapshot, for both tag kinds and duplicate-current
  variants. Separate Data traces now execute reachable value parsing.

## Boundaries and upstream evidence

Prerequisite PR #15 supplies the existing tag/attribute storage and shared
emission/fixture support. Historically, PR #20 integrated onto main
`81eecf0955187a6b9be74a1a1d68afaa4b7ba399` (merged PRs #16/#17/#18/#19),
retaining both quoted value implementations, After attribute value (quoted),
and the unquoted value state, and adding only After attribute name. Source
snapshots still verify the closing quote independently; subsequent whitespace
recovery executes After attribute name and checks exact selected-tag emission,
end-tag diagnostics, Data continuation (including NUL), suspension, and terminal
EOF snapshots. Malformed adjacency reports missing whitespace and schedules
reconsumption without repeating preprocessing. Combined quoted-source
emission and EOF checks live in `after_attribute_value_quoted_state.rs`.
Character reference remains a terminal typed `UnimplementedState` destination.
Repeated blocked steps are effect-free, including after direct cursor
inspection/mutation. A blocked Execution cannot be reconstructed/resumed.

PR #21 adds Before attribute value onto main `2983fbe`, connecting real Data
input to this state. Seeded contexts remain isolated source-step tests, alongside
real Data completion, EOF abandonment, and Character reference blocking.
The runner's initial-state conventions do not include value states; historically
this value-state PR alone added no upstream passes. The combined integration's
new passes and reviewed identities are recorded in
[Before attribute value evidence](tokenizer-before-attribute-value.md).
All prior case-level baselines (including PR #20's boolean-attribute additions)
remain enforced unchanged. Upstream fixtures/revisions are unchanged.

Run all [contributing checks](../CONTRIBUTING.md) and the
[upstream runner](tokenizer.md#fixture-runner). The PR report records exact
executed/passed/failed/blocked counts separately from local tests and fixture
inventory. Local branch/streaming tests do not establish whole-tokenizer
conformance, tree-builder behavior, or browser security.
