# RCDATA end tag name state

This change implements **only `State::RcdataEndTagName`**. After integration
with main `c72c7e5014d0b61d4759e06823b5f21941725375` (PRs #26, #25, #24, #23),
its two RCDATA predecessors and both RAWTEXT predecessors are preserved from
main. The full RCDATA closing-tag/fallback chain is reachable. RAWTEXT end tag
name and Character reference remain terminal stubs. There is no automatic mode
selection or tree-builder behavior.

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
attributes, and no detached duplicate; isolated tests initialize these defaults,
and integrated tests inspect the real predecessor's creation before name accumulation. The normal initial buffer is empty;
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

Fallback `&` is reconsumed by RCDATA before execution stops terminally at
Character reference. Fallback `<` now reaches the implemented less-than-sign
state, so repeated partial candidates can recover and close a later tag.
No output prefix is treated as success. Tests retain actual remaining terminal
blocks (including after additional input is fed) and never reconstruct/resume
blocked execution. RAWTEXT end tag name and Character reference have explicit
effect-free terminal-stub checks.

`real_rcdata_entry_preserves_each_source_boundary_and_returns_to_data` inspects
RCDATA `<`, less-than sign `/` (only buffer clearing), end tag open's empty token
creation/reconsumption, each original/normalized name letter, emission, and Data's
subsequent **literal** NUL output at normalized positions. The sibling source-state
tests retain their snapshots and now inspect actual name accumulation, fallback,
and completion instead of obsolete terminal-name assertions.
`real_rcdata_closing_tag_and_literal_fallback_paths_complete_exactly` checks
matching/mismatching/absent context, repeated partial candidates, non-ASCII/NUL
fallback, attributes, flag diagnostics, and both EOF outcomes.
`real_rcdata_entry_all_splits_and_single_point_chunks_preserve_every_step` adds
complete RCDATA-entry traces across all two-chunk splits and single-point chunks,
including malformed input, CRLF/non-scalars, recovery, and real later stubs.
`explicit_modes_do_not_switch_automatically_and_remaining_destinations_stay_terminal`
checks that `<title>`/`<textarea>` start tags do not choose RCDATA, an RCDATA tag-like
start does not emit a start token, and a matching end tag explicitly switches to
Data. New Data start-tag emissions update `last_start_tag` normally. Character
reference return states differ before/after closing, while RAWTEXT end tag name
and Markup declaration open continue to stop terminally.

## Integrated fixture review

The original isolated PR started at `bcae7e5c916470319ef0838dc72dfd71a95b8181`
and added zero upstream passes because its predecessors were stubs. Integration
with main `c72c7e5014d0b61d4759e06823b5f21941725375` preserves all four merged
siblings and makes the state reachable from the existing RCDATA initial state.
No initial-state vocabulary, capability/policy exclusions, comparison logic,
fixture expectation, or pin is changed.

All 18 previously name-blocked RCDATA runs were examined directly in the
unchanged fixtures, independently of deriving expectations from tokenizer output.
The following **17** manually selected pass identities are added in
`crates/html/tests/baselines/rcdata_end_tag_name.json`, all `tests` group,
`RCDATA state`, initial-state index **0**, with last start tag **`xmp`**.
`C(text)` denotes unchanged Character output; `E(name)` denotes an EndTag.
No errors are expected except the two explicitly listed EOF diagnostics.

| File / case | Unchanged input → expected output | Pinned branch reasoning |
| --- | --- | --- |
| `contentModelFlags.test` 2 | `foo</xmp>` → C(`foo`), E(`xmp`) | Appropriate `>` emits; switch to Data. |
| same, 3 | `foo</xMp>` → C(`foo`), E(`xmp`) | ASCII `M` normalizes in name, stays original in buffer; `>` matches. |
| same, 4 | `foo</xmp ` → C(`foo`); `eof-in-tag` at (1,10) | Appropriate SPACE hands off; destination EOF abandons tag, not literal fallback. |
| same, 5 | `foo</xmp` → C(`foo</xmp`) | Even matching name falls back literally at name-state EOF; no `eof-in-tag`. |
| same, 6 | `foo</xmp/` → C(`foo`); `eof-in-tag` at (1,10) | Appropriate `/` hands off; Self-closing start tag EOF abandons, without setting flag. |
| same, 7 | `foo</xmp<` → C(`foo</xmp<`) | `<` triggers name fallback, reconsumed RCDATA then less-than sign; EOF emits delayed `<` then reconsumed EOF. |
| same, 8 | `</foo>bar</xmp>` → C(`</foo>bar`), E(`xmp`) | Mismatching `>` is reconsumed literally; later slash clears buffer before real candidate. |
| same, 9 | `</xmp</xmp</xmp>` → C(`</xmp</xmp`), E(`xmp`) | Both partial names fall back on `<`; final appropriate `>` emits. |
| same, 10 | `</foo>bar</xmpaar>` → C(`</foo>bar</xmpaar>`) | Prefix equality is insufficient; both nonmatching delimiters recover literally. |
| same, 11 | `foo</xmp></baz>` → C(`foo`), E(`xmp`), E(`baz`) | Closing `xmp` switches to Data; ordinary Data end tags need not match last start tag. |
| `domjs.test` 27 | `</XMP>` → E(`xmp`) | ASCII-only normalization permits matching `>`. |
| same, 29 | `</xm>` → C(`</xm>`) | Shorter name is inappropriate; literal `>` reconsumption. |
| same, 30 | `</xm ` → C(`</xm `) | Inappropriate SPACE falls back; subsequent RCDATA EOF is not tag abandonment. |
| same, 31 | `</xm/` → C(`</xm/`) | Inappropriate `/` is literal, not a flag handoff. |
| `escapeFlag.test` 0 | `foo<!--</xmp>--></xmp>` → C(`foo<!--`), E(`xmp`), C(`-->`), E(`xmp`) | No comment mode is entered in RCDATA; after first close Data handles the second end tag normally. |
| same, 1 | `foo<!-->baz</xmp>` → C(`foo<!-->baz`), E(`xmp`) | Literal RCDATA markup-like prefix followed by appropriate close. |
| same, 4 | `foo<!-- x --x>x-- >x--!>x--<></xmp>` → C(`foo<!-- x --x>x-- >x--!>x--<>`), E(`xmp`) | Every non-solidus less-than fallback stays literal; final name matches. |

The eighteenth affected run, `escapeFlag.test` **2**, input
`foo<!--></xmp><!-->baz</xmp>`, now emits its first closing tag and switches to
Data, then blocks at **Markup declaration open** on the second `<!`. Its fixture
expects a comment and `abrupt-closing-of-empty-comment` at (1,19), but that state
chain is not implemented; this run is **not** comparable or a pass and is not
added. `escapeFlag.test` 3 still blocks earlier at Character reference. The
corresponding 18 RAWTEXT initial-state-index-1 runs still block at RAWTEXT end tag
name. No stub is reconstructed/resumed to inspect hypothetical continuation.

All **869 prior identities**, baseline files, and existing registry entries are
preserved. With these 17 additions, the protected union is **886** identities.
There is no automatic pass capture. New runner totals versus integrated main:

- 6,827 fixture cases; 7,053 runs; 6,190 dispatch-attempted.
- **1,042 executed = 1,042 passed + 0 failed; 6,011 blocked**.
- Main before this state: 1,025 executed/passed, 0 failed, 6,028 blocked.
- Remaining reasons: CharacterReference 4,692; TokenVariants 858;
  MarkupDeclarationOpen 241; ScriptData 89; CdataSection 56;
  ProcessingInstructionOpen 52; RawtextEndTagName 18; XmlCoercion 4;
  PendingSpecReview 1.

Local branch/streaming, adapter, and inventory tests are not upstream conformance
executions. This remains partial standalone tokenization, not full conformance
or a browser-security claim. Use the [runner command/count definitions](tokenizer.md#fixture-runner).

## Integration caveats

Production sibling algorithms, source branch evidence, upstream fixtures,
registry ordering, and pins are preserved. Only obsolete destination expectations
are migrated to real name-state effects; actual later stubs retain explicit
terminal assertions. All CONTRIBUTING checks and the pinned runner are rerun.
Module/dispatch, support inventory, and baseline-list edits remain narrow and
additive. Future sibling merges require reviewing moved terminal blocks and
rerunning checks; never hide newly comparable failures behind adapter exclusions.
No RAWTEXT end tag name, Character reference, automatic mode switching, tree
builder, dependency, token variant, or shared API rewrite is added.
