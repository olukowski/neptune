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
  and closed input and absent/Start/End selectors are exercised.
- Actual EOF versus repeated open exhaustion after an already consumed `<`,
  with/without a CRLF prefix: only `<` is emitted at EOF and conceptual EOF is
  scheduled for reconsumption at the position after `<`.
- Default context: no destination token/storage creation or EOF is invented.
- Suspension with no current item, subsequent feeding, and terminality that
  cannot be cleared by feeding/finishing input after a stub.
- Every two-chunk split (with empty chunks) and single-code-point chunking of
  representative `/`, `!`, malformed/literal fallback, NUL, Unicode/non-scalars,
  CRLF, consecutive/trailing CR, and EOF inputs matches closed source-step and
  terminal-stub snapshots. Remaining chunks are fed before the first destination
  dispatch; no blocked execution is reconstructed or resumed.

Isolated entry initializes the cursor as if `<` was already consumed; it does
not simulate Script data's algorithm. `ScriptData`, `ScriptDataEndTagOpen`, and
`ScriptDataEscapeStart` all remain **terminal `UnimplementedState`** destinations
in this independent branch, including when the cursor is open/exhausted or at
actual EOF. Repeated terminal steps have no effects. Tests inspect cursor
reconsumption/tails directly after checking terminality, never execute a fake
destination or restart execution after a stub. These are source-state/local
streaming checks, not tokenizer conformance passes.

## Upstream report and validation

At base `c8ec2da` with only this state added, the pinned runner reports:

- 6,827 fixture cases / 7,053 case-initial-state runs;
- 6,190 dispatch-attempted;
- **1,059 executed = 1,059 passed + 0 failed**;
- **5,994 blocked**: Character reference 4,692; Markup declaration open 242;
  Script data 89; CDATA section 56; Processing instruction open 52;
  unsupported token variants 858; XML coercion 4; pending-spec review 1.

There are **zero newly passing upstream identities**: the runner's six initial
states do not include this internal state, and Script data remains a stub, so
no upstream run can reach this handler yet. All 903 previously protected
identities still pass. No baseline file/registration, upstream fixture, fixture
revision, or specification revision is changed. Inventory tests execute no
conformance cases; local tests and blocked prefixes are not upstream passes.
This partial report makes no full-conformance or browser-security claim.

All required checks passed: `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets --locked -- -D warnings`,
`cargo test --workspace --locked`, and
`RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`.
The separate `cargo test --workspace --locked pinned_upstream_runner -- --nocapture`
report also passed. The fixture submodule was already initialized at the pinned
revision `c777c408b61078ea2eb4acefc2535f54dbc8b28a`.

## Scope and sibling integration

Exactly one state is implemented. No Script data, end-tag-open/name, escape-start,
other escape states, automatic mode switching, tree builder, API/storage helper,
or dependency is added. There is no missing shared prerequisite.
Module/dispatch and independent support-inventory additions register only this
state; shared documentation changes only identify support and buffer usage.

The four parallel PRs start independently from main. No sibling changes are
imported here. The user must review them before later integration/rebase.
After neighboring implementations merge, retain the source snapshots but review
obsolete terminal-block expectations in these local tests, rerun all checks and
the upstream report, and review any newly comparable identities individually.
Escape start remains an explicit block even with the other three sibling states.
Dispatch, module/inventory lists, and status documentation can have small textual
conflicts; resolve additively without silently keeping stale block expectations.
