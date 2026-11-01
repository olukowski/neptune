# Comment less-than sign bang dash dash

## Pinned branch evidence

Normative source: WHATWG HTML revision
`404b3eb14a7a936dddefb2d78e224dbeb01d2358`, recorded in
[sources.toml](sources.toml),
[Comment less-than sign bang dash dash state, lines 144642–144657](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144642-L144657)
(`#comment-less-than-sign-bang-dash-dash-state`). The implementation was derived
from that immutable source, not a living-standard snapshot or another engine.
Also checked: [preprocessing/current-next input, lines 142560–142594](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142594),
[Tokenization/reconsumption, lines 143151–143190](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143151-L143190),
and [nested-comment definition, lines 141335–141339](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L141335-L141339).

Algorithm: `crates/html/src/tokenizer/comment_less_than_sign_bang_dash_dash.rs`.
Focused tests: `crates/html/tests/comment_less_than_sign_bang_dash_dash_state.rs`.
`tokenizer/comment.rs` remains the private **shared helper**, not the Comment
state implementation. No helper, token/storage variant, dependency, sibling
algorithm, fixture, specification pin, or baseline is changed here.

| Consumed input | Source effects | Focused evidence |
| --- | --- | --- |
| `>` U+003E | Reconsume in CommentEnd, no state error | Exhaustive ASCII test, suspension/feed test, source-before-destination test, chunk traces |
| EOF | Reconsume conceptual EOF in CommentEnd, no state error | Open-exhaustion/explicit-EOF test, source-before-destination test, empty/suppressed-LF chunk traces |
| Anything else | Report `nested-comment` at the consumed normalized position; reconsume in CommentEnd | All other ASCII points, NUL, malformed punctuation, Unicode/scalars, surrogates, noncharacters and controls; exact diagnostics and chunk traces |
| Open exhausted input | Infrastructure suspension, not specification EOF; no effects | Repeated suspension with/without cursor history, empty chunks, suppressed LF after CR, later input/finish |

Every source branch preserves pending-comment data literally: no deferred `--`
append, NUL replacement, emission, token creation, or comment take occurs here.
In particular EOF neither emits EOF nor reports `eof-in-comment` until the real
CommentEnd destination executes. The shared `comment::emit` helper is the
existing take-on-emission path, but this source never calls it.

Direct entry supplies initialized `PendingComment`, empty or seeded with literal
NUL/CR/LF/non-scalars and delimiter-like text. Tests compare exact complete output,
state/cursor diagnostics, normalized positions and unrelated context (both pending
tag kinds, retained/detached attributes, selector, scratch, return state and
last-start-tag bookkeeping). Prior comment token boundaries/order and prior
state/preprocessing diagnostics stay unchanged. Cursor-only probes **before**
any destination attempt prove same-point/position reconsumption exactly once,
without duplicate preprocessing errors, and an unconsumed suffix; EOF is probed
separately. Probed executions are never dispatched afterward.

## Historical isolated review

Initial head `bef1e4871d16b4d4ed204bbb819297a655ce54ef`, based on prerequisite
#46 (`47c53e8dcf3e69d920c4be290036ffb70af72cc9`), received independent review
with **no findings**, full checks and an independent oracle. Only BangDashDash
was supported in the ordinary-comment family on that isolated head; CommentEnd
was then a terminal destination stub. Its six focused tests and 387 total tests
passed. Those support/validation boundaries are historical, not current status.

## Final integration onto #47–55

