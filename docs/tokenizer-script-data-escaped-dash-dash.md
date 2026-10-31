# Script data escaped dash dash state

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


## Scope and immutable evidence

Only `State::ScriptDataEscapedDashDash` is implemented here, in
`crates/html/src/tokenizer/script_data_escaped_dash_dash.rs`. The historical
standalone baseline was main `28131e11437941928eb7246885fb70e1160b2700` (PR #31);
this state was a stub. PR #33 now rebases onto merged PRs #32 (Escaped) and #34
(EscapedDash), main `04283260b9d467d198ab92cb8ccd5721c51fe21e`, preserving both
production algorithms unchanged. No unmerged sibling branch is imported.
Existing Context, cursor, tokens, diagnostics and Execution suffice; no shared framework, dependency, storage or adapter change.

Retrieved and inspected the immutable WHATWG HTML revision
`404b3eb14a7a936dddefb2d78e224dbeb01d2358` directly from its
[raw source](https://raw.githubusercontent.com/whatwg/html/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source).
Relevant source sections/definitions:

- [Assigned state, #script-data-escaped-dash-dash-state, lines 143837–143869](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143837-L143869): complete six-branch algorithm.
- [Preprocessing/input definitions, lines 142560–142611](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142611): surrogate, noncharacter and control errors, deferred NUL handling, newline normalization, next/current input, conceptual EOF.
- [Tokenization, lines 143149–143188](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143149-L143188): consuming versus reconsuming, temporary/return-state context, character/EOF tokens and emission.
- [unexpected-null-character, lines 141479–141482](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L141479-L141482).
- [eof-in-script-html-comment-like-text, lines 141138–141150](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L141138-L141150): EOF in script text resembling an HTML comment, not comment-token parsing.

Pins and upstream fixtures remain unchanged. This is standalone tokenizer work,
not JavaScript parsing, automatic mode switching, tree construction, full
conformance, or a browser-security claim.

## Branch mapping

| Consumed input | Pinned lines | Source effects / intended target | Focused tests |
| --- | --- | --- | --- |
| `-` U+002D | 143844–143845 | Emit `-`; remain in assigned state. | ASCII, open/EOF, streaming |
| `<` U+003C | 143847–143848 | Switch to Script data escaped less-than sign; emit nothing. | ASCII, terminal destination, streaming |
| `>` U+003E | 143850–143852 | Switch to Script data; emit `>`. | ASCII, real Script data continuation, streaming |
| NUL | 143854–143858 | One `unexpected-null-character`; switch to Script data escaped; emit U+FFFD. | ASCII, real Escaped destination, normalized positions, streaming |
| EOF | 143860–143863 | One `eof-in-script-html-comment-like-text`; emit EOF and finish, without changing state. | Open/EOF, normalized positions, streaming |
| Anything else | 143865–143867 | Switch to Script data escaped; emit current point literally. | ASCII, Unicode/non-scalars, real Escaped destination, streaming |

Every branch consumes exactly once; none schedules reconsumption. `<` is not
emitted here, and nothing consumes or preprocesses its following point eagerly.
NUL replacement is distinct from literal U+FFFD. Ampersands, quotes, whitespace,
non-ASCII letters and punctuation take anything-else, not character-reference or
JavaScript processing. Surrogates/noncharacters/controls retain their original
values, with exactly their cursor preprocessing diagnostics; they are not
converted through scalar strings. CR/CRLF become LF in the existing cursor.

Open exhausted input suspends without effects, retaining current input and state;
empty chunks are not conceptual EOF. Explicit `finish()` is required for EOF.
Diagnostics use the repository's one-based normalized code-point coordinates,
including EOF immediately after the final point. Repeated terminal steps do not
duplicate tokens/errors.

All branches preserve unrelated selected/pending start/end tags (including flags,
retained and detached attributes), pending comment, last start tag, return state,
and temporary buffer. Existing tokens/errors are appended to, never replaced.

## Focused and streaming evidence

`crates/html/tests/script_data_escaped_dash_dash_state.rs` contains eight local
tests after integration (seven in the historical standalone revision):

- All 128 ASCII points under both open and finished input: exact output, source
  target, normalized current point, state/preprocessing diagnostics and
  multiplicities, unrelated storage, and next untouched point/no reconsumption.
- Unicode scalars, non-ASCII punctuation/whitespace, literal BOM/U+FFFD,
  actual high/low surrogates, BMP/supplementary noncharacters and controls:
  exact original values/errors, positions, source effects and untouched NUL
  suffix. The real Escaped destination then replaces/diagnoses that NUL at 1:4
  and emits EOF/error at 1:5, with exact snapshots and no duplicate diagnostics.
- NUL, `<` and ampersand branches after hyphens: unchanged source effects,
  followed by real Escaped consumption of `tail` / `amp;` and EOF/error at 1:8;
  only `<` remains terminal at escaped less-than sign, without fake EOF or an
  extra recoverable error.
- Empty input and hyphen runs: repeated effect-free suspension followed by exactly
  one EOF diagnostic/token, terminal idempotence and preserved storage.
- Assigned-state NUL and EOF after preconsumed scalar/supplementary/CRLF input:
  explicit normalized line/column assertions.
- `>` through existing Script data: CRLF normalization, repeated NUL diagnostics,
  surrogate diagnostic, exact output/EOF/positions, and a later explicit escape
  start block after `<!`, with source assertions intact.
- Fixed specification-derived source-step snapshots from each of Escaped,
  EscapedDash and EscapedDashDash through repeated hyphen/literal/NUL cycles:
  `>` returns to real Script data (CRLF/NUL/EOF), while `<` reaches the remaining
  terminal escaped-less-than-sign stub. Full snapshots preserve unrelated storage.
- Every two-chunk split, including empty chunks, and single-code-point streaming
  against every closed-input source-step snapshot. Cases include every branch,
  hyphen runs, repeated three-state cycles, CRLF splits, consecutive/trailing CR,
  malformed input, actual non-scalars/controls, real end-tag continuation and
  later escaped-less-than-sign / Character reference / escape-start blocks.
  Suspensions must preserve snapshots. The harness stops at terminal stubs and
  never reconstructs or resumes a blocked execution.

Support inventory registration is one additive entry, independent of dispatch.
On the historical standalone base, only the generic inventory assumed this stub.
The merged EscapedDash suite now has two such assumptions: its exhaustive branch
and default-context destination tests are deliberately updated to execute
DashDash (including suspension, NUL/surrogate consumption and return to Escaped),
with source snapshots and later explicit escaped-less-than-sign blocks intact.
All three suites extend streaming inputs through real three-state cycles; no
expectations depend dynamically on support, and no test is skipped.
Inventory/local tests are not upstream conformance cases.

## Validation and upstream report

All required commands passed on historical standalone revision
`e9d4660dc3325ba7694da576f79ef6f6d416ac31` against the original common baseline.
All are rerun and passed (exit 0) against the integrated base
`04283260b9d467d198ab92cb8ccd5721c51fe21e` plus this one-state implementation.
The integrated workspace suite passes 268 Rust tests, 0 failed, 0 ignored;
21 are the three focused suites (8 DashDash, 7 Dash, 6 Escaped). These Rust test
counts are separate from upstream conformance runs. There are no local
validation/infrastructure blockers.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo test --workspace --locked pinned_upstream_runner -- --nocapture
```

Pinned fixtures were already initialized at
`c777c408b61078ea2eb4acefc2535f54dbc8b28a`. The historical standalone report and
the integrated report have identical counts; the table compares current merged
main (#32/#34) with this integrated PR:

| Category | Baseline | This state | Change |
| --- | ---: | ---: | ---: |
| Fixture cases | 6827 | 6827 | 0 |
| Runs | 7053 | 7053 | 0 |
| Dispatch-attempted | 6190 | 6190 | 0 |
| Executed | 1117 | 1117 | 0 |
| Passed | 1117 | 1117 | 0 |
| Failed | 0 | 0 | 0 |
| Blocked | 5936 | 5936 | 0 |

Executed = passed + failed; blocked runs are not passes. All 921 protected prior
passing identities remain intact and pass the independent case-level regression
check. There are no new comparable upstream runs, so no new pass-identity file is
applicable and no existing baseline is edited or execution auto-blessed.
The 31 runs blocked at Script data escape start remain blocked there: the fixture
adapter has no escaped-dash-dash initial state, and the earlier escape-start
states are stubs. Inspected unchanged `domjs.test` zero-based cases 5, 9, 13 and 15:
case 5's third NUL (after `--`) expects U+FFFD and exactly
`unexpected-null-character` at 1:36; case 9 (`<!--test--`) expects literal output
and exactly `eof-in-script-html-comment-like-text` at 1:11; cases 13/15
(`<!-- - -->`, `<!--test--->`) expect literal output without errors, covering
literal fallback, repeated hyphens and `>` return. They remain blocked at escape
start, not new comparable identities. No fixture-derived expected output is
inferred from execution. No fixture, pin, policy or adapter rule is changed to
bypass that boundary.

## Remaining stubs and parallel integration

Isolated entry uses `Execution::new(State::ScriptDataEscapedDashDash, context)`.
Historically, Script data escape start, escape start dash, escaped, escaped dash,
and escaped less-than sign were stubs on the standalone baseline. After inheriting
merged #32/#34, Escaped and EscapedDash now consume real input, handle NUL/EOF and
suspension, and can return to DashDash; only escaped less-than sign is a direct
terminal destination here. It returns `UnimplementedState` on the next step, not
a recoverable HTML error or successful EOF. On PR #33's base, escape start and
escape start dash prevented normal entry into this cycle. PR #35 now adds escape
start dash on merged main `a0cf68b77662fb482373f78fa15b69a540b53623`, preserving
the trio algorithms and source assertions; escape start remains unsupported and
still prevents normal Script data entry. Isolated escape-start-dash entry now
executes this cycle; escaped less-than sign remains terminal. See
[PR #35 integrated tests and validation](tokenizer-script-data-escape-start-dash.md#focused-and-streaming-tests-after-integration).
Ordinary
comments, character references, double-escaped script states and other unsupported
states also remain stubs.

This began as one of five standalone state PRs. The user subsequently authorized
sequential merges by the parent, with a fresh independent review immediately
before each merge and findings returned to the implementation thread. This
thread rebases onto merged #32/#34, resolving additive module/dispatch, inventory
and status-link conflicts by preserving all three registrations. No unmerged
sibling work is imported and no reviewer/subagent is started here. The production
scope remains one new state. Parent will freshly review the exact pushed revision;
this thread does not merge or enable auto-merge. Future integration must rerun
all checks/report and review changed terminal boundaries while preserving
source-step assertions and later explicit blocks.
Do not dynamically choose expectations from support, skip tests, or reconstruct
an execution that already hit a stub. Newly comparable upstream identities need
individual fixture/spec review before any state-specific baseline addition.
