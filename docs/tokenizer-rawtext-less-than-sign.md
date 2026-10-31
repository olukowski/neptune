# RAWTEXT less-than sign state

## Immutable source and branch mapping

Normative baseline: WHATWG HTML revision
[`404b3eb14a7a936dddefb2d78e224dbeb01d2358`](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143570-L143586),
**RAWTEXT less-than sign state** (`#rawtext-less-than-sign-state`). The production
algorithm is solely `crates/html/src/tokenizer/rawtext_less_than_sign.rs`.

| Consumed input | Source effects | Intended destination |
| --- | --- | --- |
| `/` U+002F | Empty the existing code-point temporary buffer; do not emit, create a tag, or reconsume. | RAWTEXT end tag open |
| Anything else, including EOF | Emit exactly one literal `<`; preserve the buffer; schedule the current code point or conceptual EOF for reconsumption. | RAWTEXT |
| Exhausted open input | No effects; repeatedly return `NeedMoreInput`, not EOF. | Same state |

The first two rows exhaust the pinned algorithm. The third is the existing
incremental cursor contract, not an extra specification branch. This state
reports no HTML parse errors. In particular, neither an invalid-tag-name error
nor `eof-before-tag-name` applies. No pending tag/comment or selector is created,
replaced, abandoned, or emitted. Last start tag and return state are untouched.

Relevant definitions inspected in that same immutable source:

- [Preprocessing, next/current input, and conceptual EOF, lines 142560–142607](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142607):
  CR/CRLF normalize before tokenization; non-scalars/noncharacters/controls
  retain the cursor's preprocessing diagnostics. NUL handling belongs to states.
- [Tokenization/reconsumption, scratch storage, and creation versus emission,
  lines 143149–143191](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143149-L143191):
  reconsumption supplies the current item instead of the next item. Clearing the
  temporary buffer is not end-tag creation.
- [RAWTEXT, lines 143314–143334](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143314-L143334):
  only the subsequent destination step handles NUL replacement/error, a second
  `<`, literal `&`, or terminal EOF. Source EOF recovery itself is not completion.

## Focused coverage and boundaries

`crates/html/tests/rawtext_less_than_sign_state.rs` initializes isolated entry
with an already consumed `<`, both default and deliberately populated context.
It checks both branches, all other ASCII values, Unicode scalars and actual lone
surrogates/noncharacters/controls, exact source output and normalized positions,
preprocessing-error multiplicity, NUL's destination-only error/replacement,
EOF reconsumption, untouched tags/attributes/detached duplicates/comment/return
state/last-start-tag, buffer preservation versus clearing, and absence of token
creation. Source and destination effects are asserted separately.

Repeated open exhaustion has no effects. Repeated terminal EOF/stub steps have
no additional effects. After integration with [RAWTEXT end tag open](tokenizer-rawtext-end-tag-open.md),
the `/` source snapshot still clears only the buffer. The next source step can
suspend, recover literally on non-alpha/EOF, or create an empty selected end tag
before [RAWTEXT end tag name](tokenizer-rawtext-end-tag-name.md) normalizes/appends
on the next step. Integrated tests assert creation and name accumulation separately,
including reconsumption of the first letter, destination-only NUL replacement and
its exact diagnostic. No blocked execution is reconstructed or resumed. RCDATA less-than sign and end tag
open are preserved from merged PRs #26 and #24;
[RCDATA end tag name now completes that separate chain](tokenizer-rcdata-end-tag-name.md).
The upstream counts below record PR #25's historical integration, before #24.

Every two-chunk split and single-code-point chunking of representative isolated
and real RAWTEXT-entry inputs compares every non-suspension source-step snapshot,
including `<`/`/` boundaries, consecutive `<`, literal recovery, CRLF boundaries,
consecutive/trailing CR, NUL, non-scalars, EOF, and terminal blocks.
`tests/support/rawtext_less_than_sign.rs` adds state-local adapter checks for
complete literal fallback, exact diagnostics, rejection of wrong output, and
complete alpha/name output comparison and matching-prefix rejection at genuine
later Character reference blocks. Prior RAWTEXT source tests still inspect their
source boundary; alpha now requires actual normalization/completion, not a name
stub. EOF/non-alpha adapter cases likewise require complete output and errors.

PR #25 implemented only `State::RawtextLessThanSign`; no adjacent algorithm,
shared API/storage, automatic mode selection, tree builder, or dependency is added.
Module/dispatch/support-inventory and test-runner registrations are narrow.

## Upstream evidence and integration

The counts in this section record PR #25's historical integration, before
RAWTEXT end tag open was implemented. For current reachability, the additional
reviewed RAWTEXT passes and updated counts, see
[PR #22 name-state integration evidence](tokenizer-rawtext-end-tag-name.md#integrated-fixture-review).
[PR #23](tokenizer-rawtext-end-tag-open.md#integrated-fixture-review) records the
intermediate one-pass end-tag-open integration.

At baseline `bcae7e5c916470319ef0838dc72dfd71a95b8181`, the pinned upstream runner
reports 6,827 cases / 7,053 runs, 6,190 dispatch-attempted, **1,023 executed,
1,023 passed, 0 failed, 6,030 blocked**. With this state those counts remain
unchanged: the 19 `RawtextLessThanSign` blocks become `RawtextEndTagOpen` blocks.
All 19 `<`-containing RAWTEXT runs were inspected in the unchanged fixtures
(`contentModelFlags.test`, `domjs.test`, `escapeFlag.test`): each eventually
contains `</` and cannot complete while end tag open remains a stub. Literal
fallback can progress before that terminal block, but a matching prefix is not
a pass. There are **zero new upstream passes**, so no baseline file is added or
changed. Existing protected passing identities remain intact.

Integration with main `aef33450e9f4cb3b243a17c588d00700f3bdad14` (PR #26)
preserves both less-than-sign algorithms, state-local tests/evidence, and their
independent module/dispatch/inventory/adapter registrations. The documentation
status conflict is resolved additively. With both states, counts above remain
unchanged; the 19 former RCDATA less-than-sign blocks additionally move to RCDATA
end tag open. All fixture pins and passing-run baseline files remain unchanged.

These are partial conformance counts, not inventory counts or a full-conformance
claim. Rebase after neighboring state merges and rerun all checks plus the
upstream report: a destination implementation can legitimately move a block or
admit new comparable runs. Shared dispatch, inventory, docs, and runner
registrations may have small textual conflicts; review integration behavior
instead of retaining obsolete stub expectations or automatically blessing passes.
