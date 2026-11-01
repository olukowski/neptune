# Comment end dash state

## Pinned normative evidence and scope

Implements only **Comment end dash state** (`#comment-end-dash-state`) from
[WHATWG HTML source 404b3eb14a7a936dddefb2d78e224dbeb01d2358,
lines 144660–144678](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144660-L144678),
fetched and read at that exact revision. The preprocessing rules and conceptual
EOF definition are at lines 142560–142607; current/next input, reconsumption,
comment data and token creation/emission are at lines 143149–143188.
The source pin and upstream fixtures are unchanged.

Prerequisite PR #46 is present at merged commit
`47c53e8dcf3e69d920c4be290036ffb70af72cc9`. After independent review of the
original head `55a7b1df965ebbf882bd6790de339dfc8f15ef39`, this branch was rebased
onto `3f1ea04c8c5204d049cf50fb18981dac0f5fa844` (merged #47/#48/#49).
The implementation lives in
`crates/html/src/tokenizer/comment_end_dash.rs` and reuses private
`comment::data_mut` and `comment::emit`, existing `PendingComment` and
`Token::Comment`. The shared helper `comment.rs` is unchanged; it is not the
core Comment state implementation. No sibling state, storage/token variant,
dependency or adapter capability is added.

Direct isolated entry supplies initialized pending-comment context. Existing
data is literal code-point storage, including NUL, CR, surrogates and
noncharacters; it is never preprocessed again. The independent support inventory
already initializes `pending_comment` for supported states, including this one.

## Branch mapping

| Consumed input | Source effects | Focused proof |
| --- | --- | --- |
| U+002D `-` | Switch to CommentEnd; do not append, emit, diagnose or reconsume. | Exhaustive ASCII/source snapshot test; pre-destination cursor probe sees the untouched suffix, not a replayed hyphen. |
| EOF | Report `eof-in-comment` at EOF; take and emit current comment, then EOF. No withheld hyphen append, reconsumption or switch to Data. | Empty/seeded comment tests at fresh and normalized-history positions, exact token boundaries/order and diagnostics, terminal repeats, caller-reconsumed EOF. |
| Anything else | Append exactly one U+002D to existing comment data; reconsume current point in Comment. Do not append that point or perform destination effects. | Every ASCII point plus Unicode, NUL, controls, actual surrogates/noncharacters; exact source snapshots and cursor replay/position/input-error multiplicity probes. |
| Open exhaustion (execution contract) | NeedMoreInput, with no effects; not the EOF branch. | Repeated empty chunks/suspension, then explicit finish or supplied input; all chunk splits. |

Focused tests are entirely local in
`crates/html/tests/comment_end_dash_state.rs`. They preserve unrelated start/end
tags, retained and detached attributes, tag selector (absent/start/end), last
start tag, character-reference return state, scratch and prior diagnostics/output.
EOF emission removes only the pending comment and preserves literal data and
separate comment boundaries. Positions count normalized code points, not UTF-8
bytes or UTF-16 units. CR becomes LF; suppression of an LF following a consumed
CR is also exercised across the entry/chunk boundary.

Both destinations remain unsupported on the rebased base. Tests inspect source
effects **before** attempting Comment/CommentEnd, then assert exact terminal
`UnimplementedState` with no subsequent effects. Separate cursor probes prove
consumption/reconsumption before destination dispatch and end without dispatching
with a probed cursor. No blocked execution is reconstructed or resumed.

Closed-input traces are compared with every two-chunk split (including empty
chunks) and single-code-point chunks of representative hyphen, fallback, EOF,
malformed, CRLF, consecutive/trailing CR, NUL, scalar and non-scalar inputs.
A source step immediately leaves this state; traces stop honestly at a destination
stub, rather than inventing later comment processing. Caller-scheduled input and
conceptual EOF reconsumption are covered without simulating a predecessor state.

## Rebased predecessor integration

Merged #48's CommentLessThanSignBangDash fallback now reaches this state.
Its state-local tests retain their original source snapshots (no append,
diagnostic, emission or comment take in that predecessor), then execute the real
CommentEndDash destination. Non-EOF fallback appends only `-` and schedules the
same point in still-unsupported Comment, with no duplicate input errors. EOF is
reconsumed at its original normalized position, diagnoses `eof-in-comment`, takes
and emits the unchanged comment, emits EOF and finishes exactly once. The
predecessor's hyphen branch still blocks at CommentLessThanSignBangDashDash.

The exhaustive ASCII/non-scalar, resumed-feed, EOF and all-chunk-split suites in
`comment_less_than_sign_bang_dash_state.rs` now check these actual destinations,
with source/destination effects independently inspectable and cursor probes before
remaining stub attempts. Its state-local documentation records the new boundary.
The reviewed CommentEndDash algorithm and its six original tests are unchanged;
no additional production algorithm or gateway is added by the rebase fixes.

## Validation and conformance boundary

All CONTRIBUTING checks pass: pinned submodule initialization, rustfmt check,
Clippy (all targets, warnings denied), locked workspace tests, and rustdoc
(warnings denied). The rebased workspace has 403 passing tests, zero failures
and zero ignored tests, including the six unchanged focused tests and five
updated predecessor/integration tests.

The separately run `pinned_upstream_runner -- --nocapture` reports, identically
before and after this change:

- 6,827 fixture cases; 7,053 case/initial-state runs.
- 6,190 dispatch-attempted.
- **Executed 1,148; passed 1,148; failed 0; blocked 5,905.**
- Blocked: CharacterReference 4,692; MarkupDeclarationOpen 242;
  CdataSection 56; ProcessingInstructionOpen 52; TokenVariants 858;
  XmlCoercion 4; PendingSpecReview 1.

No new upstream passing identities are admitted; every existing pass baseline
is unchanged and remains protected by the runner regression tests. Inventory
checks and focused/synthetic tests are not counted as conformance executions.

Normal Data entry for `<!--text-->` still stops terminally at
MarkupDeclarationOpen after consuming `<!`, without creating a pending comment
or emitting comment/EOF tokens; the local suite asserts this explicitly.
No new Data-entry coverage, complete comment-family support, whole-tokenizer
streaming conformance, full HTML conformance or browser-security claim is made.
Neighbor implementations can legitimately move terminal blocks on later rebases;
source-step expectations must remain independent of those integration changes.
