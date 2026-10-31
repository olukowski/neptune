# RAWTEXT end tag open

Only `State::RawtextEndTagOpen` is implemented here. The starting HEAD and
`origin/main` were both `bcae7e5c916470319ef0838dc72dfd71a95b8181`.
PR #23 subsequently merged main `a4721f341f272168dae177a0b6224588c77e50f7`
after PRs #26, #25 and #24 (both less-than-sign states and RCDATA end tag open).
Their algorithms, registrations, source evidence and prior baselines are retained;
no end-tag-name state is implemented here.

## Pinned branch evidence

Normative source: WHATWG HTML revision
`404b3eb14a7a936dddefb2d78e224dbeb01d2358`, as recorded in
[sources.toml](sources.toml). Inspected source sections:

- [Assigned state, lines 143589–143607](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143589-L143607).
- [Preprocessing, current/next input and conceptual EOF, lines 142560–142608](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142608).
- [Reconsumption, temporary buffer, creation versus emission and creation defaults, lines 143150–143189](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143150-L143189).
- [Appropriate end tag definition, lines 143212–143216](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143212-L143216).
  This state does not perform that comparison or change `last_start_tag`.

| Consumed input | Source effects | Intended destination |
| --- | --- | --- |
| ASCII alpha (`A–Z`, `a–z`) | Replace only pending end-tag storage with an empty name, empty attributes, unset self-closing flag and no detached current attribute; select End; schedule reconsumption. No name append, lowercasing or emission. | RAWTEXT end tag name |
| Anything else, including EOF | Emit literal `<` then `/`; preserve pending storage and selector; schedule reconsumption. No source parse error or EOF token. | RAWTEXT |
| Exhausted open input | Effect-free suspension; not the specification's conceptual EOF. | Stay in RAWTEXT end tag open |

Production implementation: `crates/html/src/tokenizer/rawtext_end_tag_open.rs`.
It reuses existing storage, cursor and execution support. Neither branch clears
or appends to `temporary_buffer`; the predecessor's buffer clearing is not
repeated here. NUL replacement and its diagnostic belong to RAWTEXT on the next
step. Preprocessing diagnostics occur once at first consumption, not again on
reconsumption. Only destination RAWTEXT emits EOF after fallback on conceptual EOF.

## Focused tests and limits

`crates/html/tests/rawtext_end_tag_open_state.rs` initializes isolated source entry
with an already-consumed `</` cursor prefix and empty temporary buffer, without
simulating neighboring algorithms. Additional real RAWTEXT-entry tests execute
the merged predecessors, asserting each source snapshot before creation/fallback.
Separate seeded-storage tests verify nonempty buffer preservation without
reprocessing stored non-scalars/NUL.

Coverage includes all 52 ASCII letters and every other ASCII point; alpha
boundaries; punctuation, non-ASCII letters/whitespace, supplementary scalars,
actual surrogates, noncharacters and controls; exact source/destination tokens,
diagnostic codes and normalized positions; CR/CRLF; creation defaults, absent
and stale storage, selected start/end tags, unrelated attributes and detached
attributes; literal fallback, unconsumed tails and EOF reconsumption; repeated
suspension and terminal behavior. Every two-chunk split and single-code-point
chunking of representative isolated and real RAWTEXT-entry inputs compares productive source-step
snapshots and terminal results against closed input. No blocked execution is
reconstructed or resumed.

RAWTEXT less-than sign is now merged and implemented. Alphabetic input
creates/selects the empty pending tag before the next step blocks terminally at
RAWTEXT end tag name, without consuming the reconsumed letter or producing EOF.
Literal `<` fallback now reaches the real less-than-sign algorithm via RAWTEXT;
its source effects and reconsuming continuation are checked rather than retaining
an obsolete stub expectation. End-tag-name states remain unsupported, and no
blocked execution is reconstructed or resumed.

