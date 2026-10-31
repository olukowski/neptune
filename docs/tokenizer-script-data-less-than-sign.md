# Script data less-than sign state

## Pinned evidence and branch mapping

Normative baseline: WHATWG HTML revision
[`404b3eb14a7a936dddefb2d78e224dbeb01d2358`](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143654-L143673),
**Script data less-than sign state** (`#script-data-less-than-sign-state`), source
lines 143654–143673. The immutable raw `source` was retrieved and these lines
inspected, together with the definitions below. Existing RCDATA/RAWTEXT code
supplies execution/storage conventions, not normative algorithm evidence.
The entire production algorithm is in
`crates/html/src/tokenizer/script_data_less_than_sign.rs`.

| Consumed input | Source-step effects | Intended destination |
| --- | --- | --- |
| `/` U+002F (lines 143661–143663) | Empty the temporary buffer; no token creation, emission, or reconsumption. | Script data end tag open |
| `!` U+0021 (lines 143665–143667) | Emit `<` then `!` as separate character tokens; preserve the buffer; no reconsumption. | Script data escape start |
| Anything else, including conceptual EOF (lines 143669–143671) | Emit exactly `<`; preserve the buffer; schedule the current input for reconsumption. | Script data |
| Exhausted **open** input | No effects; `NeedMoreInput`, not EOF. | Same source state |

The first three rows exhaust the pinned algorithm; the fourth is the existing
incremental input contract, not an additional normative branch. The executor
returns `Progress::Continue(target)` after all source effects, before attempting
the destination on the next step. In particular, `!` must emit both tokens even
though escape start is unimplemented. EOF fallback must not emit an EOF token,
finish, report `eof-before-tag-name`, or invent a recoverable error.

Relevant definitions inspected in the same pinned source:

- [Preprocessing and next/current input, lines 142560–142594](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142594):
  CR/CRLF normalize before tokenization. Surrogates, noncharacters, and controls
  retain their preprocessing errors; NUL handling belongs to later states.
- [Conceptual EOF, lines 142602–142607](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142602-L142607):
  EOF is a conceptual input item, so it takes the anything-else branch here.
- [Tokenization and reconsumption, lines 143149–143167](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143149-L143167):
  the destination receives the current item instead of the next item; temporary
  buffer and return state are distinct storage.
- [Token creation/emission, lines 143169–143191](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143169-L143191):
  clearing scratch storage is not creating a tag. This state creates no pending
  tag or comment and leaves creation defaults unchanged.
- [Script data, lines 143338–143359](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143338-L143359):
  the already-implemented destination handles reconsumed NUL replacement/errors,
  ordinary character emission, another `<`, and terminal EOF on its own step.

This state has **no state-generated parse errors**. NUL is neither replaced nor
diagnosed here; non-scalars remain code points rather than scalar strings.
Only cursor preprocessing reports the applicable diagnostic, once at the
original normalized position. Pending tags (including attributes and detached
duplicates), current-tag selector, pending comment, last emitted start-tag name,
return state, and prior output/diagnostics are preserved. Only `/` clears the
buffer; only the explicitly specified character tokens are appended.

## Focused tests and explicit blocks

`crates/html/tests/script_data_less_than_sign_state.rs` contains five local tests:

- Every ASCII input, representative Unicode scalars, actual lone surrogates,
  noncharacters and controls: exact source snapshots, normalized positions,
  diagnostics, emitted order, buffer behavior, and unconsumed tails. Both open
  and closed input and absent/Start/End selectors are exercised. Fallback then
  executes the real Script data destination, checking NUL replacement/error at
  the original position, another `<` transition, once-only preprocessing errors,
  the following character, suspension, and terminal EOF in separate snapshots.
- Actual EOF versus repeated open exhaustion after an already consumed `<`,
  with/without a CRLF prefix: only `<` is emitted at EOF and conceptual EOF is
  scheduled for reconsumption at the position after `<`.
- Default context: no destination token/storage creation or EOF is invented.
- Suspension with no current item, subsequent feeding, and terminality that
  cannot be cleared by feeding/finishing input after a stub.
- Every two-chunk split (with empty chunks) and single-code-point chunking of
  representative `/`, `!`, malformed/literal fallback, NUL, Unicode/non-scalars,
  CRLF, consecutive/trailing CR, and EOF inputs matches closed source-step and
  terminal/finished snapshots through every real source step. Both isolated
  entry and real Script data entry are tested, including splits before/after
  the initial `<`. Open exhaustion is checked at every step. A stub immediately
  ends the trace; it is never fed, reconstructed, or resumed.

