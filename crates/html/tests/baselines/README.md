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

There is deliberately no updater. Add individually reviewed new pass identities
in state PRs; do not remove existing entries, infer expected tokens/errors from
execution, or widen exclusions to obtain a pass. New passes need not match an
old aggregate total, but every comparable mismatch fails. Specification/fixture
revision changes and any baseline removals require dedicated review.
Use the runner command and count definitions in
[docs/tokenizer.md](../../../../docs/tokenizer.md#fixture-runner).
