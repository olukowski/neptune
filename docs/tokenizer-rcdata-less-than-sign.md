# RCDATA less-than sign state

Normative source: immutable WHATWG HTML revision
[404b3eb14a7a936dddefb2d78e224dbeb01d2358, lines 143486–143502](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143486-L143502)
(`#rcdata-less-than-sign-state`). Supporting definitions inspected:
[preprocessing, current/next input and conceptual EOF, lines 142560–142607](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142607),
[reconsumption, temporary buffer, token creation versus emission, lines 143153–143194](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143153-L143194),
and [RCDATA, lines 143286–143312](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143286-L143312).
Pins and upstream fixtures are unchanged.

## Branch mapping

| Consumed input | Source effects | Next step |
| --- | --- | --- |
| U+002F SOLIDUS `/` | Clear the temporary buffer; consume without reconsumption; emit nothing. | RCDATA end tag open remains a terminal `UnimplementedState`. |
| Anything else, **including EOF** | Emit literal U+003C `<`; preserve the buffer; schedule the current item for reconsumption in RCDATA. | RCDATA handles that same item/position, including NUL replacement, `&` transition, another `<` transition, or terminal EOF. |

Open exhaustion is not the anything-else branch: it repeatedly returns
`NeedMoreInput` without clearing storage, emitting, or changing current input.
Neither specification branch reports a state diagnostic or creates a token other
than the literal character on fallback. In particular, `/` does **not** create an
end tag, select `current_tag`, or emit the delayed `<`. Existing pending tokens,
their flags/retained and detached attributes, last start tag, return state, and
prior output/diagnostics remain untouched. Empty default storage stays absent;
there is no new tag creation/default initialization in this state.

## Focused evidence

`crates/html/tests/rcdata_less_than_sign_state.rs` tests isolated entry with an
already-consumed `<` and normalized positions after raw CRLF. It separately
asserts source effects before attempting destinations:

- Solidus with empty/seeded scratch storage, all selector choices, default/seeded
  pending tokens, open/closed input, and unread non-scalar tails; exact repeated
  terminal blocks, including after subsequent context mutation.
- Every other ASCII point plus Unicode scalars, actual surrogates, noncharacters,
  controls and NUL. Source diagnostics and storage are checked independently of
  RCDATA's subsequent errors/output/return-state changes. Reconsumption retains
  the exact position, never duplicates preprocessing errors, and leaves tails unread.
- Repeated open-input suspension versus explicit EOF, literal fallback before
  the destination's single EOF, repeated finished behavior, and split CRLF
  suppression followed by exact normalized NUL diagnostics.
- Every two-chunk split and single-code-point chunks of representative literal,
  malformed, non-scalar, CR/CRLF, consecutive/trailing CR, EOF, repeated `<`,
  solidus and ampersand inputs. Traces compare every non-suspension source step
  from both isolated entry and real RCDATA entry. Stubs are never reconstructed
  or resumed.

State-local adapter tests in `tests/support/rcdata_less_than_sign.rs` require
complete literal output and exact diagnostic multiplicities/positions, reject
deliberate mismatches, and retain explicit blocks after matching prefixes.
The preexisting RCDATA tests still inspect its source transition independently;
only the now-obsolete less-than-sign stub expectation is removed. Character
reference and RCDATA end tag open block assertions remain explicit.

## Upstream and integration scope

No new upstream passes are added and all prior passing identities remain
unchanged. Independent fixture inspection shows every pinned RCDATA run with
`<` also reaches `</` (RCDATA end tag open) or an earlier `&` (Character
reference); executing a longer prefix is not conformance. No baseline file or
policy is changed. Use the [runner command/count definitions](tokenizer.md#fixture-runner)
for reproducible executed/passed/failed/blocked counts.

This implements exactly one production state. RCDATA end tag open/name and all
three neighboring RAWTEXT states remain stubs; no shared API/storage prerequisite,
dependency, token variant, automatic mode selection or tree builder is added.
After sibling merges, rebase and rerun all CONTRIBUTING checks and the pinned
upstream runner. Resolve small module/dispatch/support/docs registration conflicts
additively and review legitimate changes to destination blocks; never weaken
them or resume blocked executions to force passes. This is partial standalone
tokenization, not full conformance or browser-security evidence.
