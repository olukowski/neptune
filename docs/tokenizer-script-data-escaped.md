# Script data escaped state

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


## Scope and pinned evidence

PR #32 implemented only `State::ScriptDataEscaped`, anchor
`#script-data-escaped-state`, on main baseline
`28131e11437941928eb7246885fb70e1160b2700` (PR #31). It uses existing
Context, Cursor, character tokens, diagnostics and Execution; no shared framework,
dependency, storage, adapter, fixture, pin, or policy changes are needed.

The immutable HTML Standard source revision is
[`404b3eb14a7a936dddefb2d78e224dbeb01d2358`](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source).
The complete source was retrieved from that revision's raw GitHub URL, not the
living Standard or another engine. Reviewed sections/definitions:

- [Assigned state, lines 143775–143802](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143775-L143802).
- [Preprocessing, next/current input and conceptual EOF, lines 142560–142607](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142607).
  Newline normalization is delegated to the existing Cursor; CR/CRLF become LF,
  including across chunks. Surrogates, noncharacters and controls retain their
  code points and receive preprocessing diagnostics on consumption, not on push.
- [Tokenization and reconsumption, lines 143149–143179](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143149-L143179):
  character and EOF token output, switching versus reconsumption. No branch of
  this state requests reconsumption.
- [Parse errors, lines 140986–141008](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L140986-L141008),
  [unexpected-null-character, lines 141479–141482](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L141479-L141482), and
  [eof-in-script-html-comment-like-text, lines 141138–141149](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L141138-L141149).
  State algorithms are normative; error-table explanations are non-normative.

## Branch mapping

Implementation: `crates/html/src/tokenizer/script_data_escaped.rs`.
Focused tests: `crates/html/tests/script_data_escaped_state.rs`.

| Consumed input | Required source effects / destination | Focused evidence |
| --- | --- | --- |
| `-` U+002D | Emit `-`; switch to Script data escaped dash | Exhaustive ASCII source snapshot, untouched suffix, separate real destination test |
| `<` U+003C | Emit nothing; switch to Script data escaped less-than sign | Exhaustive ASCII source snapshot, untouched suffix, separate terminal destination test |
| NUL U+0000 | Report `unexpected-null-character`; emit U+FFFD; stay | Exhaustive branch and repeated NUL/newline test with exact error positions and multiplicity |
| EOF | Report `eof-in-script-html-comment-like-text`; emit EOF | Empty/nonempty and seeded/default contexts, normalized EOF position, repeated terminal steps |
| Anything else | Emit current point unchanged; stay | All other ASCII points, Unicode/non-scalars, literal entity-like and malformed script text |

Every source step consumes exactly one normalized point (or conceptual EOF).
Delimiters are consumed, not reconsumed. The suffix probe is a source-boundary
inspection, not an attempt to resume a stub. Actual destination attempts are
separate tests. Historically both destinations were terminal stubs in PR #32.
With PR #34 integrated, Script data escaped dash executes its own source step,
consumes a surrogate suffix once, emits it unchanged with its preprocessing error,
and returns to Script data escaped. The next NUL is replaced/diagnosed there,
then open input suspends or closed input emits one EOF/error. Script data escaped
less-than sign remains terminal `UnimplementedState`, with no suffix consumption,
diagnostic, substitute EOF, or reconstructed execution. Closing input cannot
revive that stub. PR #34 changes no production algorithm in this state.

Exhausted open input and empty chunks are effect-free `NeedMoreInput`, preserving
the previous current item/position. Only explicit `finish` supplies EOF. EOF adds
exactly one error and one EOF token, even on repeated terminal calls. Diagnostic
coordinates follow the repository's normalized one-based code-point convention,
not byte/UTF-16 offsets. NUL is replaced here, not by preprocessing; literal
U+FFFD, surrogates, noncharacters and controls are not replaced here.

Full snapshots assert unrelated storage preservation for every branch: selected
tag, both pending tags (including retained/detached attributes and flags), pending
comment, temporary buffer, last start tag and return state. Existing token/error
prefixes remain ordered. No comment/tag is created, abandoned or emitted.
Ampersands are literal and do not invoke Character reference or alter return state.

## Streaming and entry boundaries

The six focused tests cover every branch, all ASCII points, Unicode/non-scalars,
exact diagnostics/positions/multiplicities, malformed input, consumption, repeated
suspension/completion, and default/seeded contexts. All two-chunk splits and
single-code-point streaming compare every non-suspension source-step snapshot
against closed input. Inputs include CRLF splits, consecutive/trailing CR,
repeated NUL, surrogate/noncharacter/control points, literal entity-like text,
and transitions through `-` into Script data escaped dash and back. With PR #33,
`--` executes real Script data escaped dash dash: hyphens stay there, NUL/literal
points return to Escaped, and `>` returns to Script data. Repeated three-state
cycles include NUL/CRLF boundaries and both real Script data continuation and
later explicit escaped-less-than-sign / escape-start terminal blocks. `-<`
still blocks at escaped less-than sign with its tail unconsumed.
Suspensions must preserve the complete snapshot; trace collection stops at a
stub without feeding it again.

Entry is isolated through `Execution::new(State::ScriptDataEscaped, context)`.
Script data escape start and escape start dash were stubs on PR #33's baseline.
PR #35 now adds escape start dash on merged main
`a0cf68b77662fb482373f78fa15b69a540b53623`, preserving this implementation and
its source assertions. Escape start alone still blocks normal Script data entry;
isolated escape-start-dash entry now reaches the real escaped trio. See
[PR #35 integrated tests and validation](tokenizer-script-data-escape-start-dash.md#focused-and-streaming-tests-after-integration). PR #32's source
assertions remain unchanged. PR #34 deliberately updates its old escaped-dash
stub assumption to the actual destination effects described above and extends
streaming traces through both states. PR #33 extends those unchanged source
assertions with real three-state streaming cycles; the less-than-sign stub
remains explicit.
The independent support inventory is registered additively.
No tree-builder mode switching or JavaScript parsing is implemented.

## Upstream fixture review and results

Fixtures remain pinned at `c777c408b61078ea2eb4acefc2535f54dbc8b28a`.
Inspected unchanged `domjs.test` cases 7–12: all start in Script data and expect
literal comment-like text plus `eof-in-script-html-comment-like-text` at EOF.
Case 7 is `<!--test` with its error at 1:9; cases 8/9 end after dash/dash-dash
and cases 10–12 reach double-escaped forms. These need earlier and/or later
states as well: all still stop at Script data escape start, not a comparable
prefix pass. They are not added to a pass baseline.

The historical standalone PR #32 runner report was (the integrated PR #34
report has the same upstream counts; see its state-local evidence):

- 6,827 fixture cases; 7,053 runs; 6,190 dispatch-attempted.
- **Executed 1,117 = passed 1,117 + failed 0; blocked 5,936.**
- 31 runs still block at Script data escape start.
- Delta from parent: executed +0, passed +0, failed +0, blocked +0.

No newly comparable upstream identity exists for this isolated state; no
state-specific pass-baseline file is warranted. All existing reviewed baseline
identities remain unchanged and passing. Local tests/support inventory are not
upstream conformance executions. Neither full conformance nor browser security
is claimed.

## Validation and parallel integration

Historical standalone PR #32 validation: all required commands passed locally
(exit 0); the workspace suite passed 253 Rust tests with 0 failed and 0 ignored,
including the six focused state tests. These Rust test totals are separate from
upstream run counts. For the rerun after PR #34 integration, see
[Script data escaped dash evidence](tokenizer-script-data-escaped-dash.md#upstream-and-validation).
For the subsequent PR #33 integration onto merged PRs #32/#34, see
[escaped dash dash validation](tokenizer-script-data-escaped-dash-dash.md#validation-and-upstream-report).

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo test --workspace --locked pinned_upstream_runner -- --nocapture
```

PR #32 originally implemented only this state against the common baseline.
PR #34 inherited merged PR #32 from main
`e2a9b926938d022820613c752b467e16130521bc` and adds Script data escaped dash;
on that historical baseline, the escape-start states, escaped dash dash, and
escaped less-than sign still blocked. PR #33 then inherited both merged states
from main `04283260b9d467d198ab92cb8ccd5721c51fe21e` and added escaped dash dash;
on that baseline, both escape-start states and escaped less-than sign remained
those boundaries. PR #35's current entry boundary is described above.
No unmerged sibling branch is imported. Module/dispatch/support inventory
registration is additive. Future integrations must rerun all checks/upstream
reporting and review moved terminal blocks or newly comparable cases without
skipping tests, auto-blessing identities, or resuming blocked execution.
The user now authorizes sequential integration by the parent, with a fresh
independent review of each final revision before merge.
