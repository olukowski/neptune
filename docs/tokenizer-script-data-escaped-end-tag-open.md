# Script data escaped end tag open

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


## Current integration and historical baseline

The user authorized sequential integration with independent final-head review.
This PR is rebased onto exact main **`56056932b73bcac03f3633937705ed1ed77221d0`**,
after merged #43, #38, #39, #40, #42, #37 and #44. All seven merged production
algorithms, registrations, independent inventory setup and scope documentation
are preserved additively. Only `origin/main` was fetched; #45 was not imported.
The assigned production algorithm is unchanged from standalone head
`dec41ce0e9892520197fdd98df1a3d6a6793fd5b`. Its **291-test / seven-focused-test**
report and terminal-name destination were historical, not current boundaries.
The exact integration review head is recorded in [PR #41](https://github.com/olukowski/neptune/pull/41).

Current focused coverage executes the real EscapedEndTagName destination, with
its original source-step assertions retained. All 52 alpha tests now separately
assert normalized name/original buffer append at the same reconsumed position,
then literal mismatch and destination EOF. Fresh suspended alpha entry now
continues through name accumulation, suspension and literal EOF recovery.
Three additional tests check exact-history appropriate/inappropriate delimiters,
real attributes/duplicate removal/NUL/solidus recovery/tag EOF, and matching-name
EOF/non-scalar mismatch recovery. The merged name suite's obsolete isolated-open
stub assertion now checks real empty creation, reconsumed mixed-case accumulation,
appropriate emission and complete Data continuation/EOF with full snapshots.
Its genuine Script data/Escaped predecessor blocks remain unchanged.

## Scope and entry boundary

Implements exactly `State::ScriptDataEscapedEndTagOpen`, in
`crates/html/src/tokenizer/script_data_escaped_end_tag_open.rs`, on verified main
original base **`39d772e31b98e6e955a2e00f546a8db3506eb1dd`** (after PR #36);
the current merged base is recorded above.
No sibling state, dependency, API/storage redesign, fixture, pin, adapter or
baseline change is included. Existing Context, cursor, tokens and execution
infrastructure suffice. Shared code changes are additive module/dispatch and
independent support-inventory registration, with narrow current-scope links.

Normal Script data entry still stops at `ScriptDataEscapedLessThanSign`, the
immediate predecessor. Isolated entry uses
`Execution::new(State::ScriptDataEscapedEndTagOpen, context)` with the cursor
positioned after the already-consumed `</` and an empty temporary buffer as
specified for the predecessor's `/` branch. No pending tag or selector is
required: ASCII alpha creates/selects the end tag. For robustness, seeded tests
also prove that this state itself preserves a nonempty original-spelling buffer.
Direct input is the suffix, not another `</`. Prefix positioning in local tests
consumes cursor input only, not a simulated predecessor algorithm.

The alpha destination `ScriptDataEscapedEndTagName` now executes: it accumulates
an ASCII-normalized candidate name and original-spelling scratch at the same
reconsumed position. Appropriate delimiters enter existing attribute/tag support
or emit into Data; mismatches/EOF recover literally into Escaped. The original
letter remains current and reconsumption is scheduled at the source boundary,
before any name append. Fallback executes the already implemented Escaped state. Its `<` still reaches terminal EscapedLessThanSign. No execution
that reaches a stub is reconstructed/resumed. There is no tree builder, automatic
element-mode switching, Comment-token behavior, or JavaScript parsing.

## Retrieved specification evidence

Retrieved and inspected the immutable WHATWG HTML
[raw source at revision `404b3eb14a7a936dddefb2d78e224dbeb01d2358`](https://raw.githubusercontent.com/whatwg/html/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source),
not another engine or recollection:

- [Assigned state, lines 143895–143911](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143895-L143911),
  anchor `#script-data-escaped-end-tag-open-state`.
- [Preprocessing, current/next input and conceptual EOF, 142560–142607](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142607):
  original surrogate/noncharacter/control diagnostics, deferred NUL handling and
  newline normalization. The linked Infra
  [normalize newlines](https://infra.spec.whatwg.org/#normalize-newlines)
  definition was inspected: replace CRLF with LF, then remaining CR with LF.
- [Tokenization, reconsumption, temporary buffer, tag creation defaults and
  creation versus emission, 143149–143189](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143149-L143189).
- [Imported ASCII definitions, 2297–2300](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L2297-L2300),
  and the linked Infra [ASCII alpha](https://infra.spec.whatwg.org/#ascii-alpha)
  and upper/lower definitions: exactly U+0041–U+005A and U+0061–U+007A.
- [Predecessor, 143872–143892](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143872-L143892),
  [Escaped fallback destination, 143775–143802](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143775-L143802),
  and [name destination, 143914–143954](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143914-L143954).
  Name normalization/buffer accumulation belong to that later state, not this one.
- [After attribute value (quoted), 144402–144432](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144402-L144432):
  closing a quoted value then reaching EOF reports `eof-in-tag` in that state,
  without an extra transition to After attribute name.
- Destination diagnostic definitions:
  [unexpected-null-character, 141479–141482](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L141479-L141482),
  [eof-in-script-html-comment-like-text, 141138–141149](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L141138-L141149).

## Complete source-branch mapping

Focused tests: `crates/html/tests/script_data_escaped_end_tag_open_state.rs`.

| Input / pinned branch | Implementation effects | Focused test evidence |
| --- | --- | --- |
| ASCII alpha, 143901–143903 | Replace only pending end tag with empty name, unset flag, empty attributes/no detached attribute; select End; reconsume in EscapedEndTagName. No append, lowercase, emission or diagnostic. | `all_ascii_letters_create_empty_selected_end_tag_and_reconsume_without_normalizing`: all 52 letters, absent/stale candidate, each selector, open/closed input; exact creation defaults before real name append, literal mismatch recovery and escaped EOF snapshots. |
| Anything else, 143905–143908 | Emit `<`, `/` in order; reconsume in Escaped. No tag creation/abandonment, buffer mutation or source error. | `all_non_alpha_ascii_and_non_scalars_fall_back_then_execute_real_escaped_step`: all other ASCII, Unicode letters/whitespace/scalars, actual surrogates/noncharacters/controls, CR normalization, exact source/destination snapshots and unread tail at the `<` stub. |
| Conceptual EOF (included in anything else) | Emit only `</`; reconsume EOF, without EOF token/error here. | `open_exhaustion_is_not_eof_and_eof_is_reconsumed_before_destination_error`: empty/prefixed/CRLF positions; only the real Escaped step emits EOF and one escaped EOF error at the same position. |
| Open exhaustion (incremental contract, not a spec branch) | Effect-free NeedMoreInput, retaining source/current input. | Repeated suspension/empty chunks; `fresh_direct_entry_can_resume_suspension_with_either_branch` checks fresh default context resumption with alpha, punctuation and NUL. |

Snapshots include tokens, both diagnostic vectors, normalized current item and
position, state, selector, both pending tags/flags/retained and detached attributes,
pending comment, last start tag, return state and temporary buffer. Existing
output/error prefixes and unrelated context remain unchanged. Reconsumption is
explicitly asserted (including EOF); the next real Escaped step sees the same
current point, not the tail. Preprocessing errors occur once on first consumption.
NUL is not replaced/diagnosed at this source; Escaped emits U+FFFD and its error.
`repeated_nul_and_non_scalar_after_crlf_have_exact_once_only_diagnostics` fixes
exact expected positions/multiplicities across CRLF, two NULs, a surrogate and EOF.
Repeated terminal calls never duplicate output/errors or consume a tail.

`real_script_entry_stops_at_unimplemented_predecessor_without_consuming_solidus`
checks every real step of `<!--</A>` from Script data through EscapeStart,
EscapeStartDash and EscapedDashDash to the predecessor stub. It cannot supply
real entry to this assigned state yet; no manual restart is substituted.

`every_two_chunk_split_and_single_point_chunking_matches_closed_source_trace`
compares every productive source-step snapshot and terminal result for all
splits (including empty chunks) and single-code-point chunks. Cases cover both
branches, EOF, malformed text, literal ampersands, NUL/non-scalars, CRLF,
consecutive/trailing CR, a preconsumed CR suppressing a following LF, hyphen cycles,
`-->` returning to Script data and later terminal stubs. Feeding stops at the
first terminal result; unread tails beyond a stub are not execution coverage.
Streaming now additionally traverses real appropriate/mismatching mixed-case names,
attributes, duplicate removal, attribute NUL, malformed solidus, name/tag EOF,
CRLF delimiters and later CharacterReference/MarkupDeclarationOpen blocks.
Each trace is repeated with absent, matching and case-distinct history and with
both proper empty and deliberately stale scratch, preserving all old comparisons.
The ten local open-state tests and inventory are not upstream conformance cases.

## Validation and upstream categories

All checks passed (exit 0):

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo test --workspace --locked pinned_upstream_runner -- --nocapture
```

Current integrated workspace: **364 passed, 0 failed, 0 ignored**, including
**ten focused open-state tests** and all **15 merged name-state tests**.
The standalone report was 291/seven; it is historical.
The fixture submodule was already initialized at pinned
`c777c408b61078ea2eb4acefc2535f54dbc8b28a`; no initialization/pin change was needed.

Upstream report: **6,827 fixture cases; 7,053 runs; 6,190 dispatch-attempted;
1,126 executed; 1,126 passed; 0 failed; 5,927 blocked**.
Executed = passed + failed; blocked dispatch attempts are not comparable passes.
Counts match current merged main (and the reviewed main-after-PR-36 report):
zero newly passing identities.
All 930 previously protected identities and baseline registrations are preserved.
No empty/new baseline is added, because no independently passing new identity
exists; the runner has no assigned-state initial entry and every real path to it
hits the predecessor stub first.

Blocked reasons: CharacterReference 4,692; MarkupDeclarationOpen 242;
ScriptDataEscapedLessThanSign 22; CdataSection 56; ProcessingInstructionOpen 52;
TokenVariants 858; XmlCoercion 4; PendingSpecReview 1.

Independently inspected unchanged `domjs.test` tests[16], Script data initial
index 0: `lastStartTag: script`, input `<!-- </script> --></script>`, expected
Character `<!-- `, EndTag `script`, Character ` -->`, EndTag `script`, no errors.
The pinned predecessor `/` branch would reach this state's alpha creation and
then the name state, but today it blocks at escaped less-than sign before `/`
is consumed. `domjs.test` tests[20–23] include incomplete/unclosed tag spellings
inside escaped/double-escaped text; they likewise reach the same earlier stub,
not a comparable fallback prefix. Neither these nor other blocked cases are
added to the baseline. Partial results imply neither full conformance nor
browser security.

## Later integration boundaries

This PR now inherits the seven reviewed merged implementations from main; no
unmerged sibling was fetched/cherry-picked/merged. In particular #45 is not
imported. EscapedLessThanSign is the one remaining Script data gateway stub;
implementing it will enable normal entry and may admit new upstream identities.
The name destination is real, so obsolete terminal-name assertions have been
replaced with actual execution while retaining every original creation/default,
context-preservation and reconsumption source assertion. No comparisons, pins,
fixtures or baseline registrations were weakened/changed.

All CONTRIBUTING checks and the pinned runner were rerun on this integrated tree.
The coordinator owns independent final-head review and any merge. This thread
neither merges/enables auto-merge nor spawns. Later gateway integration must review
moved terminal boundaries and independently inspect newly comparable fixtures
before adding state-specific baselines; never dynamically detect support, skip
blocks or reconstruct/resume terminal executions. No shared prerequisite or local
validation blocker was found.
