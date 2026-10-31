# RAWTEXT end tag name state

## Pinned evidence and scope

This PR implements only `State::RawtextEndTagName`, following WHATWG HTML
revision `404b3eb14a7a936dddefb2d78e224dbeb01d2358`:

- [Assigned algorithm, source lines 143608–143651](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143608-L143651).
- [Current/next input, preprocessing, conceptual EOF, lines 142560–142610](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142610).
- [Reconsumption, scratch storage, creation and emission, lines 143150–143217](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143150-L143217).
  In particular, appropriate means exact equality with the name of the last
  **emitted** start tag; no previous start tag means no appropriate end tag.

The algorithm is in `crates/html/src/tokenizer/rawtext_end_tag_name.rs` and its
focused evidence is in `crates/html/tests/rawtext_end_tag_name_state.rs`.
No adjacent state, shared storage/API, fixture adapter, dependency, token variant,
tree-builder behavior, or automatic mode switching is added.

## Entry and branch map

Isolated entry requires `current_tag = Some(TagKind::End)`, an initialized
`pending_end_tag` with the normalized candidate name so far, and
`temporary_buffer` containing its original spelling. Empty candidate/buffer
storage models entry immediately before the first reconsumed letter; seeded
prefixes model subsequent steps. The optional `last_start_tag` is supplied
unchanged. This is not reconstruction or resumption after a terminal stub.

| Pinned branch | Source effects and focused evidence |
| --- | --- |
| ASCII upper alpha (143635–143638) | Append code point + 0x20 to candidate; append the original to scratch. All 26 uppercase letters and seeded name/storage preservation are asserted. |
| ASCII lower alpha (143640–143643) | Append unchanged to both stores. All 26 lowercase letters are asserted. No Unicode folding or non-ASCII name accumulation. |
| TAB, LF, FF, SPACE (143616–143623) | If appropriate, consume and switch to Before attribute name; otherwise literal fallback. CR arrives as LF. Every delimiter is checked with matching, absent, case-distinct, prefix/suffix, Unicode-distinct and unrelated last-start names. |
| `/` (143625–143628) | If appropriate, consume and switch to Self-closing start tag; no flag mutation or diagnostic in this source. Otherwise literal fallback. |
| `>` (143630–143633) | If appropriate, consume, switch to Data and use existing selected-tag emission. Ordered retained attributes/flag and both emission diagnostics are checked, including normalized locations and detached-duplicate exclusion. Otherwise literal fallback. |
| Anything else (143645–143650) | Emit `<`, `/`, then scratch points in order; reconsume in RAWTEXT. Exhaustive non-letter ASCII, Unicode letters/whitespace, actual surrogates, noncharacters and controls are covered. NUL is not replaced or diagnosed here; RAWTEXT does that next step. Scratch and the un-emitted candidate remain inspectable and unchanged. |
| Conceptual EOF | Also anything else, even for an appropriate candidate. Source emits only the literal fallback and schedules EOF reconsumption; RAWTEXT alone emits terminal EOF on the next step, without `eof-in-tag`. |
| Open exhausted input | Repeated `NeedMoreInput` without effects; never EOF, literal fallback, candidate emission, or an appropriateness decision. Feeding can continue until explicit finish. |

Tests separately inspect source snapshots before destination processing, cursor
consumption/reconsumption (including EOF), exact diagnostics and normalized
positions, pending-tag defaults, unrelated pending start/comment and return-state
storage, original buffer contents (including seeded non-scalars), and unchanged
last-start bookkeeping. Successful emission takes only the selected end tag.
Destination attribute parsing, self-closing errors, and destination EOF abandonment
are tested separately from the assigned algorithm.

All two-chunk splits and single-code-point chunking of representative names,
appropriate/inappropriate delimiters, malformed input, CRLF/consecutive/trailing
CR, NUL, non-scalars, explicit EOF, and later terminal blocks compare complete
source-step traces. Repeated suspension, completion and terminal errors are
asserted. These are local state checks, not full tokenizer conformance.

## Connected RAWTEXT evidence

