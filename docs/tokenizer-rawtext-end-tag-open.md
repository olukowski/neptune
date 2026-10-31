# RAWTEXT end tag open

Only `State::RawtextEndTagOpen` is implemented here. The starting HEAD and
`origin/main` were both `bcae7e5c916470319ef0838dc72dfd71a95b8181`.

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

`crates/html/tests/rawtext_end_tag_open_state.rs` initializes isolated entry with
an already-consumed `</` cursor prefix and empty temporary buffer. It does not
execute or simulate neighboring state algorithms. Separate seeded-storage tests
verify nonempty buffer preservation without reprocessing stored non-scalars/NUL.

Coverage includes all 52 ASCII letters and every other ASCII point; alpha
boundaries; punctuation, non-ASCII letters/whitespace, supplementary scalars,
actual surrogates, noncharacters and controls; exact source/destination tokens,
diagnostic codes and normalized positions; CR/CRLF; creation defaults, absent
and stale storage, selected start/end tags, unrelated attributes and detached
attributes; literal fallback, unconsumed tails and EOF reconsumption; repeated
suspension and terminal behavior. Every two-chunk split and single-code-point
chunking of representative assigned-state inputs compares productive source-step
snapshots and terminal results against closed input. No blocked execution is
reconstructed or resumed.

RAWTEXT less-than sign and RAWTEXT end tag name remain terminal
`UnimplementedState` stubs. Alphabetic input creates/selects the empty pending
tag before the next step blocks at RAWTEXT end tag name, without consuming the
reconsumed letter or producing EOF. Literal `<` fallback reaches the separate
less-than-sign stub via RAWTEXT. Existing support inventory and RAWTEXT tests
retain explicit terminal-stub assertions.

No fixture runner initial-state mapping or adapter change is needed: upstream
fixtures enter RAWTEXT, whose less-than-sign destination still blocks before
this assigned state. Consequently this change admits **zero new upstream
passes**, and all prior passing-run identities remain unchanged. Inventory and
these isolated tests are not upstream conformance. See the PR report for executed,
passed, failed and blocked run counts, and [tokenizer.md](tokenizer.md#fixture-runner)
for definitions.

Rebase and rerun all required checks and the upstream runner after sibling
state PRs merge. Module/dispatch/support-inventory and status-document edits may
conflict; a newly implemented neighbor can legitimately move a terminal block.
Review that change rather than suppressing the block or resuming an execution
that has already stopped at a stub. There is no automatic mode switching,
tree-builder behavior, new token variant, dependency or shared infrastructure.
