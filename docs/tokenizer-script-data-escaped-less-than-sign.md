# Script data escaped less-than sign state

## Final integration after all eight siblings

The user authorized coordinator-managed sequential integration. This PR was
rebased onto exact merged main **`022baca4abb8593974d29c018597f9d5678231a6`**,
containing #43, #38, #39, #40, #42, #37, #44 and #41. Only `origin/main` was
fetched; no unmerged sibling checkout was imported. The original common base and
standalone review head (`a8fe92d0500e907efcd566b261b8a191e202232e`) below are
historical. The exact final review head is recorded in
[PR #45](https://github.com/olukowski/neptune/pull/45)'s body.

All eight merged production algorithms, module/dispatch registrations, independent
support-inventory entries and direct-entry setup are preserved. This gateway's
33-line algorithm is unchanged from its standalone review head. There is no
shared prerequisite or sibling correctness fix hidden in this integration.
Production changes relative to main remain just the new gateway, its registration
and a buffer-use comment. **Every Script data state is now reachable from explicit
Script data entry**, not just isolated entry. Data still does not automatically
switch modes when a script start tag emits. No tree builder, JavaScript parser,
Comment tokens, API redesign or new dependency is introduced.

### Current source-step and integration evidence

The immutable HTML source was re-read at revision
`404b3eb14a7a936dddefb2d78e224dbeb01d2358`, especially the complete
[escaped gateway/end-tag/double-escape algorithms, lines 143872–144142](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143872-L144142),
plus the input/reconsumption/creation and predecessor definitions cited below.

- `/` still consumes without emission/reconsumption and clears scratch; only the
  next EscapedEndTagOpen step creates/selects an empty End candidate. The name
  state reconsumes the original letter, stores ASCII-normalized names and
  original-spelling scratch, and emits only an appropriate tag into Data.
- Alpha still clears scratch, emits only `<`, and reconsumes the unchanged letter
  in DoubleEscapeStart. That state, not the gateway, appends normalized scratch
  while emitting original spelling. Only exact `script` plus a specified delimiter
  selects double escaping. DoubleEscapeEnd mirrors that matching to leave it;
  nested script-like text is not a nesting-depth counter.
- Fallback still emits only `<`, preserves unrelated context and scratch, and
  reconsumes in Escaped, including conceptual EOF. NUL replacement/diagnosis and
  escaped EOF diagnosis/emission remain destination-owned. Cursor preprocessing
  errors occur once, without replacing non-scalars or duplicating on reconsumption.

All seven gateway tests retain their original source snapshots; the former
terminal-destination test is now
`slash_and_alpha_destinations_execute_separate_source_steps_and_complete_exactly`.
It checks separate real creation/letter steps, open exhaustion, appropriate
emission, DoubleEscaped selection, NUL recovery and exact final context/errors.
Real predecessor tests likewise execute the destinations after asserting gateway
source effects. All ASCII/non-scalar, suffix/reconsumption and split-streaming
coverage remains intact.

Every obsolete ScriptData-family stub assertion throughout the state suites and
support adapter is replaced with real execution, without detecting support
dynamically or accepting output prefixes. Original predecessor/source boundaries
remain asserted. Real `CharacterReference` and `MarkupDeclarationOpen` blocks
still terminate without EOF or further effects, including after a complete escaped
or double-escaped path and appropriate close. Terminal executions are never
reconstructed/resumed. Existing simple fallback inputs also now require their
complete output and exact EOF diagnostics in the adapter, with deliberately wrong
outputs and missing/duplicate diagnostic negative checks.

`crates/html/tests/script_data_integration.rs` adds seven explicit-entry tests:

- Per-source creation/selection, normalized name versus original scratch, and
  whitespace departure; inherited attributes with retained versus detached
  duplicate identity/value; new-attribute detachment reset; unquoted value,
  self-closing end tag, exact emission diagnostics and following Data NUL.
- Exact emitted-history matching versus absent/case-distinct history; mismatch
  literal recovery and EOF, without implicit mode selection.
- Mixed-case double escaping, script-like nesting, partial/wrong names,
  malformed missing delimiters, every start/end delimiter (including CRLF),
  returns through Escaped and Script data into Data, and exact emitted text/tags.
- EOF after each gateway, end-tag/name, double-start/end, less-than and dash
  boundary, with exactly one destination-owned error/EOF token.
- Repeated NUL across DoubleEscaped, Escaped and Data (replacement versus original
  NUL), actual surrogate/noncharacter/control preservation, and exact normalized
  diagnostic positions/multiplicities.
- Genuine later unrelated terminal blocks, plus every two-chunk split and
  single-code-point chunking of representative integrated inputs, including CRLF,
  malformed code points, incomplete names, attributes/duplicates and self-closing
  emission. All non-suspension source snapshots match closed-input traces.

### Individually reviewed integration fixture identities

The unchanged fixtures remain pinned at
`c777c408b61078ea2eb4acefc2535f54dbc8b28a`. No fixture, gitlink, specification pin,
comparison or adapter policy is changed. Each identity below was inspected in its
original fixture and reasoned from the immutable branches before addition to
`baselines/script_data_escaped_less_than_sign_integration.json`. All are Script
data initial-state entry index 0. The two original fallback identities below
remain in their original file, with all prior baselines/registrations intact.

| File / zero-based tests index | Unchanged tokens/errors and pinned branch review |
| --- | --- |
| `domjs.test` 6 | `doubleEscaped` fixture decoding supplies NUL, not UTF-16 joining. Three `<script>` double-start matches reach DoubleEscaped, Dash and DashDash NUL replacement respectively; character text is otherwise literal. Only `unexpected-null-character` at 1:13, 1:30, 1:48. Each `-->` returns to Script data. |
| `domjs.test` 10 | `<!--<script>-` stays literal; DoubleEscapedDash owns the sole `eof-in-script-html-comment-like-text` at 1:14. |
| `domjs.test` 11 | `<!--<script>--` stays literal; DoubleEscapedDashDash owns that sole EOF error at 1:15. |
| `domjs.test` 12 | `<!--<script>` stays literal; DoubleEscaped owns that sole EOF error at 1:13. |
| `domjs.test` 16 | With `lastStartTag=script`, escaped `</script>` emits EndTag after `<!-- `, returns to Data; following ` -->` is text and the final ordinary close emits another EndTag. No errors. |
| `domjs.test` 17 | Double-start `script>` enters DoubleEscaped; the first `</script>` is literal and returns to Escaped. `-->` returns to Script data; the final close emits EndTag. One literal prefix `<!-- <script></script> -->`, no errors. |
| `domjs.test` 18 | Nested `<script>` in DoubleEscaped is literal, not another depth level. First close leaves DoubleEscaped; second emits an escaped EndTag. Expected prefix `<!-- <script><script></script>`, EndTag, text ` -->`, EndTag; no errors. |
| `domjs.test` 19 | DoubleEscapedDashDash `>` abruptly returns to Script data before the first close. Expected prefix `<!-- <script>-->`, EndTag, text ` -->`, EndTag; no errors. |
| `domjs.test` 20 | `<scrip>` scratch mismatch returns to Escaped. Expected literal `<!--<scrip>`, appropriate escaped EndTag, text `-->`; no errors. |
| `domjs.test` 21 | `<script` followed by `<` has no matching delimiter: DoubleEscapeStart reconsumes `<` in Escaped. Expected literal `<!--<script`, appropriate escaped EndTag, text `-->`; no errors. |
| `domjs.test` 22 | DoubleEscapeEnd `scrip>` mismatch stays DoubleEscaped. Later `-->` returns to Script data. Entire `<!--<script></scrip>-->` is one literal character output; no errors. |
| `domjs.test` 23 | `</script-->` has no delimiter after `script`; DoubleEscapeEnd reconsumes `-` in DoubleEscaped, then dash/dash-dash `>` returns to Script data. Entire input is literal, no errors. |
| `test1.test` 38 | `<!-- </ test -->`: gateway `/` clears scratch; EndTagOpen SPACE fallback emits `</`, reconsumes SPACE in Escaped. Entire input literal, `-->` returns to Script data; no errors. |
| `test1.test` 39 | `<!-- <test> -->`: DoubleEscapeStart `test>` mismatch returns to Escaped; entire input literal, no errors. |
| `test1.test` 40 | `<!-- </test> -->`: no emitted start-tag history, so normalized `test` is inappropriate and original spelling recovers literally. Entire input literal, no errors. |
| `test1.test` 41 | `<!--<script>-</script>-->`: DoubleEscapedDash `<` emits, LessThanSign `/` clears scratch/emits slash; DoubleEscapeEnd `script>` returns to Escaped. Entire input literal, no errors. |
| `test1.test` 42 | Same exact literal recovery/return with two hyphens, through DoubleEscapedDashDash `<`; no errors. |
| `test1.test` 43 | Same with three hyphens: extra `-` remains in DoubleEscapedDashDash before `<`; no errors. |
| `test1.test` 44 | `<!--<script> - </script>-->`: spaces are literal; post-hyphen SPACE returns to DoubleEscaped before the matched double-end. Entire input literal, no errors. |
| `test1.test` 45 | Same with two spaced hyphens: SPACE in DoubleEscapedDashDash returns to DoubleEscaped; matched double-end returns to Escaped. Entire input literal, no errors. |

Cases 16–23 have unchanged `lastStartTag=script`; case 6 alone uses fixture
`doubleEscaped`; cases 10–12 and all eight `test1.test` additions omit history.
Cases without errors listed above have no fixture `errors` field. No expectation
was generated from execution. All 930 pre-wave protected identities plus the two
original gateway identities still pass; the protected union is now **952**.

### Final checks and remaining limitations

All CONTRIBUTING checks and the separate pinned runner report pass. Workspace:
**379 Rust harness tests passed, 0 failed, 0 ignored**, including seven gateway
and seven new integration tests. Harness/inventory tests are not extra upstream
conformance runs.

Pinned upstream: **6,827 cases / 7,053 case-initial-state runs**;
**6,190 dispatch-attempted**; **1,148 executed = 1,148 passed + 0 failed**;
**5,905 blocked**. None block on a ScriptData-family state. Remaining categories:
Character reference 4,692; Markup declaration open 242; Processing instruction
open 52; CDATA section 56; unsupported token variants 858; XML coercion 4;
pending-spec review 1. Main before this gateway had 1,126 executed/passed,
0 failed and 5,927 blocked: exactly the 22 reviewed identities now complete.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo test --workspace --locked pinned_upstream_runner -- --nocapture
```

This is complete reachability of the Script data state family within a partial
standalone tokenizer, **not** automatic element-mode switching, a tree builder,
JavaScript parsing, browser security or full tokenizer conformance. Coordinator
owns independent review of the exact final pushed head and any merge. This thread
does not merge, enable auto-merge, or create a subagent.

## Historical standalone evidence

**Everything below preserves the original PR-local source/test/report record.
Its stub boundaries, test names, support claims and counts are historical, not
current integrated status. Normative branch evidence remains applicable.**

## Scope and immutable evidence

This PR implements exactly `State::ScriptDataEscapedLessThanSign`, in
`crates/html/src/tokenizer/script_data_escaped_less_than_sign.rs`, against reviewed
main `39d772e31b98e6e955a2e00f546a8db3506eb1dd` (after PR #36). Initial HEAD was
verified equal before implementation. No sibling changes are imported.

The complete HTML source was retrieved from the immutable revision
[`404b3eb14a7a936dddefb2d78e224dbeb01d2358`](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source),
not the living algorithm or another engine. Reviewed evidence:

- [Assigned section, lines 143872–143892](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143872-L143892),
  anchor `#script-data-escaped-less-than-sign-state`.
- [Input preprocessing, next/current input and conceptual EOF, lines 142560–142607](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142607).
  The existing cursor supplies newline normalization and preprocessing errors.
  Its referenced Infra [normalize newlines](https://infra.spec.whatwg.org/#normalize-newlines)
  definition replaces CRLF with LF and then remaining CR with LF.
- [Tokenization, reconsumption, temporary buffer, output and token creation, lines 143149–143189](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143149-L143189).
  Reconsumption provides the current item again instead of the next item; it is
  not an additional consume or immediate destination execution.
- [HTML's imported ASCII-alpha definitions, lines 2297–2299](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L2297-L2299)
  reference Infra [ASCII alpha](https://infra.spec.whatwg.org/#ascii-alpha):
  U+0041–U+005A or U+0061–U+007A, inclusive. Non-ASCII letters do not qualify.
- [Real escaped predecessors and fallback destination, lines 143775–143869](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143775-L143869).
  The escaped destination, not this gateway, replaces NUL/reports its error or
  reports `eof-in-script-html-comment-like-text` and emits EOF.

## Branch-to-code/test map

Focused evidence lives in
`crates/html/tests/script_data_escaped_less_than_sign_state.rs`.

| Consumed input | Required source effects | Focused tests |
| --- | --- | --- |
| `/` | Empty temporary buffer; switch to escaped end tag open. No emission, reconsumption, tag creation or selection. | `every_ascii_branch_and_non_scalars_preserve_exact_source_effects_and_spelling`; `slash_and_alpha_destinations_are_terminal_with_no_destination_effects` |
| ASCII alpha | Empty buffer; emit only `<`; reconsume in double escape start. Do not append, lowercase or emit the alpha. | Same two tests, including every upper/lower letter and unchanged original spelling |
| Anything else, including EOF | Emit only `<`; preserve buffer; reconsume in escaped. No source parse error, NUL replacement or EOF emission. | Exhaustive source test; `fallback_reconsumption_defers_null_handling_and_never_duplicates_preprocessing`; `open_exhaustion_and_reconsumed_eof_have_distinct_exact_effects` |
| Open exhausted input | Effect-free `NeedMoreInput`, not conceptual EOF. | Repeated suspension in the EOF test, real predecessor test and streaming trace |

The source snapshot tests cover all ASCII points and boundaries, punctuation,
non-ASCII letters/whitespace, NUL, replacement character, supplementary scalars,
actual lone surrogates, noncharacters and controls. They assert current input,
normalized positions, intended target, exact output/error prefixes, buffer
clearing versus preservation, suffix consumption and reconsumption. Full storage
snapshots preserve the selected tag, both pending tags (including retained and
detached attributes/flags), pending comment, last emitted start tag, return state,
and existing tokens/diagnostics. No pending token is created, abandoned or emitted.

`malformed_fallback_stream_has_exact_normalized_diagnostic_positions_and_multiplicities`
checks repeated NUL, CRLF and CR, surrogate/noncharacter/control errors, a later
escaped `<`, and EOF with independently specified complete tokens and exact
one-based normalized diagnostic coordinates. Reconsumption adds no duplicate
preprocessing diagnostic. Repeated terminal calls cannot add errors/output.

Direct entry explicitly consumes the predecessor's `<` in cursor setup before
constructing the execution; it initializes no fake tag or destination algorithm.
The independent support inventory registers the state additively and supplies
this setup for open exhaustion and EOF. `all_real_predecessors_reach_the_gateway_before_its_source_step`
executes Script data → escape start → escape start dash and each real
escaped/escaped-dash/escaped-dash-dash entry, preserving predecessor snapshots
before testing gateway effects and destination blocks.

`every_two_chunk_split_and_single_point_chunking_match_every_source_snapshot`
compares every non-suspension source-step snapshot against closed input for direct
entry and all three real predecessor paths. It covers every two-chunk split,
including empty chunks, and single-code-point chunks: CRLF boundaries,
consecutive/trailing CR, malformed text, NUL/non-scalars, EOF and both terminal
stubs. Traces stop immediately at terminal execution; none reconstructs or resumes
it. Existing predecessor tests retain their source boundaries and now inspect
real fallback or the later terminal block instead of expecting this state's stub.
Local tests and inventory checks are not upstream conformance runs.

## Independently reviewed upstream identities

Fixture revision remains `c777c408b61078ea2eb4acefc2535f54dbc8b28a`; the submodule
was already initialized at that pin. No fixture or specification pin is modified.
Two newly comparable identities are added to the state-specific baseline file,
registered without changing any prior baseline or registration:

| File / zero-based case | Description / unchanged input and output | Pinned branch reasoning |
| --- | --- | --- |
| `domjs.test`, `tests[14]` | “Dash less-than in script HTML comment”; `<!-- -< -->` | Real escaped dash consumes `<`; this gateway consumes SPACE, emits `<`, and reconsumes SPACE in escaped. The later `-->` returns to Script data, which emits EOF. |
| `test1.test`, `tests[37]` | “< in script HTML comment”; `<!-- < test -->` | Real escaped consumes `<`; this gateway takes the same SPACE fallback. Remaining text emits literally; `-->` returns to Script data before EOF. |

Both have exactly one Script data initial-state entry (index 0), no
`lastStartTag`, no `doubleEscaped`, and no expected diagnostics. Output is one
Character string identical to input; terminal EOF is omitted only by the existing
fixture format adapter. Expected results above were inspected in unchanged
fixtures and reasoned from the pinned branches, not generated from execution.
The protected baseline grows from 930 to 932 identities.

## Validation and remaining boundaries

All CONTRIBUTING checks and the separate upstream report passed:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo test --workspace --locked pinned_upstream_runner -- --nocapture
```

The workspace Rust harness passed 291 tests, with zero failed/ignored, including
seven focused state tests and the upstream runner test. Harness/inventory counts
are distinct from the following conformance-run counts.

Upstream: 6,827 fixture cases / 7,053 case-initial-state runs;
6,190 dispatch-attempted; **1,128 executed = 1,128 passed + 0 failed**;
**5,925 blocked**. The verified common base had 1,126 executed/passed, zero
failed and 5,927 blocked. All 930 prior protected identities still pass.

Blocked categories: Character reference 4,692; Markup declaration open 242;
Processing instruction open 52; CDATA section 56; Script data double escape
start 17; Script data escaped end tag open 3; unsupported token variants 858;
XML coercion 4; pending-spec review 1. Of the base's 22 escaped-less-than-sign
blocks, two now pass and twenty reach the two explicit terminal destinations.
No blocked prefix is counted as success. This remains partial conformance,
not full tokenizer conformance or a browser-security claim.

There is no missing shared prerequisite. Production additions are this module,
dispatch registration and a buffer-use comment; no storage/API redesign,
helper framework, dependency, tree builder, automatic mode switch, JavaScript
parsing or Comment-token behavior is introduced. The other eight sibling states
remain untouched and unimplemented on this standalone branch.

Later sibling merges/rebases must preserve additive module/dispatch/inventory and
baseline registration, then review moved terminal assertions and newly comparable
fixtures, rerunning every check and the upstream report. Escaped end-tag-open and
double-escape-start source effects must not be attributed to this gateway. No
sibling checkout is fetched, cherry-picked or merged here. Integration and merging
are left to the coordinator after the user's signal; this PR is review-only.
