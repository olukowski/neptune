# Comment state

## Current CommentEnd integration (PR #55)

CommentEnd now closes core `-->` and reaches EndBang recovery on `--!`.
Nine ordinary-comment states are supported from initialized isolated entry;
BangDashDash alone remains unsupported in this family. Normal Data entry still
blocks at MarkupDeclarationOpen. See [current integration evidence](tokenizer-comment-end.md#integration-onto-47-54)
for the updated independent core/isolated outcome table, source snapshots,
every-split traces, validation and unchanged upstream counts. Original source
expectations and all production siblings/helpers are preserved. The PR #54
integration boundaries/counts below describe its historical base, not current
reachability; pinned algorithm branch evidence remains normative.

## Scope and pinned evidence

Only `State::Comment` is implemented here, in
`crates/html/src/tokenizer/comment_state.rs`, separate from the private shared
`comment.rs` helper module. The base includes prerequisite PR #46,
`47c53e8dcf3e69d920c4be290036ffb70af72cc9`. No sibling state, storage, token
variant, dependency, fixture adapter, or pin is added or changed.

The immutable WHATWG HTML source at revision
[`404b3eb14a7a936dddefb2d78e224dbeb01d2358`, lines 144561–144587](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144561-L144587)
was retrieved and inspected directly. Its heading is `Comment state` (source
`dfn id="comment"`; registry/navigation anchor `#comment-state`). Branch mapping:

| Consumed input | Source effects | Result |
| --- | --- | --- |
| `<` U+003C (144568–144570) | Append the current point, including the `<`. | Comment less-than sign |
| `-` U+002D (144572–144573) | Do not append anything. | Comment end dash |
| NUL U+0000 (144575–144578) | Report `unexpected-null-character`; append U+FFFD. | Stay in Comment |
| Conceptual EOF (144580–144582) | Report `eof-in-comment`; emit/take the comment, then emit EOF. | Finished; state unchanged |
| Anything else (144584–144585) | Append the original point unchanged. | Stay in Comment |
| Exhausted open input | No mutation, diagnostic, or emission. | NeedMoreInput |

Every normative branch consumes once; none schedules reconsumption. `>` does
not close this state, and `&` is literal data, not a character reference.
CommentLessThanSign and CommentEndDash now execute real destination steps,
including suspension on exhausted open input and EOF handling on explicit finish.
They were terminal stubs on the initially reviewed prerequisite base; that is no
longer the integration boundary. Source effects are inspected before the next
destination attempt; genuine blocked executions are never reconstructed or resumed.

Related definitions inspected in the same pinned source:

- [Preprocessing, current/next input and conceptual EOF, 142560–142607](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142607):
  CR/CRLF normalization, once-only surrogate/noncharacter/control diagnostics,
  and state-specific NUL handling.
- [Tokenization, reconsumption, token data and creation/emission distinction,
  143149–143191](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143149-L143191).
- [`eof-in-comment`, 141119–141122](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L141119-L141122).

Direct isolated entry requires an initialized `PendingComment`; this state does
not create or clear it. Existing data (including NUL, CR and non-scalars) is
preserved literally, without preprocessing it again. Appending and emission reuse
`comment::data_mut` and `comment::emit`; EOF takes only the pending comment.
Unrelated tag/attribute storage, detached duplicates, tag selector, last start
tag, scratch and character-reference return state are preserved.

## Integration onto #47–53

The original head `1bf08068fb6c41b38e8ead3b2dba01c255e5e636` received independent
review with no findings. At the coordinator's instruction, PR #54 rebases onto
`f2ca599065ec7fa70cc437f5c2ea6b9e2ee68883` (merged #47–53). Additive conflicts
preserve every merged module, dispatch registration, inventory entry and evidence
link. Comment's algorithm is byte-for-byte unchanged from that original review;
all seven merged sibling algorithms and shared `comment.rs` remain unchanged.

Semantic integration delta, all from the same pinned
[source lines 144514–144732](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144514-L144732):

- CommentStart, StartDash, LessThanSign, Bang, EndDash and EndBang fallback now
  enter real Comment, not a development stub. Each affected state suite retains
  the original source snapshot before asserting actual literal append, transition,
  NUL replacement/error, or EOF diagnostic/comment-take/EOF emission. Preprocessing
  errors and current positions remain unchanged across reconsumption.
- BangDash fallback now executes EndDash and Comment; EOF is emitted only by
  EndDash. Its old cursor-tail-only chunk harness now collects real source-step
  traces through the supported loop. Separate cursor probes never dispatch an
  implemented state after manual consumption.
- Comment's `<` and `-` now consume the next point in real LessThanSign/EndDash.
  A literal next point returns/reconsumes in Comment; EndDash first appends its
  withheld hyphen. Open destinations suspend, and explicit EOF follows the real
  less-than fallback or EndDash EOF branch.
- CommentEnd and BangDashDash are the only remaining ordinary-comment stubs.
  Core Comment on `-->` stops after the second hyphen in CommentEnd, before `>`;
  `<!-->` stops in BangDashDash, before a nested-comment error or later closure.
  No unsupported branch is inferred, skipped, or treated as completion.
- Only isolated Start/StartDash/EndBang entry can currently reach a closing `>`
  emission into Data; Comment cannot reach EndBang while CommentEnd is a stub.
  Integrated closure expectations cover unchanged literal seeded data, no withheld
  delimiter append, abrupt/incorrect-close errors, comment take, Data's original
  NUL output, EOF ordering, and matching comment prefixes followed by genuine
  CharacterReference/MarkupDeclarationOpen blocks.
- README and the authoritative opening support list reflect all eight supported
  ordinary-comment states. Inventory still initializes pending data and the
  already-consumed/appended `<`/`<!` predecessor context where required.

## Focused tests

`crates/html/tests/comment_state.rs` contains eight local tests:

- Every ASCII point plus representative Unicode, actual lone surrogates,
  noncharacters and controls: exact source-step snapshots, diagnostics/positions,
  open/closed input, absent/Start/End tag selectors, and untouched unrelated data.
  Literal branches consume a following `x`, proving no outgoing reconsumption;
  transitions preserve source effects before executing actual destination steps
  and the following Comment reconsumption.
  Separate cursor-only probes after the source (before any destination attempt)
  prove delimiters are not reconsumed and surrogate tails remain unconsumed and
  undiagnosed until the probe. No execution continues after those probes.
- CRLF, repeated NUL, supplementary characters, literal `>`/`&`, precise normalized
  locations, literal seeded data, ordered separate comment tokens, EOF and take
  semantics, and effect-free repeated termination.
- Empty/seeded comment data with repeated open exhaustion and empty chunks,
  including trailing CR; only explicit finish diagnoses/emits EOF. Minimal
  isolated entry supplies only initialized pending-comment storage.
- Incoming reconsumption of NUL, both transition delimiters, `>`, a surrogate,
  noncharacter, control and conceptual EOF: positions and preprocessing errors
  remain unchanged; state diagnostics occur once on the source step. A following
  EOF is consumed, rather than reconsuming the input again.
- Open transition destinations suspend repeatedly, then handle explicit EOF with
  exact source/destination ordering and take semantics. Real Data entry on
  `<!--comment-->` still stops at MarkupDeclarationOpen after `!`, before creating
  or executing an ordinary comment. This is not new Data-entry coverage.
- Every two-chunk split (including empty chunks) and single-code-point chunks
  match closed-input source-step traces for literals, malformed comment-like
  input, NUL, Unicode/non-scalars, CRLF, consecutive/trailing CR, EOF and actual
  destination loops. Trace collection stops permanently at a genuine stub.
- An independently expected integrated case table covers all eight entry states,
  literal/fallback loops, withheld delimiters, NUL/preprocessing/locations, seeded
  literal data, closures, take/EOF order, genuine blocks, and comment emission
  followed by later stubs. Every case is checked at every two-chunk split and as
  single-code-point chunks, not merely compared against production's closed run.
- An explicit `<!-x-` trace asserts each source snapshot independently: BangDash
  schedules `x`, EndDash appends only `-`, Comment reconsumes/appends `x`, and the
  final EndDash EOF emits `<!-x` (without the trailing withheld hyphen), then EOF.

These local checks and the independent support inventory are not upstream
conformance executions. This PR adds only Comment to the inventory, preserving
all merged entries and initialized pending-comment context for isolated EOF.

## Independent fixtures and validation

Pinned html5lib-tests remains at `c777c408b61078ea2eb4acefc2535f54dbc8b28a`.
Independently inspected upstream expectations (zero-based case indices):

- `test1.test` 17, `Simple comment`: `<!--comment-->` expects comment data
  `comment` (closing delimiters still require the unsupported CommentEnd).
- `test1.test` 21, `Unfinished comment`: `<!--comment` expects comment data
  `comment` and `eof-in-comment` at 1:12.
- `test3.test` 89, `<!--\\u0000`: `<!--` followed by NUL expects comment data
  U+FFFD, `unexpected-null-character` at 1:5 and `eof-in-comment` at 1:6.

All remain blocked from Data at MarkupDeclarationOpen. They are independent
branch evidence, not newly passing runs or claims that this PR executes their
ordinary-comment entry paths.

Both the original review and this integration's separately executed
`pinned_upstream_runner` report:
**6,827 fixture cases; 7,053 runs; 6,190 dispatch-attempted;
1,148 executed = 1,148 passed + 0 failed; 5,905 blocked.**
Blocked distribution: CharacterReference 4,692; MarkupDeclarationOpen 242;
ProcessingInstructionOpen 52; CdataSection 56; TokenVariants 858; XmlCoercion 4;
PendingSpecReview 1. All deltas are zero. Prior identity baselines are retained
and pass; no newly passing identities exist to add. No fixtures or pins change.

Checks passed on mba: submodule initialization, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets --locked -- -D warnings`,
`cargo test --workspace --locked`,
`RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`, and
`cargo test --workspace --locked pinned_upstream_runner -- --nocapture`.

This is partial isolated-state support, not full tokenizer conformance or a
browser-security claim. Normal Data comment entry remains blocked.
