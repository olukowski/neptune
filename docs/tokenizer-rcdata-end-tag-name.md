# RCDATA end tag name state

This change implements **only `State::RcdataEndTagName`**. Its two RCDATA
predecessors and all three corresponding RAWTEXT states remain terminal stubs.
There is no automatic mode selection or tree-builder behavior.

## Immutable source and branch evidence

Normative source: WHATWG HTML revision
[`404b3eb14a7a936dddefb2d78e224dbeb01d2358`](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source),
as pinned by `docs/sources.toml`:

- [RCDATA end tag name, lines 143524–143565](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143524-L143565).
- [Appropriate end tag definition, lines 143212–143216](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143212-L143216):
  exact candidate-name equality with the last **emitted** start-tag name; no
  emitted start tag means no appropriate end tag. Do not normalize this context again.
- [Tokenization, lines 143151–143210](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143151-L143210):
  reconsumption, scratch storage, creation versus emission, creation defaults,
  and the two independent end-tag emission errors.
- [Preprocessing and current/next input, lines 142560–142595](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142595):
  code-point diagnostics and newline normalization precede this state.

Production code is state-local in `crates/html/src/tokenizer/rcdata_end_tag_name.rs`;
branch tests are in `crates/html/tests/rcdata_end_tag_name_state.rs`.

| Pinned branch | Source effects | Focused evidence |
| --- | --- | --- |
| TAB, LF, FF, SPACE; appropriate | Consume separator; switch to Before attribute name. No name/buffer append, emission, attribute creation, or reconsumption. CR is already LF. | `every_delimiter_checks_exact_appropriateness_and_consumes_or_reconsumes`, `matching_delimiters_hand_off_to_existing_tag_support_not_rcdata` |
| `/`; appropriate | Consume; switch to Self-closing start tag without setting the flag. | Same delimiter tests; flag/emission effects belong to the destination. |
| `>`; appropriate | Consume; emit through existing selected-tag support; switch to Data. | Same delimiter tests, including retained attributes/flag, detached duplicate exclusion, independent emission diagnostics, and untouched unrelated storage. |
| Any of those delimiters; inappropriate | Use the literal fallback, including reconsumption of the delimiter. | Delimiter matrix covers absent, prefix, longer, case-different, and matching last-start-tag context; seeded names separately check exact code-point equality, including non-scalars and Unicode normalization distinctions. |
| ASCII upper alpha | Append point + `0x20` to candidate name; append **original** point to temporary buffer; stay here. | `all_ascii_letters_normalize_only_the_name_and_append_original_spelling`, `accumulation_preserves_seeded_non_scalars_attributes_flags_and_detached_storage` |
| ASCII lower alpha | Append original point to both name and buffer; stay here. | Same letter tests, all 52 ASCII letters. No appropriateness check on an unfinished name. |
| Anything else, including EOF | Emit `<`, `/`, then one character token per buffer point, in order. Reconsume current input in RCDATA. Do not append it to the name/buffer, clear scratch storage, take the pending tag, diagnose, or emit EOF here. | `anything_else_is_literal_even_when_candidate_matches_and_diagnostics_belong_to_destination`, `empty_open_input_and_partial_name_suspend_but_eof_falls_back_then_is_reconsumed` |

Open exhaustion is `NeedMoreInput`, not specification EOF: repeated suspension
preserves every field and the previous current input. On explicit EOF the source
performs literal fallback; RCDATA subsequently reconsumes conceptual EOF and emits
one terminal EOF, with no `eof-in-tag`. Conversely, EOF **after** an appropriate
whitespace or `/` handoff belongs to the destination tag states, which abandon
the selected tag and report `eof-in-tag`:
`delimiter_handoff_eof_is_destination_abandonment_not_source_fallback`.

## Entry, storage, and diagnostics

Isolated entry supplies `current_tag = Some(TagKind::End)`, an initialized
`pending_end_tag`, and the corresponding original-spelling `temporary_buffer`.
The normal predecessor creates an empty-name end tag with an unset flag, empty
attributes, and no detached duplicate; tests initialize and assert these defaults
without implementing that predecessor. The normal initial buffer is empty;
partial-name tests seed it explicitly. This state neither creates a token nor
clears the buffer. Existing name/buffer code points are never preprocessed again.

Only ASCII letters accumulate. Non-ASCII letters, digits, NUL, vertical tab,
NBSP, punctuation, surrogates, and noncharacters all use literal fallback even
when the candidate already matches. Cursor diagnostics occur on first consumption
only. In particular, NUL replacement and `unexpected-null-character` belong to
RCDATA's next step, not this source. The tests inspect these steps separately,
including exact diagnostics and normalized positions. `normalized_positions_and_no_duplicate_preprocessing_on_reconsumption`
and `matching_tag_emission_errors_use_normalized_destination_position` independently
assert CRLF handling and destination emission-error locations.

Accumulation and fallback preserve pending-tag flags/attributes/detached storage,
the selector, unrelated pending start tag/comment, last-start-tag context, return
state, and existing output/diagnostics. Successful emission takes only the selected
end tag, keeps the temporary buffer, and never updates `last_start_tag`.

## Incremental and blocked execution evidence

`every_two_chunk_split_and_single_point_chunking_preserve_source_step_traces`
compares every non-suspension source step against closed-input execution for all
two-chunk splits and single-code-point chunking, with and without last-start-tag
context. Samples include partial/exact/mismatching names, each delimiter path,
attributes and malformed solidus recovery, EOF after delimiters, CRLF splits,
consecutive/trailing CR, NUL, non-scalars, preprocessing errors, and later stubs.
It checks repeated empty feeds/suspension and repeated terminal results.

Fallback `&` and `<` are reconsumed by RCDATA before execution stops terminally
at Character reference or RCDATA less-than sign. No output prefix is treated as
success. Tests preserve these explicit blocks, including after additional input
is fed, and never reconstruct/resume a blocked execution. A separate test checks
all five neighboring wave states are still effect-free terminal stubs in this PR.
Those assertions must be reviewed when integrating their separate PRs.

## Upstream scope and integration

The pinned upstream runner cannot enter this state directly: its initial-state
vocabulary is unchanged, and RCDATA less-than sign/end tag open remain stubs.
Consequently this state adds **zero new upstream passes**; no pass-baseline file,
fixture, expectation, or pin is modified. Existing case-level pass protections
remain in force. Local branch/streaming and support-inventory checks are not
upstream conformance executions. Use the runner's separate executed, passed,
failed, and blocked counts in the PR report; no full-conformance claim is made.

This PR starts from `bcae7e5c916470319ef0838dc72dfd71a95b8181`. Module/dispatch,
independent support-inventory, and status-documentation edits are deliberately
small but can conflict with sibling PRs. Rebase after neighboring merges, review
changed terminal-block assertions, and rerun every contributing check plus the
upstream runner; do not hide newly reachable execution behind adapter exclusions.
