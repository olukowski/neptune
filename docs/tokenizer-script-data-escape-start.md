# Script data escape start state

## Current final Script data integration (PR #45)

On merged main `022baca4abb8593974d29c018597f9d5678231a6` plus PR #45,
the complete Script data state family is reachable from explicit Script data
entry. EscapedLessThanSign is no longer a stub: escaped closing tags and the
full double-escaped loop execute, including returns to Escaped, Script data and
Data. This state's production algorithm and source-branch assertions are
preserved; tests now execute newly reachable destinations instead of stopping at
obsolete script-state blocks. Character reference/Markup declaration and other
unrelated unsupported states remain genuine terminal limitations.

See [PR #45's final integration and individually reviewed fixture audit](tokenizer-script-data-escaped-less-than-sign.md#final-integration-after-all-eight-siblings)
and `crates/html/tests/script_data_integration.rs` for current paths, source-step
effects, exact diagnostics, attributes/duplicates, nesting/case/delimiters and
chunk-splitting evidence. This remains a standalone tokenizer, not automatic
mode switching, tree construction, JavaScript parsing or full conformance.

**All support boundaries, counts and PR-local “current” labels below are
historical snapshots. Their normative branch evidence remains applicable.**


## Scope, historical baseline and integrated base

PR #36 implements exactly `State::ScriptDataEscapeStart`, in
`crates/html/src/tokenizer/script_data_escape_start.rs`, plus additive module,
dispatch and independent inventory registrations. No adjacent algorithm, shared
framework/storage, dependency, fixture, pin, adapter or policy rule is changed.

Historical standalone head `2bb39eae38383fc43e19408cc98212ff1f9121ab` was based on
main `28131e11437941928eb7246885fb70e1160b2700` (PR #31). EscapeStartDash was then
a terminal stub: 1,118 executed/passed, 0 failed, 5,935 blocked; one reviewed new
identity, `test1.test` tests[34]. Its four focused tests/251 workspace tests were
local checks, not additional conformance cases.

The integrated base is main **`da69390807532870f62ac6e8c2e88bc2fe608ae4`**,
containing reviewed/merged #32 (Escaped), #34 (EscapedDash), #33
(EscapedDashDash), and #35 (EscapeStartDash). Those four production algorithms
and their source assertions are preserved. Rebase conflicts in module/dispatch,
inventory and current status were resolved additively. No unmerged sibling branch
was imported. The final head/base pair is recorded in PR #36's body.

Normal explicit Script data entry now traverses `<!` → EscapeStart → `-` →
EscapeStartDash → `-` → EscapedDashDash. The escaped trio emits character tokens,
handles NUL and escaped EOF, and returns to Script data on `>` in EscapedDashDash.
An escaped `<` still consumes once without emission and reaches terminal
`ScriptDataEscapedLessThanSign`. Escaped end tags and double escaping remain
unsupported; no tree builder, automatic element-based mode switching, Comment
token emission or JavaScript parsing is implied.

## Pinned evidence and branch mapping

Retrieved and inspected the immutable WHATWG HTML raw source revision
[`404b3eb14a7a936dddefb2d78e224dbeb01d2358`](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143741-L143755),
**Script data escape start state** (`#script-data-escape-start-state`), lines
143741–143755. The pinned algorithm is unchanged on integration. Referenced
definitions and real predecessor/destination sections were also inspected:

- [Preprocessing, next/current input and conceptual EOF, lines 142560–142607](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142607):
  CR/CRLF normalize before state matching; non-scalars/controls retain their
  input-stream diagnostics; NUL handling belongs to tokenization states.
- [Tokenization, reconsumption and token output, lines 143149–143191](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143149-L143191):
  reconsumption supplies the current item, not the following one. Temporary and
  return storage are distinct and neither changes here.
- [Script data, lines 143338–143359](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143338-L143359),
  and [less-than-sign, lines 143654–143673](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143654-L143673):
  predecessor emission and fallback NUL replacement/EOF belong to their own steps.
- [Escape start dash and escaped trio, lines 143758–143869](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143758-L143869):
  merged destination branches supply integration expectations, not new production
  scope. Their escaped EOF diagnostic differs from ordinary Script data EOF.

| Consumed input | Assigned source effects | Intended destination |
| --- | --- | --- |
| U+002D `-` (143748–143750) | Emit exactly one hyphen character token; no reconsumption. | Script data escape start dash |
| Anything else, including conceptual EOF (143752–143753) | Emit nothing; schedule the current item for reconsumption. | Script data |
| Exhausted open input | Effect-free `NeedMoreInput`, not EOF. | Same state |

The first two rows exhaust the normative algorithm; suspension is the existing
incremental contract. `Progress::Continue(target)` exposes source effects before
the destination executes. This state generates **no parse errors**, replaces no
NUL, and emits no EOF. Its destination alone handles reconsumed NUL/EOF. Closed
input `<!-` now reconsumes EOF through EscapeStartDash in ordinary Script data
without an escaped EOF error; `<!--` instead emits EOF with exactly one
`eof-in-script-html-comment-like-text` from EscapedDashDash. Preprocessing errors
occur only on first consumption at normalized code-point positions, never again
on reconsumption; non-scalars remain code points.

Unrelated pending tags, retained/detached attributes, current-tag selection,
pending comment, history, return state, temporary buffer and prior output/errors
are preserved. Only the specified hyphen token is appended at this source step.

## Focused and streaming tests after integration

`crates/html/tests/script_data_escape_start_state.rs` has **seven** local tests:

- Exhaustive ASCII plus Unicode/hyphen lookalikes, lone surrogates, noncharacters
  and controls, open/closed input and absent/Start/End selectors. Exact source
  snapshots preserve storage, positions and errors. The hyphen branch now follows
  the real EscapeStartDash `z` fallback and reconsumption; other branches follow
  Script data replacement/emission/less-than-sign effects separately.
- Repeated open suspension versus conceptual EOF after a CRLF/`<!` cursor prefix,
  with default/seeded storage. Only Script data emits the terminal EOF at 2:3.
- No-current-item entry suspends, accepts a hyphen, exposes its emission, then
  suspends in the real destination or falls back through EOF at 1:2. No stale
  dash-state stub assertion remains.
- Fixed specification-derived source snapshots from real Script data through
  CRLF, `<!--`, repeated hyphen cycles, escaped NUL/surrogate/noncharacter/control,
  `>` return and ordinary NUL/non-scalars. Every normalized current item,
  emission, state/input error position and multiplicity is explicit. All unrelated
  context remains intact, and ordinary EOF adds no escaped EOF diagnostic.
- Real-entry EOF cases distinguish incomplete `<!`/`<!-` and `<!-->` ordinary EOF
  from EOF in all three escaped states, with exact final state/output/position
  and a single escaped EOF diagnostic only where specified.
- All three escaped `<` source boundaries remain terminal: the `<` is consumed
  without emission; no EOF or recoverable error substitutes for the stub. Repeated
  steps/attempted feeding after terminality cannot revive execution.
- Every two-chunk split (including empty chunks) and single-code-point streaming
  equals closed-input source-step snapshots through isolated and real entry.
  Inputs include escaped EOF in each state, cycles and `>` return, repeated NUL,
  actual non-scalars, CRLF/consecutive/trailing CR, appropriate closing tags,
  escaped `<` from each state, and later Character reference / Markup declaration
  open blocks. Suspensions preserve snapshots. Traces stop at the first genuine
  stub and never feed, reconstruct or resume it.

Existing Script data, less-than-sign, end-tag-open/name, EscapeStartDash,
EscapedDash and EscapedDashDash suites deliberately replace obsolete block
assumptions with actual destination steps. Source snapshots remain intact; EOF
errors and terminal escaped-less-than-sign/later Data/attribute blocks are exact.
The Escaped suite's source assertions are unchanged. Adapter-local tests require
complete literal/comment-like output and exact diagnostics; matching prefixes
still block at unsupported destinations. No dynamic implementation detection,
skipped tests, fixture edits or adapter/policy weakening is used. Local Rust tests
and inventory checks are not upstream conformance cases.

## Reviewed upstream identities

The nine newly comparable runs were individually inspected in the unchanged
pinned fixtures before their identities were added. Every run below has the
single `Script data state` initial entry (index 0), no `lastStartTag`, and literal
character output as described. Only `domjs.test` tests[5] uses `doubleEscaped`.

| Fixture / zero-based case | Input and expected output/diagnostics | Pinned branch reasoning |
| --- | --- | --- |
| `test1.test` 34, `<! in script data` | Input/output `<!test-->`; no errors. | EscapeStart reconsumes `t` in Script data; the rest emits literally. This identity was already protected on the standalone head. |
| `test1.test` 35, `<!- in script data` | Input/output `<!-test-->`; no errors. | EscapeStart emits the first `-`; EscapeStartDash reconsumes `t` in Script data. No escaped state is entered. |
| `test1.test` 36, `Escaped script data` | Input/output `<!--test-->`; no errors. | Two hyphens enter EscapedDashDash; `t` enters Escaped; closing hyphens then `>` return to Script data before EOF. |
| `domjs.test` 5, `NUL in script HTML comment` | Double-escaped input `<!--test\u0000--><!--test-\u0000--><!--test--\u0000-->`; output replaces each `\u0000` with `\uFFFD`. Exactly `unexpected-null-character` at 1:9, 1:22, 1:36. | The NULs occur in Escaped, EscapedDash and EscapedDashDash respectively; each replaces once and enters/stays Escaped. Every `-->` returns to Script data; no escaped EOF error. |
| `domjs.test` 7, `EOF in script HTML comment` | Input/output `<!--test`; exactly `eof-in-script-html-comment-like-text` at 1:9. | EOF in Escaped. |
| `domjs.test` 8, `EOF in script HTML comment after dash` | Input/output `<!--test-`; exactly the escaped EOF error at 1:10. | One trailing hyphen enters EscapedDash; EOF reports once. |
| `domjs.test` 9, `EOF in script HTML comment after dash dash` | Input/output `<!--test--`; exactly the escaped EOF error at 1:11. | Two trailing hyphens enter EscapedDashDash; EOF reports once. |
| `domjs.test` 13, `Dash in script HTML comment` | Input/output `<!-- - -->`; no errors. | SPACE leaves DashDash for Escaped; hyphen/SPACE cycles through Dash/ Escaped; final `-->` returns to Script data. |
| `domjs.test` 15, `Dash at end of script HTML comment` | Input/output `<!--test--->`; no errors. | Third trailing hyphen stays in DashDash; `>` returns to Script data before EOF. |

`baselines/script_data_escape_start.json` contains exactly these nine reviewed
identities, not generated expectations. Its registration is additive; all 921
prior protected identities and their files/registrations are unchanged (930 total).
This adds eight identities over the historical standalone head, nine over the
integrated base. No result is auto-blessed.

The other 22 formerly EscapeStart-blocked runs remain blocked, now at
EscapedLessThanSign: `domjs.test` tests[6, 10–12, 14, 16–23] and `test1.test`
tests[37–45], all Script data entry 0. Their inspected inputs reach escaped `<`
before completion. Matching output prefixes and later expected diagnostics are
not compared or added as passes.

## Validation and current counts

All required commands passed on the integrated base plus this implementation:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo test --workspace --locked pinned_upstream_runner -- --nocapture
```

The workspace suite passes 284 Rust tests, 0 failed, 0 ignored (separate from
upstream runs). Pinned fixtures were already initialized at
`c777c408b61078ea2eb4acefc2535f54dbc8b28a`. There is no validation/delivery blocker.

Upstream: 6,827 fixture cases / 7,053 runs / 6,190 dispatch-attempted on both bases.

| Result | Integrated main base | PR #36 integrated | Change |
| --- | ---: | ---: | ---: |
| Executed | 1,117 | 1,126 | +9 |
| Passed | 1,117 | 1,126 | +9 |
| Failed | 0 | 0 | 0 |
| Blocked | 5,936 | 5,927 | -9 |

Blocked reasons: Character reference 4,692; Markup declaration open 242;
EscapedLessThanSign 22; CDATA section 56; Processing instruction open 52;
unsupported token variants 858; XML coercion 4; pending-spec review 1.
Executed = passed + failed; blocks are not passes. Previously passing paths could
not have reached the old EscapeStart stub; only that dispatch entry is added and
all other algorithms remain unchanged. The runner also enforces the complete
930-identity protected union and exact comparable tokens/diagnostics. This is a
partial tokenizer, not full conformance or a browser-security claim.

## Sequential integration and review

This started as one of five standalone state PRs. The user subsequently authorized
sequential merges by the parent after fresh independent reviews. All four siblings
are now reviewed and merged; PR #36 is last and connects their normal-entry path.
This thread starts no subagent/reviewer, does not merge or enable auto-merge, and
stops after updating/pushing this PR. Parent will freshly review the exact final
head/base before merge. Future rebases must preserve algorithms, source-step
assertions and reviewed baseline identities, then rerun every check/report;
never dynamically detect implementation support, compare blocked prefixes or
reconstruct/resume an execution that reached a stub.