Isolated entry initializes the cursor as if `<` was already consumed; it does
not simulate Script data's algorithm. Real entry runs the merged Script data
handler. `ScriptDataEndTagOpen` and `ScriptDataEscapeStart` remain **terminal
`UnimplementedState`** destinations, including when input is open/exhausted or
at actual EOF. Source snapshots are still asserted before any destination step.
Repeated terminal steps have no effects. Only the real Script data destination
emits terminal EOF after fallback; the less-than-sign source itself never does.

`script_data_state.rs` retains its delimiter-only source boundary checks and now
checks the real less-than-sign → escape-start terminal boundary. Its streaming
trace includes the newly executable fallback steps rather than truncating at
less-than-sign. `tests/support/script_data.rs` requires full output and exact
errors for literal fallback (including NUL, EOF, CRLF and non-scalars), rejects
wrong output and missing/duplicate diagnostics, and keeps explicit end-tag-open
and escape-start blocks even after exact output/error prefixes. None of these
expectations depends dynamically on dispatch support. These are local tests,
not additional upstream conformance runs.

## Upstream report and validation

After rebasing onto main `c9c43fd0676d5dc39035a8da9317328b6e06745b`
(PR #30, Script data), the pinned runner reports:

- 6,827 fixture cases / 7,053 case-initial-state runs;
- 6,190 dispatch-attempted;
- **1,111 executed = 1,111 passed + 0 failed**;
- **5,942 blocked**: Character reference 4,692; Markup declaration open 242;
  Script data end tag open 6; Script data escape start 31; CDATA section 56;
  Processing instruction open 52; unsupported token variants 858; XML coercion 4;
  pending-spec review 1.

Main after PR #30 had 1,110 executed/passed, 0 failed, 5,943 blocked; its 38
less-than-sign blocks now become **one newly comparable pass** and 37 explicit
destination blocks. All 914 prior protected identities remain unchanged and
passing. The single newly reviewed identity below brings the protected union
to 915. No upstream fixture, comparison/policy rule, fixture revision, or
specification revision is changed. Inventory tests execute no conformance
cases; local tests and blocked prefixes are not upstream passes. This partial
report makes no full-conformance or browser-security claim.

### Reviewed upstream identity

`baselines/script_data_less_than_sign.json` adds exactly:
`["test1.test", "tests", 33, "< in script data", "Script data state", 0]`.
The unchanged pinned fixture was inspected individually: input `<test-->`,
expected one character string `<test-->`, no `errors`, no `doubleEscaped`,
and a single Script data initial-state entry (index 0). Script data consumes
`<`; less-than-sign consumes `t`, emits `<`, and reconsumes `t` in Script data;
`t`, `e`, `s`, `t`, `-`, `-`, `>` then emit literally, followed by terminal EOF
(omitted from fixture output). This follows the pinned anything-else branch
and Script data algorithm, not execution-generated expectations. There is no
closing-tag or escape branch in this case. Every other `<`-containing Script
data fixture reaches `/` or `!` and remains blocked. Prior baseline files and
identities are not changed or re-blessed.

All required checks passed: `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets --locked -- -D warnings`,
`cargo test --workspace --locked`, and
`RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`.
The separate `cargo test --workspace --locked pinned_upstream_runner -- --nocapture`
report also passed. The fixture submodule was already initialized at the pinned
revision `c777c408b61078ea2eb4acefc2535f54dbc8b28a`.

## Scope and sibling integration

Exactly one additional state is implemented. Script data is inherited unchanged
from main PR #30; no end-tag-open/name, escape-start, other escape state,
automatic mode switching, tree builder, API/storage helper, or dependency is
added. There is no missing shared prerequisite.
Module/dispatch and independent support-inventory additions register only this
state; shared documentation changes only identify support and buffer usage.

The original four PRs started independently. This branch was user-directed to
rebase onto current main after independently reviewed Script data PR #30 merged.
Narrow module/dispatch, independent inventory, and status conflicts were resolved
additively, preserving both states. No unmerged sibling branch was imported.
Tests that assumed a Script data stub now execute the real destination and assert
its effects; source-step assertions are preserved. No tests are skipped and no
expected behavior is selected dynamically from support availability.

End-tag-open/name remain separate PRs. Before each future merge the parent must
independently review the exact pushed revision. After those merges, deliberately
review/update destination-block tests, rerun every check and the upstream report,
and individually review newly comparable identities. Escape start remains an
explicit block even after the other three sibling states integrate. This thread
does not merge or start a reviewer/subagent.
