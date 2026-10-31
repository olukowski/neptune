# Script data escaped less-than sign state

## Scope and immutable evidence

This PR implements exactly `State::ScriptDataEscapedLessThanSign`, in
`crates/html/src/tokenizer/script_data_escaped_less_than_sign.rs`, against reviewed
main `39d772e31b98e6e955a2e00f546a8db3506eb1dd` (after PR #36). Initial HEAD was
verified equal before implementation. No sibling changes are imported.

The complete HTML source was retrieved from the immutable revision
[`404b3eb14a7a936dddefb2d78e224dbeb01d2358`](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source),
not the living algorithm or another engine. Reviewed evidence:

- [Assigned section, lines 143872–143892](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143872-L143892),
  anchor `#script-data-escaped-less-than-sign-state`.
- [Input preprocessing, next/current input and conceptual EOF, lines 142560–142607](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142607).
  The existing cursor supplies newline normalization and preprocessing errors.
  Its referenced Infra [normalize newlines](https://infra.spec.whatwg.org/#normalize-newlines)
  definition replaces CRLF with LF and then remaining CR with LF.
- [Tokenization, reconsumption, temporary buffer, output and token creation, lines 143149–143189](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143149-L143189).
  Reconsumption provides the current item again instead of the next item; it is
  not an additional consume or immediate destination execution.
- [HTML's imported ASCII-alpha definitions, lines 2297–2299](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L2297-L2299)
  reference Infra [ASCII alpha](https://infra.spec.whatwg.org/#ascii-alpha):
  U+0041–U+005A or U+0061–U+007A, inclusive. Non-ASCII letters do not qualify.
- [Real escaped predecessors and fallback destination, lines 143775–143869](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143775-L143869).
  The escaped destination, not this gateway, replaces NUL/reports its error or
  reports `eof-in-script-html-comment-like-text` and emits EOF.

## Branch-to-code/test map

Focused evidence lives in
`crates/html/tests/script_data_escaped_less_than_sign_state.rs`.

| Consumed input | Required source effects | Focused tests |
| --- | --- | --- |
| `/` | Empty temporary buffer; switch to escaped end tag open. No emission, reconsumption, tag creation or selection. | `every_ascii_branch_and_non_scalars_preserve_exact_source_effects_and_spelling`; `slash_and_alpha_destinations_are_terminal_with_no_destination_effects` |
| ASCII alpha | Empty buffer; emit only `<`; reconsume in double escape start. Do not append, lowercase or emit the alpha. | Same two tests, including every upper/lower letter and unchanged original spelling |
| Anything else, including EOF | Emit only `<`; preserve buffer; reconsume in escaped. No source parse error, NUL replacement or EOF emission. | Exhaustive source test; `fallback_reconsumption_defers_null_handling_and_never_duplicates_preprocessing`; `open_exhaustion_and_reconsumed_eof_have_distinct_exact_effects` |
| Open exhausted input | Effect-free `NeedMoreInput`, not conceptual EOF. | Repeated suspension in the EOF test, real predecessor test and streaming trace |

The source snapshot tests cover all ASCII points and boundaries, punctuation,
non-ASCII letters/whitespace, NUL, replacement character, supplementary scalars,
actual lone surrogates, noncharacters and controls. They assert current input,
normalized positions, intended target, exact output/error prefixes, buffer
clearing versus preservation, suffix consumption and reconsumption. Full storage
snapshots preserve the selected tag, both pending tags (including retained and
detached attributes/flags), pending comment, last emitted start tag, return state,
and existing tokens/diagnostics. No pending token is created, abandoned or emitted.

`malformed_fallback_stream_has_exact_normalized_diagnostic_positions_and_multiplicities`
checks repeated NUL, CRLF and CR, surrogate/noncharacter/control errors, a later
escaped `<`, and EOF with independently specified complete tokens and exact
one-based normalized diagnostic coordinates. Reconsumption adds no duplicate
preprocessing diagnostic. Repeated terminal calls cannot add errors/output.

Direct entry explicitly consumes the predecessor's `<` in cursor setup before
constructing the execution; it initializes no fake tag or destination algorithm.
The independent support inventory registers the state additively and supplies
this setup for open exhaustion and EOF. `all_real_predecessors_reach_the_gateway_before_its_source_step`
executes Script data → escape start → escape start dash and each real
escaped/escaped-dash/escaped-dash-dash entry, preserving predecessor snapshots
before testing gateway effects and destination blocks.

`every_two_chunk_split_and_single_point_chunking_match_every_source_snapshot`
compares every non-suspension source-step snapshot against closed input for direct
entry and all three real predecessor paths. It covers every two-chunk split,
including empty chunks, and single-code-point chunks: CRLF boundaries,
consecutive/trailing CR, malformed text, NUL/non-scalars, EOF and both terminal
stubs. Traces stop immediately at terminal execution; none reconstructs or resumes
it. Existing predecessor tests retain their source boundaries and now inspect
real fallback or the later terminal block instead of expecting this state's stub.
Local tests and inventory checks are not upstream conformance runs.

## Independently reviewed upstream identities

Fixture revision remains `c777c408b61078ea2eb4acefc2535f54dbc8b28a`; the submodule
was already initialized at that pin. No fixture or specification pin is modified.
Two newly comparable identities are added to the state-specific baseline file,
registered without changing any prior baseline or registration:

| File / zero-based case | Description / unchanged input and output | Pinned branch reasoning |
| --- | --- | --- |
| `domjs.test`, `tests[14]` | “Dash less-than in script HTML comment”; `<!-- -< -->` | Real escaped dash consumes `<`; this gateway consumes SPACE, emits `<`, and reconsumes SPACE in escaped. The later `-->` returns to Script data, which emits EOF. |
| `test1.test`, `tests[37]` | “< in script HTML comment”; `<!-- < test -->` | Real escaped consumes `<`; this gateway takes the same SPACE fallback. Remaining text emits literally; `-->` returns to Script data before EOF. |

Both have exactly one Script data initial-state entry (index 0), no
`lastStartTag`, no `doubleEscaped`, and no expected diagnostics. Output is one
Character string identical to input; terminal EOF is omitted only by the existing
fixture format adapter. Expected results above were inspected in unchanged
fixtures and reasoned from the pinned branches, not generated from execution.
The protected baseline grows from 930 to 932 identities.

## Validation and remaining boundaries

All CONTRIBUTING checks and the separate upstream report passed:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo test --workspace --locked pinned_upstream_runner -- --nocapture
```

The workspace Rust harness passed 291 tests, with zero failed/ignored, including
seven focused state tests and the upstream runner test. Harness/inventory counts
are distinct from the following conformance-run counts.

Upstream: 6,827 fixture cases / 7,053 case-initial-state runs;
6,190 dispatch-attempted; **1,128 executed = 1,128 passed + 0 failed**;
**5,925 blocked**. The verified common base had 1,126 executed/passed, zero
failed and 5,927 blocked. All 930 prior protected identities still pass.

Blocked categories: Character reference 4,692; Markup declaration open 242;
Processing instruction open 52; CDATA section 56; Script data double escape
start 17; Script data escaped end tag open 3; unsupported token variants 858;
XML coercion 4; pending-spec review 1. Of the base's 22 escaped-less-than-sign
blocks, two now pass and twenty reach the two explicit terminal destinations.
No blocked prefix is counted as success. This remains partial conformance,
not full tokenizer conformance or a browser-security claim.

There is no missing shared prerequisite. Production additions are this module,
dispatch registration and a buffer-use comment; no storage/API redesign,
helper framework, dependency, tree builder, automatic mode switch, JavaScript
parsing or Comment-token behavior is introduced. The other eight sibling states
remain untouched and unimplemented on this standalone branch.

Later sibling merges/rebases must preserve additive module/dispatch/inventory and
baseline registration, then review moved terminal assertions and newly comparable
fixtures, rerunning every check and the upstream report. Escaped end-tag-open and
double-escape-start source effects must not be attributed to this gateway. No
sibling checkout is fetched, cherry-picked or merged here. Integration and merging
are left to the coordinator after the user's signal; this PR is review-only.