At the coordinator's instruction, #56 rebases onto
`6fb60bf893e41a1526d42234c449c01b536f72ec` (merged #47–55). All additive
registrations and inventory context are preserved. The assigned production
algorithm is byte-for-byte unchanged from its initially reviewed head; all
merged production siblings and shared helpers are unchanged from this new base.
The independent inventory now explicitly asserts **51 implemented states**,
including Bogus comment and all **ten ordinary-comment states**.

The complete ordinary-comment family is reachable from explicit CommentStart
entry with initialized pending-comment context. Each of the ten states also
supports initialized isolated entry. The pinned
[full family source, lines 144514–144732](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144514-L144732)
was read to derive the integration expectations, not inferred from output.

Semantic delta from the initial review:

- BangDash's second `-` now reaches this state and executes the next item,
  not a replayed hyphen. `>`/EOF do **not** report `nested-comment`; every other
  point does, including NUL, extra `-`, `!`, surrogates and noncharacters.
- Source snapshots still precede real CommentEnd. That destination reconsumes
  the same point/EOF: `>` takes/emits the existing comment into Data; EOF reports
  `eof-in-comment` and emits Comment then EOF; `-` appends one; `!` enters EndBang;
  fallback restores exactly two hyphens and replays in core Comment. Only core
  Comment replaces/diagnoses NUL or appends the original literal point.
- The Bang/BangDash predecessor suites retain their original source and cursor
  expectations. Obsolete BangDashDash blocks are replaced with real next-point
  processing, exact nested-comment/error positions, End restoration/Comment replay,
  or repeated open suspension followed by explicit EOF emission/take. Cursor-only
  probes remain separate from destination execution.
- The core Comment suite's formerly blocked nested-opener cases now assert
  independently derived complete outcomes: `<!-->` emits `<!` without a nested
  error; `<<!---->` emits `<<!--` with `nested-comment` at 1:6; BangDash `-` and
  EndBang `<!--` finish via real End EOF without appending withheld hyphens.
  The same literal outcomes run at every split and as single-point chunks.

### Final family integration tests

`crates/html/tests/comment_family_integration.rs` adds three focused tests:

1. `all_ten_entries_and_full_comment_start_match_literal_outcomes_at_every_split`
   checks hand-derived complete outcomes across all ten entries, both empty and
   literal seeded data, and absent/Start/End unrelated selectors. Cases include
   abrupt empty closure, nested/repeated nested openers, greater-than/EOF exceptions,
   extra/withheld/restored hyphens, EndBang fallback and dash loops, malformed close,
   literal ampersands/punctuation, NUL, CRLF/consecutive/trailing CR, actual
   surrogates/noncharacters and precise preprocessing/state diagnostics. Complete
   tokens, final state/current input/normalized position, take semantics and all
   unrelated context are checked independently of trace equivalence. Every
   two-chunk split (including empty chunks) and single-code-point chunks matches
   a truly closed, unsplit source-step trace. No expected output is blessed from
   execution.
2. `explicit_start_nested_opener_has_independent_source_snapshots_and_once_only_preprocessing`
   checks every step of raw `CRLF <!-- SURROGATE NUL NONCHARACTER CRLF --!> NUL`:
   Start replay, nested prefix withholding, nested-comment/surrogate at 2:5,
   two further same-position replays without duplicate preprocessing errors,
   exact `--` restoration, core NUL replacement/error at 2:6, noncharacter at
   2:7, malformed close at 3:4, one comment take/emission, original Data NUL/error
   at 3:5, then one terminal EOF at 3:6. Existing literal seeded NUL/CR/surrogate
   data and unrelated pending tags/attributes/diagnostics remain intact.
3. `full_start_reaches_and_repeatedly_suspends_in_every_family_state_without_taking_comment`
   reaches all ten states from real CommentStart, checks repeated empty-chunk
   suspension with independently expected pending data/withheld suffix and no
   emission/take, then explicit EOF's exact destination/error/comment/EOF order.

Emitted nested-comment prefixes followed by `&`, `<!`, or `<?` still stop at
real CharacterReference, MarkupDeclarationOpen or ProcessingInstructionOpen stubs.
The expected Data return-state assignment and TagOpen scratch clearing are checked.
Every blocked trace stops permanently; no further chunk is fed and no execution
is reconstructed/resumed. Source-local tests retain their narrower same-item
probes; the separate family suite executes complete representative paths.

README, the authoritative support list and sibling current-status notices reflect
actual reachability. Historical PR-local support/count reports are explicitly
marked historical; their pinned source-branch evidence is retained.

### Remaining gateway boundary

Normal Data `<!--` entry still consumes `<!` and blocks at
MarkupDeclarationOpen, before pending-comment creation or output. No gateway,
sibling, helper or adapter implementation is added. Completing the family from
explicit initialized CommentStart adds **no Data-entry comment coverage**, full
tokenizer conformance, tree-builder behavior or browser-security claim.

## Final validation

All CONTRIBUTING checks pass: pinned submodule initialization, rustfmt check,
Clippy (`--workspace --all-targets --locked -- -D warnings`), workspace tests
(**454 passed, 0 failed/ignored**, including six assigned-state and three new
family integration tests), and rustdoc with warnings denied. Explicit support
inventory, pinned fixture inventory, pinned upstream runner and `git diff --check`
also pass. The support inventory checks all 84 entries and the 51-state support
set; fixture inventory reports 14 files/6,827 cases and **zero conformance
executions**. Neither inventory is counted as tokenizer conformance.

`cargo test --workspace --locked pinned_upstream_runner -- --nocapture` reports:

- 6,827 fixture cases; 7,053 case/initial-state runs; 6,190 dispatch-attempted.
- **Executed 1,148; passed 1,148; failed 0; blocked 5,905.**
- Blocks: CharacterReference 4,692; MarkupDeclarationOpen 242;
  ProcessingInstructionOpen 52; CdataSection 56; TokenVariants 858;
  XmlCoercion 4; PendingSpecReview 1.

No supported upstream initial-state run newly enters this family. There are no
new upstream passes or baseline additions; all 952 previously protected passing
identities remain intact and passing. Upstream fixtures and pins are unchanged.
