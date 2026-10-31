# Script data double escaped dash dash state

## Current PR #40 integration update

On exact main **`eecf0a3992d820f95990dcdca9201e1378e52a8f`** (merged #43/#38/#39)
plus #40, Dash is implemented without changing any merged production algorithm
or removing source-step assertions.

Dash now enters this state on `-`, with its own hyphen emission inspected before
this source executes. The `>`/CRLF/malformed-input test retains original DashDash
entry and also executes Dash entry with exact initial source snapshots. The full
streaming suite runs both entry states through real LessThanSign/DoubleEscapeEnd,
scratch/name handling, escaped cycles, EOF and later genuine blocks.

DoubleEscaped and the ordinary-entry EscapedLessThanSign gateway remain genuine
stubs. All required checks pass: **319 Rust tests, 0 failed, 0 ignored**; upstream
**1,126 executed/passed, 0 failed, 5,927 blocked**, zero new passes. Fixtures,
pins and all baseline files/registrations remain unchanged. See
[#40's current integration evidence](tokenizer-script-data-double-escaped-dash.md#current-integration-after-prs-43-38-and-39)
for exact paths, tests, counts and final-head location. Coordinator final-head
review is separate; this thread does not merge or start reviewers.

**The #39 and older reports below are historical for support and test totals.**

## Historical PR #39 report

The user authorized sequential integration. PR #39 is rebased onto exact main
base **`98607c3a282cc8336c41312db042e8f87b8c2bcf`**, containing merged #43
(DoubleEscapeEnd) and #38 (DoubleEscapedLessThanSign). The original standalone
head **`58a9d938dd01c88595c22a89f253a8f3614a340a`** and its original base/report
are historical below. The final delivered head is recorded in
[PR #39](https://github.com/olukowski/neptune/pull/39)'s body. Only current
`origin/main` was fetched; no unmerged sibling was imported. All three production
algorithms are unchanged. Module/dispatch/inventory/docs registrations and the
merged inventory's direct-entry scratch setup are preserved additively.

Current behavior: DashDash emits `<` before real LessThanSign executes. That
neighbor emits `/` and clears scratch before real DoubleEscapeEnd executes;
ASCII letters accumulate normalized scratch while emitting original spelling.
A matching `script` delimiter reaches real Escaped, a mismatch reaches terminal
DoubleEscaped. LessThanSign's non-slash fallback, and DoubleEscapeEnd's other
fallback (including EOF), reconsume in the same genuine DoubleEscaped stub.
DashDash's NUL/literal branches still block there after their own source effects;
its hyphen, `>` and EOF behavior is unchanged. No implemented predecessor yet
enters DashDash; ordinary ScriptData still stops at EscapedLessThanSign.

The original source-step assertions remain independent of destination execution.
The old less-than destination-block test now checks the neighbor's actual NUL
consumption and scheduled reconsumption, with no premature replacement or state
error, before the genuine DoubleEscaped block. Two new tests add:

- Full source snapshots through repeated hyphens, `<`, `/`, every name letter and
  each delimiter (TAB/LF/FF/SPACE/solidus/greater-than/normalized CR). Mixed-case
  and lowercase matches reach real Escaped; near matches/empty names block at
  DoubleEscaped. Scratch is preserved until `/`, then cleared and lowercased;
  output preserves spelling. Real continuation checks CRLF, two NUL replacements,
  surrogate/noncharacter/control diagnostics, exact normalized positions and
  multiplicity, unrelated storage, and exactly one escaped EOF error/token.
- Open exhaustion after `<`, `</`, and `</script`, followed by explicit EOF.
  Actual neighbors reconsume conceptual EOF before DoubleEscaped; they do not
  emit EOF or invent an escaped EOF diagnostic at a terminal stub.

Every two-chunk split and single-point stream now extends through these real
neighbors, including matched/mismatched names, name EOF/NUL, CRLF, non-scalars,
return through Escaped to ScriptData, and later genuine blocks. No support
introspection, weaker comparison, reconstruction or resumption of terminal
executions is used. The merged LessThanSign suite retains all direct-entry
source assertions and adds a real DashDash predecessor test: `<` emits once,
slash clears scratch on its own step, suspension is effect-free, then EOF
reconsumes into the genuine remaining stub. Its stale predecessor comment is
updated. The merged DoubleEscapeEnd suite needs no assertion changes.

All CONTRIBUTING checks and the explicit pinned runner pass (exit 0): fmt,
warning-denying Clippy, workspace tests, warning-denying rustdoc, upstream report,
and `git diff --check`. **310 Rust tests pass, 0 failed, 0 ignored** (nine in
DashDash, nine in LessThanSign, eight in DoubleEscapeEnd). These are local checks,
not additional upstream conformance runs.

Upstream remains **6,827 fixture cases; 7,053 runs; 6,190 dispatch-attempted;
1,126 executed = 1,126 passed + 0 failed; 5,927 blocked**. Blocks:
CharacterReference 4,692; MarkupDeclarationOpen 242; EscapedLessThanSign 22;
CdataSection 56; ProcessingInstructionOpen 52; TokenVariants 858; XmlCoercion 4;
PendingSpecReview 1. New upstream passes: **0**. All 930 protected identities,
fixture pins and baseline files/registrations are unchanged. There is no
validation/delivery blocker. Coordinator final-head review and merge are
separate; this thread does not merge, enable auto-merge or start reviewers.

## Historical standalone evidence

**Everything below describes the original standalone head, not current
integrated destination support, test names or test counts.** The assigned
algorithm, immutable branch evidence and its source-step effects remain valid.

## Scope and immutable evidence

This PR implements exactly `State::ScriptDataDoubleEscapedDashDash` in
`crates/html/src/tokenizer/script_data_double_escaped_dash_dash.rs`, from reviewed
main **`39d772e31b98e6e955a2e00f546a8db3506eb1dd`** (after PR #36). Initial HEAD
was verified before editing. No sibling implementation is imported. Existing
Context, code-point cursor, diagnostics, tokens and Execution provide everything
needed; there is no new dependency, API/storage redesign or adjacent algorithm.

Retrieved and read the immutable WHATWG HTML source revision
**`404b3eb14a7a936dddefb2d78e224dbeb01d2358`** from its
[raw source](https://raw.githubusercontent.com/whatwg/html/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source).
Normative evidence and referenced definitions:

- [Assigned state, lines 144057–144090](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144057-L144090): the complete six-branch algorithm.
- [Preprocessing, lines 142560–142607](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L142560-L142607): newline normalization, input-stream diagnostics, current/next input and conceptual EOF.
- [Tokenization, lines 143149–143191](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143149-L143191): consumption, reconsumption, scratch/return context, character/EOF token output and creation versus emission.
- [unexpected-null-character, lines 141479–141482](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L141479-L141482) and [eof-in-script-html-comment-like-text, lines 141138–141150](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L141138-L141150): state errors, not Comment-token parsing.
- [Script data, lines 143338–143359](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143338-L143359): real continuation after `>`; ordinary EOF adds no escaped EOF diagnostic.
- [Double escaped / dash predecessor, lines 143993–144054](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143993-L144054), [double escaped less-than sign, lines 144093–144109](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144093-L144109), and [escaped less-than sign, lines 143872–143892](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143872-L143892): transition/entry boundaries only, not implemented here.

## Branch-to-code/test mapping

The module matches these branches in source order. Test names below are in
`crates/html/tests/script_data_double_escaped_dash_dash_state.rs`.

| Input / source lines | Source effects and intended target | Focused tests |
| --- | --- | --- |
| `-`, 144064–144065 | Emit `-`; remain in assigned state. | `every_ascii_branch…`, `open_exhaustion_hyphen_runs…` |
| `<`, 144067–144069 | Emit `<`; switch to DoubleEscapedLessThanSign. Unlike EscapedDashDash, the less-than character **is emitted here**. | `every_ascii_branch…`, `less_than_null_and_literal_effects…` |
| `>`, 144071–144073 | Emit `>`; switch to ScriptData, not Escaped. | `every_ascii_branch…`, `greater_than_returns_to_real_script_data…` |
| NUL, 144075–144079 | Exactly one `unexpected-null-character`; emit U+FFFD; switch to DoubleEscaped. | `every_ascii_branch…`, `less_than_null_and_literal_effects…`, `explicitly_reconsumed_entry…` |
| EOF, 144081–144084 | Exactly one `eof-in-script-html-comment-like-text`; emit one terminal EOF without changing state. | `open_exhaustion_hyphen_runs…`, `explicitly_reconsumed_entry…` |
| Anything else, 144086–144088 | Emit the current code point literally; switch to DoubleEscaped. | `every_ascii_branch…`, `unicode_non_scalars_and_controls…` |

Every branch consumes once; none schedules reconsumption. Tests inspect source
snapshots before destination execution, including untouched suffixes and exact
current item. Explicitly preconsumed/reconsumed code-point and EOF entry is also
tested: preprocessing diagnostics do not repeat. Open exhaustion returns
`NeedMoreInput` without changing state, current item, output or errors. Empty
chunks are not EOF; repeated finish/terminal steps cannot duplicate output/errors.

All ASCII points are checked under open and finished input. Additional points
include non-ASCII letters/whitespace/punctuation, literal BOM and U+FFFD, actual
high/low surrogates, BMP/supplementary noncharacters, supplementary scalars and
controls. Original spelling is preserved (no ASCII lowercasing or Unicode folding).
NUL alone becomes U+FFFD. The cursor supplies preprocessing errors at normalized,
one-based code-point coordinates; state errors have exact positions and multiplicity.
CR/CRLF normalize through the existing cursor, not the state.

Seeded snapshots preserve selected and pending tags, self-closing flags, retained
and detached attributes (including non-scalars), comment data, last-start-tag
history, return state and mixed-case/NUL/replacement scratch data. Prior output
and diagnostics are appended to, not overwritten. Default context is also valid;
**direct entry needs no pending tag, selector, temporary-buffer initialization or
other setup** beyond `Execution::new(State::ScriptDataDoubleEscapedDashDash, context)`.
The independent support inventory adds just this state; its existing unrelated
tag/comment setup is not a prerequisite for this state.

The streaming test compares every source-step snapshot for every two-chunk split
(including empty chunks) and single-code-point chunks against closed input.
Inputs include hyphen runs, every branch, CRLF splits, trailing/consecutive CR,
NULs, non-scalars and malformed text, real Script data continuation/closing tags,
and later explicit escaped-less-than-sign / Character reference blocks.
Suspensions are effect-free; traces stop at terminal errors and never reconstruct
or resume a blocked execution. These are local state checks, not upstream runs.

## Reachability, remaining stubs and integration boundaries

There is no implemented predecessor into this state on the required base.
The real Script data `<!--<script>-->` path is tested through the existing escape
start/trio chain: it stops at `ScriptDataEscapedLessThanSign` after consuming the
second `<`, before any double escaping. Direct isolated entry therefore supplies
the assigned state's evidence without implementing neighbors.

`ScriptDataDoubleEscaped` and `ScriptDataDoubleEscapedLessThanSign` remain terminal
`UnimplementedState` errors on the next step, even after emitted literal text or
NUL recovery. They are not recoverable HTML errors, EOF, or successful prefixes.
Repeated steps preserve the source boundary; no comparison hides the block.
The `>` branch executes existing Script data, with CRLF/NUL/non-scalar diagnostics
and ordinary EOF checked separately. Only the generic support inventory had an
obsolete assigned-state stub assumption; existing state tests remain unchanged.

This is one of nine independently owned states. Later coordinator integration
must preserve all additive module/dispatch/inventory registrations and status
links. Implemented predecessors/destinations will change reachability and terminal
boundaries: review fixed source assertions separately from new destination steps,
retain later real stubs, rerun every check/report, and independently review any
newly comparable fixture identities before state-specific baseline additions.
Do not dynamically detect support, weaken comparisons or resume terminal executions.
No sibling fetch/cherry-pick/merge, subagent, merge or auto-merge is performed here.
No tree builder, automatic mode switching, JavaScript parsing, Comment-token
behavior, full-conformance or browser-security claim is implied.

## Validation and upstream evidence

The pinned fixture submodule was already initialized at
`c777c408b61078ea2eb4acefc2535f54dbc8b28a`; fixtures and both pins are unchanged.
All required commands passed (exit 0):

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo test --workspace --locked pinned_upstream_runner -- --nocapture
```

Workspace: **291 Rust tests passed, 0 failed, 0 ignored**, including the seven new
focused tests. Local/inventory/runner-infrastructure tests are not additional
conformance cases. The explicit upstream command reports:

- Fixture cases: **6,827**; runs: **7,053**; dispatch-attempted: **6,190**.
- Executed: **1,126**; passed: **1,126**; failed: **0**; blocked: **5,927**.
- Blocks: CharacterReference 4,692; MarkupDeclarationOpen 242;
  ScriptDataEscapedLessThanSign 22; CdataSection 56; ProcessingInstructionOpen 52;
  TokenVariants 858; XmlCoercion 4; PendingSpecReview 1.

Executed = passed + failed; blocked runs are not passes. These match the common
base's [PR #36 integrated report](tokenizer-script-data-escape-start.md#validation-and-current-counts).
There are **zero newly passing upstream runs**: none of the supported upstream
initial states enters this isolated state through implemented production paths.
All 930 protected identities remain passing; no baseline file/registration is
changed or auto-blessed.

Independently inspected unchanged `domjs.test` zero-based tests[6], [11] and [19]:
[6]'s third double-escaped NUL expects U+FFFD and `unexpected-null-character` at
1:48; [11] (`<!--<script>--`) expects literal output and the escaped EOF error at
1:15; [19]'s abrupt double-escaped `-->` expects literal text then a real `script`
end tag. These are relevant future NUL/EOF/`>` integration evidence, but all still
block at escaped less-than sign on this PR. No output-prefix pass or newly passing
fixture identity is inferred. There is no validation/delivery blocker; predecessor
and destination stubs are the deliberate remaining functional boundary.