The original isolated PR began at common baseline
`bcae7e5c916470319ef0838dc72dfd71a95b8181`, with zero new upstream passes because
its entry states were stubs. Integration merges main
`cebd054f5410bc1f929bf39b9ad37efe4ac0a73f` after PRs #26, #25, #24, #23 and #27.
All five sibling production algorithms and all 886 protected identities are
preserved exactly; only obsolete RAWTEXT name-stub expectations are migrated.
RCDATA end tag name is not reimplemented or generalized.

Thirteen state-local tests now include four integrated RAWTEXT-entry tests:

- `real_rawtext_entry_preserves_source_boundaries_creation_and_original_spelling`
  inspects RAWTEXT `<`, less-than sign `/` (only scratch clearing), end-tag-open
  empty creation/reconsumption, each original/normalized name point, existing
  emission into Data, and Data's subsequent literal NUL/error.
- `real_rawtext_matching_mismatch_eof_attributes_and_literal_ampersands_complete_exactly`
  independently specifies matching/mismatching/absent/case-distinct context,
  prefix and non-ASCII mismatches, repeated partial candidates, NUL, name-state
  literal EOF versus destination `eof-in-tag`, retained attributes/duplicate
  removal, flag errors, normalized locations, and literal RAWTEXT ampersands.
- `explicit_modes_do_not_switch_automatically_and_real_later_stubs_stay_terminal`
  verifies Data `<style>` does not choose RAWTEXT, RAWTEXT tag-like starts stay
  literal, and a matching closing tag explicitly chooses Data. Later Data start
  tags update last-start context normally. Character reference still blocks in
  attributes or Data, and Markup declaration open still blocks after a close,
  including after more input/finish. RAWTEXT ampersands do not invoke references.
- `real_rawtext_entry_all_splits_and_single_code_point_chunks_preserve_every_step`
  compares every productive source step from real RAWTEXT entry across all
  two-chunk splits and single-point chunks, absent/matching/case-distinct last
  context, preprocessing/non-scalars, EOF, attributes, literal fallback, and
  genuine later terminal blocks. Repeated suspension is effect-free.

Sibling source-state snapshots still inspect empty creation before destination
normalization and original-spelling append. Adapter tests require complete
matching/mismatching/EOF output and reject deliberately truncated results instead
of retaining obsolete name-state blocks. RCDATA tests retain all RCDATA branch
assertions; only their RAWTEXT-stub assumptions now use real later blocks.
No output prefix is accepted as completion, and no blocked execution is resumed.

## Integrated fixture review

The existing RAWTEXT initial-state mapping needs no change. All 18 previously
name-blocked RAWTEXT runs were inspected directly in the unchanged pinned
fixtures; expectations below come from those fixtures and the pinned algorithm,
not tokenizer output. Last start tag is `xmp` throughout. `C(text)` denotes
Character output and `E(name)` an EndTag. The following **17** identities are
manually recorded in `crates/html/tests/baselines/rawtext_end_tag_name.json`,
all in group `tests`, state `RAWTEXT state`, initial-state index **1**.
No errors are expected except the two explicitly listed EOF diagnostics.

