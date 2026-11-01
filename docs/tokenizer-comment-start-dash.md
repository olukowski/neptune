# Comment start dash state

## Pinned source and scope

Normative source: WHATWG HTML at
`404b3eb14a7a936dddefb2d78e224dbeb01d2358`,
[Comment start dash, lines 144535–144559](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144535-L144559).
Read from that immutable source, not the living page or another engine.
The related [input/preprocessing definitions, lines 142560–142607](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142607)
define current/next input, conceptual EOF, newline normalization and input
diagnostics. [Tokenization, lines 143149–143188](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143149-L143188)
defines reconsumption and separates token creation from emission.

Only `CommentStartDash` is implemented here, in
`crates/html/src/tokenizer/comment_start_dash.rs`. The private
`comment::data_mut` and `comment::emit` helpers from prerequisite #46
(`47c53e8dcf3e69d920c4be290036ffb70af72cc9`, an ancestor of this branch)
remain in the shared `comment.rs` module, not a core Comment algorithm.
No storage, token variants, dependencies, fixtures, pins or pass baselines change.

Direct isolated entry supplies initialized `pending_comment`; it may contain
literal data. The support inventory supplies that same invariant, including at
EOF. This state neither creates comments nor chooses a tag. No tree builder or
automatic tokenizer-mode selection is implied.

## Branch map

All test names below are in `crates/html/tests/comment_start_dash_state.rs`.

| Consumed input / pinned lines | Source effects | Focused test |
| --- | --- | --- |
| `-`, 144542–144543 | Switch to CommentEnd. Do not append the delimiter or deferred hyphen, emit, diagnose, or reconsume. | `hyphen_consumes_without_appending_emitting_or_reconsuming_then_blocks` |
| `>`, 144545–144548 | Report `abrupt-closing-of-empty-comment` at the consumed point; switch to Data and emit/take the current comment, without adding a hyphen. | `greater_than_emits_and_takes_exact_comment_then_real_data_continues` |
| EOF, 144550–144552 | Report `eof-in-comment`; emit/take the comment, then one EOF. No state switch, deferred hyphen append or reconsumption. | `open_exhaustion_has_no_effects_but_explicit_eof_diagnoses_emits_and_takes` |
| Anything else, 144554–144556 | Append exactly one U+002D; reconsume the point in Comment. The point itself is not appended/replaced/diagnosed by this source state. | `anything_else_appends_only_one_hyphen_and_reconsumes_literal_input` |

Open exhaustion repeatedly yields NeedMoreInput without side effects; empty
chunks do not imply EOF. The explicit EOF branch is distinct from fallback,
including when incoming EOF was scheduled for reconsumption. The
`incoming_reconsumption_of_code_points_and_eof_is_consumed_once` test also
checks already-preprocessed input without duplicate diagnostics.

Snapshots include exact output, error codes/multiplicities, normalized positions,
current input, pending comment and unrelated context (both tag kinds, retained
and detached attributes, flags, selector, scratch, return state and last-start-tag
bookkeeping). Empty and seeded comment emission takes only the comment and
preserves preceding token boundaries. Seeded NUL, CR and non-scalars remain
literal, without reprocessing. All ASCII fallback points plus Unicode letters,
combining marks, emoji, surrogates, noncharacters and controls are checked.
NUL follows fallback here: no `unexpected-null-character` or replacement is
borrowed from the unsupported Comment destination. Cursor probes verify the
scheduled original point, its unchanged position and single preprocessing error,
then untouched tail/EOF. Those probes do not dispatch or resume blocked execution.

Source snapshots are asserted before destination attempts. Comment and CommentEnd
remain terminal `UnimplementedState` destinations on this base. Repeated blocked
steps are effect-free. The implemented Data continuation consumes only the tail,
including Data's original-NUL output/error and terminal EOF. Completion is
idempotent; an emitted/taken comment is never emitted twice.

`every_two_chunk_split_and_single_point_chunks_match_closed_source_traces`
compares source-step traces for every two-chunk split (with empty chunks) and
single-code-point chunks of representative branch, EOF, malformed-input, CRLF,
trailing/consecutive CR, NUL, non-scalar and downstream-block cases, both at
initial positions and after a consumed newline/hyphen prefix. Streaming stops at
the first terminal block; no blocked execution is reconstructed.

## Integration onto #47–51

After initial independent review with no findings at
`0dc2a7a52c3414eec82270ff9fa9b411effd9105`, rebase base is
`fb249f0e4273ed1626ce2f28ddc9f9cd8218dc78`. Additive conflicts preserve
all merged modules, dispatch, independent inventory and documentation.
The authoritative implemented-state list includes all 47 dispatched states.

The reviewed CommentStartDash algorithm and its seven isolated tests are
unchanged. The already-merged CommentStart algorithm is also unchanged, but its
tests now preserve the exact predecessor source assertions and then execute
CommentStartDash rather than falsely expecting its stub. All four destination
branches are independently asserted through actual predecessor entry, including
empty/seeded data, open exhaustion vs EOF, exact normalized diagnostic locations,
comment take semantics, Data continuation and EOF order, and genuine
Comment/CommentEnd blocks. Predecessor chunk traces are expanded correspondingly.
See [the predecessor integration evidence](tokenizer-comment-start.md#commentstartdash-integration-52).

## Entry and conformance limits

`normal_data_comment_entry_is_still_terminally_blocked_before_comment_creation`
checks actual Data entry stops at MarkupDeclarationOpen, before any comment
creation or this state. No sibling algorithm or gateway is added by this PR.
No new Data-entry comment coverage or new upstream passing identity is claimed.

The pinned upstream runner has no CommentStartDash initial-state name; focused
isolated tests above are local branch/streaming evidence, not upstream runs.
Run the separate upstream report per CONTRIBUTING.md; inventory is not
conformance, and blocked paths are not passes. Prior pass identities must remain
passing. This partial implementation makes no full-conformance or browser-security
claim.
