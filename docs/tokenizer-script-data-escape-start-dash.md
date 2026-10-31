# Script data escape start dash state

## Scope, historical baseline and integrated base

PR #35 implements only `State::ScriptDataEscapeStartDash`, in
`crates/html/src/tokenizer/script_data_escape_start_dash.rs`. Existing Context,
Cursor, character tokens, diagnostics and Execution suffice. No adjacent algorithm,
shared framework, dependency, storage, adapter/policy, fixture or pin is changed.

Historical standalone head `588b925e1b994142903c0f64865d01e86cd356bb` used main
`28131e11437941928eb7246885fb70e1160b2700` (PR #31); both the assigned state and
its escaped-dash-dash destination were stubs on that original base. That head's
eight local tests and 255 workspace tests passed before integration.

The integrated base is main **`a0cf68b77662fb482373f78fa15b69a540b53623`**,
containing merged PRs #32 (Escaped), #34 (EscapedDash), and #33
(EscapedDashDash). All three production implementations, source assertions and
registrations are preserved. Module/dispatch, inventory and status-link conflicts
were resolved additively; no unmerged sibling branch was imported. The exact
final integrated head is recorded in PR #35's body, paired with this base.

EscapeStart remains unsupported, so entry into the assigned state is still
isolated. After this state's hyphen branch, real EscapedDashDash now executes:
literal/NUL returns to Escaped, hyphens cycle through the escaped trio, and `>`
returns to Script data. EscapedLessThanSign remains a terminal stub.

## Immutable evidence and branch mapping

Retrieved and inspected the exact [HTML Standard source revision
404b3eb14a7a936dddefb2d78e224dbeb01d2358](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source),
not the living algorithm or another engine. Pins in [sources.toml](sources.toml)
are unchanged. Reviewed sections and referenced definitions:

- [Assigned state, lines 143758–143773](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143758-L143773),
  anchor `#script-data-escape-start-dash-state`.
- [Preprocessing and input characters, lines 142560–142607](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142607):
  surrogate/noncharacter/control diagnostics, state-local NUL handling, the linked
  Infra normalize-newlines operation, current/next input and conceptual EOF.
  Existing Cursor owns preprocessing; reconsumption neither repeats it nor advances.
- [Tokenization, lines 143150–143182](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143150-L143182):
  source steps, reconsumption in the specified state, and character-token output.
- [Script data, lines 143338–143359](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143338-L143359):
  existing fallback destination owns NUL replacement/error, `<` transitions,
  literal output (including `&`) and EOF emission.
- [Escape start, lines 143741–143755](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143741-L143755):
  a hyphen would reach the assigned state, but that predecessor remains a stub.
- [Escaped trio, lines 143775–143869](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143775-L143869):
  already merged destinations used for integrated test expectations. Their
  algorithms are not changed here. In particular, escaped EOF reports
  `eof-in-script-html-comment-like-text`, whereas ordinary Script data EOF does not.

| Consumed input | Assigned source effects | Focused evidence |
| --- | --- | --- |
| U+002D `-` (143765–143767) | Emit exactly one hyphen; switch to EscapedDashDash, without reconsumption. | Unchanged exhaustive source snapshots and cursor-only untouched-tail probe; real destination snapshots, suspension/EOF and three-state cycles. |
| Anything else (143769–143770) | No output or state diagnostic; reconsume in Script data, including conceptual EOF. | All ASCII points and representative Unicode/non-scalars; exact scheduling, destination current point/position and untouched tail; EOF and real continuation tests. |
| Open-input exhaustion (execution contract, not a third spec branch) | Effect-free NeedMoreInput, retaining state/current item/output/diagnostics/storage. | Repeated suspension with/without cursor history; later feed/finish and streaming snapshots. |

This source never replaces NUL, reports a state parse error or emits EOF. NUL
replacement and its one `unexpected-null-character` diagnostic occur only in the
reconsuming Script data destination. Surrogates/noncharacters/controls retain their
point and single preprocessing diagnostic. Positions count normalized code points;
CR/CRLF handling remains on Cursor. Unrelated selected/unselected pending tags,
retained/detached attributes, comments, history, return state, temporary buffer,
prior tokens and diagnostics are preserved at the source boundary.

## Focused and streaming tests after integration

`crates/html/tests/script_data_escape_start_dash_state.rs` has **nine** local tests
(eight on the historical standalone head):

- Every ASCII point plus Unicode scalars, U+0080/U+009F controls, U+00A0,
  U+FEFF, supplementary input, actual U+D800/U+DFFF surrogates and noncharacters.
  Open/closed input and absent/start/end selectors verify exact source snapshots,
  all unrelated storage, state/cursor diagnostics, normalized positions, one
  reconsumption and the destination step separately.
- The exhaustive hyphen branch retains its original source snapshot, then real
  EscapedDashDash consumes the untouched `z` suffix at 2:5 and enters Escaped.
  Open exhaustion suspends; closed input emits exactly one EOF/error at 2:6.
- A separate cursor-only hyphen probe, before destination execution, proves exactly
  one point consumed and no scheduled reconsumption. It is not used as an
  integration execution with a skipped suffix.
- Empty open entry repeatedly suspends with no current item or with prior history;
  only explicit, idempotent finish takes EOF fallback. Only the Script data step
  emits EOF, with no escaped EOF error; repeated terminal steps have no effects.
- Feeding suspended entry neither invents predecessor output nor creates storage.
  After a hyphen, real EscapedDashDash repeatedly suspends on open exhaustion;
  explicit finish adds one escaped EOF/error at 1:2. No stale destination-stub
  expectation remains.
- CRLF fallback followed by two NULs, surrogate, noncharacter, control and literal
  ampersand checks exact token order, positions and diagnostic multiplicities.
- Real fallback continuation completes literal text, or explicitly stops at later
  EscapeStart / CharacterReference stubs without accepting prefixes.
- New fixed, specification-derived snapshots enter the escaped trio through this
  state's hyphen, then cycle through literal points, NUL in DashDash and Dash,
  split-normalizable CRLF, surrogate/noncharacter/control points and hyphen runs.
  Every step asserts exact state, current point/position, output, diagnostics and
  preserved storage. Three exits separately test escaped EOF at 3:7, `>` back to
  Script data followed by CRLF/NUL and error-free EOF at 4:2, and terminal
  EscapedLessThanSign at 3:7 with no `<` output or suffix consumption. Open/closed
  variants distinguish suspension from explicit EOF at each completing exit.
- Every two-chunk split (including empty chunks) and single-code-point streaming
  compares all non-suspending snapshots with closed input. Cases now include
  literal comment-like suffixes, all three escaped-less-than-sign paths, repeated
  escaped cycles, `>`/CRLF/NUL continuation, non-scalars, matching end tags and
  later explicit escaped-less-than-sign / CharacterReference / EscapeStart blocks.
  Collection stops at the first block without feeding/reconstructing/resuming it.
- Actual Script data entry on `<!--tail` still emits `<!`, then stops at
  EscapeStart before consuming a hyphen; isolated entry remains intentional.

Three old hyphen-destination stub assertions failed on initial rebase as expected.
They were deliberately replaced by the real-destination checks above while
retaining all assigned-source assertions. The merged trio tests did not assume
EscapeStartDash was unsupported and remain unchanged. Later explicit blocks are
preserved; no expectation uses dynamic support detection or test skipping.
Inventory/local tests are not upstream conformance cases.

## Upstream evidence and exact validation

Fixtures remain pinned at `c777c408b61078ea2eb4acefc2535f54dbc8b28a`.
Inspected examples include `test1.test` tests[35], “<!- in script data”, whose
input/output is literal `<!-test-->` with no errors, and tests[36], “Escaped script
data”, whose input/output is `<!--test-->` with no errors. Local tests execute
suffixes `test-->` and `-test-->` from isolated entry, not their blocked upstream
initial-state runs. `domjs.test` tests[5] (NUL comment-like text) and tests[7–9]
(EOF in comment-like text) were inspected as independent fixture evidence. They
still block at EscapeStart; their later diagnostics do not belong to this source.

Historical standalone results, current merged base and integrated PR #35 have
identical upstream counts:

| Category | Integrated base | Integrated PR #35 | Delta |
| --- | ---: | ---: | ---: |
| Fixture cases | 6,827 | 6,827 | 0 |
| Runs | 7,053 | 7,053 | 0 |
| Dispatch-attempted | 6,190 | 6,190 | 0 |
| Executed | 1,117 | 1,117 | 0 |
| Passed | 1,117 | 1,117 | 0 |
| Failed | 0 | 0 | 0 |
| Blocked | 5,936 | 5,936 | 0 |

Executed = passed + failed. Blocks remain CharacterReference 4,692;
TokenVariants 858; MarkupDeclarationOpen 242; CdataSection 56;
ProcessingInstructionOpen 52; ScriptDataEscapeStart 31; XmlCoercion 4;
PendingSpecReview 1. All 921 protected prior passing identities remain unchanged
and passing. No new comparable identity exists, so no state-specific baseline
file is applicable. No fixture/pin/adapter/policy exclusion was changed, nor any
execution auto-blessed. The 31 earlier blocks do not promise 31 eventual passes.

All validation commands passed (exit 0) after integration:

```sh
cargo test --locked -p neptune-html --test script_data_escape_start_dash_state
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo test --workspace --locked pinned_upstream_runner -- --nocapture
```

Integrated focused tests: **9 passed, 0 failed**. Integrated workspace
unit/integration/doc tests: **277 passed, 0 failed, 0 ignored**, including the
unchanged 21 tests in the inherited escaped trio suites. Historical standalone:
8 focused and 255 workspace tests passed. Rust test totals are separate from
upstream conformance runs. No local validation/infrastructure blocker remains.

## Sequential integration and remaining limits

The user authorized sequential integration by the parent with fresh independent
review immediately before each merge. PRs #32/#34/#33 are already merged; this PR
is next and EscapeStart PR #36 follows. This thread rebases onto the exact merged
base, starts no subagent/reviewer/thread/workflow and does not merge or enable
auto-merge. Parent will review the final exact pushed revision.

EscapeStart still prevents normal entry into the new handler and escaped trio.
EscapedLessThanSign and other unsupported states remain terminal UnimplementedState,
never substitute EOF or recoverable parse errors. PR #36 integration must preserve
source assertions, review moved blocks/newly comparable identities individually,
and rerun all checks/report; never auto-bless cases or resume a blocked execution.
There is no tree builder, automatic script mode switching or JavaScript parser.
This is partial standalone tokenization, not full conformance or browser-security
evidence.
