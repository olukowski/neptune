# Script data end tag open

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


## PR #36 integration update

The PR-local reports and earlier entry/stub descriptions below are **historical**.
On merged main `da69390807532870f62ac6e8c2e88bc2fe608ae4` plus PR #36,
EscapeStart now connects ordinary Script data entry through EscapeStartDash and
the escaped trio; `>` in EscapedDashDash returns to Script data. Escaped `<`
still stops terminally at EscapedLessThanSign; escaped end tags and double escaping
remain unsupported. This state's production algorithm and source assertions are
preserved. Tests now follow actual destinations, EOF diagnostics and later stubs.
See [PR #36's current integration/fixture audit](tokenizer-script-data-escape-start.md)
for the nine reviewed new passes and final counts; old reports below are not
current conformance totals.


Implements exactly `State::ScriptDataEndTagOpen`. Originally independent from
main `c8ec2da`, PR #28 subsequently rebased onto main
`3fc6b310a6ea3b3e229a795336eaca72218a14ca`, after independently reviewed
Script data PR #30 and less-than-sign PR #29 merged. PR #31 now rebases onto
main `59bc9ec055074a04419af7fb7068f95a836ff13d`, inheriting all three algorithms
unchanged and adding end tag name. Only escape start remains a stub in the simple
script chain. No unmerged sibling branch is imported. See the
[current name-state report](tokenizer-script-data-end-tag-name.md#validation-and-current-limits);
the PR #28 merge counts and fixture review below are explicitly historical.

## Retrieved and inspected specification evidence

Normative baseline: WHATWG HTML source revision
`404b3eb14a7a936dddefb2d78e224dbeb01d2358`, unchanged in
[sources.toml](sources.toml). The immutable raw source was retrieved and these
sections inspected directly, rather than treating an existing state as evidence:

- [Assigned algorithm, lines 143676–143692](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143676-L143692):
  `#script-data-end-tag-open-state`.
- [Preprocessing, current/next input and conceptual EOF, lines 142560–142608](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142608).
- [Reconsumption, temporary buffer, token defaults and creation versus emission,
  lines 143150–143189](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143150-L143189).
- [Appropriate end tag definition, lines 143212–143216](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143212-L143216).
  This state does not compare names or alter the last emitted start tag.
- [Imported ASCII definitions, lines 2297–2300](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L2297-L2300)
  link to Infra. The linked [ASCII alpha definition](https://infra.spec.whatwg.org/#ascii-alpha)
  and its upper/lower definitions were also retrieved and inspected: exactly
  U+0041–U+005A or U+0061–U+007A, not Unicode alphabetic classification.

| Input | Source effects | Intended destination |
| --- | --- | --- |
| ASCII alpha | Replace pending end tag with empty name, unset self-closing flag, empty attributes and no detached current attribute; select End; reconsume the original point. No append, lowercase conversion, emission or diagnostic. | Script data end tag name |
| Anything else, including EOF | Emit `<` then `/` character tokens; reconsume the current point or conceptual EOF. No tag creation, storage abandonment, source parse error or EOF token. | Script data |
| Open exhausted input | Effect-free `NeedMoreInput`, retaining current input and source state. This is not conceptual EOF. | No transition |

`crates/html/src/tokenizer/script_data_end_tag_open.rs` reuses the existing
`PendingEndTag::default()`, selector, cursor and execution contract; no shared
prerequisite is missing. Only alpha creation replaces end-tag storage. Other
pending storage, prior output/diagnostics, return state, last start tag and the
temporary buffer are preserved. This state never clears or appends the buffer.
NUL takes literal fallback without replacement or a state diagnostic here.
Surrogates, noncharacters and controls retain the cursor's original diagnostics
and normalized positions; reconsumption does not duplicate them.

## Focused tests and explicit limits

`crates/html/tests/script_data_end_tag_open_state.rs` contains five local tests:

- All 52 ASCII letters, unchanged at the source boundary, with absent/stale
  candidate storage, all selector choices, and open/closed input. Creation
  defaults are independently explicit, including the cleared detached attribute.
  The next name step separately checks normalized accumulation and original
  scratch spelling at the same position; following literal mismatch recovery
  and Script data reconsumption/EOF preserve unrelated storage.
- Every other ASCII point plus non-ASCII letters/whitespace, supplementary
  scalars, actual surrogates, noncharacters and controls. Exact literal token
  order, source diagnostics, preprocessing codes/positions, CR normalization,
  unrelated context and current-input identity are asserted. The next real
  Script data step separately checks NUL replacement/error, once-only
  preprocessing, literal emission or another `<` transition; the tail, repeated
  suspension and exact EOF/escape-start terminal boundary are also asserted.
- Repeated open-input suspension versus explicit EOF, including empty entry and
  already-consumed prefixes at different normalized positions. EOF emits only
  `</` here and schedules EOF reconsumption; only the following Script data
  step emits terminal EOF, at the same conceptual position.
- Resumption from suspension with fresh default context for both branches,
  including NUL. Alpha creates an empty candidate, then the real name step
  accumulates it; open exhaustion suspends before actual EOF literal recovery.
  Only the following Script data step emits EOF.
- Every two-chunk split and single-code-point chunking of representative suffixes
  matches closed-input productive source-step snapshots and terminal results.
  Empty chunks, CRLF, consecutive/trailing CR, EOF, malformed input and non-scalars
  are included. An already-consumed trailing CR additionally checks suppression
  of a following LF before either alpha or fallback/EOF selection.

Isolated cursor prefixes are positioned directly, not simulated algorithms.
`script_data_state.rs` additionally asserts each real Script data → less-than-sign
→ end-tag-open source boundary, buffer clearing, repeated end-tag-open
suspension, candidate creation versus fallback/NUL/EOF destination steps and
exact preserved context. Name reconsumption, original-spelling accumulation,
suspension and literal EOF recovery now execute with separate exact snapshots.
`script_data_less_than_sign_state.rs` retains all source assertions and checks
real end-tag-open creation on a following letter, name accumulation and EOF fallback;
its split/single-point traces include real entry, non-alpha `</` recovery, NUL,
CRLF and later blocks. No expectations dynamically detect implementation support.

The production-adapter checks require complete literal fallback output and exact
errors, deliberately reject wrong prefixes/missing or duplicated diagnostics,
and require complete matching-tag and literal name recovery/EOF results.
Escape-start and later Data/attribute blocks persist despite matching prefixes.
Stored non-scalars are never preprocessed again. Repeated terminal EOF/stub
steps preserve their result and all effects. Real name steps confirm alpha's
original reconsumed item before the tail is consumed; no blocked execution is
reconstructed/resumed. Traces execute every real step through completion or the
first genuine stub; unread tails after a stub are not coverage. These are local
streaming checks, not whole-tokenizer conformance.

## Historical PR #28 checks and upstream results

At PR #28's merge, all required checks passed (current counts are in the
[name-state report](tokenizer-script-data-end-tag-name.md#validation-and-current-limits)):

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked` — 230 tests passed, 0 failed/ignored,
  including all five state-local tests, the integrated source-boundary test,
  merged tests and the prior pass-identity floor.
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`
- `cargo test --workspace --locked pinned_upstream_runner -- --nocapture`

The fixture submodule was already initialized at the pinned
`c777c408b61078ea2eb4acefc2535f54dbc8b28a` revision. Report:
**6,827 fixture cases; 7,053 runs; 6,190 dispatch-attempted;
1,112 executed; 1,112 passed; 0 failed; 5,941 blocked**.
Executed = passed + failed; dispatch attempts ending at stubs are not passes.

Blocked reasons: CharacterReference 4,692; TokenVariants 858;
MarkupDeclarationOpen 242; ScriptDataEscapeStart 31; ScriptDataEndTagName 5;
CdataSection 56; ProcessingInstructionOpen 52; XmlCoercion 4; PendingSpecReview 1.
Local tests and support inventory are not upstream conformance passes. No fixture,
pin, adapter execution/comparison or policy is changed. This is partial coverage,
not full conformance or browser security.

## Integrated fixture review

This is the historical PR #28 fixture audit. The original isolated PR reported
zero new passes because Script data was a stub. Main after PR #29 had 1,111
executed/passed, 0 failed, 5,942 blocked. All six end-tag-open-blocked Script data
runs were inspected in the unchanged pinned fixtures. At PR #28's merge five
then blocked at Script data end tag name; one completed:

| Fixture / tests index | Initial-state index | Unchanged input/output and pinned branch reasoning |
| --- | --- | --- |
| `domjs.test` 28, “bad endtag (space before name)” | 2, Script data | `lastStartTag: xmp`; input and sole Character output `</ XMP>`; no errors or double escaping. Script data consumes `<`; less-than-sign consumes `/`, clearing the buffer. End tag open consumes SPACE (not alpha), emits `</`, and reconsumes SPACE in Script data. SPACE and `XMP>` emit literally, then terminal EOF (not fixture output). No candidate, appropriateness check or parse error applies. |

Only this individually reviewed identity is added in
`baselines/script_data_end_tag_open.json` and registered alongside every prior
file. All **915** previous protected identities remain unchanged and passing;
the union is now **916**. Expectations are not generated from execution.

The other five runs are `domjs.test` cases 27, 29–31 (Script data initial-state
index 2) and 42 (index 0). Their unchanged inputs are `</XMP>`, `</xm>`,
`</xm `, `</xm/`, and `<b>hello world</b>`, respectively. Each reached ASCII alpha
after `</`, created an empty selected end tag and then blocked terminally at
name at PR #28's merge. Case 42 first emitted `<b>hello world` literally, but
that matching prefix was not a pass. PR #28 added none of those identities.
PR #31 now implements the name state and individually reviews all five in its
[separate audit/baseline](tokenizer-script-data-end-tag-name.md#integrated-fixture-review).
The 31 escape-start blocks and broader limitations remain explicit.

## Sibling integration considerations

PR #28 added no name-state logic, additional predecessor algorithm, automatic
element-based mode switching, tree builder, dependency, abstraction or shared storage.
Module/dispatch and independent support-inventory conflicts were resolved
additively, preserving the merged states. Current-scope comments/docs and explicit
source/destination tests are updated; no tests are skipped or implementation-
detection expectations introduced.

The user authorized sequential integration, independently reviewing each exact
pushed revision immediately before merge. PRs #30/#29/#28 are now inherited from
main. PR #31 adds name recognition/recovery, preserving each source effect while
updating obsolete destination-stub assertions across all four suites/adapters.
All checks/upstream reporting are rerun; the five new pass identities are
individually reviewed. The parent will freshly review the exact pushed revision;
this branch neither merges, starts reviewers/subagents nor enables auto-merge.
Escape start remains an explicit block. Never resume an execution that hit a stub.
