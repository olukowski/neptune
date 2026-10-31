# Before attribute value evidence

## Scope and immutable source

PR #21 adds only `State::BeforeAttributeValue`, in
`crates/html/src/tokenizer/before_attribute_value.rs`. Originally based on shared
prerequisite PR #15 (`046565d7c033ed2bff4934b83f689284113bb1b4`), the final batch
integration rebases onto main `2983fbe809621963f8399e6e94072ccfc04937ce` (PRs
#16–#20). It preserves After attribute name, all three value states, and After
attribute value (quoted), without changing their algorithms. No adjacent state,
dependency, fixture, pin, runner comparison, or exclusion is added/changed.

Normative evidence is the immutable HTML Standard revision
`404b3eb14a7a936dddefb2d78e224dbeb01d2358`, not the living standard:

- [Before attribute value, lines 144266–144293](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144266-L144293).
- [Destination value/after-quoted rules, lines 144296–144432](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144296-L144432).
- [Reconsumption and creation/emission/abandonment, lines 143150–143210](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143150-L143210).
- [Removed duplicates remain current, lines 144218–144229](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144218-L144229).
- [Preprocessing and next/current input, lines 142560–142600](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142600).

## Every algorithm branch

Seven focused tests live in
[`before_attribute_value_state.rs`](../crates/html/tests/before_attribute_value_state.rs).
Names below identify tests by distinctive prefix.

| Pinned branch | Code | Focused evidence |
| --- | --- | --- |
| TAB, LF, FF, SPACE: ignore | First `CodePoint` arm; stay here | `whitespace_quotes_and_anything_else…`, `open_exhaustion…`, `all_two_chunk_splits…`: each separator, runs, normalized CR/CRLF, consumption. |
| `"`: switch to double-quoted value | Second arm; no reconsumption | Point matrix and `attribute_name_detachment…`: consumed quote, unchanged current value/storage; destination appends only on its own next step. |
| `'`: switch to single-quoted value | Third arm; no reconsumption | Same tests, independently checking single-quoted target. |
| `>`: missing-value error, Data, emit tag | Fourth arm; shared `tag::emit_tag` | `greater_than…`, `real_entry…`: exact error locations, selected-token take, retained order/values, duplicate exclusion, both end-tag errors, last-start-tag bookkeeping, Data continuation, one EOF. |
| Anything else: reconsume in unquoted value | `CodePoint(_) \| Eof` arm | Point matrix, `open_exhaustion…`, `real_non_scalar_and_control…`, `real_entry…`: malformed punctuation/NUL, Unicode whitespace, non-scalars/controls, EOF identity and position. |

Open exhaustion is an execution contract, not an additional spec EOF branch:
NeedMoreInput has no effects, preserves current input/storage, and can resume after
feeding. Empty chunks and repeated suspension are tested.

**EOF boundary:** this state has **no EOF-specific branch**. EOF falls through
anything else and is reconsumed in unquoted value. The source snapshot retains
the pending tag and emits/reports nothing. On the **next** step, the implemented
unquoted destination reports `eof-in-tag`, abandons only the selected tag, and
emits one terminal EOF. `<p a=` now completes by abandonment, not by emitting a
tag or reporting a missing-value error. The original PR21 report's unquoted-stub
block was historically correct before PRs #16–#20 were integrated, not the current
combined behavior. Abandonment must never be moved into Before attribute value.

No branch initializes, clears, or appends an attribute value. The ASCII/non-scalar
point matrix checks source snapshots and directly inspects cursor consumption;
it deliberately does not dispatch destinations in every seeded matrix case.
Separate tests execute destinations: duplicate detachment remains current,
value appends use `tag::current_attribute_mut`, and retained predecessors are
unchanged. Only `>` takes the selected tag through shared emission. Unrelated
pending tags/comments, return/scratch state, prior output/diagnostics are preserved.
Both tag kinds/flags and detached-only storage are covered; detached-only storage
does not trigger `end-tag-with-attributes`.

## Combined Data-entry evidence

Data now reaches quoted/unquoted/boolean attribute parsing. All affected earlier
local and runner assertions follow real destination behavior: complete tokens and
diagnostics, destination-owned EOF abandonment, or a still-terminal non-attribute
stub. Source-step snapshots remain inspectable before those destination effects.
No merged algorithm needed a fix; no expected error is suppressed.

Three tests in `attribute_value_integration.rs` additionally check:

- Mixed quoted/unquoted/boolean attributes, duplicate followed by a value and a
  new attribute, missing values after duplicates, no-whitespace recovery, exact
  error positions and multiplicity, both tag kinds, ordered retained values,
  shared end-tag diagnostics and start-tag-only last-start-tag updates.
- Real source-step lifecycle: duplicate detachment at `=`, value mutation on the
  detached identity, then `start_attribute` clearing it before appending the new
  empty attribute; the predecessor's value remains unchanged.
- Ampersands in all three value states, including detached duplicates: correct
  return state and pending values, unconsumed tail, no fake EOF or parse error,
  and repeated effect-free terminal CharacterReference dispatch.

Literal `/` inside an unquoted value is distinct from structural `/` after a
quoted value or whitespace. End-tag attributes/solidus errors coexist at emission;
EOF abandonment reports neither emission error. Quoted local suites now check
real Data completion, EOF, NUL replacement and terminal ampersands; malformed
After-quoted recovery tests inspect Before-value EOF reconsumption separately
from unquoted abandonment. Runner regressions reject missing diagnostics and
matching prefixes, including prefixes before abandoned tags and real stubs.

Every two-chunk split and single-code-point chunking compares complete source-step
snapshots through first termination, including the newly reachable mixed-value,
duplicate, no-whitespace, missing-value, end-tag/solidus and ampersand samples.
CRLF splits, consecutive/trailing CR, EOF, NUL, actual surrogates/noncharacters/
controls remain covered. Seeded storage tests are independent local support
checks, not themselves end-to-end or upstream conformance evidence. Blocked
executions are never reconstructed/resumed.

## Original eight reviewed upstream identities

Pinned html5lib revision remains `c777c408b61078ea2eb4acefc2535f54dbc8b28a`.
All identities below use root `tests`, implicit `Data state`, initial-state index
0, and zero-based case indices. Inputs/expected tokens/errors were re-inspected
in the unchanged fixture JSON against the pinned source, not inferred from actual
execution. `before_attribute_value.json` retains these eight identities:

| File / case | Independent input/expectation review |
| --- | --- |
| `test3.test` / 1317, 1442 | Two distinct `<a a=>` identities: tag `a`, `a=""`; only `missing-attribute-value` at 1:6. |
| `test3.test` / 1320 | TAB between `=` and `>`; same token, error 1:7. |
| `test3.test` / 1321 | LF; same token, error 2:1. |
| `test3.test` / 1323 | FF; same token, error 1:7 (FF does not advance line). |
| `test3.test` / 1324 | CR normalizes to LF; same token, error 2:1. |
| `test3.test` / 1326 | SPACE; same token, error 1:7. |
| `test4.test` / 4 | `<z ==>`: tag `z`, `=""`; unexpected-equals-before-name at 1:4, missing-value at 1:6. |

The **historical PR21-only** report on prerequisite #15 was 665 executed/passed,
0 failed, 6,388 blocked, versus 657 prior passes. Its quoted/unquoted destinations
were then stubs. These totals are not current combined support.

## Combined integration review

`integrated_attribute_values.json` adds **42** manually selected identities,
independently inspected in the pinned fixtures and reconciled with the source
branches above. This is an explicit review allowlist, not execution-driven
blessing. No previous baseline entry/file is removed or rewritten. All 817 main
identities plus the original eight remain protected; total protected identities
are **867**. Descriptions, case indices and initial states remain exact in JSON.

| File / cases | Expected tokens and diagnostics, independently reviewed |
| --- | --- |
| `test1.test` / 9, 10 | `h` with `a=b` in single-quoted/unquoted form; no errors. |
| `test1.test` / 13 | Start `h`, end `h`; end-tag attribute error at 1:13 (end fixture projects fields, local tests retain them). |
| `test1.test` / 14, 15 | `h` with `a=b,c=d`; only case 15 lacks whitespace, error at 1:9. |
| `test1.test` / 16 | Repeated `a`: retain `b`, discard new value `d`; duplicate at 1:11. |
| `test1.test` / 68 | `a` with `a=f<`; unquoted forbidden `<` appended with error at 1:7. |
| `test2.test` / 26, 27, 28, 39 | Respectively self-closing `br foo=bar` (no errors); malformed `h/a=b` (solidus error 1:4); double-quoted `h a=b`; boolean `a` followed by uppercase `B=''` normalized to `b`. Last two have no errors. |
| `test3.test` / 1318, 1322 | Unquoted NUL becomes U+FFFD with unexpected-null at 1:6; VT is retained with only control-input error at 1:6. |
| `test3.test` / 1328, 1329, 1331 | Double-quoted empty/NUL/LF values: respectively empty without error, U+FFFD with null error 1:7, literal LF without error. |
| `test3.test` / 1365, 1366 | Single-quoted empty/NUL: respectively no error, U+FFFD with null error 1:7. |
| `test3.test` / 1377, 1381 | Empty quoted `a`, followed by NUL/VT name: new U+FFFD/VT boolean attribute; missing-whitespace at 1:8 plus null/control-input error at 1:8. |
| `test3.test` / 1383, 1387 | CR after quote normalizes to whitespace, no new attribute/error; double quote after quote starts literal `"` boolean name, missing-whitespace and unexpected-name-character both 1:8. |
| `test3.test` / 1392, 1397, 1401 | Empty quoted `a` followed by `/` (flag true, no error), `=` (new boolean name `=`, missing-whitespace and unexpected-equals both 1:8), or `A` (duplicate removed, missing-whitespace 1:8 and duplicate 1:9). |
| `test3.test` / 1436, 1441 | Unquoted `/` is the value, not a flag (no error); unquoted `=` is appended with forbidden-value-character error 1:6. |
| `test4.test` / 5, 6 | `<z ===>` / `<z ====>`: name `=`, value `=` / `==`; unexpected-equals-before-name 1:4, forbidden-value-character 1:6 (and 1:7 in case 6), preserving multiplicity. |
| `test4.test` / 17, 18 | `foo a=b'c` / `b"c`; embedded quote appended with forbidden-value-character at 1:9. |
| `test4.test` / 19, 20 | Quoted `a=b` immediately followed by boolean `c`; missing-whitespace at 1:11. |
| `test4.test` / 21, 22 | `br` / `bar` with `a=b` and flag true; no tokenizer error. Tree-builder acknowledgement is outside scope, even for `bar`. |
| `test4.test` / 47 | `x x=1 x=2 X=3`: retain `x=1`, two duplicates at 1:9 and 1:13, no value leakage. |
| `test4.test` / 75 | `a=aa` plus literal grave accent; forbidden-value-character 1:8. |
| `test4.test` / 80–84 | EOF before value, in double-/single-/unquoted value, after quoted value: no tag, exactly `eof-in-tag` at 1:7, 1:9, 1:9, 1:8, 1:10 respectively. Case 80 reconsumes EOF before abandonment. |

The current runner compares **1,023 executed = 1,023 passed + 0 failed**, with
6,030 blocked out of 7,053 runs / 6,827 fixture cases; 6,190 dispatch-attempted.
This is **206 new passes over main's 817**, including the original eight PR21
identities and 198 additional integration passes. Only **42** of those 198 are
added to the reviewed regression allowlist here; new passes are not synonymous
with baseline additions. Every comparable run is still checked, protected or not.
There were **zero newly comparable mismatches** to suppress or exclude.

## Validation and limits

Run all [CONTRIBUTING checks](../CONTRIBUTING.md), the upstream report with
`--nocapture`, and `git diff --check`. Exact final check evidence and blocked
reason counts are recorded in the PR report, separately from local test functions.
Character reference remains terminal inside values and Data; comments/declarations,
script/CDATA and other unsupported states and adapter policies retain explicit
blocks. No tree builder, whole-tokenizer conformance, or browser-security claim.
