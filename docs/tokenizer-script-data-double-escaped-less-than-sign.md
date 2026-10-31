# Script data double escaped less-than sign state

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


## Current PR #37 integration update

On exact merged main **`b67a7ee7445965257d0733569ac3c4e4f0bde8c8`** (#43/#38/#39/#40/#42)
plus #37, isolated DoubleEscapeStart now enters the completed double-escaped loop
at exact `script` delimiters. All five merged algorithms, registrations, inventory
setup and test suites remain unchanged; new cross-state source snapshots live in
the DoubleEscapeStart suite. EscapedLessThanSign remains the real ordinary-entry
gateway stub. All checks pass: **339 Rust tests, 0 failed, 0 ignored**; upstream
**1,126 executed/passed, 0 failed, 5,927 blocked**, zero new passes.
See [#37's current integration evidence](tokenizer-script-data-double-escape-start.md#current-integration-after-merged-43-38-39-40-and-42)
for exact paths, hashes, checks and final-head location. Reports below are historical
for support boundaries and test totals; their original source evidence is preserved.

<a id="current-pr-42-integration-update"></a>

## Historical PR #42 integration update

On exact main **`76add983570aba0fb8fc769aa8ea4f2510d37965`** (merged #43/#38/#39/#40)
plus #42, DoubleEscaped completes the isolated double-escaped loop. All merged
production algorithms, registrations, inventory scratch setup and original
source-step evidence are preserved. No unmerged sibling is imported.

All source snapshots remain independent of destination effects. Fallback now
executes real DoubleEscaped on every ASCII/non-scalar point and its untouched
suffix, including less-than/hyphen transitions and exact NUL/EOF diagnostics.
Slash still clears scratch before DoubleEscapeEnd consumes the next name point.
All three real predecessors are now tested: DoubleEscaped, Dash and DashDash.
All nine local tests retain malformed-input and full split-streaming coverage.

The ordinary-entry EscapedLessThanSign gateway remains a genuine terminal stub.
All required checks pass: **329 Rust tests, 0 failed, 0 ignored**; upstream
**1,126 executed/passed, 0 failed, 5,927 blocked**, zero new passes. Fixtures,
pins, comparisons and baseline files/registrations are unchanged. See
[#42's current integration evidence](tokenizer-script-data-double-escaped.md#current-pr-42-integration-after-merged-43-38-39-and-40)
for exact source paths, all affected tests, counts and final-head location.
Coordinator independent final-head review is separate; this thread does not
merge or start subagents.

**Everything below is historical for support boundaries and test totals.**

<a id="current-pr-40-integration-update"></a>

## Historical PR #40 integration update

On exact main **`eecf0a3992d820f95990dcdca9201e1378e52a8f`** (merged #43/#38/#39)
plus #40, Dash is implemented without changing any merged production algorithm
or removing source-step assertions.

Both Dash and DashDash are now real immediate predecessors. The existing
predecessor source test runs both, preserving `<` emission, slash scratch reset,
suspension and EOF reconsumption snapshots. The Dash suite additionally checks
per-letter normalized scratch versus original output, malformed-input positions,
cycles and name-state EOF through this state's real algorithm.

DoubleEscaped and the ordinary-entry EscapedLessThanSign gateway remain genuine
stubs. All required checks pass: **319 Rust tests, 0 failed, 0 ignored**; upstream
**1,126 executed/passed, 0 failed, 5,927 blocked**, zero new passes. Fixtures,
pins and all baseline files/registrations remain unchanged. See
[#40's current integration evidence](tokenizer-script-data-double-escaped-dash.md#current-integration-after-prs-43-38-and-39)
for exact paths, tests, counts and final-head location. Coordinator final-head
review is separate; this thread does not merge or start reviewers.

**The #39 and older reports below are historical for support and test totals.**

## Historical PR #39 integration update

On main `98607c3a282cc8336c41312db042e8f87b8c2bcf` (merged #43/#38) plus #39,
DoubleEscapedDashDash is now a real immediate predecessor: it consumes/emits `<`
before this state's source step. The suite adds a full-snapshot predecessor test
for slash clearing/emission, open suspension and EOF reconsumption into the
remaining DoubleEscaped stub. Existing source tests and both merged production
algorithms are unchanged. Ordinary ScriptData still blocks at EscapedLessThanSign.
All checks pass: **310 Rust tests, 0 failed, 0 ignored**; upstream remains
**1,126 executed/passed, 0 failed, 5,927 blocked**, zero new passes/baselines.
See [#39's integrated evidence](tokenizer-script-data-double-escaped-dash-dash.md#historical-pr-39-report)
for exact integrated base, final-head location, current chain and test evidence.
The PR #38 report below is historical for predecessor support and test totals.

## Historical PR #38 integrated evidence after PR #43

PR #38 was rebased onto main **`542a5060d085bd546387e2781bbf10039fb2e197`**
(PR #43, DoubleEscapeEnd). The original standalone review head was
`2dc06ca9b2eb2880b8d40facb8363d5bc8f9596a` on common base
`39d772e31b98e6e955a2e00f546a8db3506eb1dd`; those original support and test
claims are historical, retained below. The exact final integrated review head
is recorded in the PR body. No other unmerged sibling was imported, and no
production algorithm in either state changed during integration.

Current support: `/` still clears scratch and emits only `/` in the assigned
source step. Its next step now executes real DoubleEscapeEnd rather than a stub.
That neighbor accumulates ASCII-lowercased scratch while emitting original
spelling, then either reaches real Escaped on a matching `script` delimiter or
stops at terminal DoubleEscaped on a mismatch/fallback/EOF. Anything-else in the
assigned state still reconsumes in terminal DoubleEscaped, without output or
state diagnostics. All three immediate predecessors remain stubs. Ordinary
ScriptData fixture entry still blocks earlier at EscapedLessThanSign.

All original source-step checks are retained separately from destination effects.
The exhaustive slash case now consumes the untouched `z` in DoubleEscapeEnd,
checks scratch/output/current input, suspends on open exhaustion, then reconsumes
EOF into the genuine DoubleEscaped stub. The empty-storage slash case likewise
executes the real destination's suspension and EOF transition. Two additional
focused tests exercise all 64 spellings of `script` at every delimiter (including
normalized CR), preserving original spelling versus normalized scratch in full
snapshots, and exact destination diagnostics through CRLF, two NULs, surrogate,
noncharacter, control and EOF. In the malformed-suffix case state errors occur
at 3:1/3:2 and 3:6; input diagnostics occur at 3:3–3:5, once each.

The streaming harness now records result plus full snapshot through every real
destination step, completion and genuine remaining block, stopping immediately
at terminal results. All two-chunk splits and single-point chunking cover matched
and mismatched names, delimiter boundaries, EOF after letters, CRLF suppression,
consecutive/trailing CR, malformed input, non-scalars, real Escaped hyphen cycles
and return to ScriptData, and later EscapedLessThanSign blocks. It never detects
support dynamically or resumes a terminal execution.

The merged DoubleEscapeEnd suite's obsolete immediate-predecessor stub assertion
now executes real DoubleEscapedLessThanSign, checking scratch reset and slash
emission before the next state, then each name letter, delimiter and actual
Escaped EOF diagnostic/token. Its earlier ordinary-entry block remains intact.
Module/dispatch, independent inventory and scope docs were resolved additively,
preserving all merged DoubleEscapeEnd registrations and scratch-context setup.
README and the neighbor's state-local evidence now identify the real boundary.

All CONTRIBUTING checks plus the explicit pinned runner pass on this integrated
revision: fmt check, warning-denying Clippy, workspace tests, warning-denying
rustdoc, `pinned_upstream_runner`, and `git diff --check`. **300 Rust tests pass,
0 fail, 0 ignored**, including eight tests in each of the two state-local suites.
These are not upstream conformance counts.

The integrated pinned report is unchanged: **6,827 fixture cases; 7,053 runs;
6,190 dispatch-attempted; 1,126 executed = 1,126 passed + 0 failed;
5,927 blocked**. Blocks: CharacterReference 4,692; MarkupDeclarationOpen 242;
EscapedLessThanSign 22; CdataSection 56; ProcessingInstructionOpen 52;
TokenVariants 858; XmlCoercion 4; PendingSpecReview 1. New upstream passes: **0**.
All prior fixtures, pins, baseline files and registrations remain unchanged.
No validation/shared-prerequisite blockers remain. Coordinator review and merge
are separate; this thread neither merges nor starts reviewers. Future merges
must extend only the remaining genuine stub assertions and rerun all checks.

## Historical standalone evidence

**Everything below describes the original standalone head, not current
integrated destination support or current Rust test counts.** Pinned branch
mapping and assigned-state source effects remain valid unchanged.

## Scope and immutable evidence

This change implements exactly `State::ScriptDataDoubleEscapedLessThanSign` in
`crates/html/src/tokenizer/script_data_double_escaped_less_than_sign.rs`.
Initial HEAD was verified as the required common reviewed base
`39d772e31b98e6e955a2e00f546a8db3506eb1dd` (main after PR #36).
No sibling implementation, checkout, dependency, storage/API redesign, tree
builder, automatic element-mode switching, JavaScript parsing or Comment-token
behavior is introduced. Existing Context and cursor infrastructure suffice.

Retrieved and read the immutable WHATWG HTML source revision
[`404b3eb14a7a936dddefb2d78e224dbeb01d2358`](https://raw.githubusercontent.com/whatwg/html/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source),
including:

- [Assigned section, lines 144093–144108](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144093-L144108),
  `#script-data-double-escaped-less-than-sign-state`: the complete two-branch algorithm.
- [Preprocessing and next/current input, lines 142560–142594](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142594):
  newline normalization, once-only surrogate/noncharacter/control diagnostics,
  and deferred NUL handling.
- [Conceptual EOF, lines 142602–142607](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142602-L142607):
  EOF takes anything-else here; exhausted open input is not EOF.
- [Tokenization, reconsumption, temporary/return storage and token emission,
  lines 143149–143191](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143149-L143191):
  reconsumption provides the current item instead of the next; scratch storage
  is independent of return-state context and pending tags.
- [Double escaped predecessors, lines 143995–144090](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143995-L144090):
  DoubleEscaped, DoubleEscapedDash and DoubleEscapedDashDash all consume **and
  emit** `<` before entering the assigned state. They are not implemented here.
- [Double escape end, lines 144111–144143](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144111-L144143):
  subsequent ASCII-normalized scratch accumulation belongs to that destination,
  not this state. The slash itself is emitted unchanged, not added to scratch.

## Branch-to-code/test mapping

| Consumed item | Pinned lines | Complete source-step effects | Focused tests |
| --- | --- | --- | --- |
| `/` U+002F | 144100–144102 | Clear temporary buffer; switch to DoubleEscapeEnd; emit exactly `/`. No reconsumption, tag creation or error. | Exhaustive ASCII/context snapshots; cursor probes; default entry; all chunk splits |
| Anything else, including EOF | 144104–144105 | Reconsume in DoubleEscaped. No emission, scratch/name mutation or state diagnostic. | Exhaustive ASCII/Unicode/non-scalar snapshots; cursor probes; open/EOF; all chunk splits |
| Exhausted open input | Existing incremental contract | `NeedMoreInput`; preserve state, current item, storage, diagnostics and output. | Repeated suspension, empty chunks and subsequent feeding |

`Execution::step` completes only the assigned source step and returns
`Progress::Continue(target)` before attempting its destination. Both destinations
remain terminal `UnimplementedState` errors, even when they would otherwise need
more input or handle EOF. EOF does not finish, emit an EOF token, or diagnose
`eof-in-script-html-comment-like-text` in this state. NUL is not replaced or
diagnosed here. The preceding `<` is never emitted a second time.

Slash clears even seeded non-scalar/NUL scratch data. Fallback preserves scratch
literally; it does not apply ASCII folding, append letters, or recognize `script`.
Selected and unrelated pending tags, names (including spelling), flags, ordered
retained and detached attributes, pending comment, last emitted start-tag name,
return state, prior tokens and diagnostics are preserved. Cursor preprocessing
retains its exact normalized code-point positions and multiplicities.

## Focused tests and entry boundary

`crates/html/tests/script_data_double_escaped_less_than_sign_state.rs` has six
local tests (not upstream conformance cases):

1. Every ASCII point under open/closed input and absent/Start/End selectors,
   plus Unicode letters/whitespace, BOM, literal U+FFFD, supplementary scalars,
   actual high/low surrogates, BMP/supplementary noncharacters and controls.
   Full source snapshots at normalized position 2:2 assert exact token ordering,
   all context, buffer initialization/preservation, and both diagnostic vectors.
   Every destination attempt is explicitly terminal and repeated attempts have
   no effects; no output prefix is treated as success.
2. Cursor-only probes **before any destination attempt** verify that slash is
   consumed, whereas fallback is reconsumed at its original position before the
   untouched suffix. Preprocessing errors occur once, not again on reconsumption.
   These probes do not execute a destination or resume a terminal execution.
3. Repeated open exhaustion with empty chunks after preconsumed `<`, with and
   without a CRLF prefix; explicit EOF then schedules conceptual EOF reconsumption
   at 1:2 or 2:2 with no state error/output. Cursor-only EOF probing precedes the
   terminal destination assertion.
4. Default storage needs neither a selected tag nor initialized scratch. Open
   entry can suspend and accept slash, or explicit EOF, without inventing storage.
5. Every two-chunk split (including empty chunks) and single-code-point chunking
   matches closed-input source and terminal snapshots. Inputs cover both branches,
   mixed-case slash suffix, malformed punctuation, NUL, surrogates/noncharacters,
   controls, CRLF, consecutive/trailing CR, and EOF. The harness stops immediately
   at a stub, without feeding later chunks, reconstructing or resuming it. Thus
   suffix processing/CRLF suppression inside a destination is **not** claimed;
   these are assigned-state source-step streaming checks only.
6. Real ScriptData entry on `<!--<script></script>` follows all implemented
   source steps and stops at the actual earlier EscapedLessThanSign stub, with
   exact `<!--` output and current `<` at 1:5. It never reaches this assigned state.

Isolated entry explicitly initializes the cursor after consumed `<` and includes
its already-emitted character token, matching all three predecessor contracts.
There is no real implemented immediate predecessor on this base. The independent
support inventory adds only this state and supplies that direct-entry setup
before input is finished; its existing effect-free stub checks still cover every
unimplemented predecessor and destination. No old destination-block assertions
needed updating outside the inventory.

## Validation and upstream accounting

All final required checks pass (exit 0):

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo test --workspace --locked pinned_upstream_runner -- --nocapture
```

The workspace suite passes **290 Rust tests**, 0 failed, 0 ignored, including the
six new focused tests and independent support inventory. An initial local
inventory setup attempt pushed `<` after `finish()` and failed with
`InputAlreadyFinished`; moving explicit direct-entry setup before finish fixed
that test setup. All final checks above were then run successfully.

The pinned fixture submodule was already initialized at
`c777c408b61078ea2eb4acefc2535f54dbc8b28a`; no fixture, pin, adapter, baseline or
registration was modified. Upstream report:

- 6,827 fixture cases; 7,053 case/initial-state runs; 6,190 dispatch-attempted.
- **Executed: 1,126; passed: 1,126; failed: 0; blocked: 5,927.**
- Blocks: CharacterReference 4,692; MarkupDeclarationOpen 242;
  EscapedLessThanSign 22; CdataSection 56; ProcessingInstructionOpen 52;
  TokenVariants 858; XmlCoercion 4; PendingSpecReview 1.

There are **zero newly passing upstream identities**: no supported fixture
initial-state name enters this isolated state, and reachable double-escape paths
remain blocked earlier. Prior protected identities and registrations are
unchanged. Consequently no new state-specific baseline file is warranted.
Inventory/local tests and attempted-but-blocked runs are not conformance passes.

Independently inspected unchanged `domjs.test` cases with descriptions
`</script> in script HTML comment - double escaped` and
`Incomplete end tag in script HTML comment double escaped`. Their inputs are
`<!-- <script></script> --></script>` and `<!--<script></scrip>-->` respectively;
expected outputs retain the internal double-escaped closing sequence literally
(the former later emits a final EndTag `script`), with no expected errors.
Both currently block at EscapedLessThanSign before this state's slash branch.
Their output expectations were read from fixtures, not inferred from execution.
No blocked prefix is compared or silently skipped.

## Later integration boundary

The coordinator owns later merging/rebasing after the user's signal. No sibling
changes were fetched, cherry-picked or merged here. Additive module/dispatch,
independent inventory and scope-list registrations may need additive conflict
resolution. When DoubleEscaped/DoubleEscapedDash/DoubleEscapedDashDash merge,
review real predecessor paths and `<` emission without changing this source
algorithm. When DoubleEscapeEnd or DoubleEscaped merges, terminal assertions
must be deliberately extended through the real destination, retaining source-step
snapshots, reconsumption/diagnostic checks and any later typed blocks. Rerun all
checks and the upstream report, then independently review any newly comparable
fixture identities rather than auto-blessing results.

No shared prerequisite or validation blocker remains. End-to-end double escaping
and full tokenizer conformance remain out of scope; passing this partial suite
makes no full-conformance or browser-security claim.
