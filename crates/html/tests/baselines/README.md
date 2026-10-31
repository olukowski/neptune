# Previously passing upstream runs

`passing.json` is a reviewed regression floor, not generated expectations or a
conformance claim. Each row is:

`[file, root_group, case_index, description, initial_state, initial_state_index]`

Indices are zero-based; an absent `initialStates` uses Data at index 0.
File/group/case index disambiguate repeated descriptions at the pinned revision;
initial state and its entry index disambiguate repeated initial-state runs.
Descriptions are retained literally (including trailing spaces) for review.
Missing or changed identities fail, rather than silently dropping protection.

Migration source: main `fc7ea75595d7ecff1e50890b46d3e94278811f39`, after PRs #8–#10.
All old explicit subgroup, identity, total, and reason assertions passed before
capturing these 555 distinct passing runs (503 Data, 52 PLAINTEXT). The migration
retains every one, including all earlier milestone groups. The fixture gitlink
remains `c777c408b61078ea2eb4acefc2535f54dbc8b28a`; normative provenance is in
[docs/sources.toml](../../../../docs/sources.toml).

State PRs may add a state-specific JSON file and register it in
`support/regression.rs`; all files use the same identity format and are checked
as one duplicate-free regression floor. Keep existing files unchanged.

`rawtext.json` adds independently reviewed RAWTEXT identities: literal entity-like
text (`contentModelFlags.test`), NUL replacement and literal U+FEFF (`domjs.test`),
and empty input, whitespace, punctuation (including `&`), case-preserved letters,
and supplementary scalar input (`test3.test`). The latter's JSON surrogate pairs
are scalar input, not the fixture's `doubleEscaped` convention. Only U+000B has a
preprocessing diagnostic in that group. No `<`-containing run is a new pass:
those terminate at RAWTEXT less-than sign, still a stub.

`rcdata.json` records reviewed RCDATA identities: `domjs.test` cases 3 and 24,
and `test3.test` cases 0–16, 20–66, and 1576–1604 (even indices only). The former
check NUL replacement with its exact error and literal U+FEFF. The latter check
empty input, literal whitespace/punctuation/letters/supplementary scalars, and
VT's preprocessing error. Case 18 (`&`) is deliberately absent: Character
reference is a terminal block, not a pass. See the
[RCDATA branch evidence](../../../../docs/tokenizer-rcdata.md).

`after_attribute_name.json` adds the 160 reviewed Data identities listed and
explained in [After attribute name evidence](../../../../docs/tokenizer-after-attribute-name.md#independently-reviewed-fixture-identities).
They cover boolean attributes, whitespace-only endings, EOF abandonment,
duplicate removal, end-tag diagnostics, and malformed-solidus recovery. Existing
baseline files remain unchanged. Its historical PR #20 report predates reachable
Data-to-value parsing; PR #21 now connects that path.

`before_attribute_value.json` adds eight individually reviewed Data identities:
`test3.test` cases 1317, 1320, 1321, 1323, 1324, 1326, and 1442, plus
`test4.test` case 4. These are missing-value `>` recovery (including each HTML
whitespace and CR normalization), not quoted/unquoted value implementations.
The two identical `<a a=>` descriptions are separate upstream case identities.
See the [branch and fixture review](../../../../docs/tokenizer-before-attribute-value.md).

`integrated_attribute_values.json` adds 42 explicitly reviewed Data identities
for PR #21's newly reachable valued-attribute integration. The
[review table](../../../../docs/tokenizer-before-attribute-value.md#combined-integration-review)
maps each selection to unchanged fixture tokens/diagnostics and pinned branches.
All 817 prior identities and the original eight PR21 identities remain intact;
the integrated protected baseline is 867 identities. Baseline additions are
**not** the same count as new passes: combined support admits 206 new passes
over main (including those eight), of which 42 further identities are protected.

`rcdata_end_tag_open.json` adds exactly one manually reviewed identity after
PRs #26/#25 make end tag open reachable: `domjs.test` tests[28], “bad endtag
(space before name)”, RCDATA initial-state index 0. Input `</ XMP>` takes
non-alpha SPACE fallback and emits the full string literally with no errors;
no name matching or tag creation applies. The
[state-local audit](../../../../docs/tokenizer-rcdata-end-tag-open.md#integrated-fixture-review)
explains this sole newly comparable run and the 18 remaining alpha/name-state
blocks. All previous 867 protected identities remain unchanged (868 total).

There is deliberately no updater. Add individually reviewed new pass identities
in state PRs; do not remove existing entries, infer expected tokens/errors from
execution, or widen exclusions to obtain a pass. New passes need not match an
old aggregate total, but every comparable mismatch fails. Specification/fixture
revision changes and any baseline removals require dedicated review.
Use the runner command and count definitions in
[docs/tokenizer.md](../../../../docs/tokenizer.md#fixture-runner).