The merged less-than-sign tests retain the `/` source snapshot (only buffer
clearing), then separately check end-tag-open suspension/fallback/creation,
destination RAWTEXT's NUL diagnostic/replacement, and alpha's terminal name-state
block. State-local production-adapter checks in
`crates/html/tests/support/rawtext_end_tag_open.rs` require complete EOF and
non-alpha recovery, exact diagnostic locations/multiplicity, and rejection of
wrong/truncated output or wrong/duplicated diagnostics. Prior matching-prefix
checks now block at RAWTEXT end tag name only for alpha; the original EOF/NUL
inputs instead must compare complete recovery output. No runner policy is weakened.

## Integrated fixture review

The original isolated PR report (1,023 executed/passed, 0 failed, 6,030 blocked)
predates merged predecessors. Main `a4721f341f272168dae177a0b6224588c77e50f7`
reports **1,024 executed, 1,024 passed, 0 failed, 6,029 blocked** after RCDATA end
tag open. With this PR, the 19 RAWTEXT end-tag-open blocks become **18 RAWTEXT
end-tag-name blocks and one new pass**. No direct end-tag-open fixture initial
state or adapter capability change is needed.

All 19 affected RAWTEXT fixture runs were inspected directly in the unchanged
pinned fixtures, not selected by deriving expectations from tokenizer output.
The newly comparable identity is:

| Fixture identity | Unchanged input/context/output | Pinned branch reasoning |
| --- | --- | --- |
| `domjs.test`, `tests[28]`, “bad endtag (space before name)”, `RAWTEXT state`, initial-state index 1 | Input `</ XMP>`, last start tag `xmp`; output one Character string `</ XMP>`; no errors. | RAWTEXT consumes `<`; less-than sign consumes `/` and clears the buffer. End tag open consumes SPACE (not ASCII alpha), emits `</`, and reconsumes SPACE in RAWTEXT. RAWTEXT emits SPACE and the remaining `XMP>` literally, then EOF. No tag is created, no appropriate-end-tag comparison occurs, and neither state nor preprocessing errors apply. |

Only that manually reviewed identity is added in
`crates/html/tests/baselines/rawtext_end_tag_open.json` and registered alongside
all existing files. The sibling RCDATA `domjs.test` case 28 identity (initial-state
index 0) and its registry addition are preserved unchanged. All prior 868 protected
identities remain intact, for 869 total. This is not an automatic pass capture.

The other 18 RAWTEXT runs are `contentModelFlags.test` cases 2–11 (10),
`domjs.test` cases 27 and 29–31 (4), and `escapeFlag.test` cases 0–2 and 4 (4),
all initial-state index 1. Each reaches an ASCII letter immediately after `</`,
so creates an empty end tag and blocks at RAWTEXT end tag name. They remain
blocked, not compared, even after a matching literal prefix. The output and
errors requiring name-state behavior have not been blessed as passes.

Historical PR #23 pinned upstream report (before PR #27): 6,827 fixture cases / 7,053 runs;
6,190 dispatch-attempted; **1,025 executed, 1,025 passed, 0 failed, 6,028 blocked**.
Blocked reasons: CharacterReference 4,692; TokenVariants 858;
MarkupDeclarationOpen 240; ScriptData 89; CdataSection 56;
ProcessingInstructionOpen 52; RawtextEndTagName 18; RcdataEndTagName 18;
XmlCoercion 4; PendingSpecReview 1. Executed = passed + failed.
PR #27 subsequently implements RCDATA end tag name only; see its
[current integrated report and fixture audit](tokenizer-rcdata-end-tag-name.md#integrated-fixture-review).
The 18 RAWTEXT end tag name blocks above remain unchanged.
Inventory and local tests are not upstream conformance; partial passes imply
neither full conformance nor browser security. See
[tokenizer.md](tokenizer.md#fixture-runner) for definitions.

## Integration caveats

The production state algorithm is unchanged by integration; only conflict
resolution, reachability tests, reviewed baseline registration and evidence are
updated. All CONTRIBUTING checks and the pinned runner are rerun on the integrated
branch. Module/dispatch/support-inventory and status-document edits may still
conflict with remaining sibling PRs. Rebase and rerun after neighbor merges: a
new destination can legitimately move a terminal block. Review the integration
change rather than suppressing the block or resuming stopped execution. There is
no automatic mode switching, tree-builder behavior, new token variant, dependency
or shared infrastructure.
