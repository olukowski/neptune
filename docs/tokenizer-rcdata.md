# RCDATA state coverage

Normative baseline: [HTML Standard source at
404b3eb14a7a936dddefb2d78e224dbeb01d2358](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source),
**RCDATA state** (`#rcdata-state`), with the shared **Preprocessing the input
stream** and opening **Tokenization** contracts. Revisions remain those in
[sources.toml](sources.toml).

| Consumed input | Effects, in specification order |
| --- | --- |
| `&` U+0026 | Set return state to RCDATA; switch to Character reference. |
| `<` U+003C | Switch to RCDATA less-than sign; preserve return state. |
| NUL U+0000 | Report `unexpected-null-character`; emit U+FFFD, not Data's literal NUL. |
| EOF | Emit one terminal EOF. |
| Anything else | Emit the current code point unchanged; stay in RCDATA. |

Every branch consumes once; none reconsumes. The two transition branches do not
emit or report state diagnostics. Their source effects are inspectable before
the next step attempts the destination. [RCDATA less-than sign](tokenizer-rcdata-less-than-sign.md)
and [RCDATA end tag open](tokenizer-rcdata-end-tag-open.md) now execute;
Character reference and RCDATA end tag name remain terminal `UnimplementedState`
stubs. Never reconstruct or resume blocked execution.

Select RCDATA explicitly with `Execution::new(State::Rcdata, context)`. The
fixture runner already recognizes `RCDATA state`; no mapping, input, execution,
token/storage, or emission API changes are needed. There is no tree-builder
mode selection or automatic `<title>` / `<textarea>` handling. Attribute name
and RAWTEXT are implemented independently; their handlers, tests, inventory
entries, and existing additive baselines are preserved.

`crates/html/tests/rcdata_state.rs` covers all branches, exact emitted tokens,
current input/normalized positions, state and preprocessing diagnostic codes and
multiplicities, return-state overwrite/preservation (including absent values),
and all unrelated context/pending token/attribute/scratch storage. Actual
surrogates, noncharacters, and controls pass through unchanged with exactly the
cursor's preprocessing diagnostics; NUL replacement affects output only.

Open exhausted input repeatedly suspends without changing context or emitting
EOF. Only explicit finish supplies EOF; finish and terminal execution are
idempotent. Every two-chunk split and single-code-point chunks of representative
inputs preserve each source-step snapshot, including CRLF splits, consecutive
and trailing CR, Unicode/non-scalars, NUL, `&` / `<` boundaries, unconsumed tails,
and repeated terminal blocks. These are RCDATA-local streaming checks, not
unsupported destination resumability or whole-tokenizer conformance.

`crates/html/tests/support/rcdata.rs` exercises the existing production adapter
with explicit/repeated RCDATA initial states, Data defaults, empty input,
double-escaped non-scalars and NUL, and matching output prefixes that must still
block at Character reference or RCDATA end tag open. These local tests are not upstream conformance runs.
The additive [RCDATA baseline](../crates/html/tests/baselines/rcdata.json) records
reviewed upstream identities without changing prior passes or upstream fixtures.
Use the central [runner command and count definitions](tokenizer.md#fixture-runner)
for reproducible execution evidence; blocked cases are never passes.

After neighboring merges, rebase and rerun all contributing checks plus
the pinned upstream report;
resolve shared registration/baseline-list edits additively and review legitimate
destination-block changes rather than weakening assertions. This remains a
partial standalone tokenizer, not full conformance or a browser-security claim.
