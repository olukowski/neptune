# Script data escaped dash state

## Scope and pinned evidence

Implements only `State::ScriptDataEscapedDash` in
`crates/html/src/tokenizer/script_data_escaped_dash.rs`, from WHATWG HTML revision
[`404b3eb14a7a936dddefb2d78e224dbeb01d2358`, source lines 143805–143835](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143805-L143835),
anchor `#script-data-escaped-dash-state`. The immutable raw source was retrieved
and inspected, not inferred from another implementation. Historical standalone
baseline: main `28131e11437941928eb7246885fb70e1160b2700` (merged PR #31);
the assigned state was a stub. PR #34 rebased onto main
`e2a9b926938d022820613c752b467e16130521bc` (merged PR #32, Script data escaped).
PR #34 added only the assigned state's production algorithm, inheriting PR #32
unchanged. PR #33 subsequently rebases onto merged PRs #32/#34 at
`04283260b9d467d198ab92cb8ccd5721c51fe21e`; neither inherited production algorithm
changes.

| Input | Source-step effects | Result |
| --- | --- | --- |
| `-` U+002D (143813–143815) | Emit one `-` character token. | Script data escaped dash dash |
| `<` U+003C (143817–143818) | No emission. | Script data escaped less-than sign |
| NUL U+0000 (143820–143824) | Report `unexpected-null-character`; emit one U+FFFD character token. | Script data escaped |
| Conceptual EOF (143826–143829) | Report `eof-in-script-html-comment-like-text`; emit one EOF token. | Finished; state unchanged |
| Anything else (143831–143833) | Emit the consumed code point unchanged. | Script data escaped |
| Open input exhaustion | No effects, no errors or EOF. | NeedMoreInput; state unchanged |

All normative branches consume once; none schedules reconsumption. `>` and `&`
are ordinary characters here, not a return to Script data or a character reference.
The last row is the existing incremental contract, not an extra spec branch.
`Execution::step` exposes completed source effects before destination dispatch.
Historically all three non-EOF destinations were terminal stubs on the standalone
baseline. After PR #34 rebased, Script data escaped was implemented by
merged PR #32: it consumes subsequent characters, replaces/diagnoses NUL, suspends
on open exhaustion, and reports/emits its own conceptual EOF on explicit finish.
It may return through `-` to this state. PR #33 now implements Script data escaped
dash dash, consuming its own point after this state's `-`: hyphens stay there,
NUL/literal points return to Escaped, and `>` returns to Script data. Only Script
data escaped less-than sign remains a **terminal `UnimplementedState`**
destination here, even with exhausted open input. No substituted EOF, recoverable
parse error, adjacent state implementation, or resumption after a stub is supplied.

Referenced definitions inspected in the same pinned source:

- [Preprocessing and next/current input, 142560–142594](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142594):
  newline normalization precedes tokenization; surrogates, noncharacters, and
  non-whitespace/non-NUL controls have preprocessing diagnostics. NUL handling
  is state-specific. Non-scalars are retained as code points, not scalar strings.
- [Conceptual EOF, 142602–142607](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142602-L142607).
- [Tokenization, reconsumption, and token emission, 143149–143191](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143149-L143191):
  current and next input are distinct; incoming reconsumption does not advance
  preprocessing. Character tokens are emitted, not stored as pending tags.
- [EOF error definition, 141138–141149](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L141138-L141149)
  and [NUL error definition, 141479–141482](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L141479-L141482).

The implementation reuses Context, Cursor, character tokens, diagnostics, and
Execution. No dependency, storage, shared framework, fixture adapter, policy, or
pin changes. Existing output and diagnostics are appended to, not overwritten.
Pending tags/attributes/detached duplicates, selector, pending comment, temporary
buffer, return state, and last emitted start-tag name remain untouched.

## Local branch and streaming checks

`crates/html/tests/script_data_escaped_dash_state.rs` contains seven focused tests:

- Every ASCII code point plus representative Unicode, actual lone surrogates,
  noncharacters, and controls: exact tokens, state, current input, diagnostic
  codes/normalized positions/multiplicities, preserved storage, absent/Start/End
  selectors, and open/closed input. The source snapshot preserves a surrogate
  tail untouched. After NUL/anything-else, the real Script data escaped destination
  consumes that tail exactly once with its original code point and preprocessing
  diagnostic/position, then suspends on open input or emits one EOF/error on finish.
  After `-`, PR #33's real DashDash consumes the same surrogate tail, transitions
  to Escaped, then suspends or emits EOF/error. Original source snapshots remain
  intact. After `<`, the remaining stub is asserted literally and repeatedly; a
  direct tail probe only inspects its unconsumed input, never resumes it.
  No expectation is selected by dynamic support detection.
- Empty open input and exhaustion after a prior current item suspend repeatedly;
  only explicit finish yields the EOF diagnostic and one terminal EOF. Positions
  include supplementary characters and CR/CRLF prefixes. Finished execution is
  effect-free on repeated steps.
- Default-context entry requires no pending storage. The real Escaped or DashDash
  destination suspends on open exhaustion, handles a newly fed NUL at 1:2 with
  exact error multiplicity (including a prior source NUL), then consumes `<` and
  stops at the remaining less-than-sign stub. Open destination stubs do not become
  suspension; feeding/finishing cannot clear terminal status.
- Incoming reconsumption of NUL, surrogates, noncharacters, controls, delimiters,
  and conceptual EOF retains positions and once-only preprocessing diagnostics.
- Split CRLF independently checks suppressed LF followed by source NUL or EOF at
  2:1. After source NUL, the real Script data escaped step reports/emits EOF at
  2:2; both source and destination snapshots are exact.
- Real Script data entry still blocks at earlier Script data escape start after
  emitting `<!`, without consuming the first `-` or reaching this state.
- Every two-chunk split (including empty chunks) and single-code-point streaming
  matches closed-input source-step/terminal snapshots for every branch, malformed
  input, CRLF/consecutive/trailing CR, NUL, Unicode/non-scalars, and EOF, including
  a prefix ending with CR. Integrated traces now execute Script data escaped and
  repeated `EscapedDash`/`Escaped`/`EscapedDashDash` cycles, including repeated NUL
  and CRLF splits, `>` returning to Script data, and later `<!` escape-start blocks.
  `-<` and `--<` retain explicit terminal escaped-less-than-sign stubs. Trace collection stops immediately at a stub and never feeds,
  reconstructs, or resumes it.

Isolated entry uses the existing cursor directly to establish a preceding current
item; it does not simulate an earlier state's algorithm or resume a blocked run.
Historically no focused test on the original common baseline assumed this state
was a stub. Merged PR #32 added such an assumption: its destination test is
updated deliberately to execute the real escaped-dash step on a surrogate, then
Script data escaped's NUL handling, suspension, and EOF. Its original source-step
snapshots and remaining less-than-sign block are preserved. Its streaming inputs
now exercise real return paths. PR #33 additionally replaces this suite's two
old dash-dash stub expectations with exact real destination snapshots, keeping
the source assertions and less-than-sign blocks. All three suites' streaming
inputs now exercise complete cycles and later remaining blocks. The
independent inventory adds only this state alongside the inherited registration.
Earlier escape-start assertions remain unchanged. These local tests and inventory
checks are not upstream conformance cases.

## Upstream and validation

The pinned fixture submodule is initialized at
`c777c408b61078ea2eb4acefc2535f54dbc8b28a`; no upstream fixture is modified.
Original standalone baseline and merged PR #32 baseline each report:
6,827 fixture cases, 7,053 runs, 6,190 dispatch-attempted;
**1,117 executed = 1,117 passed + 0 failed; 5,936 blocked**.
Relevant unchanged upstream evidence inspected: `domjs.test` zero-based cases
5 (the middle NUL after a dash expects U+FFFD and `unexpected-null-character`
at 1:22), 8 (`<!--test-` expects literal output and the EOF error at 1:10),
13 (`<!-- - -->`, literal dash/space output), and 14 (`<!-- -< -->`, literal
output through escaped less-than). These are independent fixture evidence, not
new comparable passes: all still block at earlier Script data escape start.

There are 31 blocks at Script data escape start. That earlier state prevents
reaching the assigned state from the runner's supported entry states. No new
upstream pass identities are expected or auto-blessed; prior baselines remain
unchanged.

After PR #34's rebase/integration, the separately executed upstream runner reported
the same 7,053 runs (also unchanged from the historical standalone PR #34 report):
**1,117 executed, 1,117 passed, 0 failed, 5,936 blocked** (all deltas zero),
with 6,190 dispatch-attempted. The blocked distribution is unchanged:
Character reference 4,692; Markup declaration open 242; Processing instruction
open 52; CDATA section 56; Script data escape start 31; token variants 858;
XML coercion 4; pending-spec review 1. All prior passing identities remain
protected and pass. No state-specific pass-baseline file is applicable because
there are no new upstream pass identities.

Historical PR #34 checks were rerun and passed locally after rebasing onto merged PR #32:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked` — 260 Rust tests passed, 0 failed, 0 ignored;
  includes all seven state-local tests and six inherited Script data escaped tests
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`
- `cargo test --workspace --locked pinned_upstream_runner -- --nocapture`

PR #33's later integrated validation and unchanged upstream results are recorded
in [escaped dash dash evidence](tokenizer-script-data-escaped-dash-dash.md#validation-and-upstream-report).

There are no local validation/infrastructure blockers. Destination and earlier
stubs are intentional scope limits, not failing tests or comparable passes.

## Parallel integration caveat

This PR began as one of five standalone parallel state PRs. The user subsequently
authorized clean sequential integration by the parent with a fresh independent
review immediately before each merge. PR #34 rebased onto merged PR #32's exact
base, preserving its module/dispatch, inventory registration, status link,
production implementation, and source assertions. Additive registration/status
conflicts were resolved by retaining both implemented states. No unmerged sibling
branch is imported and this thread starts no reviewers/subagents or merges.

On PR #33's base, the earlier escape-start states and escaped less-than sign
still blocked; escaped dash dash now executed. PR #35 rebases onto merged main
`a0cf68b77662fb482373f78fa15b69a540b53623` and adds escape start dash while
preserving this algorithm and all source assertions. Escape start remains the
normal-entry block; isolated escape-start-dash entry now runs the escaped trio,
with escaped less-than sign still terminal. See
[PR #35 integrated validation](tokenizer-script-data-escape-start-dash.md#upstream-evidence-and-exact-validation).
Future sibling integration must review newly executable paths and moved
terminal blocks, deliberately update affected explicit tests while preserving
source assertions, and rerun every check/upstream report. Any new upstream pass
identity requires individual fixture inspection/review, not aggregate auto-blessing.
Parent will review this final pushed revision before merging; auto-merge is not
enabled. This partial implementation makes no full-conformance or browser-security
claim.
