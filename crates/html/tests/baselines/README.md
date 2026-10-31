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
at that baseline's original review, those terminated at RAWTEXT less-than sign,
then a stub. Later state-specific additions retain these original identities.

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

`rawtext_end_tag_open.json` adds the one independently reviewed RAWTEXT run:
`domjs.test`, `tests[28]`, “bad endtag (space before name)”, initial-state index 1.
The literal input/output `</ XMP>` takes the non-alpha fallback without errors;
see the [state-local review](../../../../docs/tokenizer-rawtext-end-tag-open.md#integrated-fixture-review).
The sibling RCDATA case 28 identity (entry index 0), its registration and every
prior baseline are unchanged. There are now 869 protected identities.

`rcdata_end_tag_name.json` adds 17 individually reviewed RCDATA identities:
`contentModelFlags.test` cases 2–11, `domjs.test` cases 27 and 29–31, and
`escapeFlag.test` cases 0, 1, and 4, all initial-state index 0. The
[state-local fixture audit](../../../../docs/tokenizer-rcdata-end-tag-name.md#integrated-fixture-review)
records their unchanged expected tokens/errors and pinned branch reasoning,
including literal EOF fallback versus tag-state EOF abandonment. The eighteenth
formerly name-blocked run (`escapeFlag.test` case 2) now blocks at Markup
declaration open and is not added. All 869 prior identities, files, and registrations
remain unchanged; the protected union now has 886 identities. No result-derived
baseline generation or policy weakening is used.

`rawtext_end_tag_name.json` adds 17 individually reviewed RAWTEXT identities:
`contentModelFlags.test` cases 2–11, `domjs.test` cases 27 and 29–31, and
`escapeFlag.test` cases 0, 1, and 4, all initial-state index 1. The
[state-local audit](../../../../docs/tokenizer-rawtext-end-tag-name.md#integrated-fixture-review)
records every unchanged expected output/error and the pinned branch reasoning.
The eighteenth formerly name-blocked RAWTEXT run (`escapeFlag.test` case 2)
now blocks at Markup declaration open and is not added. All 886 prior identities,
including every RCDATA name-state entry and registration, remain unchanged.
The protected union now has 903 identities; no result-derived baseline generation,
fixture/pin edits, comparison changes, or policy weakening is used.

`script_data.json` adds eleven individually reviewed Script data identities:
`domjs.test` cases 3 and 24 (initial-state index 3), and `test3.test` cases
0, 2, 4, 6, 8, 18, 66, 1584, and 1604 (initial-state index 4). They cover
NUL replacement/error, literal U+FEFF, empty input, whitespace, VT's preprocessing
error, literal `&`, case preservation, and supplementary scalar input. The
[state-local audit](../../../../docs/tokenizer-script-data.md#reviewed-fixture-identities)
records unchanged expected tokens/errors and pinned branch reasoning. All 903
prior identities/files remain unchanged (914 protected identities total).
The baseline is a reviewed subset, not all newly passing runs or a conformance claim.

`script_data_less_than_sign.json` adds exactly one independently reviewed identity:
`test1.test`, `tests[33]`, “< in script data”, Script data initial-state index 0.
Input/output `<test-->` is literal with no errors: less-than-sign fallback emits
`<` and reconsumes `t` in the existing Script data state; the remaining points
emit literally before EOF. The
[state-local integration audit](../../../../docs/tokenizer-script-data-less-than-sign.md#reviewed-upstream-identity)
records the unchanged fixture and pinned branches. All 914 prior identities and
baseline files remain unchanged (915 protected identities total). At that PR's
merge, end-tag-open and escape-start runs still blocked explicitly; no prefix
was counted as a passing run.

`script_data_end_tag_open.json` adds exactly one individually reviewed identity:
`domjs.test`, `tests[28]`, “bad endtag (space before name)”, Script data entry
index 2. The unchanged input/output `</ XMP>` is literal with no errors: SPACE
takes end-tag-open fallback, emitting `</` and reconsuming SPACE in Script data,
which emits the remaining input and EOF. The
[state-local fixture audit](../../../../docs/tokenizer-script-data-end-tag-open.md#integrated-fixture-review)
records all six affected upstream runs and this sole newly comparable identity.
All 915 prior protected identities/files remain unchanged (916 total at PR #28's
merge). Its five alpha runs then blocked at Script data end tag name;
escape-start blocks remain.

`script_data_end_tag_name.json` adds exactly five individually inspected identities:
`domjs.test` cases 27 and 29–31 (Script data entry index 2), and case 42 (entry
index 0). The [state-local fixture audit](../../../../docs/tokenizer-script-data-end-tag-name.md#integrated-fixture-review)
records each unchanged input/output and its pinned branch reasoning: mixed-case
appropriate emission; inappropriate `>`, SPACE and `/` literal recovery; and
absent start-tag history keeping tag-like input literal. All lack fixture errors
and `doubleEscaped`. No output or expectation was generated from execution.
All 916 previous identities/files remain unchanged and passing (921 protected
identities total). No escape-start block is accepted as a pass.

There is deliberately no updater. Add individually reviewed new pass identities
in state PRs; do not remove existing entries, infer expected tokens/errors from
execution, or widen exclusions to obtain a pass. New passes need not match an
old aggregate total, but every comparable mismatch fails. Specification/fixture
revision changes and any baseline removals require dedicated review.
Use the runner command and count definitions in
[docs/tokenizer.md](../../../../docs/tokenizer.md#fixture-runner).
