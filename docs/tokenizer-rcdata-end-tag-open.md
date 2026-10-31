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
input, alpha/name accumulation/completion, and fallback to later stubs. These are local state
checks, not upstream conformance executions.

Production scope added by this PR is **only** `State::RcdataEndTagOpen`.
RCDATA less-than sign (merged PR #26) and RAWTEXT less-than sign (merged PR #25)
are preserved unchanged. [RCDATA end tag name](tokenizer-rcdata-end-tag-name.md)
is now implemented separately in PR #27: source creation snapshots remain,
followed by actual normalization, literal fallback, or tag emission. Character
reference and other still-unsupported states remain terminal development errors.
Isolated entry initializes the cursor after the consumed `</` prefix and uses an
empty temporary buffer, or deliberately seeds unrelated storage to verify its
preservation. No blocked execution is reconstructed or resumed. There is no
automatic mode selection, tree builder, shared API change, or fixture adapter
expansion.

## Integrated fixture review

The runner still needs no direct initial-state mapping for this state: ordinary
RCDATA entry now reaches it through the merged less-than-sign state. The original
isolated PR report (1,023 executed/passed, 0 failed, 6,030 blocked) predates this
integration. At main `651cf0747ede47cd401aba7e38c57d0db90841fb`, the 19 RCDATA
end-tag-open blocks now become **18 end-tag-name blocks and one new pass**.
All previous baseline files and fixture pins remain unchanged.

The newly comparable fixture was inspected directly, independently of deriving
expected results from tokenizer output:

| Fixture identity | Unchanged input/context/output | Pinned branch reasoning |
| --- | --- | --- |
| `domjs.test`, `tests[28]`, “bad endtag (space before name)”, `RCDATA state`, initial-state index 0 | Input `</ XMP>`, last start tag `xmp`; output one Character string `</ XMP>`; no errors. | RCDATA consumes `<`; less-than sign consumes `/` and clears the buffer. End tag open consumes SPACE (not ASCII alpha), emits `</`, and reconsumes SPACE in RCDATA. RCDATA emits the space and remaining `XMP>` literally, then EOF. No tag is created or compared for appropriateness, and no state or preprocessing error applies. |

Only that manually selected identity is added in
`crates/html/tests/baselines/rcdata_end_tag_open.json`; existing 867 protected
identities remain intact, for 868 total. This is not an automatic pass capture.

The other affected runs were inspected in the unchanged fixtures:
`contentModelFlags.test` cases 2–11 (10), `domjs.test` cases 27 and 29–31 (4),
and `escapeFlag.test` cases 0–2 and 4 (4), all RCDATA initial-state index 0.
Each reaches an ASCII letter immediately after `</`, so creates an empty end tag
and at PR #24's merge blocked at RCDATA end tag name. They were **not**
comparable/pass identities for that PR. PR #27 now reviews their newly reachable
outcomes in the [name-state audit](tokenizer-rcdata-end-tag-name.md#integrated-fixture-review):
17 new passes, one later Markup declaration open block. `escapeFlag.test` case 3
still blocks earlier at Character reference.

Historical PR #24 report (before PRs #23/#27): 6,827 fixture cases / 7,053 runs; 6,190 dispatch-attempted;
**1,024 executed, 1,024 passed, 0 failed, 6,029 blocked**.
Use the [runner command and count definitions](tokenizer.md#fixture-runner).
Inventory checks are not conformance, and partial passes imply neither full
conformance nor browser security.

Integrated tests retain the less-than-sign source snapshots (only the scratch
buffer clears on `/`) before inspecting this state's creation or fallback.
Open exhaustion suspends; explicit EOF emits the prefix before RCDATA's EOF;
non-scalar fallback diagnoses once and reconsumes unchanged; alpha creates an
empty selected tag before the name-state step accumulates the reconsumed letter. Both isolated and real RCDATA streaming
traces cover all two-chunk splits and single-point chunks, including
`</ XMP>`, `</0>`, NUL/CRLF, and further less-than signs.
The assigned-state test no longer treats the merged less-than-sign state as a
stub: its source effects and reconsuming RCDATA continuation are checked.
State-local adapter tests in `tests/support/rcdata_end_tag_open.rs` require
complete EOF/non-alpha fallback and alpha/name completion, exact diagnostic
multiplicity/positions, and rejection of deliberate mismatches. Real later stubs
retain explicit terminal checks.

## Integration caveats

Started at the requested common baseline
`bcae7e5c916470319ef0838dc72dfd71a95b8181` (HEAD and origin/main), then rebased
onto `651cf0747ede47cd401aba7e38c57d0db90841fb` after PRs #26 and #25 merged.
Both sibling implementations, registrations, tests, and branch evidence are
preserved; obsolete destination blocks are updated explicitly. No neighboring
end-tag-name algorithm is implemented. All CONTRIBUTING checks and the pinned
upstream runner are rerun for this integrated scope.
Dispatch, module registration, support inventory, and the central documentation
have deliberately small additive edits that can conflict with sibling PRs.
Rebase after neighboring merges and rerun every CONTRIBUTING check and the
upstream runner. Destination implementations can legitimately move a terminal
block; review and update those integration assertions explicitly, never remove
blocking semantics or simulate continuation through a stub.
