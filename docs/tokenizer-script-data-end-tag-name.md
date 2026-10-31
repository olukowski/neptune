# Script data end tag name state

## PR #36 integration update

The PR-local reports and earlier entry/stub descriptions below are **historical**.
On merged main `da69390807532870f62ac6e8c2e88bc2fe608ae4` plus PR #36,
EscapeStart now connects ordinary Script data entry through EscapeStartDash and
the escaped trio; `>` in EscapedDashDash returns to Script data. Escaped `<`
still stops terminally at EscapedLessThanSign; escaped end tags and double escaping
remain unsupported. This state's production algorithm and source assertions are
preserved. Tests now follow actual destinations, EOF diagnostics and later stubs.
See [PR #36's current integration/fixture audit](tokenizer-script-data-escape-start.md)
for the nine reviewed new passes and final counts; old reports below are not
current conformance totals.


## Immutable evidence and scope

Only `State::ScriptDataEndTagName` is implemented here, from WHATWG HTML
revision `404b3eb14a7a936dddefb2d78e224dbeb01d2358`. The source was retrieved
from the immutable raw-source URL and inspected, including these definitions:

- [Assigned algorithm, lines 143695–143738](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143695-L143738).
- [Preprocessing, current/next input and conceptual EOF, lines 142560–142607](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142607).
- [Reconsumption and temporary buffer, lines 143160–143168](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143160-L143168).
- [Tag creation defaults, creation versus emission, and independent end-tag emission errors, lines 143174–143210](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143174-L143210).
- [Appropriate end tag definition, lines 143213–143215](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143213-L143215):
  exact equality with the last **emitted** start-tag name; absent history means
  no appropriate end tag.
- [Predecessor context, lines 143661–143686](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143661-L143686):
  slash clears scratch; end tag open creates an empty end tag and reconsumes
  the first ASCII letter. These predecessor algorithms are inherited unchanged
  from the independently reviewed and merged PRs #30, #29 and #28.

Algorithm: `crates/html/src/tokenizer/script_data_end_tag_name.rs`.
Focused tests: `crates/html/tests/script_data_end_tag_name_state.rs`.
RCDATA/RAWTEXT code supplies storage/execution conventions, not normative evidence.
No shared storage/API, tag helper, production adapter, dependency, fixture,
pinned revision, other algorithm, tree-builder behavior, or automatic mode
switching is changed. Source-local tests and adapter regressions are integrated
across all four simple Script data states.

## Entry invariants and branch evidence

Fresh isolated entry requires `current_tag = Some(TagKind::End)` and matching
`pending_end_tag` storage. Its candidate name is normalized so far, while
`temporary_buffer` contains original spelling. Empty stores model the first
letter; seeded prefixes model later steps. `last_start_tag` is independent
optional context, never inferred from unrelated pending start-tag storage.
Default `Context` alone is not initialized tag-state entry. Tests explicitly
check creation defaults and valid initialized entry, preserved selection and
unrelated storage; this state neither creates nor resets a tag or scratch.
Synthetic seeded storage probes also verify literal/non-scalar preservation;
they do not claim such prefixes are reachable via ASCII accumulation.

| Source branch | Implementation and focused evidence |
| --- | --- |
| TAB, LF, FF, SPACE, 143703–143710 | If appropriate, consume and choose Before attribute name; otherwise literal fallback. All separators, plus preprocessed CR, are checked. No attribute creation or destination diagnostic occurs in the source step. |
| `/`, 143712–143715 | If appropriate, consume and choose existing Self-closing start tag. The source neither sets the flag nor diagnoses it. Inappropriate slash falls back. |
| `>`, 143717–143720 | If appropriate, consume, choose Data and reuse `tag::emit_tag`. Tests check ordered retained attributes, exclusion of detached duplicates (including detached-only storage), both independent emission errors exactly once at the normalized delimiter position, selected-storage take, and unchanged last-start history. Otherwise literal fallback. |
| ASCII upper alpha, 143722–143725 | Append point + 0x20 to candidate and original point to scratch. Every uppercase letter is checked step by step against seeded names and unrelated storage. |
| ASCII lower alpha, 143727–143730 | Append unchanged to both stores. Every lowercase letter is checked; no Unicode case folding. |
| Anything else, 143732–143736 | Emit `<`, `/`, then each scratch point in order. Leave candidate/scratch unchanged and schedule reconsumption in Script data. Exhaustive non-letter ASCII, inappropriate delimiters, punctuation boundaries, Unicode letters/whitespace, NUL, real surrogates, noncharacters and controls are checked. The mismatching point is not emitted by this state. |
| Actual EOF | Also anything else, even for a matching candidate. Emit only literal recovery and reconsume conceptual EOF; do not emit EOF, abandon the tag, or report `eof-in-tag` here. |
| Open exhausted input | Repeated effect-free `NeedMoreInput`, not EOF or a delimiter decision. Empty chunks do not change that. Input can resume until explicitly finished. |

Appropriateness tests distinguish absent history, exact match, case-distinct
history, shorter/longer names, Unicode-distinct names, and unrelated un-emitted
start storage. Synthetic seeded probes establish that comparison uses candidate
code points, not scratch or scalar strings. Non-scalar input keeps its cursor
preprocessing error and normalized coordinates; reconsumption must not repeat it.
Scratch itself is not input and is neither preprocessed nor NUL-replaced.

Matching paths exercise existing attribute/self-closing/Data destinations through
complete emission, malformed-solidus recovery, duplicate removal, NUL value
replacement and exact diagnostic positions. Destination EOF abandonment produces
`eof-in-tag`, unlike name-state EOF fallback. Snapshot assertions preserve unrelated
pending start/comment data, return state, last-start history and creation defaults.
Every split into two chunks, empty chunks and single-code-point chunks of
representative inputs compares every productive source-step snapshot, including
CRLF splits, repeated/trailing CR, malformed input, non-scalars, EOF and blocks.
These are local execution tests, not upstream conformance passes.

## Integrated Script data execution

PR #31 is user-directed to rebase onto main
`59bc9ec055074a04419af7fb7068f95a836ff13d`, after independently reviewed PRs
#30 (Script data), #29 (less-than sign), and #28 (end tag open) merged. Shared
module/dispatch and independent support-inventory conflicts are resolved
additively. All three inherited production algorithms remain unchanged; only
obsolete source comments are corrected. No unmerged sibling was imported or
cherry-picked.

All four state suites now execute the real destination instead of obsolete name
or Script data stub assertions. Source boundaries remain explicit: creation is
empty before reconsumption; normalization and original-spelling append occur only
on name steps; literal fallback precedes Script data reconsumption; only Script
data emits EOF after name-state EOF. Open input repeatedly suspends without
fallback. Source-local tests separately retain exact NUL/preprocessing diagnostics,
normalized positions, unconsumed tails, and selected/unrelated storage.

Four additional name-local integrated tests begin at **ScriptData**, not an
isolated name context:

- `real_script_entry_preserves_each_creation_accumulation_emission_and_data_boundary`
  snapshots every step from `<` through slash clearing, empty creation, each
  original/normalized letter, matching `>` emission and following Data NUL.
- `real_script_entry_completes_matching_mismatched_eof_delimiters_and_malformed_paths_exactly`
  independently specifies mixed/exact/absent/case-distinct history, partial/longer
  and Unicode mismatches, repeated candidates, literal ampersands, actual EOF
  fallback, whitespace and solidus handoffs, destination EOF abandonment,
  attributes/flag diagnostics, malformed solidus, NUL and real non-scalars.
- `real_script_close_selects_data_and_genuine_escape_reference_markup_blocks_stay_terminal`
  checks following Data tag creation/last-start updates and exact terminal
  escape-start, Character reference and Markup declaration open boundaries.
- `real_script_entry_every_split_and_single_point_chunks_match_all_integrated_steps`
  compares every productive source-step snapshot at every two-chunk split and
  single-point chunking, including CRLF, malformed input, non-scalars and blocks.

The adapter now requires complete matching-tag and literal mismatch/EOF results,
including exact diagnostics, and deliberately rejects truncated output and
missing/duplicate errors. Matching prefixes before genuine later stubs remain
blocked. No support-detection expectations, skipped tests or reconstruction after
a stub are introduced. Escape start and all other escaped-script states remain
terminal; a literal `<!` is not accepted as complete tokenization. Character
references after a matching close or in attributes and Data markup declarations
also remain explicit terminal blocks.

### Historical isolated evidence

The original isolated PR started from main `c8ec2da`, with **zero** new upstream
passes: Script data and its predecessors were still stubs. Its mismatch/EOF tests
then stopped terminally at Script data after inspecting source effects. That
historical report had 1,059 executed/passed, 0 failed and 5,994 blocked. Those
obsolete blocks are now replaced by real reconsumption/EOF behavior, never by
resuming an old blocked execution. All old baseline files and registrations are
preserved.

## Integrated fixture review

The pinned `domjs.test` entries were inspected directly, independently of
production output. These are the **five** formerly name-blocked Script data
runs from main after PR #28. All lack `errors` and `doubleEscaped`; absent errors
means none expected. `C(text)` means Character output, `E(name)` an EndTag.

| tests index / description | Script data initial-state index / history | Unchanged input → expected output; pinned reasoning |
| --- | --- | --- |
| 27, `lowercase endtags` | 2 / `xmp` | `</XMP>` → E(`xmp`). Uppercase accumulation lowercases only candidate; appropriate `>` emits and chooses Data. |
| 29, `bad endtag (not matching last start tag)` | 2 / `xmp` | `</xm>` → C(`</xm>`). Shorter candidate is inappropriate; emit original scratch, reconsume `>` literally, then Script data EOF. |
| 30, `bad endtag (without close bracket)` | 2 / `xmp` | `</xm ` → C(`</xm `). Inappropriate SPACE uses literal fallback, not attribute-state entry or `eof-in-tag`; Script data emits EOF. |
| 31, `bad endtag (trailing solidus)` | 2 / `xmp` | `</xm/` → C(`</xm/`). Inappropriate `/` is literal, not self-closing state entry; no flag/error. |
| 42, `HTML tag in script data` | 0 / absent | `<b>hello world</b>` → C(`<b>hello world</b>`). Opening markup is literal; it never emits a start tag or sets history. Closing `b` has no appropriate history and also recovers literally. |

Only these manually selected identities are added to
`baselines/script_data_end_tag_name.json`. All **916** previous protected
identities and their files remain unchanged and pass; the protected union is
**921**. Fixture/pin, adapter comparison and policy limitations are unchanged.

## Validation and current limits

All CONTRIBUTING checks and the explicit pinned runner passed; see the PR for
commands. Workspace tests: **247 passed, 0 failed**, including **15** name-local
tests. Fixture submodule remains pinned at
`c777c408b61078ea2eb4acefc2535f54dbc8b28a`.

- 6,827 fixture cases; 7,053 runs; **6,190 dispatch-attempted**.
- **1,117 executed = 1,117 passed + 0 failed; 5,936 blocked**.
- Delta from main `59bc9ec`: **+5 newly passing/comparable**, no new failures;
  blocked decreases by 5, dispatch-attempted unchanged.
- Blocked: CharacterReference 4,692; TokenVariants 858; MarkupDeclarationOpen 242;
  ScriptDataEscapeStart 31; CdataSection 56; ProcessingInstructionOpen 52;
  XmlCoercion 4; PendingSpecReview 1.

Inventory and local checks are not conformance passes; this remains partial
standalone tokenization, not full conformance or browser-security evidence.
There is no automatic mode switching, tree builder, JavaScript parser or escaped
script algorithm. No missing shared prerequisite or operational blocker remains.
The parent will freshly review the exact pushed revision before deciding whether
to merge. This branch starts no subagent/reviewer and neither merges nor enables
auto-merge. Future destination implementations require deliberate source-test
and independently reviewed baseline updates, followed by all checks again.
