# Script data double escape end state

## PR #38 integration update

The original standalone evidence below is historical for predecessor support.
On base `542a5060d085bd546387e2781bbf10039fb2e197` (merged PR #43) plus PR #38,
DoubleEscapedLessThanSign is now implemented and reaches this state on `/`.
Its source step clears the temporary buffer and emits `/`; this state then
executes its existing algorithm unchanged. The old immediate-predecessor stub
test now checks that real source step, each `script` accumulation step, matched
`>` transition to Escaped, and actual Escaped EOF/error, with exact snapshots.
Ordinary ScriptData entry still blocks at EscapedLessThanSign, and
DoubleEscaped remains a terminal destination stub. All original state-local
branch tests and the normal-entry earlier block are preserved.

Current integrated validation: all CONTRIBUTING checks and the pinned runner
pass; **300 Rust tests passed, 0 failed, 0 ignored**. Upstream remains **1,126
executed/passed, 0 failed, 5,927 blocked**, with no new identities or baseline
changes. See [PR #38's current integrated evidence](tokenizer-script-data-double-escaped-less-than-sign.md#current-integrated-evidence-after-pr-43)
for exact base, original standalone head, current counts and integration limits;
the final integrated review head is recorded in the PR body. The remainder of
this document records PR #43's original standalone audit, including its former
predecessor-stub statements and original 292-test total, not current support.

## Scope and source

This PR implements exactly `State::ScriptDataDoubleEscapeEnd`, anchor
`#script-data-double-escape-end-state`, on reviewed main base
`39d772e31b98e6e955a2e00f546a8db3506eb1dd` (after PR #36).
Implementation: `crates/html/src/tokenizer/script_data_double_escape_end.rs`.
Local tests: `crates/html/tests/script_data_double_escape_end_state.rs`.
No sibling implementation, fixture/pin change, dependency, storage redesign,
runner policy, or baseline-registration change is included.

The complete HTML source was retrieved from the immutable raw GitHub URL for
revision `404b3eb14a7a936dddefb2d78e224dbeb01d2358`, not the living Standard or
another engine. Reviewed normative sections and referenced definitions:

- [Assigned algorithm, lines 144110–144142](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144110-L144142).
- [Immediate predecessor, lines 144093–144107](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144093-L144107):
  double-escaped less-than sign clears scratch on `/`, emits `/`, and switches
  here. That predecessor is still a stub on this branch.
- [Preprocessing, current/next input, conceptual EOF, lines 142560–142607](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142607).
  CR/CRLF become LF through the existing cursor. Surrogates, noncharacters and
  non-whitespace/non-NUL controls retain code-point identity and receive input
  diagnostics on first consumption. NUL handling is left to state algorithms.
- [Tokenization, reconsumption, scratch and output, lines 143149–143179](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143149-L143179).
  Switching consumes the next item; reconsumption supplies the current item
  without advancing or repeating input diagnostics.
- [HTML's imported Infra definitions, lines 2284–2305](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L2284-L2305):
  [ASCII upper alpha](https://infra.spec.whatwg.org/#ascii-upper-alpha) is
  U+0041–U+005A, [ASCII lower alpha](https://infra.spec.whatwg.org/#ascii-lower-alpha)
  is U+0061–U+007A. These linked definitions and
  [normalize newlines](https://infra.spec.whatwg.org/#normalize-newlines) were
  also read from Infra. The assigned pinned HTML algorithm explicitly specifies
  adding 0x20, not Unicode folding, and lists the six delimiters itself.
- [Escaped destination, lines 143775–143802](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143775-L143802), and
  [double-escaped destination, lines 143993–144023](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143993-L144023).
  Only the former is implemented on this branch. Destination NUL/EOF behavior
  must not be performed by this source step.

## Branch mapping

| Pinned branch | Source effects | Local evidence |
| --- | --- | --- |
| TAB, LF, FF, SPACE, `/`, `>` (144117–144127) | If scratch is exactly lowercase `script`, switch to Escaped; otherwise DoubleEscaped. Emit the current delimiter unchanged. Do not append, clear, or emit scratch; do not reconsume. | `exhaustive_source_branches_…`, `all_script_case_spellings_…`, `delimiter_equality_is_exact_…`; real Escaped continuation and independent DoubleEscaped terminal assertions |
| ASCII uppercase (144129–144132) | Append point + 0x20 to scratch; emit original point; remain here. | All 26 uppercase letters, all 64 case spellings of `script`, per-letter snapshots, seeded scratch append |
| ASCII lowercase (144134–144137) | Append original point to scratch; emit original point; remain here. | All 26 lowercase letters, per-letter snapshots and source traces |
| Anything else (144139–144140), including conceptual EOF | Preserve scratch, emit/diagnose nothing, reconsume in DoubleEscaped. | Every other ASCII point, Unicode, actual surrogates/noncharacters/controls, NUL and malformed input; duplicate-reconsume rejection, current input/position, suffix probes, EOF and typed terminal block |

Open exhaustion is **NeedMoreInput**, not the anything-else branch or EOF.
The handler consumes once, with state-local effects complete before a destination
is attempted. It never creates, changes, abandons or emits a tag/comment, changes
last-start-tag/return-state, replaces NUL, emits EOF, or reports a state error.
Pre-existing token/error prefixes remain intact. Input diagnostics retain exact
normalized one-based positions and multiplicity.

Scratch comparison is exact code-point equality, independent of last-start-tag,
selected tag or pending end-tag name. Existing scratch is not normalized again:
`SCRIPT`, `Script`, prefixes/suffixes, NUL-containing strings and Unicode lookalikes
fail the exact delimiter test. Newly consumed ASCII letters alone are lowercased
in scratch; emitted characters retain original spelling. Unlike end-tag-name
states, scratch is not an original-spelling fallback buffer here, and no current
tag is required. Even a buffer already equal to `script` must take the fallback
branch on `-`, `<`, `&`, NUL or EOF, not the delimiter branch.

## Entry, tests and streaming

Direct initial entry uses
`Execution::new(State::ScriptDataDoubleEscapeEnd, context)` with an empty
`temporary_buffer`, as the eventual predecessor establishes. Mid-state isolated
entry supplies its already accumulated prefix. The independent support inventory
registers this state additively and explicitly supplies normalized prefix scratch
for open-exhaustion/EOF entry checks. No shared prerequisite is missing.

Eight focused Rust tests cover:

- Every ASCII point plus non-ASCII boundaries, Unicode whitespace/letters,
  surrogate/noncharacter/control code points, NUL and malformed script-like input.
  Full snapshots preserve selected Start/End/None and unrelated pending tags,
  retained/detached attributes, flags, comment, last tag, return state and prefixes.
- Consumption versus reconsumption, unchanged tails, current input and normalized
  positions, including CRLF cursor history. Cursor-only suffix probes occur before
  destination dispatch, never after a terminal block.
- All 64 case spellings of `script` at every delimiter; accumulation is inspected
  before the delimiter, followed by an actual Escaped step consuming the suffix.
- Repeated effect-free suspension, empty chunks, explicit/idempotent finish,
  EOF reconsumption (including with exact `script` scratch), repeated typed stub
  errors, and no invented EOF token/escaped EOF diagnostic at a stub.
- Real Escaped continuation after a match, with two NUL replacements/errors at
  2:1 and 2:2, surrogate/noncharacter/control input diagnostics at 2:3–2:5, and
  exactly one destination EOF/error at 2:6. State-local effects are inspected
  separately before destination execution; unrelated storage remains unchanged.
- All two-chunk splits and single-code-point chunking compare every non-suspension
  source-step snapshot to closed input for default/seeded context and four scratch
  prefixes. Samples include ASCII letter/delimiter boundaries, mixed case, exact
  and near-miss names, malformed input, EOF after letters, CRLF, consecutive and
  trailing CR, NUL/non-scalars, real escaped hyphen cycles/return to Script data,
  and later explicit terminal blocks. Trace collection stops at the first terminal
  result, never feeds/resumes/reconstructs a blocked execution, and never detects
  support dynamically or compares only output prefixes.

No real predecessor reaches this state on the reviewed base. A normal Script
data path through `<!--<script></script> -->` reaches the still-unimplemented
EscapedLessThanSign at the first escaped `<`, with only `<!--` emitted. A separate
immediate-predecessor test proves DoubleEscapedLessThanSign remains effect-free
and terminal, without consuming `/` or clearing supplied scratch. These are
boundary tests, not implementations or simulated resumption of neighboring states.
No old destination-block assertion referred to this assigned state, so none
outside the independent support inventory needed changing.

## Pinned upstream evidence and validation

Fixtures remain pinned at `c777c408b61078ea2eb4acefc2535f54dbc8b28a`; the
submodule was already initialized. No fixtures, pins or prior baselines changed.
Reviewed unchanged `domjs.test` zero-based cases:

- tests[17], `</script> in script HTML comment - double escaped`, expects literal
  `<!-- <script></script> -->` followed by an end tag. The inner `</script>` would
  exercise this state's exact delimiter decision after the missing predecessors.
- tests[18], nested `<script>`, distinguishes double-escaped inner text from a
  later real end tag. tests[22], incomplete end name `</scrip>`, requires a
  nonmatching delimiter decision; tests[23], `</script-->`, requires fallback
  rather than treating the `script` prefix as a completed name.

All these whole fixture runs still block at EscapedLessThanSign. They are not
new passes and are not added to a state-specific pass file. All 930 previously
protected identities and their registrations remain unchanged and passing.
Zero new upstream passes is the truthful standalone result for this state.

All checks passed (exit 0):

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo test --workspace --locked pinned_upstream_runner -- --nocapture
```

Workspace Rust tests: **292 passed, 0 failed, 0 ignored**, including eight new
focused tests. These and the support/fixture inventory are not upstream runs.
The separate pinned runner reports 6,827 fixture cases, 7,053 initial-state runs,
and 6,190 dispatch-attempted runs:

- **Executed: 1,126 = passed 1,126 + failed 0.**
- **Blocked: 5,927.** Unchanged from the base's recorded report; new passes: 0.
- Blocks: CharacterReference 4,692; MarkupDeclarationOpen 242;
  EscapedLessThanSign 22; CdataSection 56; ProcessingInstructionOpen 52;
  TokenVariants 858; XmlCoercion 4; PendingSpecReview 1.

No full conformance, browser-security, tree-builder mode switching, JavaScript
parsing, or Comment-token behavior is claimed.

## Later integration boundaries

This branch intentionally does not fetch/cherry-pick/merge siblings, rebase across
their changes, merge a PR, or enable auto-merge. The coordinator owns later
integration after the user's signal. There is no delivery/check blocker; the
remaining execution boundaries are the predecessor and DoubleEscaped stubs plus
normal-entry EscapedLessThanSign. Escaped itself is real on this base.

Later sibling merges will share narrow module/dispatch/inventory and status-doc
registration sites. Preserve this algorithm's source-step assertions, then review
and update terminal boundary tests to actual destination effects when those
states become implemented. Rerun every required check and the upstream report;
review newly comparable fixture identities individually before adding a separate
state-local baseline. Never weaken comparisons, replace previous baselines,
resume a terminal execution, or claim today's blocked cases as passes.
