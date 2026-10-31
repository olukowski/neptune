# Script data escaped end tag name

## Current integration after merged #43, #38, #39, #40, #42 and #37

PR #44 is rebased onto exact main **`fccf89c1e1d40d345acf16c3b7731801b245bdee`**:
DoubleEscapeEnd (#43), DoubleEscapedLessThanSign (#38), DoubleEscapedDashDash (#39),
DoubleEscapedDash (#40), DoubleEscaped (#42), and DoubleEscapeStart (#37) are all
inherited unchanged. Only origin/main was fetched; unmerged EscapedEndTagOpen and
EscapedLessThanSign were not imported and remain genuine terminal stubs. Module,
dispatch, independent inventory and current scope docs were resolved additively.
The assigned production algorithm is unchanged from standalone head
`e12564e2d8b628c1e18e2cf10c3b6f1f4c60c498`; that head and the 297-test standalone
report below are historical. The exact final review head is recorded in
[PR #44](https://github.com/olukowski/neptune/pull/44)'s body.

All 13 original tests still pass without weakening any source-step, appropriate
end-tag, original-spelling, NUL, EOF or reconsumption assertion. Their actual
Escaped destinations already executed on the original base; no obsolete
DoubleEscaped stub assertion existed in this suite. Two additional tests make
the current integrated boundaries explicit:

- `literal_fallback_follows_real_escaped_return_and_ordinary_closing_tag_into_data`
  inspects assigned fallback before reconsumption, then real Escaped → Dash →
  DashDash → ScriptData source snapshots. A later ordinary mixed-case closing tag
  emits through the existing tag path and selects Data, whose following NUL is
  emitted **unchanged** with its exact diagnostic at 1:14. EOF at 1:15 has no
  comment-like EOF error. Literal fallback does not prematurely abandon storage.
- `merged_double_escaped_loop_returns_to_real_escaped_predecessor_and_blocks`
  starts explicitly at DoubleEscapeStart (not a reconstructed blocked run),
  snapshots `ScRiPt>--a</ScRiPt><` through all six inherited algorithms, then stops
  at the genuine EscapedLessThanSign gateway. Normalized scratch versus original
  emission, slash-only scratch clearing, preserved pending End name, no duplicate
  `<` emission and exact untouched-tail/terminal behavior remain explicit. There
  is still **no real path into the assigned name state** through that stub.

Every two-chunk split and single-point chunking now also covers the complete
fallback/ordinary-close path and the inherited double-escaped loop, including
CRLF after the matched double-escape end delimiter. Source evidence for the
merged loop was reread at pinned [lines 143958–144142](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143958-L144142).
No terminal execution is resumed, no output prefix is accepted as a pass, and no
dynamic support detection is used. No other state implementation or suite changed.

All CONTRIBUTING checks plus the explicit pinned runner passed (exit 0): fmt,
warning-denying Clippy, workspace tests, warning-denying rustdoc and upstream
reporting. **354 Rust tests passed, 0 failed, 0 ignored**, including **15 assigned
state-local tests**. Local/inventory tests are not upstream conformance runs.

Current pinned upstream results: **6,827 fixture cases; 7,053 runs; 6,190
dispatch-attempted; 1,126 executed = 1,126 passed + 0 failed; 5,927 blocked**.
Blocked categories remain CharacterReference 4,692; TokenVariants 858;
MarkupDeclarationOpen 242; CdataSection 56; ProcessingInstructionOpen 52;
ScriptDataEscapedLessThanSign 22; XmlCoercion 4; PendingSpecReview 1.
New passes versus current merged main: **0**. All 930 protected identities,
baseline files/registration, fixtures, pins, adapter and comparison policy are
unchanged. No validation/shared-prerequisite blocker remains. The coordinator
owns independent review of the exact final head and any subsequent merge; this
thread neither merges nor spawns. Partial support is not full conformance or a
browser-security claim.

## Scope and immutable evidence

Implements exactly `State::ScriptDataEscapedEndTagName`. The original standalone
base was `39d772e31b98e6e955a2e00f546a8db3506eb1dd` (verified before editing);
current merged-base integration is described above. No dependency, fixture, pin,
adapter, storage/API redesign or shared tag refactor is included.

The complete HTML source was retrieved from the immutable raw GitHub URL at
revision `404b3eb14a7a936dddefb2d78e224dbeb01d2358`, then the assigned section and
referenced definitions were read:

- [Assigned algorithm, lines 143913–143956](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143913-L143956).
- [Appropriate end tag, lines 143213–143215](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143213-L143215):
  exact candidate-name equality with the last **emitted** start tag, if any.
  Neither original scratch spelling nor unrelated pending start-tag storage is history.
- [Reconsumption and temporary buffer, lines 143149–143168](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143149-L143168).
- [Token creation/emission and end-tag errors, lines 143174–143210](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143174-L143210):
  shared selected-tag emission retains attributes/flag, excludes detached duplicates,
  and independently reports end-tag attributes and trailing solidus.
- [Preprocessing, current/next input and conceptual EOF, lines 142560–142607](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142607).
  CR/CRLF normalization and one-time control/noncharacter/surrogate diagnostics
  remain in the existing cursor. NUL handling belongs to tokenizer states.
- [Predecessor setup, lines 143872–143910](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143872-L143910):
  escaped less-than sign clears scratch on `/`; escaped end tag open creates an
  empty end tag and reconsumes the first ASCII letter. **Both remain stubs here.**
- [Fallback destination, lines 143775–143802](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143775-L143802):
  Script data escaped handles reconsumed NUL and EOF; EOF reports
  `eof-in-script-html-comment-like-text`, not `eof-in-tag`.

The pinned source imports Infra's [ASCII upper alpha](https://infra.spec.whatwg.org/#ascii-upper-alpha),
[ASCII lower alpha](https://infra.spec.whatwg.org/#ascii-lower-alpha), and
[normalize newlines](https://infra.spec.whatwg.org/#normalize-newlines) definitions,
which were also read: U+0041–U+005A and U+0061–U+007A, not Unicode case folding;
CRLF becomes LF, then remaining CR becomes LF. No living HTML algorithm or other
engine is the implementation baseline.

## Entry and branch mapping

Algorithm: `crates/html/src/tokenizer/script_data_escaped_end_tag_name.rs`.
Focused evidence: `crates/html/tests/script_data_escaped_end_tag_name_state.rs`.
The independent support inventory adds only this state and its direct-entry setup.

Direct entry requires `current_tag = Some(TagKind::End)`, matching initialized
`pending_end_tag`, and original-spelling `temporary_buffer`, as the predecessor
would establish. `last_start_tag` is optional. Tests seed this context explicitly;
they never reconstruct or resume a terminal execution. Name accumulation creates
no token and resets no storage.

| Pinned branch | Code/source effects | Focused tests |
| --- | --- | --- |
| TAB, LF, FF, SPACE | If appropriate, consume delimiter and select Before attribute name; otherwise literal fallback | `each_appropriate_delimiter_consumes_once_and_only_greater_than_emits`; `appropriate_means_exact_candidate_name_not_scratch_or_unemitted_storage` |
| `/` | If appropriate, consume and select Self-closing start tag; otherwise fallback | Same delimiter tests, plus `existing_destinations_complete_attributes_flags_malformed_recovery_and_eof` |
| `>` | If appropriate, emit through existing selected-tag helper and select Data; otherwise fallback | Delimiter tests; `emission_uses_selected_tag_helper_including_both_independent_errors` |
| ASCII upper alpha | Append point + 0x20 to candidate name, original point to scratch; stay | `initialized_end_and_original_spelling_are_required_entry_context_not_creation` exhausts all 26 uppercase letters |
| ASCII lower alpha | Append unchanged to both; stay | Same test exhausts all 26 lowercase letters; scheduled-first-letter entry is tested separately |
| Anything else, including inappropriate delimiters and EOF | Emit `<`, `/`, and scratch in insertion order; preserve candidate/scratch; reconsume in Script data escaped | `exhaustive_nonletters_fall_back_before_escaped_reconsumption_and_diagnostics`; `scratch_is_emitted_in_order_without_preprocessing_and_without_including_mismatch`; `open_input_suspends_but_explicit_eof_recovers_literally_even_for_matching_name` |

No fallback appends the mismatching point, clears scratch, abandons the selected
tag, replaces NUL, emits EOF, or diagnoses `eof-in-tag`. Those source effects are
checked **before** the destination step. Original scratch contents (even seeded
CR, NUL, surrogate and noncharacter points) emit literally without preprocessing.
Reconsumption includes conceptual EOF and does not duplicate input diagnostics.
Open exhausted input repeatedly returns effect-free `NeedMoreInput`; only
explicit finish supplies EOF. Escaped EOF adds exactly one diagnostic and EOF,
while a successful closing `>` selects Data, whose EOF has no escaped-EOF error.

The original 13 local tests inspect all ASCII branches and alpha boundaries, non-ASCII
letters/whitespace, actual non-scalars/noncharacters/controls, exact diagnostic
positions/multiplicity, selected and unrelated storage, token/error prefixes,
retained/detached attributes and flags, scheduled reconsumption and untouched
tails. Existing attribute destinations execute normally, including duplicate
removal, quoted values/NUL, malformed solidus recovery, and destination EOF
abandonment. Shared emission is checked both with retained attributes and with
only a detached duplicate, including coexistence of independent emission errors.

Every two-chunk split and single-code-point chunking compares complete productive
source-step traces, including CRLF splits, consecutive/trailing CR, mixed spelling,
matching/absent/case-distinct history, mismatches, NUL, non-scalars, attributes,
EOF and terminal blocks. Separate real Script data and escaped-entry traces,
including chunk splits, stop at the actual escaped-less-than-sign predecessor.
There is no supported real path **into** this state on the current rebased base.
Later Character reference, Markup declaration open and escaped less-than sign
blocks remain terminal typed `UnimplementedState`, never prefix success or EOF.

## Upstream evidence and validation

The fixture checkout remains at `c777c408b61078ea2eb4acefc2535f54dbc8b28a`.
The pinned `domjs.test` case `</script> in script HTML comment` was inspected:
`<!-- </script> --></script>` with Script data initial state and history `script`
expects Character `<!-- `, EndTag `script`, Character ` -->`, EndTag `script`,
without errors. It still blocks at escaped less-than sign before this state.
The neighboring double-escaped closing-tag cases also need the missing gateway
predecessors. The six double-escape algorithms now exist on merged main, but the
gateway still blocks ordinary entry. No output prefix is compared or blessed.

The original standalone pinned runner results below remain numerically unchanged
on the current merged base:

- 6,827 fixture cases; 7,053 runs; 6,190 dispatch-attempted.
- **Executed 1,126 = passed 1,126 + failed 0; blocked 5,927.**
- Blocked: CharacterReference 4,692; TokenVariants 858; MarkupDeclarationOpen 242;
  CdataSection 56; ProcessingInstructionOpen 52; ScriptDataEscapedLessThanSign 22;
  XmlCoercion 4; PendingSpecReview 1.
- **Zero new upstream passes** versus the reviewed base. No state-specific pass
  baseline is added; all prior baseline files/registrations remain unchanged.

Local tests and inventory are not conformance runs. This is partial standalone
tokenization, not full HTML conformance or browser-security evidence. There is no
tree builder, automatic element-mode switching, JavaScript parser or invented
Comment-token behavior.

Historical standalone validation: all CONTRIBUTING checks and the explicit
pinned report passed locally (exit 0). The workspace suite then passed
**297 Rust tests, 0 failed, 0 ignored**, including
**13 state-local tests**. These counts are separate from upstream run results.
The already initialized fixture submodule required no checkout changes.

Executed commands:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo test --workspace --locked pinned_upstream_runner -- --nocapture
```

## Later integration boundaries

No shared prerequisite is missing. The operational entry block is intentionally
unimplemented predecessors, not an invitation to implement neighbors. The
user has now authorized sequential integration with independent final-head review.
The coordinator owns that review and merge, not this implementation thread.
Module/dispatch and independent support-inventory edits have combined additively.
When predecessor PRs merge, deliberately review the obsolete predecessor-block
assertions and add real entry coverage, rather than weakening comparisons or
resuming a previously blocked run. Rerun every check and the pinned report, and
independently review any newly comparable fixture identities before adding a
state-local baseline. This PR is for user review only: no merge or auto-merge.
