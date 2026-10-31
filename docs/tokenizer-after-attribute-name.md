# After attribute name evidence

## Scope and immutable source

The opening scope below records PR #20's historical integration. PR #21 rebases
onto main `2983fbe809621963f8399e6e94072ccfc04937ce` and adds only Before
attribute value; all previously merged state algorithms remain unchanged.

Only `State::AfterAttributeName` is added, in
`crates/html/src/tokenizer/after_attribute_name.rs`. Originally based on
prerequisite PR #15 (`046565d7c033ed2bff4934b83f689284113bb1b4`), this change
is integrated onto main `81eecf0955187a6b9be74a1a1d68afaa4b7ba399` (merged
PRs #16/#17/#18/#19). Both quoted value states, After attribute value (quoted),
and the unquoted value state are preserved. No adjacent state,
shared storage algorithm, dependency, fixture revision, or upstream expectation
is implemented/changed.

Normative algorithm: HTML Standard revision
`404b3eb14a7a936dddefb2d78e224dbeb01d2358`,
[After attribute name, source lines 144232–144270](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144232-L144270).
Also reviewed the adjacent
[Before attribute name and Attribute name rules](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144145-L144229)
and the opening
[Tokenization abandonment/emission rules](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143174-L143210).
The living standard is not the implementation baseline.

## Branch map

Every row is a separate match arm in `after_attribute_name::step`.
Tests below are in `crates/html/tests/after_attribute_name_state.rs`.

| Pinned branch | Code effects | Focused test |
| --- | --- | --- |
| TAB, LF, FF, SPACE | Consume and ignore; stay here. CR normalization belongs to cursor. | `whitespace_solidus_equals_consume_without_touching_storage_even_without_current_attribute` |
| `/` | Consume; switch to Self-closing start tag without flag mutation or reconsumption. | Same test; `real_paths_finish_boolean_attributes_duplicates_and_abandon_unfinished_values` exercises the implemented destination. |
| `=` | Consume; switch to Before attribute value without modifying the current attribute. | Same source-step matrix, including detached-current identity and exact consumption; real Data tests now continue through value parsing/EOF abandonment. |
| `>` | Consume; switch to Data and call shared `tag::emit_tag`. | `emission_and_eof_use_selected_tag_helpers_preserving_unrelated_context` |
| EOF | Report `eof-in-tag` at conceptual EOF; call shared `tag::take_tag` to abandon the selected unfinished token; emit one terminal EOF. | Same test, plus `open_input_suspends_without_finalizing_or_losing_detached_current` |
| Anything else | `tag::start_attribute(Attribute::default())`; reconsume unchanged input in Attribute name. | `anything_else_creates_empty_attribute_clears_detached_current_and_reconsumes_literally` exhausts ASCII and tests Unicode/non-scalars. |

Before attribute name reaches this state on `/`, `>`, or EOF **without creating
an attribute**. Only a selector and its matching pending tag are required for
isolated entry here, not a current attribute. No value operation or current-value
lookup is needed. The `=` arm only switches; it does not accidentally select a
retained predecessor of a detached duplicate. Starting another attribute clears
the detached slot through the existing lifecycle helper. All actual future value
operations must still use `tag::current_attribute_mut`.

Emission tests cover both tag kinds, retained order/values, detached duplicate
exclusion (including detached-only storage), existing flag preservation, both
end-tag emission errors independently and together, and last-start-tag update
only for emitted start tags. EOF emits neither a tag nor emission errors. Full
snapshots check unrelated pending tags/comments, prior tokens/errors, return
state, scratch buffer, selector, input position, and last-start-tag preservation.
Source tests distinguish consumption from reconsumption; NUL replacement,
ASCII lowercasing, and quote/less-than errors occur in Attribute name, not here.
Surrogates, noncharacters, and controls retain exact preprocessing diagnostics
once at their original normalized positions. EOF and emission positions are also
checked after CRLF and a supplementary scalar.

`all_two_chunk_splits_and_single_points_match_complete_source_step_traces`
compares every source step, tokens, storage, errors, positions, and terminal
results with closed input, for all two-chunk splits and single-code-point chunks.
It covers both isolated tag kinds and real Data entry, CRLF/consecutive/trailing
CR, NUL, malformed input, duplicate names, non-scalars, EOF, and terminal stubs.
Open exhaustion repeatedly returns NeedMoreInput without finalizing anything.
A blocked execution is never reconstructed or resumed. Seeded value-bearing
storage tests are isolated support evidence, not reachable value-parsing paths.
Integration assertions inspect source effects before implemented destinations,
assert complete output or destination-state EOF abandonment, or continue to a
real Character reference block; no matching prefix is a complete pass.
Rebased tests for both quoted value sources preserve their pre-destination
snapshots, then compare exact emission, end-tag diagnostics, Data continuation,
suspension, and EOF. After quoted/unquoted whitespace recovery tests now verify
complete final snapshots for emission/selected-tag abandonment, duplicate
exclusion, new-attribute lifecycle, and unrelated-context preservation. Their
streaming traces continue through the newly implemented destination. Adapter
regressions also retain the formerly blocked whitespace/solidus inputs as exact
completion tests, rejecting missing output or diagnostics.

## Independently reviewed fixture identities

`crates/html/tests/baselines/after_attribute_name.json` adds 160 Data identities
(all group `tests`, initial-state index 0), reviewed from the pinned upstream
inputs, expected tokens, and diagnostics against the source above and existing
Attribute name/Self-closing start tag rules. The explicit reviewed selection is:

- `test2.test`: 38, 40, 41 (whitespace-only ending; new quote/apostrophe attributes).
- `test3.test`: 1217–1218, 1220–1221, 1223–1287, 1289–1316,
  1493–1512, 1520–1536, 1538–1550.
- `test4.test`: 0, 3, 24, 45, 46, 48, 49, 77, 78, 79.

In `test3`, these are exact whitespace versus name-character boundaries,
empty-valued attributes, ASCII-only folding, punctuation/NUL diagnostics,
control preprocessing diagnostics (columns 4/5/6), duplicate `a`/`A` removal
(column 7), self-closing flags, and malformed-solidus recovery (column 4).
The JSON surrogate pairs in the supplementary-name cases decode to scalar
U+100000; they are not `doubleEscaped` lone surrogates. `test4` additionally
checks selected-token abandonment at EOF (CR gives line 2 column 1), uppercase
end-tag attributes (error column 6), duplicate end attributes (both errors at
column 8), and EOF after Before/Attribute/After attribute name (columns 4/5/6).

PR #20 re-reviewed these same 160 identities against main `81eecf0`.
They remain protected unchanged in PR #21. Before attribute value now connects
the value states; [PR #21 evidence](tokenizer-before-attribute-value.md) separately
records newly admitted integration passes and a reviewed additional allowlist.
These identities are an explicit review allowlist, not an automatic blessing of
current output. All prior baseline files are unchanged; no exclusions or
aggregate-count substitution is used. The runner compares complete execution,
including diagnostics and terminal EOF, against unchanged upstream expectations.

## Remaining limits

Before attribute value, all three value states, and After attribute value
(quoted) now execute from Data. Character reference remains a terminal typed
stub, including in a detached duplicate's value. Failed destination dispatch
is a development limitation, not an HTML parse error. Many other tokenizer states and tree
construction are absent. Local branch/streaming tests and fixture inventory
checks are not upstream conformance runs. See the PR report for executed,
passed, failed, and blocked upstream totals and the required check results;
this work makes no full-conformance or browser-security claim.
