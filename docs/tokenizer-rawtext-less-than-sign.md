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
no additional effects. The `/` destination remains explicitly terminal
`UnimplementedState(RawtextEndTagOpen)`, including with open input, closed input,
and an unconsumed malformed tail. No blocked execution is reconstructed or
resumed. RAWTEXT end tag name and all adjacent RCDATA states remain stubs too.

Every two-chunk split and single-code-point chunking of representative isolated
and real RAWTEXT-entry inputs compares every non-suspension source-step snapshot,
including `<`/`/` boundaries, consecutive `<`, literal recovery, CRLF boundaries,
consecutive/trailing CR, NUL, non-scalars, EOF, and terminal blocks.
`tests/support/rawtext_less_than_sign.rs` adds state-local adapter checks for
complete literal fallback, exact diagnostics, rejection of wrong output, and
matching-prefix rejection at the remaining stub. Prior RAWTEXT source tests now
inspect their source boundary; terminal-block assertions move to end tag open.

This PR implements only `State::RawtextLessThanSign`; no adjacent algorithm,
shared API/storage, automatic mode selection, tree builder, or dependency is added.
Module/dispatch/support-inventory and test-runner registrations are narrow.

## Upstream evidence and integration

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

These are partial conformance counts, not inventory counts or a full-conformance
claim. Rebase after neighboring state merges and rerun all checks plus the
upstream report: a destination implementation can legitimately move a block or
admit new comparable runs. Shared dispatch, inventory, docs, and runner
registrations may have small textual conflicts; review integration behavior
instead of retaining obsolete stub expectations or automatically blessing passes.
