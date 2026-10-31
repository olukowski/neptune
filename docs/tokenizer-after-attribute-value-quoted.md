# After attribute value (quoted)

Scope: only `State::AfterAttributeValueQuoted`, implemented in
`crates/html/src/tokenizer/after_attribute_value_quoted.rs`; tests live in
`crates/html/tests/after_attribute_value_quoted_state.rs`. Initially integrated
on merged PRs #16 and #17, retaining both quoted value implementations and
prerequisite PR #15's shared storage/emission support. PR #19 rebases onto
`78e52d1e56183acb9730b200c8e5d0cd728030e6` (merged PRs #16/#17/#18)
and adds the isolated unquoted value state while preserving this algorithm.
The original After attribute value (quoted) implementation changed no adjacent
algorithm, fixture, shared storage API, or dependency. Neighbor-state tests/docs
follow this executable destination rather than expecting its former stub.

## Pinned evidence and branch map

Normative algorithm: immutable HTML Standard revision
`404b3eb14a7a936dddefb2d78e224dbeb01d2358`,
[After attribute value (quoted) state, lines 144402–144432](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144402-L144432)
(`#after-attribute-value-quoted-state`). Also read the pinned
[preprocessing/current-next-input rules](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142595),
[Tokenization creation, abandonment, emission and last-start-tag rules](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143174-L143215),
and [duplicate removal/current identity rule](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144218-L144229).
The standard, not another engine or generated expected output, supplies behavior.

Each row is a distinct match arm in the state-local `step` function:

| Consumed input | Code effect | Focused test(s) |
| --- | --- | --- |
| TAB, LF, FF, SPACE | Continue to Before attribute name; no reconsume, append, diagnostic, or emission. CR is already LF. | `whitespace_solidus_and_every_other_point_have_exact_source_effects_and_consumption`; `seeded_destination_paths_complete_or_block_without_accepting_prefixes` |
| `/` | Continue to Self-closing start tag, without reconsumption or setting the flag here. | Same tests; source snapshots preserve both initial flag values, and `/>` executes the existing destination's emission. |
| `>` | Shared `tag::emit_tag`; continue to Data. | `greater_than_uses_shared_emission_excludes_detached_duplicate_and_preserves_identity` |
| EOF | `eof-in-tag` at EOF; shared `tag::take_tag` abandons selected tag, then one terminal EOF. No tag emission. | `open_exhaustion_is_effect_free_and_eof_abandons_only_selected_tag` |
| Anything else | `missing-whitespace-between-attributes` at consumed point; schedule reconsumption in Before attribute name. No value/name mutation or attribute creation here. | Point matrix above; `missing_whitespace_precedes_attribute_creation_and_name_errors_without_reprocessing`; `equals_recovery_and_new_duplicate_keep_retained_values_untouched`; `later_name_input_diagnostics_are_not_source_errors` |
| Open exhaustion (execution contract, not spec EOF) | NeedMoreInput, no effects; subsequent feeding remains supported until terminal completion/block. | Open-exhaustion test and `all_two_chunk_splits_and_single_point_chunks_match_closed_source_step_traces` |

The point matrix covers every ASCII point except `>` (covered separately), Unicode
whitespace boundaries, non-ASCII letters/scalars, actual lone surrogates,
noncharacters, controls, and NUL, for both tag kinds and both stored flag values.
Positions are normalized code-point positions, including after a supplementary
scalar and CRLF. All existing retained values and detached duplicate identities
remain unchanged by the source. No value operation is needed by this algorithm;
there is no direct indexing/mutation of attribute values.

Recovery is inspected one real source step at a time: initial preprocessing and
missing-whitespace errors, Before attribute name's creation/clearing of the old
detached current, then Attribute name's lowercasing, NUL replacement, punctuation
errors, subsequent preprocessing, and duplicate detachment. Reconsumption never
repeats preprocessing. Equals recovery gets its separate Before attribute name
error. Retained predecessor values are never selected in place of a duplicate.

Shared emission preserves retained order, non-scalar values, existing flag and
name; detached-only storage is not an emitted attribute. End-tag attribute/solidus
errors occur once each at `>`, only if applicable. Start-tag emission alone updates
last-start-tag. Emission/EOF abandonment clear only the selected tag and selector;
unrelated pending tag/comment, return state, scratch buffer, prior output and
errors remain intact. Seeded storage is never preprocessed again.

Every two-chunk split (including empty ends) and single-code-point chunks compare
all source-step snapshots against closed input: state/current point and position,
output, both diagnostic streams, selected/unrelated storage, last-start-tag,
return state, and scratch buffer. Samples include every branch, CRLF splits,
consecutive/trailing CR, malformed names, duplicate recovery, non-scalars, emission
and later blocks. Suspension and repeated terminal steps must be effect-free.
This is state-local streaming evidence, not whole-tokenizer streaming conformance.

## Reachability, conformance, and remaining limitations

Both quoted value states now execute and transition into this state. Before
attribute value remains a stub, so no genuine Data path reaches these states yet.
Isolated contexts deliberately supply valid selected tags/attributes, including detached duplicates. They are not evidence
of reachable end-to-end quoted attribute parsing. Real `<a x='v'>` and end-tag
variants remain blocked at Before attribute value, even after a matching emitted
tag prefix. No blocked Execution is reconstructed or resumed.

After attribute name and Before attribute value remain typed terminal
`UnimplementedState` destinations. Whitespace followed by `>`/EOF reaches the
former, not direct emission/abandonment in this state. After emission, `&` still
blocks at Character reference and `<!` at Markup declaration open. Blocks never
become EOF, HTML parse errors, or prefix passes. Both quoted-value test suites
retain exact source snapshots and now verify subsequent whitespace recovery to
After attribute name, missing-whitespace diagnostics/reconsumption, or suspension
and EOF abandonment rather than treating this state as a stub.
`both_quoted_value_sources_reach_emission_or_eof_with_exact_retained_and_detached_values`
checks both quote sources, both tag kinds, retained/detached value mutation,
closing quotes, `>` / `/>` emission, exact end-tag diagnostics, and EOF abandonment
one source step at a time, including unrelated-context preservation.

The pinned upstream runner's implementation and rebased integration reports agree:
6,827 fixture cases; 7,053 runs; 6,190 dispatch-attempted; **657 executed,
657 passed, 0 failed, 6,396 blocked**. These are upstream conformance runs,
not the nine local focused test functions or the support/fixture inventory.
No newly passing upstream identities: all prior case-level baseline files remain
unchanged. No automatic blessing, exclusions, fixture edits, or baseline removals.
The fixture revision remains `c777c408b61078ea2eb4acefc2535f54dbc8b28a`.
Use the [report command and count definitions](tokenizer.md#fixture-runner)
to reproduce. Validation on this branch (all passed):

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked` (including all nine focused tests)
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`
- `cargo test --workspace --locked pinned_upstream_runner -- --nocapture`

No full-conformance or browser-security claim.
