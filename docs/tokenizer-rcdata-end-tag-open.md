# RCDATA end tag open state

## Pinned branch evidence

Normative source: WHATWG HTML revision
`404b3eb14a7a936dddefb2d78e224dbeb01d2358`, as recorded in
[sources.toml](sources.toml). Inspected the immutable source, not the living
standard or another engine:

- [Assigned state, lines 143505–143522](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143505-L143522).
- [Preprocessing and current/next input definitions, lines 142560–142594](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142594):
  normalize newlines, diagnose non-scalars/noncharacters/controls on consumption.
- [Reconsumption, creation defaults, and creation versus emission, lines 143152–143191](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143152-L143191).
- [Appropriate end tag definition, lines 143213–143217](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143213-L143217):
  this state does **not** test appropriateness or change `last_start_tag`.

| Input | Source effects | Destination |
| --- | --- | --- |
| ASCII alpha, lines 143513–143515 | Create/select an end tag with empty name, unset self-closing flag, empty attributes, no detached attribute. No append, case conversion, output, or state diagnostic. | Reconsume in RCDATA end tag name. |
| Anything else, lines 143517–143519 | Emit exactly `<`, then `/`; preserve pending storage/selector and temporary buffer. No state diagnostic. Includes EOF. | Reconsume in RCDATA. |
| Exhausted open input (execution contract) | No effects. Not specification EOF. | Repeated `NeedMoreInput` until input or explicit finish. |

Only the ASCII-alpha branch replaces pending end-tag storage. Both branches
preserve unrelated pending start tags/comments, last-start-tag bookkeeping,
return state, existing output/errors, and temporary buffer. The preceding
less-than-sign state's buffer clearing is not duplicated here. This state does
not append buffer contents on fallback.

EOF does not emit EOF here: it emits the literal prefix and schedules conceptual
EOF reconsumption; RCDATA emits the terminal EOF on its next step. Similarly,
NUL replacement/error and ampersand/less-than transitions belong to the
reconsuming RCDATA destination, not this source.

## Focused tests and limits

`crates/html/tests/rcdata_end_tag_open_state.rs` checks all 52 ASCII letters,
every other ASCII point, non-ASCII letters/scalars, surrogates, noncharacters,
controls, NUL, and newline preprocessing. Exact snapshots separate source effects
from destination effects, including diagnostics/multiplicity and normalized
positions after a consumed CRLF prefix. Tests inspect empty creation defaults,
replacement versus preservation (including detached attributes and non-scalar
scratch storage), selectors, absent/present last start tags, reconsumption and
unconsumed tails, EOF versus repeated suspension, and repeated terminal results.

Every two-chunk split and single-code-point chunking of representative inputs
matches every closed-input source-step snapshot. Samples include malformed
closing syntax, CRLF boundaries, consecutive/trailing CR, NUL/non-scalars, empty
input, alpha/name blocks, and fallback to later stubs. These are local state
checks, not upstream conformance executions.

Production scope is **only** `State::RcdataEndTagOpen`. RCDATA less-than sign and
RCDATA end tag name remain explicit terminal `UnimplementedState` stubs.
Isolated entry initializes the cursor after the consumed `</` prefix and uses an
empty temporary buffer, or deliberately seeds unrelated storage to verify its
preservation. No blocked execution is reconstructed or resumed. There is no
automatic mode selection, tree builder, shared API change, or fixture adapter
expansion.

The existing upstream runner has no direct initial-state mapping for this state;
ordinary RCDATA entry still blocks at less-than sign. No new upstream passes or
baseline identities are added; all existing baselines and fixture pins remain
unchanged. Use the [runner command and count definitions](tokenizer.md#fixture-runner).
Inventory checks are not conformance, and partial passes imply neither full
conformance nor browser security.

## Integration caveats

Started at the requested common baseline
`bcae7e5c916470319ef0838dc72dfd71a95b8181` (HEAD and origin/main).
Dispatch, module registration, support inventory, and the central documentation
have deliberately small additive edits that can conflict with sibling PRs.
Rebase after neighboring merges and rerun every CONTRIBUTING check and the
upstream runner. Destination implementations can legitimately move a terminal
block; review and update those integration assertions explicitly, never remove
blocking semantics or simulate continuation through a stub.
