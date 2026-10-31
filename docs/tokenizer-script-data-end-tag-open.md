# Script data end tag open

Implements exactly `State::ScriptDataEndTagOpen` from main
`c8ec2da` (RAWTEXT end tag name, PR #22). Script data, Script data less-than
sign, and Script data end tag name remain unimplemented on this independent
branch. No sibling branch is imported or integrated.

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
- Every other ASCII point plus non-ASCII letters/whitespace, supplementary
  scalars, actual surrogates, noncharacters and controls. Exact literal token
  order, source diagnostics, preprocessing codes/positions, CR normalization,
  unrelated context and current-input identity are asserted.
- Repeated open-input suspension versus explicit EOF, including empty entry and
  already-consumed prefixes at different normalized positions. EOF emits only
  `</` here and schedules EOF reconsumption, never a successful EOF token.
- Resumption from suspension with fresh default context for both branches,
  including NUL.
- Every two-chunk split and single-code-point chunking of representative suffixes
  matches closed-input productive source-step snapshots and terminal results.
  Empty chunks, CRLF, consecutive/trailing CR, EOF, malformed input and non-scalars
  are included. An already-consumed trailing CR additionally checks suppression
  of a following LF before either alpha or fallback/EOF selection.

Cursor prefixes are positioned directly; these tests do not simulate or execute
unimplemented predecessors. Stored non-scalars are not preprocessed again.
Both destinations are terminal `UnimplementedState` on the next execution step,
including Script data after EOF fallback. Repeated steps preserve the same block
and all effects. Cursor-only inspection confirms the original reconsumed item,
its position and unconsumed tail; no blocked execution is reconstructed/resumed.
Chunk traces stop at the first stub and do not claim coverage of unread tails,
predecessor streaming, destination recovery or whole-tokenizer conformance.

## Checks and upstream results

All required checks passed:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked` — 215 tests passed, 0 failed/ignored,
  including all five new state-local tests and the existing pass-identity floor.
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`
- `cargo test --workspace --locked pinned_upstream_runner -- --nocapture`

The fixture submodule was already initialized at the pinned
`c777c408b61078ea2eb4acefc2535f54dbc8b28a` revision. Report:
**6,827 fixture cases; 7,053 runs; 6,190 dispatch-attempted;
1,059 executed; 1,059 passed; 0 failed; 5,994 blocked**.
Executed = passed + failed; dispatch attempts ending at stubs are not passes.

Blocked reasons: CharacterReference 4,692; TokenVariants 858;
MarkupDeclarationOpen 242; ScriptData 89; CdataSection 56;
ProcessingInstructionOpen 52; XmlCoercion 4; PendingSpecReview 1.
The assigned state is not reachable from the runner's existing entry states
while Script data remains a stub: **zero new upstream passes**. No baseline
addition is warranted; all 903 existing protected identities and their files and
registrations remain unchanged. Local tests and support inventory are not
upstream conformance passes. No fixture, pin, adapter, comparison or policy is
changed. This is partial coverage, not full conformance or browser security.

## Sibling integration considerations

No name-state logic, predecessor algorithm, automatic element-based mode
switching, tree builder, new dependency, abstraction or shared storage is added.
Module/dispatch and independent support-inventory changes register only this
state; the shared status document only adds its state-local link.

The user will integrate/rebase the four independent Script data PRs later.
Once destinations are implemented, source expectations should remain unchanged,
but destination-block assertions must be reviewed and replaced with the real
next-step effects. Rerun all checks and the upstream report after integration;
review newly comparable fixture identities individually before adding a
state-specific baseline. Never resume an execution that already hit a stub.
This PR is for review only, not merge or auto-merge.
