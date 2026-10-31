# Script data state

## Current final Script data integration (PR #45)

On merged main `022baca4abb8593974d29c018597f9d5678231a6` plus PR #45,
the complete Script data state family is reachable from explicit Script data
entry. EscapedLessThanSign is no longer a stub: escaped closing tags and the
full double-escaped loop execute, including returns to Escaped, Script data and
Data. This state's production algorithm and source-branch assertions are
preserved; tests now execute newly reachable destinations instead of stopping at
obsolete script-state blocks. Character reference/Markup declaration and other
unrelated unsupported states remain genuine terminal limitations.

See [PR #45's final integration and individually reviewed fixture audit](tokenizer-script-data-escaped-less-than-sign.md#final-integration-after-all-eight-siblings)
and `crates/html/tests/script_data_integration.rs` for current paths, source-step
effects, exact diagnostics, attributes/duplicates, nesting/case/delimiters and
chunk-splitting evidence. This remains a standalone tokenizer, not automatic
mode switching, tree construction, JavaScript parsing or full conformance.

**All support boundaries, counts and PR-local “current” labels below are
historical snapshots. Their normative branch evidence remains applicable.**


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

Normative source: WHATWG HTML revision
`404b3eb14a7a936dddefb2d78e224dbeb01d2358`, retrieved directly from
[the immutable raw source](https://raw.githubusercontent.com/whatwg/html/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source),
not another engine or the neighboring RCDATA/RAWTEXT implementations.
Neither this pin nor the fixture pin in [sources.toml](sources.toml) changes.

Inspected source sections and definitions:

- [Script data state, #script-data-state, lines 143338–143359](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143338-L143359):
  the complete four-branch algorithm, in source order.
- [Preprocessing, lines 142560–142611](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142611):
  input stream, surrogate/noncharacter/control diagnostics, NUL handling deferred
  to states, newline normalization, current/next input, and conceptual EOF.
- [Tokenization, lines 143149–143188](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143149-L143188):
  state transitions versus reconsumption, return state/temporary buffer,
  character/EOF output, tag creation defaults, and tree-builder interaction.
- [unexpected-null-character, lines 141479–141482](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L141479-L141482):
  the diagnostic definition; Script data's own branch requires replacement.

Only `State::ScriptData` is implemented in `script_data.rs`. Explicit entry uses
`Execution::new(State::ScriptData, context)`; the existing fixture adapter already
recognizes `Script data state`. There is no automatic element-based mode switch,
tree builder, JavaScript parsing, character-reference processing, or implementation
of any escaped-script state.
[Script data less-than sign](tokenizer-script-data-less-than-sign.md) and
[end tag open](tokenizer-script-data-end-tag-open.md), and
[end tag name](tokenizer-script-data-end-tag-name.md) are implemented separately
and reachable from this state's `<` branch.
Existing Context/cursor/execution/storage conventions are reused without adding
fields, dependencies, or shared algorithm abstractions.

## Branch mapping

| Consumed input | Pinned lines | Source effects / intended destination | Focused evidence |
| --- | --- | --- | --- |
| U+003C `<` | 143345–143346 | Consume once; switch to Script data less-than sign, without emission, diagnostic, or reconsumption. | `less_than_consumes_only_the_delimiter_before_destination_dispatch` |
| U+0000 NUL | 143348–143351 | Report `unexpected-null-character` at the current normalized position; emit U+FFFD, not the original NUL; stay in Script data. | `null_replacement_and_exact_normalized_diagnostics_preserve_context` |
| EOF | 143353–143354 | Emit one terminal EOF, without a parse error. | NUL test plus `empty_open_input_suspends_and_only_explicit_eof_finishes_once_without_creation` |
| Anything else | 143356–143357 | Emit the current code point unchanged; stay in Script data. `&` is literal, including malformed/valid-looking reference strings. | `anything_else_including_entity_like_text_and_non_scalars_is_literal` |

No branch reconsumes. `<` itself is not emitted or saved by this state and no
following point is consumed/preprocessed here. Source effects are inspectable
before the separately implemented `ScriptDataLessThanSign` destination executes.
That destination clears the buffer on `/`, emits `<!` on `!`, or emits `<` and
reconsumes anything else (including EOF) in Script data. Open exhaustion suspends
there without emitting. End tag open creates/selects an empty candidate on ASCII
alpha and reconsumes in end tag name; otherwise it emits `</` and reconsumes in
Script data, including EOF. End tag name normalizes ASCII names, preserves
original spelling, emits appropriate end tags into Data or recovers literally
with reconsumption in Script data. Terminal `UnimplementedState` still occurs at
escape start, never accepted as a matching prefix or recoverable parse error.
Literal fallback and EOF reconsumption can complete through the real Script data
handler. Repeated terminal checks retain the same result even if context is
inspected or input is subsequently closed; no execution is reconstructed after
a stub. Source effects and destination effects remain separately asserted.

Exhausted **open** input returns `NeedMoreInput` with no observable state effect,
including retaining the current item and position. Only explicit cursor `finish`
provides actual EOF. Empty chunks are not EOF. CR and CRLF normalization belongs
to the existing cursor, including CRLF split across chunks and trailing CR.
Positions count normalized code points, one-based, with EOF just after the last
point; these are the repository diagnostic coordinates, not byte/UTF-16 offsets.

Actual surrogate code points, noncharacters, and controls are emitted unchanged
by the anything-else branch. The cursor reports their preprocessing diagnostics
once at original normalized positions; only NUL is replaced here. A literal
U+FFFD or U+FEFF remains literal. Ordinary punctuation/letters are not interpreted
as JavaScript, case-folded, or converted through scalar-only strings.

Every branch preserves last-start-tag, return state (absent or seeded), selected
tag and both pending tags (including flags, retained and detached attributes),
pending comment, and temporary buffer. Prior tokens/errors remain ordered and
are only appended to as specified. No tag/comment is created, emitted, abandoned,
or reset by this state; existing creation defaults are unchanged.

## Focused and streaming checks

`crates/html/tests/script_data_state.rs` checks the four branches, exact tokens,
state and preprocessing diagnostics/positions/multiplicities, normalized current
input and EOF, consumed delimiters and untouched suffixes, repeated suspension,
repeated terminal EOF/stub results, and unrelated context preservation.

Every two-chunk split (including empty leading/trailing chunks) and single-point
chunking of representative inputs is compared against every closed-input source
step. Cases include literal entity-like text, CRLF and consecutive/trailing CR,
NUL, Unicode scalars, actual surrogates/noncharacters/controls, `<`, `<!`, `</`,
and malformed/literal fallback and unconsumed tails at terminal destinations.
Suspensions must be effect-free. The trace harness stops at a stub, never feeds/reconstructs/resumes
it to test a subsequent state.

`crates/html/tests/support/script_data.rs` checks existing adapter mapping,
independent initial-state entries, Data defaults/no automatic switch after
`<script>`, exact output and diagnostics (with deliberately wrong expectations),
double-escaped non-scalars, literal ampersands, and terminal blocks despite exact
prefix matches at escape start or later Data/attribute stubs. Closing tags and
literal name mismatch/EOF recovery compare complete output. Newly executable literal fallback
must compare complete output/errors, rejecting wrong prefixes and missing or
duplicate diagnostics. These synthetic/local tests and the independent support
inventory are **not** upstream conformance passes or whole-tokenizer streaming
conformance.

## Reviewed fixture identities

The pinned html5lib revision is `c777c408b61078ea2eb4acefc2535f54dbc8b28a`.
Unchanged upstream expectations were inspected before selecting these eleven
identities for `baselines/script_data.json`; identities are not inferred/blessed
from production results. The baseline is a small reviewed subset of new passes.
All previous 903 protected identities and baseline files remain intact.

| File / tests index | Initial-state index | Unchanged input / expected output and diagnostic review |
| --- | --- | --- |
| `domjs.test` 3 | 3 | Double-escaped U+0000 → U+FFFD; exactly `unexpected-null-character` at 1:1 (NUL branch). |
| `domjs.test` 24 | 3 | Double-escaped U+FEFF `foo` U+FEFF `bar` passes through literally, no errors (anything else). |
| `test3.test` 0 | 4 | Empty input; no fixture output/errors; production adds terminal EOF (EOF branch). |
| `test3.test` 2 | 4 | TAB → literal TAB, no errors (anything else). |
| `test3.test` 4 | 4 | LF → literal LF, no errors (anything else). |
| `test3.test` 6 | 4 | VT → literal VT; exactly `control-character-in-input-stream` at 1:1 (preprocessing plus anything else). |
| `test3.test` 8 | 4 | FF → literal FF, no errors (anything else). |
| `test3.test` 18 | 4 | `&` → literal `&`, no errors or Character reference transition (anything else). |
| `test3.test` 66 | 4 | `;` plus U+100000 → same two code points, no errors (anything else). |
| `test3.test` 1584 | 4 | `A` → uppercase `A`, no errors (anything else). |
| `test3.test` 1604 | 4 | U+100000 → same supplementary scalar, no errors (anything else). |

In cases 66 and 1604 the JSON surrogate pair `\uDBC0\uDC00` encodes a scalar;
`doubleEscaped` is absent. It is not two actual surrogate input points. Every
listed nonempty case reaches the EOF branch after its output. No `<`-containing
identity was newly comparable in the original Script data PR #30: less-than-sign
was then a stub. Subsequent less-than-sign integration adds exactly one reviewed
identity in its [separate baseline/audit](tokenizer-script-data-less-than-sign.md#reviewed-upstream-identity).
Upstream fixtures, existing reviewed identities, comparison rules, policy blocks,
and pinned revisions are unchanged. See [baseline policy](../crates/html/tests/baselines/README.md).

## Reporting and sibling integration

Use the [documented runner command and count categories](tokenizer.md#fixture-runner):
executed = passed + failed; dispatch-attempted includes blocked stubs. Report all
four categories separately in the PR. Remaining Script data destination blocks
are explicit, not skipped or passing cases; broader adapter/policy/state blocks
remain unchanged. No full conformance or browser-security claim follows.

Script data merged as independently reviewed PR #30, commit
`c9c43fd0676d5dc39035a8da9317328b6e06745b`. Less-than-sign PR #29 was then
user-directed to rebase onto that main revision. Its integration preserves this
algorithm and delimiter-only source boundary and executes real fallback/EOF paths.
Historically PR #28 then rebased onto merged PR #29 main
`3fc6b310a6ea3b3e229a795336eaca72218a14ca`. It added real end-tag-open execution,
exact source/destination boundaries (including suspension, creation, NUL and EOF),
and moved genuine stub assertions to then-unimplemented end tag name and escape
start. Its historical report/identity is retained in the
[end-tag-open audit](tokenizer-script-data-end-tag-open.md#historical-pr-28-checks-and-upstream-results).

PR #31 now rebases onto main `59bc9ec055074a04419af7fb7068f95a836ff13d` after
PR #28 merged, adding real end-tag-name execution and updating every obsolete
stub expectation across all four suites/adapters. Source-step assertions remain:
empty creation precedes normalization; name EOF emits a literal candidate before
Script data emits EOF. Escape start remains terminal; Character reference and
Markup declaration open still block after appropriate closes where applicable.
See the [current name-state integration report](tokenizer-script-data-end-tag-name.md#validation-and-current-limits)
for counts and the five independently reviewed newly comparable identities.
Never bypass terminal semantics, automatically bless results, or resume old
blocked executions. No shared prerequisite was missing for this state.
