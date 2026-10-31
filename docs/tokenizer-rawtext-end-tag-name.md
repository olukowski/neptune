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

## Remaining blocks and integration

RAWTEXT less-than sign and RAWTEXT end tag open remain terminal
`UnimplementedState` stubs. RAWTEXT therefore cannot yet reach this implementation
from normal mode entry. Tests retain explicit adjacent-stub assertions and later
blocks at RAWTEXT less-than sign / Character reference; no output prefix is
accepted as completion and no blocked execution is reconstructed/resumed.

The upstream runner needs no registration change: it already recognizes RAWTEXT
initial entry and discovers production dispatch outcomes. This state adds no
reachable upstream passes on this baseline, so no pass-identity file is added or
changed. All existing passing-run baselines remain enforced. Inventory is not
conformance; exact executed/passed/failed/blocked counts belong in the PR report.

Sibling PRs may conflict in module/dispatch, support inventory and status docs.
Rebase after their merges and rerun every check and the upstream report. A newly
implemented entry/destination can legitimately change terminal-block assertions
and upstream reachability; review those integration effects rather than importing
neighbor states here or weakening regression policy.
