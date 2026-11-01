# Comment less-than sign bang dash dash

## Pinned branch evidence

Normative source: WHATWG HTML revision
`404b3eb14a7a936dddefb2d78e224dbeb01d2358`, recorded in
[sources.toml](sources.toml),
[Comment less-than sign bang dash dash state, lines 144642–144657](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144642-L144657)
(`#comment-less-than-sign-bang-dash-dash-state`). The implementation was derived
from that immutable source, not a living-standard snapshot or another engine.
Also checked: [preprocessing/current-next input, lines 142560–142594](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142594),
[Tokenization/reconsumption, lines 143157–143190](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143157-L143190),
and [nested-comment definition, lines 141335–141339](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L141335-L141339).

Algorithm: `crates/html/src/tokenizer/comment_less_than_sign_bang_dash_dash.rs`.
Focused tests: `crates/html/tests/comment_less_than_sign_bang_dash_dash_state.rs`.
`tokenizer/comment.rs` remains the private **shared helper**, not the Comment
state implementation. No helper, token/storage variant, dependency, sibling
algorithm, fixture, specification pin, or baseline is changed here.

| Consumed input | Source effects | Focused evidence |
| --- | --- | --- |
| `>` U+003E | Reconsume in CommentEnd, no state error | Exhaustive ASCII test, suspension/feed test, source-before-block test, chunk traces |
| EOF | Reconsume conceptual EOF in CommentEnd, no state error | Open-exhaustion/explicit-EOF test, source-before-block test, empty/suppressed-LF chunk traces |
| Anything else | Report `nested-comment` at the consumed normalized position; reconsume in CommentEnd | All other ASCII points, NUL, malformed punctuation, Unicode/scalars, surrogates, noncharacters and controls; exact diagnostics and chunk traces |
| Open exhausted input | Infrastructure suspension, not specification EOF; no effects | Repeated suspension with/without cursor history, empty chunks, suppressed LF after CR, later input/finish |

All branches preserve pending-comment data literally: no deferred `--` append,
NUL replacement, emission, token creation, or comment take occurs in this state.
In particular EOF neither emits EOF nor reports `eof-in-comment`; such destination
work must not be supplied early. The shared `comment::emit` helper remains the
existing take-on-emission path, but this source never calls it.

## Isolation and coverage boundary

Direct entry supplies initialized `PendingComment`, empty or seeded with literal
NUL/CR/LF/non-scalars and delimiter-like text. Tests compare exact complete output,
state/cursor diagnostics, normalized positions and unrelated context (both pending
tag kinds, retained/detached attributes, selector, scratch, return state and
last-start-tag bookkeeping). Prior comment token boundaries/order and prior
state/preprocessing diagnostics stay unchanged. The no-take checks keep the same
pending comment through suspension, source transition and terminal destination
attempts; existing helper unit tests separately prove actual emission/take.

Cursor-only probes **before** any destination attempt prove the same normalized
point/position is reconsumed exactly once, with no duplicate preprocessing error,
and the suffix remains unconsumed. EOF is probed separately in the same way.
Separate executions inspect source effects first, then verify CommentEnd's typed,
effect-free, repeated terminal `UnimplementedState`. None is resumed or rebuilt.

Every two-chunk split (with intervening empty chunks) and single-code-point chunks
of representative inputs compare source-step snapshots against unsplit execution,
for both open and explicitly closed exhaustion. A preconsumed raw CR tests LF
suppression across the feed boundary. Cases include EOF, suppressed-LF-only input,
`>`, NUL, malformed nested delimiters, CRLF/consecutive/trailing CR, non-scalars,
noncharacters, controls and Unicode. This state performs only one source step
before its destination block: traces stop there, never consume the remaining
suffix or imply whole-tokenizer chunk equivalence.

CommentEnd and all other ordinary comment states remain unimplemented on this
PR's prerequisite base `47c53e8dcf3e69d920c4be290036ffb70af72cc9` (#46).
A real Data-entry regression still stops at MarkupDeclarationOpen after consuming
`<!`, with no pending-comment creation or output. There is no new Data-entry
comment coverage, tree-builder behavior, full conformance or security claim.
When neighboring states merge, their actual destination effects must be tested
on the rebased head rather than preserving stale stub expectations.

## Validation on the isolated prerequisite base

All CONTRIBUTING checks pass: pinned submodule initialization, rustfmt check,
Clippy (`--workspace --all-targets --locked -- -D warnings`), workspace tests,
and rustdoc with warnings denied. The six state-local tests are local algorithm
checks, not additional upstream conformance cases.

`cargo test --workspace --locked pinned_upstream_runner -- --nocapture` reports:

- 6,827 fixture cases; 7,053 case/initial-state runs; 6,190 dispatch-attempted.
- **Executed 1,148; passed 1,148; failed 0; blocked 5,905.**
- Blocks: CharacterReference 4,692; MarkupDeclarationOpen 242;
  ProcessingInstructionOpen 52; CdataSection 56; TokenVariants 858;
  XmlCoercion 4; PendingSpecReview 1.

No upstream fixture can enter this isolated state through the supported initial
states yet. There are no new upstream passes or baseline additions; all 952
previously protected passing identities remain intact and passing. Inventory
checks execute no tokenizer conformance cases.