| File / case | Unchanged input → expected output | Pinned branch reasoning |
| --- | --- | --- |
| `contentModelFlags.test` 2 | `foo</xmp>` → C(`foo`), E(`xmp`) | Appropriate `>` emits and chooses Data. |
| same, 3 | `foo</xMp>` → C(`foo`), E(`xmp`) | ASCII `M` lowercases only in the name, not scratch. |
| same, 4 | `foo</xmp ` → C(`foo`); `eof-in-tag` at (1,10) | Appropriate SPACE hands off; destination EOF abandons tag. |
| same, 5 | `foo</xmp` → C(`foo</xmp`) | Name-state EOF takes literal fallback even for matching name; no tag error. |
| same, 6 | `foo</xmp/` → C(`foo`); `eof-in-tag` at (1,10) | Appropriate `/` hands off; Self-closing start tag diagnoses EOF without setting flag. |
| same, 7 | `foo</xmp<` → C(`foo</xmp<`) | Name fallback reconsumes `<` in RAWTEXT; less-than-sign EOF emits delayed `<`, then RAWTEXT emits EOF. |
| same, 8 | `</foo>bar</xmp>` → C(`</foo>bar`), E(`xmp`) | Inappropriate `>` reconsumes literally; later slash clears scratch for matching candidate. |
| same, 9 | `</xmp</xmp</xmp>` → C(`</xmp</xmp`), E(`xmp`) | Partial names fall back on `<`; final matching `>` emits. |
| same, 10 | `</foo>bar</xmpaar>` → C(`</foo>bar</xmpaar>`) | Prefix matches are insufficient; both delimiters fall back literally. |
| same, 11 | `foo</xmp></baz>` → C(`foo`), E(`xmp`), E(`baz`) | Matching close chooses Data; Data's end tags need not be appropriate. |
| `domjs.test` 27 | `</XMP>` → E(`xmp`) | ASCII-only normalization enables exact name match. |
| same, 29 | `</xm>` → C(`</xm>`) | Shorter name is inappropriate; literal delimiter reconsumption. |
| same, 30 | `</xm ` → C(`</xm `) | Inappropriate SPACE falls back; RAWTEXT EOF is not tag abandonment. |
| same, 31 | `</xm/` → C(`</xm/`) | Inappropriate `/` is literal, not a flag-state handoff. |
| `escapeFlag.test` 0 | `foo<!--</xmp>--></xmp>` → C(`foo<!--`), E(`xmp`), C(`-->`), E(`xmp`) | No comment mode in RAWTEXT; first close chooses Data, which handles the second end tag. |
| same, 1 | `foo<!-->baz</xmp>` → C(`foo<!-->baz`), E(`xmp`) | Literal RAWTEXT markup-like prefix followed by matching close. |
| same, 4 | `foo<!-- x --x>x-- >x--!>x--<></xmp>` → C(`foo<!-- x --x>x-- >x--!>x--<>`), E(`xmp`) | Non-solidus less-than fallback stays literal until the final matching candidate. |

The eighteenth affected run, `escapeFlag.test` **2**, input
`foo<!--></xmp><!-->baz</xmp>`, now closes the first tag and switches to Data,
then blocks at **Markup declaration open** on the second `<!`. Its expected
comment and `abrupt-closing-of-empty-comment` diagnostic at (1,19) require an
unimplemented chain: this run is not comparable, not passed, and not added.
No blocked run is reconstructed to simulate continuation. Case 3 contains no
RAWTEXT initial state (its RCDATA counterpart still blocks at Character reference).
The 17 RCDATA pass identities from PR #27 and every other existing baseline are
unchanged; no fixture, expectation, pin, comparison or policy is modified.

### Historical RAWTEXT integration report

The protected union grew from **886 to 903** identities with only these 17
reviewed additions, without result-derived generation or automatic blessing.
Historical integrated main had 1,042 executed/passed, 0 failed, 6,011 blocked.
This integrated RAWTEXT state reports:

- 6,827 fixture cases; 7,053 runs; 6,190 dispatch-attempted.
- **1,059 executed = 1,059 passed + 0 failed; 5,994 blocked**.
- Blocked reasons: CharacterReference 4,692; TokenVariants 858;
  MarkupDeclarationOpen 242; ScriptData 89; CdataSection 56;
  ProcessingInstructionOpen 52; XmlCoercion 4; PendingSpecReview 1.

Inventory/local tests are not upstream conformance runs. This remains partial
standalone tokenization, not full conformance or browser-security evidence.
Use the [central runner command/count definitions](tokenizer.md#fixture-runner).

## Remaining blocks and integration caveats

Character reference, ordinary comments/markup declarations, script escape states
and other unsupported states remain terminal `UnimplementedState` errors.
The four simple Script data states now execute; see the
[current script integration report](tokenizer-script-data-end-tag-name.md#validation-and-current-limits). RAWTEXT and
RCDATA closing-tag recognition now execute, with no automatic tree-builder mode
switching. All CONTRIBUTING checks and the pinned runner must pass after conflict
resolution. Shared module/dispatch, support inventory, docs and baseline-list
edits are additive. Preserve sibling algorithms/identities and review any future
reachability changes rather than hiding newly admitted mismatches or resuming
blocked execution.
