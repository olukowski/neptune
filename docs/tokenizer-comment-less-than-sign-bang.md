# Comment less-than sign bang state

## Current Comment integration (PR #54)

On base `f2ca599065ec7fa70cc437f5c2ea6b9e2ee68883` (merged #47–53) plus
PR #54, Comment is implemented alongside the seven merged comment states.
Bang fallback now executes real Comment; the real BangDash/EndDash fallback
also reaches Comment. Every original source snapshot is retained before actual
replacement/append/emission. BangDashDash remains a genuine terminal stub.

All affected state suites execute actual destinations and every-split streaming
paths; [core Comment integration evidence](tokenizer-comment.md#integration-onto-47-53)
records independently expected loops, closures, diagnostics, positions and genuine
remaining blocks. No production sibling algorithm or shared helper changes.
Normal Data entry still blocks at MarkupDeclarationOpen. No new upstream pass
identities or full-conformance claim. **The PR-local support boundaries/counts
below are historical; pinned branch evidence remains normative.**

## Pinned source and scope

Normative source: WHATWG HTML revision
`404b3eb14a7a936dddefb2d78e224dbeb01d2358`, recorded in
[sources.toml](sources.toml), [Comment less-than sign bang state,
lines 144610–144624](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144610-L144624).
The opening [Tokenization reconsume definition, lines 143150–143162](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143150-L143162)
defines switching states while providing the current input on the next consume.
The exact immutable source was read directly, not inferred from another engine.

Only `CommentLessThanSignBang` is implemented in
`crates/html/src/tokenizer/comment_less_than_sign_bang.rs`. The shared
`comment.rs` helper module is unchanged. This state needs neither data mutation
nor emission helpers: it preserves the existing `PendingComment` and emits no
`Token::Comment` (or other token). No storage, token variant, dependency,
fixture, specification pin, adapter, or pass baseline changes are needed.

Direct isolated entry supplies initialized pending-comment context. In normal
predecessor context, `<!` has already been consumed and appended. This state
must not append that prefix again.

## Branch evidence

| Consumed input | Source effects | Destination |
| --- | --- | --- |
| U+002D HYPHEN-MINUS | Consume once; do not append or reconsume it. | Comment less-than sign bang dash |
| Anything else, including conceptual EOF | Consume once and schedule reconsumption of the same input and position; no append, replacement, emission, or state parse error. | Comment |
| Exhausted open input | Suspend without source effects; this is not conceptual EOF. | Same source state |

The last row is the incremental execution contract, not an additional HTML
branch. In particular, NUL is not replaced here, Unicode dashes are not ASCII
hyphens, and EOF neither takes/emits the pending comment nor reports
`eof-in-comment`. Input preprocessing may normalize CR or report a diagnostic
before this source step; stored comment data is never preprocessed again.

Both destinations were terminal stubs on the original prerequisite base. After
integration onto main through PR #52, Comment remains a stub but Bang dash and
its End dash fallback execute (see below). Tests inspect source effects before
attempting destinations. No blocked execution is reconstructed or resumed.
Normal Data `<!--` entry still blocks at Markup declaration open: this PR adds
no Data-entry coverage.

## Focused verification

`crates/html/tests/comment_less_than_sign_bang_state.rs` contains local helpers
and eight focused tests:

- Every ASCII point, Unicode dash lookalikes, scalar/non-scalar points, controls,
  noncharacters and NUL; exact output, diagnostic codes/multiplicity, normalized
  current-input positions and intended targets with open and closed input.
- Empty, predecessor-prefix, and literal malformed pending data (including NUL,
  CR and lone surrogates); no append, clear, replacement or take. Existing
  emitted comment boundaries/order, both tag selectors and retained/detached
  attributes, history, scratch, return state and prior diagnostics are preserved.
- Cursor-only probes before any destination attempt prove the consumed hyphen,
  reconsumed fallback and unchanged tail; reconsumption does not duplicate
  preprocessing diagnostics. A separate probe checks conceptual EOF.
- Repeated effect-free exhaustion, explicit EOF, feeding after suspension, and
  effect-free repeated completion/terminal blocks. Implemented Bang dash suspends
  on open exhaustion; only an actual remaining stub blocks without input.
- Every two-chunk split (including empty chunks) and single-code-point chunks of
  representative inputs match closed-input source-step traces, including real
  Less-than sign entry, hyphen chains, Bang dash/End dash fallback and EOF. They
  stop at the first genuine stub or completion; no tail after a block is processed.
- Real Data source steps preserve the Markup declaration open terminal block.

The independent support inventory initializes consumed `<!` and pending-comment
data for isolated entry. Inventory and focused tests are not upstream conformance.
Existing shared-comment tests continue to prove actual emission/take semantics;
this state specifically proves that no take occurs.

## Integration on main

Rebased onto `76f91837fc17eb6de8ec313a1c5dcac906203b41`. The production algorithm
is byte-for-byte unchanged from initially reviewed `a009d68`. Merged algorithms,
shared helpers, storage and adapter capabilities remain untouched. Changes to
integration expectations are derived from the same pinned source: [Less-than sign,
Bang dash and End dash, lines 144590–144678](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144590-L144678).

- Less-than sign consumes/appends `!`, then this source consumes the next point,
  not that `!` again. Its source assertions remain intact in the predecessor suite.
- This source's `-` now reaches real Bang dash. A second `-` consumes once and
  still blocks at Comment less-than sign bang dash dash with no speculative effects.
- Bang dash fallback reconsumes in real End dash, which appends only the withheld
  `-` and reconsumes the same point in the still-unsupported Comment state. All
  ASCII and representative malformed/non-scalar followers are checked after each
  source step, with exact diagnostics and positions; preprocessing occurs once.
  In particular, neither NUL replacement nor a Comment parse error is simulated.
- A single `-` followed by EOF now completes through End dash. Bang dash schedules
  EOF reconsumption without emission/error; only End dash reports `eof-in-comment`,
  takes/emits the unchanged seeded comment, then emits EOF after prior tokens.
  The consumed hyphen is not appended on this EOF branch. Both already-closed
  input and repeated open suspension followed by explicit finish are covered.
- Every two-chunk split and single-point chunks cover both isolated entry and
  real Less-than sign entry with repeated `<`, `!`, hyphen chains, malformed input,
  CRLF boundaries, controls and non-scalars. Traces include actual destination
  effects, not reconstructed execution or matching prefixes before a stub.

The opening support list in `tokenizer.md` and the independent inventory contain
all implemented states exactly once; module/dispatch/inventory changes preserve
all merged entries and add only this state's registration and entry context.

All prior protected upstream identities are retained; no new passing identity is
introduced because this state is not reachable from upstream initial states
without Markup declaration open. See the PR report for executed/passed/failed/
blocked totals and full check results. This is partial tokenizer support, not
full conformance or a browser-security claim.
