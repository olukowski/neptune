# Script data end tag name state

## Immutable evidence and scope

Only `State::ScriptDataEndTagName` is implemented here, from WHATWG HTML
revision `404b3eb14a7a936dddefb2d78e224dbeb01d2358`. The source was retrieved
from the immutable raw-source URL and inspected, including these definitions:

- [Assigned algorithm, lines 143695–143738](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143695-L143738).
- [Preprocessing, current/next input and conceptual EOF, lines 142560–142607](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142607).
- [Reconsumption and temporary buffer, lines 143160–143168](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143160-L143168).
- [Tag creation defaults, creation versus emission, and independent end-tag emission errors, lines 143174–143210](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143174-L143210).
- [Appropriate end tag definition, lines 143213–143215](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143213-L143215):
  exact equality with the last **emitted** start-tag name; absent history means
  no appropriate end tag.
- [Predecessor context, lines 143661–143686](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143661-L143686):
  slash clears scratch; end tag open creates an empty end tag and reconsumes
  the first ASCII letter. These predecessor states are **not** implemented here.

Algorithm: `crates/html/src/tokenizer/script_data_end_tag_name.rs`.
Focused tests: `crates/html/tests/script_data_end_tag_name_state.rs`.
RCDATA/RAWTEXT code supplies storage/execution conventions, not normative evidence.
No shared storage/API, tag helper, adapter, dependency, fixture, pinned revision,
other algorithm, tree-builder behavior, or automatic mode switching is changed.

## Entry invariants and branch evidence

Fresh isolated entry requires `current_tag = Some(TagKind::End)` and matching
`pending_end_tag` storage. Its candidate name is normalized so far, while
`temporary_buffer` contains original spelling. Empty stores model the first
letter; seeded prefixes model later steps. `last_start_tag` is independent
optional context, never inferred from unrelated pending start-tag storage.
Default `Context` alone is not initialized tag-state entry. Tests explicitly
check creation defaults and valid initialized entry, preserved selection and
unrelated storage; this state neither creates nor resets a tag or scratch.
Synthetic seeded storage probes also verify literal/non-scalar preservation;
they do not claim such prefixes are reachable via ASCII accumulation.

| Source branch | Implementation and focused evidence |
| --- | --- |
| TAB, LF, FF, SPACE, 143703–143710 | If appropriate, consume and choose Before attribute name; otherwise literal fallback. All separators, plus preprocessed CR, are checked. No attribute creation or destination diagnostic occurs in the source step. |
| `/`, 143712–143715 | If appropriate, consume and choose existing Self-closing start tag. The source neither sets the flag nor diagnoses it. Inappropriate slash falls back. |
| `>`, 143717–143720 | If appropriate, consume, choose Data and reuse `tag::emit_tag`. Tests check ordered retained attributes, exclusion of detached duplicates (including detached-only storage), both independent emission errors exactly once at the normalized delimiter position, selected-storage take, and unchanged last-start history. Otherwise literal fallback. |
| ASCII upper alpha, 143722–143725 | Append point + 0x20 to candidate and original point to scratch. Every uppercase letter is checked step by step against seeded names and unrelated storage. |
| ASCII lower alpha, 143727–143730 | Append unchanged to both stores. Every lowercase letter is checked; no Unicode case folding. |
| Anything else, 143732–143736 | Emit `<`, `/`, then each scratch point in order. Leave candidate/scratch unchanged and schedule reconsumption in Script data. Exhaustive non-letter ASCII, inappropriate delimiters, punctuation boundaries, Unicode letters/whitespace, NUL, real surrogates, noncharacters and controls are checked. The mismatching point is not emitted by this state. |
| Actual EOF | Also anything else, even for a matching candidate. Emit only literal recovery and reconsume conceptual EOF; do not emit EOF, abandon the tag, or report `eof-in-tag` here. |
| Open exhausted input | Repeated effect-free `NeedMoreInput`, not EOF or a delimiter decision. Empty chunks do not change that. Input can resume until explicitly finished. |

Appropriateness tests distinguish absent history, exact match, case-distinct
history, shorter/longer names, Unicode-distinct names, and unrelated un-emitted
start storage. Synthetic seeded probes establish that comparison uses candidate
code points, not scratch or scalar strings. Non-scalar input keeps its cursor
preprocessing error and normalized coordinates; reconsumption must not repeat it.
Scratch itself is not input and is neither preprocessed nor NUL-replaced.

Matching paths exercise existing attribute/self-closing/Data destinations through
complete emission, malformed-solidus recovery, duplicate removal, NUL value
replacement and exact diagnostic positions. Destination EOF abandonment produces
`eof-in-tag`, unlike name-state EOF fallback. Snapshot assertions preserve unrelated
pending start/comment data, return state, last-start history and creation defaults.
Every split into two chunks, empty chunks and single-code-point chunks of
representative inputs compares every productive source-step snapshot, including
CRLF splits, repeated/trailing CR, malformed input, non-scalars, EOF and blocks.
These are local execution tests, not upstream conformance passes.

## Explicit blocks and sibling integration

Script data, Script data less-than sign and Script data end tag open remain
unimplemented on this independently based branch. A name mismatch or name-state
EOF therefore produces its inspectable source effects, then a **terminal
`UnimplementedState { state: ScriptData }`** on the next dispatch. It must not be
reported as a completed recovery, EOF success or recoverable parse error. Tests
also retain later Character reference and Markup declaration open terminal blocks.
Repeated steps, feeding, finish and context mutation never unblock an execution;
no test reconstructs/resumes after a stub.

There is no reachable new upstream run: upstream Script data entry still blocks
before this state. No baseline addition is warranted; all 903 existing reviewed
passing identities and registrations remain unchanged. The fixture inventory and
synthetic/local tests are not conformance runs. The pinned runner report and
required checks are recorded in the PR; this remains partial standalone
tokenization, not full conformance or browser-security evidence.

The three sibling algorithms must be reviewed/integrated separately. Module and
dispatch registration plus the independent support inventory are narrow additive
edits; merge all four registrations rather than replacing a sibling. Integration
will legitimately change Script data destination-block assertions into real
reconsumption/EOF behavior and may admit upstream runs. Review those expectations,
newly comparable diagnostics and individual pass identities against the pinned
source/fixtures, and rerun every CONTRIBUTING check plus the pinned runner after
integration. Do not accept matching output prefixes, weaken baselines, or resume
blocked execution to simulate sibling behavior. No sibling branch was imported,
merged or cherry-picked here.
