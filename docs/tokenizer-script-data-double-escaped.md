# Script data double escaped state

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

<a id="current-pr-42-integration-after-merged-43-38-39-and-40"></a>

## Historical PR #42 integration after merged #43, #38, #39 and #40

The user authorized clean sequential integration with a fresh independent
final-head review. This revision is rebased onto exact main
**`76add983570aba0fb8fc769aa8ea4f2510d37965`**, containing merged #43
(DoubleEscapeEnd), #38 (DoubleEscapedLessThanSign), #39 (DoubleEscapedDashDash),
and #40 (DoubleEscapedDash). Original standalone head
**`99609158fd54480b3e418f1478ddc7e7249c88d4`** on common base
`39d772e31b98e6e955a2e00f546a8db3506eb1dd` is historical below. The final
integrated head is recorded in [PR #42](https://github.com/olukowski/neptune/pull/42)'s
body. No unmerged sibling was imported. The assigned production algorithm and
all four merged production algorithms are unchanged. Module/dispatch/inventory
conflicts were resolved additively, retaining every merged registration and the
DoubleEscapeEnd inventory's explicit direct-entry scratch setup.

### Actual loop and source boundaries

The immutable [assigned branches, 143993–144021](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143993-L144021)
and [merged neighbors, 144024–144142](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144024-L144142)
were reread for integration. DoubleEscaped emits `-` before real Dash executes,
and emits `<` before real LessThanSign executes. Dash/DashDash NUL and literal
branches now return to the actual DoubleEscaped character loop. LessThanSign's
non-slash fallback and DoubleEscapeEnd's anything-else fallback reconsume there;
only the destination replaces NUL or reports/emits EOF. These source steps still
emit/diagnose nothing prematurely, and input diagnostics remain once-only.

`/` in LessThanSign alone clears scratch and emits `/`. DoubleEscapeEnd emits
original letter spelling while appending ASCII-lowercased scratch. A matching
`script` delimiter returns to Escaped; a mismatch returns to the real
DoubleEscaped loop without clearing scratch. Both double-escaped and escaped
dash-dash `>` branches can return to ordinary ScriptData; ordinary EOF adds no
comment-like EOF diagnostic. A name prefix (even exactly `script`) without a
delimiter reconsumes EOF in DoubleEscaped, which emits one EOF/error.

### Tests updated in all affected suites

No original source snapshot, malformed-input matrix or streaming comparison is
replaced by a support-detection shortcut or output-prefix comparison.

- **DoubleEscaped: ten tests.** Destination tests now execute Dash/LessThanSign
  on an actual surrogate and NUL tail, or on empty tails, through open suspension
  and exact EOF. Two added fixed-source-trace tests close real cycles:
  `-a--\0<\0</X>--b</ScRiPt>-->` checks Dash/DashDash returns, NUL replacement,
  less-than reconsumption, mismatched normalized `x` scratch, slash clearing,
  original mixed-case output versus per-letter `script` scratch, Escaped
  continuation and ScriptData EOF; `-->\0\r\n` separately checks the direct
  double-dash return, ordinary ScriptData NUL/error, CRLF and ordinary EOF.
- **DoubleEscapedDash: nine tests.** Exhaustive source snapshots now execute the
  surrogate suffix through DashDash, LessThanSign reconsumption, or DoubleEscaped,
  assert once-only preprocessing, suspension and exact EOF. Default entry,
  split CRLF/NUL, and name-state EOF now finish in real destinations. The old
  predecessor block now runs DoubleEscaped's real `-`, Dash's NUL replacement,
  and DoubleEscaped EOF; the separate ordinary-entry gateway block is preserved.
- **DoubleEscapedDashDash: nine tests.** Unicode source effects now continue
  through the following NUL and EOF. NUL/literal/less-than cases execute the full
  literal tail with exact replacements/positions. Near-match/empty names now
  continue through the same CRLF, repeated NUL, surrogate/noncharacter/control
  and EOF assertions as matches, in their distinct correct state. Open
  less-than/name EOF still exposes the source reconsumption before destination
  EOF/error.
- **DoubleEscapedLessThanSign: nine tests.** Every ASCII/non-scalar fallback now
  reconsumes in real DoubleEscaped, including NUL, another `<`, `-`, and the
  untouched `z` suffix. Slash still clears scratch before `z` is accumulated in
  DoubleEscapeEnd; reconsumed EOF now produces the actual destination EOF/error.
  Default/seeded suspension and EOF tests are extended, and the real predecessor
  test now includes all three DoubleEscaped/Dash/DashDash sources.
- **DoubleEscapeEnd: eight tests.** Delimiter equality checks now execute both
  matched Escaped and mismatched DoubleEscaped literal continuation, suspension
  and EOF without changing scratch. NUL/non-scalar/malformed fallback tests
  retain no-output/no-state-error source snapshots, then assert actual
  reconsumption, punctuation transitions, next-NUL handling, once-only input
  diagnostics, and EOF. Explicit EOF still reconsumes at the source, with its
  diagnostic/token supplied only by the destination.

Every affected streaming suite retains all two-chunk splits, empty chunks and
single-code-point chunking, including CRLF/consecutive/trailing CR, name
boundaries, non-scalars and malformed input. These traces now finish real
double-escaped cycles; the assigned suite adds the fixed-cycle inputs and
repeated less-than/CRLF fallback. Complete snapshots compare tokens, exact
diagnostic positions/multiplicity, current consumption and unrelated storage
including scratch. True gateway blocks stop immediately without resumption.

### Current validation, upstream report and remaining boundary

All CONTRIBUTING checks and the separate pinned runner pass (exit 0):
fmt check, warning-denying Clippy, workspace tests, warning-denying rustdoc,
`pinned_upstream_runner`, and `git diff --check`.
**329 Rust tests passed, 0 failed, 0 ignored**, including 45 focused tests in
the five suites above. These are local checks, not 329 upstream cases.

Upstream remains **6,827 fixture cases; 7,053 runs; 6,190 dispatch-attempted;
1,126 executed = 1,126 passed + 0 failed; 5,927 blocked**.
New upstream passes: **0**. The ordinary-entry EscapedLessThanSign gateway still
blocks all 22 relevant runs; DoubleEscapeStart and the escaped-end-tag gateways
also remain unimplemented, not simulated. Remaining categories:
CharacterReference 4,692; MarkupDeclarationOpen 242; ProcessingInstructionOpen 52;
CdataSection 56; TokenVariants 858; XmlCoercion 4; PendingSpecReview 1.
No fixtures, pins, expected outputs, adapter/comparison rules or prior baseline
files/registrations (930 protected identities) changed.

There is no validation/shared-prerequisite blocker. Coordinator obtains an
independent review of the final pushed base/head before any merge. This thread
does not merge, enable auto-merge or start subagents. Future gateway integrations
must preserve source evidence, review newly reachable fixture identities
individually, and rerun every check/report. Partial support is not full
conformance or a browser-security claim.

## Historical standalone PR #42 evidence

**Everything below records the original standalone head, not current
destination support, test names or Rust test counts.** Its pinned assigned-state
algorithm and source effects remain unchanged.

## Scope and immutable evidence

Implements exactly `State::ScriptDataDoubleEscaped` in
`crates/html/src/tokenizer/script_data_double_escaped.rs`, from the required common
base `39d772e31b98e6e955a2e00f546a8db3506eb1dd` (main after PR #36).
Initial HEAD was verified before implementation. No sibling algorithm or branch
is imported. Shared edits are additive module/dispatch/support registration and
small current-scope updates; no storage, API, dependency, adapter, fixture, pin,
or baseline changes are needed.

Retrieved and read the immutable WHATWG HTML raw source revision
[`404b3eb14a7a936dddefb2d78e224dbeb01d2358`, lines 143993–144021](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143993-L144021),
anchor `#script-data-double-escaped-state`. This is not an implementation from
recollection, another engine, or the moving living Standard. Referenced definitions
read in the same pinned source:

- [Preprocessing, next/current input, conceptual EOF, 142560–142607](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142607):
  CR/CRLF normalize before matching; non-scalars and controls retain code points
  and receive once-only input diagnostics; NUL handling belongs to states.
- [Tokenization, reconsumption, temporary/return buffers and token output, 143149–143191](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143149-L143191):
  a switch alone does not reconsume; character output is not pending-tag mutation.
- [Parse errors, 140986–141008](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L140986-L141008),
  [NUL error, 141479–141482](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L141479-L141482),
  and [comment-like script EOF error, 141138–141149](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L141138-L141149).
  Error-table descriptions are non-normative; state algorithms supply behavior.
- [Earlier entry boundary, 143872–143892](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143872-L143892),
  [double escape start, 143958–143990](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143958-L143990),
  and [later double-escaped states, 144024–144144](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144024-L144144)
  establish integration boundaries only; none is implemented here.

## Branch-to-code/test map

Every normative branch consumes one normalized item; none schedules reconsumption.
`Progress::Continue` exposes completed source effects before destination execution.
Tests: `crates/html/tests/script_data_double_escaped_state.rs`.

| Input / pinned source lines | Source effects / result | Focused evidence |
| --- | --- | --- |
| `-`, 144000–144002 | Emit `-`; switch to `ScriptDataDoubleEscapedDash` | Exhaustive source snapshot and terminal-destination tests |
| `<`, 144004–144006 | Emit `<`; switch to `ScriptDataDoubleEscapedLessThanSign` | Same tests; unlike Escaped, the consumed `<` is emitted here |
| NUL, 144008–144011 | Report `unexpected-null-character`; emit U+FFFD; stay | Exhaustive snapshot, repeated NUL/newlines, incoming reconsumption |
| EOF, 144013–144016 | Report `eof-in-script-html-comment-like-text`; emit EOF; finish | Empty/nonempty/default/seeded entry, exact EOF positions, reconsumed EOF, repeated terminal steps |
| Anything else, 144018–144019 | Emit current point unchanged; stay | All other ASCII points, Unicode/non-scalars, entity-like/malformed literal text |
| Open exhaustion (existing incremental contract) | Effect-free `NeedMoreInput`; stay | Repeated suspension/empty chunks before and after current input, then explicit finish |

Direct entry uses `Execution::new(State::ScriptDataDoubleEscaped, context)` and
requires **no** pending tag, selector, temporary-buffer spelling or history setup.
All such context is unrelated here and must be preserved. Snapshots compare both
pending tags including flags, retained/detached attributes, selector (None/Start/End),
comment, last-start-tag history, return state, mixed-case/non-ASCII scratch,
and existing output/diagnostic prefixes. There is no ASCII normalization of
emitted letters or scratch here. `&`, `/` and `>` are literal; no Character
reference, tag, Comment token, JavaScript parsing, tree builder or automatic
mode switching is invented.

Both outgoing destinations remain terminal typed `UnimplementedState` errors,
including with exhausted open input or closed empty tails. Tests inspect exact
source effects first, then assert repeated effect-free destination errors.
Feeding/finishing cannot clear terminal status. Separate source-boundary cursor
probes establish untouched suffix/current consumption without reconstructing or
resuming a terminal tokenizer.

## Local coverage and entry limitations

Eight focused tests cover every ASCII point plus representative scalars, actual
lone surrogates, noncharacters and controls; default/seeded context; current
input and normalized positions; exact diagnostic codes/multiplicities; literal
U+FFFD versus NUL replacement; and incoming reconsumption including EOF.
Reconsumption reuses its position and preprocessing diagnostics exactly once.
Repeated NUL and CR/CRLF runs independently assert each state diagnostic and
EOF at its normalized location. The literal-text test checks supplementary
characters count as one point, not UTF-16 units.

Every two-chunk split (including empty chunks) and single-code-point chunking of
representative inputs compares every non-suspension source-step/terminal snapshot
with closed input. Cases include all branches, malformed input, repeated NUL,
non-scalars, CRLF split across chunks, consecutive/trailing CR, literal text,
both destination blocks, and real Script data paths up to the earlier stub.
Suspension snapshots are independently effect-free;
trace collection stops immediately at a stub and never feeds/resumes it.

No implemented external predecessor reaches this state on this base. A fixed
real Script data trace for `<!--<script>` executes the existing escape-start
states and escaped dash dash, then consumes the second `<` without emission and
blocks at `ScriptDataEscapedLessThanSign`, leaving `s` unconsumed. No fabricated
predecessor or output-prefix success is used. `ScriptDataDoubleEscapeStart`,
DoubleEscapedDash, DoubleEscapedDashDash, DoubleEscapedLessThanSign and
DoubleEscapeEnd remain separate owners' stubs.

## Independent upstream evidence and report

The pinned html5lib submodule is unchanged at
`c777c408b61078ea2eb4acefc2535f54dbc8b28a`. Inspected unchanged `domjs.test`
identities (zero-based, Script data entry index 0):

- tests[6], double-escaped NUL: the first NUL after `<!--<script>` expects U+FFFD
  and `unexpected-null-character` at 1:13. Later NULs at 1:30 and 1:48 involve
  the separately owned dash states, not this implementation.
- tests[12], double-escaped EOF: input/output `<!--<script>` with one
  `eof-in-script-html-comment-like-text` at 1:13.
- tests[10–11] cover EOF in the separate dash/dash-dash states at 1:14/1:15;
  tests[17–19, 22–23] require later double-escaped transitions/end-tag support.

These are **blocked**, not new executed/passed cases: upstream has no direct
initial-state mapping for DoubleEscaped, and its earlier escaped-less-than-sign
stub prevents entry. Both the exact-base report and final report use:

```sh
cargo test --workspace --locked pinned_upstream_runner -- --nocapture
```

Exact-base and final reports agree: **6,827 fixture cases; 7,053 runs;
6,190 dispatch-attempted; 1,126 executed = 1,126 passed + 0 failed;
5,927 blocked.** All deltas are zero. Blocked categories:

| Reason | Runs |
| --- | ---: |
| Character reference | 4,692 |
| Markup declaration open | 242 |
| Processing instruction open | 52 |
| CDATA section | 56 |
| Script data escaped less-than sign | 22 |
| Token variants | 858 |
| XML coercion | 4 |
| Pending-spec review | 1 |

No new passing identity exists, so no state-specific baseline is warranted;
all previous baseline files/registrations remain intact. Local/inventory tests
are not upstream conformance runs. No full conformance or browser-security
claim is made.

## Validation

All required checks passed (exit 0):

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked` — **292 Rust tests passed, 0 failed,
  0 ignored**, including eight focused state tests; these are not 292 upstream cases.
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`
- `cargo test --workspace --locked pinned_upstream_runner -- --nocapture`
- `git diff --check`

No infrastructure/validation blocker remains. Earlier and outgoing state stubs
are intentional scope limits, not passing output prefixes or failed comparable
cases.

## Later integration boundaries

Do not merge or enable auto-merge from this thread. The coordinator handles later
integration after the user's signal. Future rebases must retain all owners'
additive module/dispatch/inventory registrations and truthful scope updates.
Implementing either outgoing destination intentionally invalidates this suite's
explicit terminal assertions: preserve source snapshots, then review actual
next-step consumption, diagnostic ownership, and later block/EOF behavior.
Implementing earlier entry states may admit upstream cases; inspect each newly
passing identity independently before adding state-specific baseline entries.
Rerun all checks and the upstream report rather than dynamically detecting
support, weakening comparisons or resuming old terminal executions.
