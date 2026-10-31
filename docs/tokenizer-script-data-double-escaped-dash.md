# Script data double escaped dash state

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

All original Dash source snapshots are preserved. Actual DoubleEscaped now
handles the surrogate/NUL tails and exact EOF after direct or DashDash return;
LessThanSign reconsumption still occurs on its own separate source step.
The direct predecessor test now runs actual DoubleEscaped hyphen emission,
Dash NUL replacement and DoubleEscaped EOF instead of a stale predecessor stub.
Name EOF is reconsumed before destination diagnosis. All nine tests, including
malformed inputs and complete split-streaming matrices, remain covered.

The ordinary-entry EscapedLessThanSign gateway remains a genuine terminal stub.
All required checks pass: **329 Rust tests, 0 failed, 0 ignored**; upstream
**1,126 executed/passed, 0 failed, 5,927 blocked**, zero new passes. Fixtures,
pins, comparisons and baseline files/registrations are unchanged. See
[#42's current integration evidence](tokenizer-script-data-double-escaped.md#current-pr-42-integration-after-merged-43-38-39-and-40)
for exact source paths, all affected tests, counts and final-head location.
Coordinator independent final-head review is separate; this thread does not
merge or start subagents.

**Everything below is historical for support boundaries and test totals.**

<a id="current-integration-after-prs-43-38-and-39"></a>

## Historical integration after PRs #43, #38 and #39

The user authorized sequential integration. PR #40 is rebased onto exact main
**`eecf0a3992d820f95990dcdca9201e1378e52a8f`**, containing merged #43
(DoubleEscapeEnd), #38 (DoubleEscapedLessThanSign), and #39
(DoubleEscapedDashDash). The original standalone head
`61f97b257ee856a9116e15105d4d3941e7034877` and its common-base support/counts below
are historical. The final review head is recorded in
[PR #40](https://github.com/olukowski/neptune/pull/40)'s body. Only `origin/main`
was fetched; no unmerged sibling was imported. All merged production algorithms
and this state's original algorithm are unchanged. Module/dispatch/inventory/docs
conflicts were resolved additively, retaining merged scratch-entry setup.

### Actual integrated paths and source boundaries

The immutable branch evidence below remains normative. Dash emits `-` and now
executes real DashDash, or emits `<` and executes real LessThanSign. DashDash
emits repeated hyphens, returns to real ScriptData on `>`, and reaches real
LessThanSign on `<`. LessThanSign emits `/` and clears scratch before real
DoubleEscapeEnd. That neighbor accumulates lowercase ASCII scratch while emitting
original spelling, then enters real Escaped on exactly `script` plus a delimiter.
Mismatch and fallback (including conceptual EOF) still reach terminal
DoubleEscaped. No DoubleEscaped implementation or gateway is supplied.

`>`/NUL/literals in Dash still transition to terminal DoubleEscaped after their
own exact source effects; `>` is not DashDash's ScriptData-return branch. With
EOF after Dash's `-`, real DashDash reports/emits the escaped EOF. With EOF after
Dash's `<` or a subsequent `</ScRiPt`, the actual neighbor reconsumes EOF into
DoubleEscaped **without** emission or a fabricated escaped EOF error. Ordinary
ScriptData still blocks earlier at EscapedLessThanSign. Terminal runs are never
reconstructed/resumed and blocked output prefixes never pass comparison.

### Updated tests, including affected merged-state suites

All original Dash source snapshots are retained before destination execution.
Its exhaustive ASCII/Unicode/non-scalar test now checks the actual destination:
DashDash emits a surrogate tail once, whereas LessThanSign reconsumes it without
emission; both stop at genuine DoubleEscaped with exactly one preprocessing error.
Default entry now tests real destination suspension and the differing EOF paths.
The direct NUL/literal DoubleEscaped blocks remain literal typed assertions.

Dash has **nine focused tests**, including two added integration tests:

- Full source snapshots from both `-</ScRiPt>` and `</ScRiPt>`: scratch remains
  intact through Dash/DashDash/less-than emission, slash alone clears it, each
  name letter emits its original spelling while appending normalized scratch,
  and matching `>` reaches real Escaped. CRLF suppression, two NUL replacements,
  surrogate/noncharacter/control errors at 2:3–2:5, exact multiplicity, unrelated
  context, escaped single-/multi-hyphen cycles, return to ScriptData, re-entry
  through `<!--x-y-->`, and one ordinary EOF at 2:22 are all explicit snapshots.
  Scratch remains exactly `script`; ordinary EOF adds no escaped EOF diagnostic.
- Real name-state suspension after `</`, `</ScRiPt`, and `</scrip`, with/without
  the preceding DashDash hop. Explicit EOF is reconsumed into the remaining stub;
  even exact `script` scratch alone does not authorize an Escaped transition.

Every two-chunk split and single-point chunking now follows real destinations,
including matched/mismatched names, cycles, name EOF/NUL/non-scalars, CRLF,
consecutive/trailing CR, and later genuine blocks. Source checks are not replaced
by streaming self-comparisons or dynamic support detection.

Affected merged suites also execute the new predecessor without losing their
existing source assertions:

- DashDash's `>`/CRLF/malformed-input continuation runs from both original
  DashDash and new Dash entry, with exact initial source snapshots. Its full
  streaming suite runs from both entry states, including slash/name/scratch,
  escaped cycles, name EOF and later blocks. Normal-entry gateway assertions stay.
- LessThanSign's predecessor test now runs from both Dash and DashDash, checking
  one `<` emission, slash reset/emission, suspension and EOF reconsumption.
- DoubleEscapeEnd retains its isolated immediate-predecessor checks and adds
  actual Dash → LessThanSign → slash reset → per-letter `script` → matched
  delimiter → real Escaped EOF snapshots in the same test.

### Current validation and upstream result

All CONTRIBUTING checks plus the separate pinned upstream runner pass (exit 0):
fmt, warning-denying Clippy, workspace tests, warning-denying rustdoc, and
`git diff --check`. **319 Rust tests pass, 0 failed, 0 ignored** (nine Dash,
nine DashDash, nine LessThanSign, eight DoubleEscapeEnd). Local/inventory tests
are not upstream conformance cases.

Upstream: **6,827 fixture cases; 7,053 runs; 6,190 dispatch-attempted;
1,126 executed = 1,126 passed + 0 failed; 5,927 blocked**. Zero new upstream
passes; the unchanged gateway still prevents double-escaped fixture execution.
Blocked reasons: CharacterReference 4,692; MarkupDeclarationOpen 242;
ProcessingInstructionOpen 52; CdataSection 56; EscapedLessThanSign 22;
TokenVariants 858; XmlCoercion 4; PendingSpecReview 1. All previous baseline
files/registrations and the 930 protected identities, fixtures and pins are
preserved. No validation/shared-prerequisite blocker remains. This is still
partial support, not full conformance or a browser-security claim.

Coordinator gets an independent review of the final pushed base/head before any
merge. This thread does not merge, enable auto-merge, or start subagents. Later
integration must extend only genuine remaining blocks, preserve source effects
and registrations, independently review any new fixture identities, and rerun all
checks/report.

## Historical standalone evidence

**Everything below records the original common-base support and validation,
not current destination support, test names or counts.** The pinned assigned
algorithm and specification branch effects remain unchanged.

## Scope and immutable source

Implements exactly `State::ScriptDataDoubleEscapedDash` in
`crates/html/src/tokenizer/script_data_double_escaped_dash.rs`, on verified common
base `39d772e31b98e6e955a2e00f546a8db3506eb1dd` (main after PR #36).
Module/dispatch and independent support-inventory registration are additive.
No neighboring algorithm, dependency, shared storage/API, adapter, fixture, pin,
or previous pass baseline is changed.

Retrieved and read the immutable WHATWG HTML source revision
[`404b3eb14a7a936dddefb2d78e224dbeb01d2358`, lines 144024–144055](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144024-L144055),
**Script data double escaped dash state** (`#script-data-double-escaped-dash-state`).
The implementation is derived from that section, not another engine or the
moving living standard.

| Input / pinned source lines | Source-step effects | Result | Focused test evidence |
| --- | --- | --- | --- |
| U+002D `-`, 144031–144033 | Emit one hyphen character token. | Double escaped dash dash | `every_ascii_branch_and_non_scalar_has_exact_source_effects_before_destination_stub`; default direct entry |
| U+003C `<`, 144035–144037 | Emit one less-than-sign character token (unlike escaped dash). | Double escaped less-than sign | Same exhaustive/default tests |
| U+0000 NUL, 144039–144043 | Report `unexpected-null-character`; emit one U+FFFD, not NUL. | Double escaped | Same exhaustive/default tests; split CRLF |
| Conceptual EOF, 144045–144048 | Report `eof-in-script-html-comment-like-text`; emit exactly one EOF. | Finished, state unchanged | `open_exhaustion_preserves_current_item_and_only_explicit_eof_reports_once`; incoming EOF reconsumption; split CRLF |
| Anything else, 144050–144052 | Emit the consumed point unchanged. | Double escaped | Exhaustive ASCII/non-scalar and incoming reconsumption tests |
| Open exhaustion (existing incremental contract) | No output, diagnostics or current-item change. | NeedMoreInput, state unchanged | Open exhaustion; default entry; streaming |

All normative branches consume once and none requests reconsumption. `>` does
not return to Script data here; `&` is literal, not a character reference.
ASCII uppercase is emitted with original spelling, not lowercased; neither
scratch storage nor pending tag names are touched. Unicode/non-scalars remain
code points. No Comment token, JavaScript parsing, tree builder, or automatic
mode switching is introduced.

Referenced definitions read in the same immutable source:

- [Preprocessing, current/next input, conceptual EOF, 142560–142607](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142607):
  CR/CRLF normalize before matching; lone surrogates, noncharacters and relevant
  controls retain their original preprocessing diagnostics. NUL is state-specific.
- [Tokenization, reconsumption, temporary buffer and output, 143149–143191](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143149-L143191):
  reconsumption supplies the existing current item, without advancing the cursor.
- [EOF diagnostic, 141138–141149](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L141138-L141149)
  and [NUL diagnostic, 141479–141482](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L141479-L141482).
- [Immediate predecessor, Double escaped, 143993–144021](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143993-L144021):
  its hyphen branch would emit `-` and enter the assigned state; that predecessor
  remains a stub and is not simulated or implemented here.
- Real Script data entry evidence: [Script data, 143338–143359](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143338-L143359),
  [less-than sign, 143654–143673](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143654-L143673),
  and [escape-start/escaped states, 143741–143869](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143741-L143869).

## Entry, source boundaries and local tests

Direct entry requires only `Execution::new(State::ScriptDataDoubleEscapedDash,
context)`; there is **no required pending-tag, selector or scratch initialization**.
Default-context tests verify this independently of the support inventory's seeded
context. Additional isolated tests establish a preceding current item by consuming
only a cursor prefix, never by reconstructing an execution stopped at a stub.

On this base all three destinations (`ScriptDataDoubleEscapedDashDash`,
`ScriptDataDoubleEscapedLessThanSign`, `ScriptDataDoubleEscaped`) remain terminal
`UnimplementedState` errors. Tests inspect complete source effects before the
next step attempts the destination, then assert the literal typed error and
repeated effect-free terminality. Even exhausted open input or closed EOF at a
destination must block, not suspend or emit EOF. Direct cursor tail probes inspect
consumption only; they never resume blocked executions or compare output prefixes
as success. No dynamic support detection or weakened comparison is used.

`crates/html/tests/script_data_double_escaped_dash_state.rs` has seven focused tests:

- Every ASCII point plus Unicode, hyphen lookalikes, controls, lone surrogates and
  noncharacters, under open/closed input and absent/Start/End selectors. Exact
  snapshots assert token values, current input, normalized positions, diagnostic
  codes/multiplicities and preserved context. A surrogate tail stays unconsumed
  until an explicit cursor-only probe, proving no outgoing reconsumption.
- Default entry checks each destination with exhausted open and closed input;
  `<` is emitted, and `>`/`&`/uppercase remain literal.
- Repeated suspension with no current item or a prior current item, followed by
  explicit finish, tests EOF position, one EOF error/token and terminality.
- Incoming NUL/delimiter/non-scalar/control reconsumption (including conceptual
  EOF) retains current positions and once-only preprocessing diagnostics.
- Split CRLF suppresses the LF before source NUL or EOF at normalized 2:1.
- Real `<!--<script>-` entry is tested through Script data → less-than sign →
  escape start → escape start dash → escaped dash dash → escaped less-than sign.
  That last source consumes `<` without output and blocks before `s`; it cannot
  reach this assigned state. Direct immediate-predecessor entry likewise blocks
  before consuming `-`. No fake predecessor implementation is provided.
- All two-chunk splits (including empty chunks) and single-code-point chunks
  match closed-input source-step/terminal traces, both isolated and from real
  Script data entry. Inputs include every branch, malformed points, CRLF,
  consecutive/trailing CR and prefixes ending in CR. Traces stop at the first
  stub; since the assigned state leaves after one point, this is not a claim
  about streaming through unimplemented destinations.

Snapshots preserve last-start-tag history, return state, selector, pending start
and end tags (including retained/detached attributes and flags), pending comment,
original-spelling scratch points, and prior tokens/errors. No existing supported
predecessor reaches the assigned state, so no obsolete destination-stub assertions
elsewhere need replacement. Local/inventory tests are not conformance runs.

## Independent upstream evidence and integration boundary

The unchanged pinned fixture checkout is
`c777c408b61078ea2eb4acefc2535f54dbc8b28a`. Inspected `domjs.test` zero-based
cases provide independent evidence, but remain blocked at the earlier
`ScriptDataEscapedLessThanSign`:

- tests[6], `NUL in script HTML comment - double escaped`: the middle segment
  `<!--<script>-\u0000-->` expects U+FFFD and `unexpected-null-character` at 1:30
  (the other two NUL errors are at 1:13 and 1:48 in neighboring states).
- tests[10], `EOF in script HTML comment double escaped after dash`:
  `<!--<script>-` expects literal character output and exactly the escaped EOF
  error at 1:14.
- tests[11], `EOF in script HTML comment double escaped after dash dash`:
  `<!--<script>--` expects literal character output and escaped EOF at 1:15;
  the second hyphen belongs to this state's transition, but the eventual EOF
  belongs to its still-unimplemented destination.

These are not newly passing identities. There is no state-specific pass-baseline
file because there are zero new upstream passes; all previous baselines and their
registration are preserved. No upstream fixture or source pin is modified.

Nine states are independently owned on this common base. Later coordinator-led
integration must preserve additive module/dispatch/inventory/status registrations,
review newly executable predecessor paths and moved terminal blocks, deliberately
update destination tests without losing source-step assertions, and rerun every
check/report. Individually inspect any newly comparable upstream identities before
adding them to state-specific baselines. Do not import sibling implementations,
resume terminal executions, auto-bless aggregate counts, merge, or enable
auto-merge as part of this standalone state PR.

## Validation

All required checks passed on this base plus the assigned implementation:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo test --workspace --locked pinned_upstream_runner -- --nocapture
```

The pinned submodule was already initialized at the required revision; no update
was needed. Workspace result: **291 Rust tests passed, 0 failed, 0 ignored**,
including seven new focused tests and the independent support inventory. These
local tests are separate from upstream conformance execution.

The upstream runner was run separately before and after implementation. Both
reports: **6,827 fixture cases; 7,053 runs; 6,190 dispatch-attempted**.

| Category | Common base | This PR | Delta |
| --- | ---: | ---: | ---: |
| Executed | 1,126 | 1,126 | 0 |
| Passed | 1,126 | 1,126 | 0 |
| Failed | 0 | 0 | 0 |
| Blocked | 5,927 | 5,927 | 0 |

Executed = passed + failed. Blocked reasons: Character reference 4,692;
Markup declaration open 242; Processing instruction open 52; CDATA section 56;
Script data escaped less-than sign 22; unsupported token variants 858;
XML coercion 4; pending-spec review 1. Blocks are not passes, including the
independently inspected double-escaped cases described above.

There is no validation/infrastructure blocker. Predecessor/destination stubs are
intentional scope limits. Later integration and review belong to the coordinator;
this standalone PR does not merge or enable auto-merge. This remains a partial
tokenizer, not full conformance or a browser-security claim.
