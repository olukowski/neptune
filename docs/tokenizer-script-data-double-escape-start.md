# Script data double escape start

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


## Current integration after merged #43, #38, #39, #40 and #42

The user authorized sequential integration with independent final-head review.
PR #37 is rebased onto exact main **`b67a7ee7445965257d0733569ac3c4e4f0bde8c8`**
(merged #43 DoubleEscapeEnd, #38 DoubleEscapedLessThanSign, #39 DoubleEscapedDashDash,
#40 DoubleEscapedDash, and #42 DoubleEscaped). Original standalone head
**`fcdb040b86fa7f1fda66e3de29dd5d04132190e3`** on common base
`39d772e31b98e6e955a2e00f546a8db3506eb1dd` is historical below. The exact final
review head is recorded in [PR #37](https://github.com/olukowski/neptune/pull/37)'s
body. Only origin/main was fetched: no unmerged sibling imports. The assigned
algorithm and all five merged production algorithms are unchanged. Registration,
inventory/direct-entry setup and scope documentation conflicts were resolved
additively, preserving merged support.

### Source and completed loop evidence

The immutable [assigned source, 143958–143991](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143958-L143991)
and [merged double-escaped loop, 143993–144142](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143993-L144142)
were reread. Exact `script` delimiters now execute real DoubleEscaped. Mismatches
and non-alpha fallback still enter Escaped; EOF remains reconsumed there. The
ordinary-entry EscapedLessThanSign gateway is still terminal, never simulated.

The assigned suite now has **ten tests**. Original exhaustive ASCII/non-scalar,
scratch-equality, spelling, context, consumption/reconsumption and source-step
snapshots remain intact. The obsolete DoubleEscaped terminal assertion after all
64 mixed-case spellings and all delimiters (including normalized CR) now asserts
real destination consumption of the untouched `TAIL`, exact EOF position/error,
and repeated terminal completion. No source snapshot was weakened.

Two new fixed integration tests extend this evidence:

- `real_double_escaped_loop_preserves_each_source_step_and_returns_via_escaped_to_script_data`
  checks every source-step snapshot of
  `ScRiPt>-a--\0<\0</X>--b</ScRiPt>-->\0\r\n` after isolated entry. It
  verifies normalized scratch versus original letters, Dash/DashDash returns,
  NUL replacement, LessThanSign no-output reconsumption, slash-only scratch reset,
  mismatched `x` scratch returning to DoubleEscaped, matched `script` returning
  to Escaped, and EscapedDashDash `>` returning to ScriptData. Exact normalized
  diagnostics are at 2:13, 2:15 and 2:35; the last NUL is still replaced by
  ScriptData. Ordinary ScriptData EOF at 3:1 adds no comment-like EOF diagnostic.
- `entered_double_escape_end_reconsumes_eof_nul_and_non_scalars_only_in_real_destination`
  traverses real DoubleEscapeStart → DoubleEscaped → LessThanSign →
  DoubleEscapeEnd. Even exact `script` scratch without a delimiter reconsumes
  EOF in DoubleEscaped, not Escaped. Source no-emission/no-state-error assertions
  precede actual destination NUL replacement, surrogate emission with once-only
  preprocessing diagnostic, literal ampersand recovery and suffix consumption.
  Open exhaustion versus explicit EOF, duplicate-reconsume rejection and exact
  terminal EOF/error multiplicity remain explicit.

All two-chunk splits and single-point chunks retain full snapshot comparisons
and now include matched/mismatched double-end cycles, CRLF delimiter boundaries,
less-than/name EOF, repeated NUL, direct DoubleEscapedDashDash `>` return to
ScriptData, and later genuine gateway blocks. No dynamic support checks, prefix
success, skipped comparisons, feeding after terminal results or reconstruction.
The five merged suites need no edits: none asserts a DoubleEscapeStart stub, and
all 45 merged focused tests retain their existing source/destination evidence.

### Current validation and remaining blocks

All CONTRIBUTING checks and the separate pinned runner passed (exit 0): fmt,
warning-denying Clippy, workspace tests, warning-denying rustdoc,
`pinned_upstream_runner`, and `git diff --check`.
**339 Rust tests passed, 0 failed, 0 ignored**, including ten assigned-state tests.
These local/inventory checks are not upstream conformance executions.

Upstream remains **6,827 fixture cases; 7,053 runs; 6,190 dispatch-attempted;
1,126 executed = 1,126 passed + 0 failed; 5,927 blocked**. New passes: **0**.
Block categories remain CharacterReference 4,692; TokenVariants 858;
MarkupDeclarationOpen 242; CdataSection 56; ProcessingInstructionOpen 52;
ScriptDataEscapedLessThanSign 22; XmlCoercion 4; PendingSpecReview 1.
The unchanged EscapedLessThanSign gateway still blocks all 22 relevant runs
before this state. No fixture, pin, adapter/comparison rule or baseline
file/registration/identity changed (930 protected identities).

No validation or shared-prerequisite blocker remains. Coordinator owns the fresh
independent final-head review and any later merge; this thread does not merge,
enable auto-merge or create subagents. Future gateway integration must preserve
source evidence and individually review newly comparable fixture identities,
then rerun every check and upstream report. Partial support is not full
conformance or a browser-security claim.

## Historical standalone scope and immutable evidence

**All following entry/destination support statements and test totals describe
the original standalone head, not the rebased integration above.**

This change implements **only `State::ScriptDataDoubleEscapeStart`**, in
`crates/html/src/tokenizer/script_data_double_escape_start.rs`, against reviewed
base `39d772e31b98e6e955a2e00f546a8db3506eb1dd` (main after PR #36).
No sibling implementation, dependency, API/storage redesign, fixture or pin change
is included. The existing temporary buffer, cursor, code-point tokens and
execution shell suffice.

The complete HTML source was retrieved from the raw GitHub URL for immutable
revision `404b3eb14a7a936dddefb2d78e224dbeb01d2358`, not the living HTML Standard
or another engine. Reviewed source and referenced definitions:

- [Assigned section, lines 143958–143991](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143958-L143991).
- [Predecessor contract, lines 143872–143892](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143872-L143892):
  EscapedLessThanSign clears scratch, emits `<`, and reconsumes an ASCII alpha
  here. That predecessor remains a stub in this PR.
- [Preprocessing/current/next input and conceptual EOF, lines 142560–142607](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142607).
  Cursor supplies CR/CRLF normalization and once-only preprocessing diagnostics.
- [Tokenization, reconsumption, temporary buffer and character tokens, lines 143149–143179](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143149-L143179).
- [Escaped destination, lines 143775–143802](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143775-L143802):
  replacement of reconsumed NUL and escaped EOF diagnostic/emission belong there,
  not to DoubleEscapeStart.
- HTML imports Infra's [ASCII upper alpha](https://infra.spec.whatwg.org/#ascii-upper-alpha),
  [ASCII lower alpha](https://infra.spec.whatwg.org/#ascii-lower-alpha),
  [ASCII alpha](https://infra.spec.whatwg.org/#ascii-alpha), and
  [normalize newlines](https://infra.spec.whatwg.org/#normalize-newlines)
  definitions. These linked definitions were read: upper alpha is U+0041–005A,
  lower alpha U+0061–007A; CRLF becomes LF, then remaining CR becomes LF.
  The algorithm/delimiters remain those explicitly stated by pinned HTML.

## Branch mapping

Focused evidence is in
`crates/html/tests/script_data_double_escape_start_state.rs` (eight local tests).

| Source branch | Handler effects | Focused evidence |
| --- | --- | --- |
| TAB, LF, FF, SPACE, `/`, `>` (143966–143975) | Compare entire scratch exactly to lowercase `script`; select DoubleEscaped if equal, Escaped otherwise; emit original normalized delimiter; preserve scratch | Exhaustive ASCII source snapshots; all six delimiters, CR normalization; 64 case spellings; exact-buffer mismatch matrix |
| ASCII upper alpha (143977–143980) | Append point + 0x20 to scratch, emit original point, stay | All 26 upper letters; 64 mixed-case spellings and every intermediate accumulation snapshot |
| ASCII lower alpha (143982–143985) | Append original point to scratch, emit original point, stay | All 26 lower letters; seeded scratch including NUL/surrogate; no early transition at complete `script` |
| Anything else (143987–143988), including EOF | Emit/diagnose nothing here; preserve scratch; reconsume in Escaped | All other ASCII points, Unicode letters/whitespace, NUL, controls, surrogates and noncharacters; explicit EOF scheduling and real Escaped continuation |

NeedMoreInput is a scaffolding suspension, not a specification EOF branch. Empty
chunks and repeated open exhaustion are effect-free, retaining current input and
position. Explicit finish after empty, partial and exact `script` scratch takes
the anything-else branch; Escaped then reports exactly one
`eof-in-script-html-comment-like-text` and emits one terminal EOF at the same
position. DoubleEscapeStart itself never reports a state parse error.

Scratch is **ASCII-normalized**, unlike the original-spelling scratch used by
end-tag-name states. Emitted letters retain their original spelling. Delimiter
comparison is exact whole-buffer equality, not prefix/suffix matching, Unicode
folding, or comparison with last-start-tag/pending-tag names. This handler does
not clear scratch on entry or departure. Direct callers must provide the desired
scratch; real predecessor entry would start empty and reconsume its first alpha.
No tag selector or pending tag is required. The support inventory explicitly
registers this state and empty direct-entry scratch setup.

Full snapshots preserve unrelated context: both pending tags, flags, retained and
detached attributes (including non-scalars), current selector (None/Start/End),
comment, last start tag, return state and prior token/error prefixes. Source-step
probes verify emitted delimiters/letters are consumed once and anything else is
reconsumed once, without duplicate preprocessing diagnostics. Real Escaped steps
separately assert exact NUL/error/EOF positions and multiplicities.

## Entry, streaming and terminal boundaries

Tests use isolated entry with explicit scratch and cursor history; one test
supplies the exact predecessor contract (empty scratch, prior `<` output,
current alpha scheduled for reconsumption) without executing or simulating a
stub. A separate real ScriptData → LessThanSign → EscapeStart → EscapeStartDash
→ EscapedDashDash → EscapedLessThanSign path verifies the actual terminal block
at the second `<` in `<!--<ScRiPt>tail`. It is not reconstructed or resumed.
There is no available implemented predecessor path into this assigned state on
the reviewed base.

On exact `script` plus a delimiter, source effects are asserted **before** the
next step reports typed terminal `UnimplementedState(ScriptDataDoubleEscaped)`.
Repeated terminal calls preserve all context and never emit substitute EOF or
consume the suffix. Nonmatching/fallback paths execute the real Escaped state;
later escaped `<` still blocks at EscapedLessThanSign. No dynamic support
detection, prefix-success comparison or skipped block is used.

Every two-chunk split and single-code-point chunking of representative inputs
matches every non-suspension closed-input source-step snapshot and final result.
Cases cover alpha accumulation, seeded/empty scratch, each delimiter, exact and
longer names, malformed script-like input, NUL, non-scalars, diagnostics, CRLF
splits, consecutive/trailing CR, reconsumed EOF, real Escaped/dash continuations,
and terminal destinations. Trace collection stops at the first terminal result;
no later chunks are fed to a blocked execution. These are local state checks,
not upstream conformance executions or whole-tokenizer streaming conformance.

## Validation and upstream evidence

All required checks passed (exit 0):

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo test --workspace --locked pinned_upstream_runner -- --nocapture
```

Workspace Rust tests: **292 passed, 0 failed, 0 ignored**, including eight focused
state tests. Inventory/local/synthetic tests are not upstream conformance runs.
The existing fixture submodule was already initialized at pinned
`c777c408b61078ea2eb4acefc2535f54dbc8b28a`; no setup/pin changes were needed.

Runner: **6,827 fixture cases; 7,053 runs; 6,190 dispatch-attempted;
1,126 executed = 1,126 passed + 0 failed; 5,927 blocked.** Block categories:

| Reason | Runs |
| --- | ---: |
| CharacterReference | 4,692 |
| TokenVariants | 858 |
| MarkupDeclarationOpen | 242 |
| CdataSection | 56 |
| ProcessingInstructionOpen | 52 |
| ScriptDataEscapedLessThanSign | 22 |
| XmlCoercion | 4 |
| PendingSpecReview | 1 |

The report is unchanged from the reviewed base: **zero newly passing upstream
runs**. Inspected unchanged `domjs.test` tests[10–12]: `<!--<script>-`,
`<!--<script>--`, `<!--<script>` expect literal output plus escaped EOF diagnostics
at 1:14, 1:15, 1:13 respectively. All still block at EscapedLessThanSign on this
base, before DoubleEscapeStart executes. Matching literal output prefixes are
not passes. No new baseline file/registration is warranted; all prior baseline
files and identities remain untouched. No full conformance/security claim.

## Later integration boundaries

Coordinator-owned later merging/rebasing must reconcile additive module/dispatch
and support inventory registrations, then rerun every check and upstream report.
When EscapedLessThanSign and DoubleEscaped merge, review moved terminal blocks
and newly comparable fixture identities explicitly; never weaken comparisons,
auto-bless baselines, or reconstruct terminal executions. No sibling changes are
imported here. No tree builder, element-mode switching, Comment tokens or
JavaScript parsing is introduced. This PR is for user review only: no merge or
auto-merge is requested.
