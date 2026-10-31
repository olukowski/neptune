# Script data double escaped dash state

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
