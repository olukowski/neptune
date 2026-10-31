# HTML tokenizer infrastructure

## Status and source

**Only Data, PLAINTEXT, Tag open, and End tag open are implemented. The other 80
states, including Tag name, Bogus comment, and Character reference, remain
`UnimplementedState` stubs.** This is a partial tokenizer, not a stable parsing API
or a usable browser. No full tokenizer-conformance or browser-security claim is made.

Implementation evidence is the immutable HTML Standard source in
[docs/sources.toml](sources.toml), specifically **Preprocessing the input stream**
and the opening **Tokenization** paragraphs (current/next input, reconsumption,
and token creation/emission), plus **Data state** (`#data-state`), **PLAINTEXT
state** (`#plaintext-state`), **Tag open state** (`#tag-open-state`), and **End tag
open state** (`#end-tag-open-state`). Newline
normalization follows the linked Infra
[normalize newlines](https://infra.spec.whatwg.org/#normalize-newlines) operation.
The test adapter follows the pinned
[upstream format](../tests/html5lib-tests/tokenizer/README.md), not another engine.
Neither pinned revision nor upstream fixtures are changed here.

The registry matches the pinned specification in order, including processing-
instruction states and no separate decimal-character-reference-start state.

## Input and execution contract

- `CodePoint` represents 0..=0x10FFFF, **including lone surrogates**. `char` is
  only a convenience for scalar input. Byte decoding, encoding sniffing,
  networking, and BOM handling belong outside this crate.
- `Cursor::push` appends decoded code points. `finish` is explicit and idempotent;
  subsequent pushes fail. Empty chunks and exhausted open input yield
  `NeedMoreInput`, never EOF.
- CRLF becomes LF and remaining CR becomes LF, including across chunk boundaries.
  A trailing CR can emit LF immediately; the next raw LF is suppressed.
- Consumption retains a current input item and its position. Suspension leaves
  that item unchanged. `reconsume` schedules it for the next consume without
  advancing or duplicating preprocessing diagnostics. It works for conceptual
  EOF too; requesting it without current input or twice before consuming fails.
- Surrogates, noncharacters, and non-whitespace/non-NUL controls produce the
  specified preprocessing errors on first consumption; they are not replaced.
  NUL handling is deferred to states. Positions count normalized code points,
  one-based, with EOF just after the last point (not byte or UTF-16 offsets).
- `Context` contains the cursor, character/EOF token outputs, state diagnostics,
  optional last start tag, and optional character-reference return state. The
  return-state field is the only supporting context addition for Data: its `&`
  branch must set this to Data before transitioning, even while the destination
  remains a stub. Tag open adds `pending_start_tag: Option<PendingStartTag>`
  (only the unfinished name) and a code-point `temporary_buffer`. Pending start
  tags are not emitted tokens: their self-closing flag is implicitly unset and
  attribute list implicitly empty. No downstream mutation, general attribute API,
  or tag-emission variant exists yet. End tag open adds matching
  `pending_end_tag: Option<PendingEndTag>` (only an unfinished name, with the same
  implicit unset flag/empty attributes) and
  `pending_comment: Option<PendingComment>` (only code-point data). These record
  creation, not emission. No comment mutation or emission is implemented. Each
  creation replaces only its own storage; other pending storage is preserved.
  This is inspectable scaffolding, not a general current-token API.
- `Execution::step` performs **one source step**, returning its intended target
  through `Progress::Continue`. Source effects can be inspected before the next
  step attempts the target. A handler schedules reconsumption on the cursor.
  `NeedMoreInput` keeps the same state and permits feeding/resuming.
- An unimplemented source or target is a terminal development error, not EOF,
  a recoverable parse error, or success. Repeated steps return that same error
  without dispatching. `Finished` is terminal too. Context remains inspectable;
  mutation does not clear terminal status. Reconstructing/resuming after a stub
  has **no supported semantics**.

Production dispatch implements only Data, PLAINTEXT, Tag open, and End tag open
and has no synthetic handlers. Unit tests inject private, non-algorithmic source
steps to validate the execution shell separately. The independent support-inventory
test explicitly lists those four as implemented and checks that the other 80 states
block without effects at both suspension and EOF; it does not infer support from
dispatch.

## Fixture runner

The runner lives in `crates/html/tests/support` as test-only infrastructure;
`serde_json` remains a dev dependency. It is deliberately limited to the
character/EOF surface emitted by Data, PLAINTEXT, Tag open, and End tag open.
Run its upstream report with:

```sh
cargo test --workspace --locked pinned_upstream_runner -- --nocapture
```

Supported conventions:

- Default Data initial state; one independent run per `initialStates` entry;
  all six upstream initial-state names recognized. `lastStartTag` is supplied
  unchanged as context, absent meaning no prior start tag.
- `doubleEscaped` input and supported output strings, including token-kind
  strings. Each literal `\uHHHH` becomes its own code point, even for adjacent
  surrogate escapes; no UTF-16 pair joining or replacement. Other text is literal.
- Raw input goes through the cursor's preprocessing, not string line helpers.
- Adjacent character outputs coalesce. Production must finish and emit exactly
  one terminal EOF, omitted from the fixture comparison. Truncated output,
  missing/duplicate EOF, output after EOF, and need-more-input on closed fixture
  input fail, rather than pass on an output prefix.
- Exact character values and parse-error codes, one-based line/column positions,
  and multiplicities are compared. Diagnostics are compared as a **multiset**,
  not by emission order; cursor and state diagnostics are both included.
  Absent `errors` means no expected errors.

Explicit limitations/policies (no silent skips or partial token comparisons):

- Any expected non-character token blocks the **whole run** as `TokenVariants`.
  DOCTYPE name/public/system IDs and force-quirks inversion, tag names,
  attributes/self-closing flags, comments, and processing instructions are
  deferred, not treated as equal or defaulted away.
- All `xmlViolationTests` runs block as `XmlCoercion`; Neptune does not apply
  DOM-to-XML-infoset coercion in this runner.
- All `pendingSpecChanges.test` runs block as `PendingSpecReview` until individually
  reconciled against our pinned specification in a separate reviewed change.
- Unknown fixture fields or initial-state names block explicitly. Malformed
  known JSON shapes fail the infrastructure test. Missing fixtures fail with
  submodule initialization instructions.
- Policy takes precedence over adapter support, which takes precedence over
  dispatch. The reported reason is the first applicable limitation, not an
  assertion that every later capability is supported.

### Counts and regression policy

A **fixture case** is one upstream JSON object; a **run** is a case/initial-state
pair. Every run has exactly one result:

- **Passed**: executor finished, complete tokens and diagnostics match.
- **Failed**: comparable execution mismatches or violates completion requirements.
- **Blocked**: explicit adapter/policy limitation or `UnimplementedState`, even
  if the source already emitted output. No comparison can turn it into a pass.

**Executed = passed + failed** (runs reaching comparison or a completion failure).
**Dispatch-attempted** additionally records entry into production execution,
including blocked stubs. This avoids describing attempted-but-blocked runs as
successfully executed conformance tests. Inventory tests execute no conformance.

Current pinned Data + PLAINTEXT + Tag open + End tag open baseline: 14 files,
6,827 cases, **7,053 runs; 5,423 dispatch-attempted; 466 executed, 466 passed,
0 failed, 6,587 blocked**:

- 4,659 blocked at Character reference (including one after Tag open recovery)
- 11 blocked at Tag name
- 25 blocked at Processing instruction open
- 262 blocked at other initial-state stubs (56 CDATA section, 61 RAWTEXT,
  63 RCDATA, 82 Script data)
- 1,625 unsupported-token runs
- 4 XML-coercion runs
- 1 pending-spec-review run

Of the 5,109 Data-initial dispatch attempts, 414 complete and compare;
4,695 are still blocked by later states, not passes. All 385 original Data passes,
52 PLAINTEXT passes, and 24 passes added by Tag open are retained and asserted
separately. All five previously End-tag-open-blocked runs now pass: `test1.test`
“Empty end tag”, `test2.test` “Unescaped </” and “Empty end tag with following
characters”, and `test3.test` “</” and “</>”. The runner asserts these case
identities explicitly, independently of dispatch. Thus 29 of the 66 runs formerly
blocked at Tag open now pass; 37 remain blocked (11 Tag name, 25 Processing
instruction open, 1 Character reference after recovery). The older Data groups
are independently identified by their first transition character. Expected
comment/end-tag fixtures still block under the existing token-variant policy;
there are no new comparable upstream runs reaching Bogus comment. Focused local
tests exercise that destination without counting it as a pass. Adapter/policy
precedence, exclusions, and dispatch-attempt counts are unchanged. No upstream
fixture or pinned revision was modified.

The baseline counts/reasons are asserted explicitly; widening blocking requires
review, not an automatic skip. Synthetic executor tests independently demonstrate
both matches and deliberate mismatches/classification; their passes are **not**
upstream tokenizer conformance passes. Local production-adapter regressions also
check that an already matching output/error prefix still blocks at a destination
stub. Only the pinned upstream runs contribute to the counts above.

## Data coverage

The implementation follows every branch of the pinned **Data state** section:
`&` consumes the character, sets return state to Data, and transitions to Character
reference; `<` consumes the character and transitions to Tag open. Neither branch
emits a token, reports a parse error, or schedules reconsumption. Character
reference blocks on the next step; Tag open now executes its own algorithm. NUL
emits `unexpected-null-character` and the original NUL character token, **not U+FFFD**. EOF emits one terminal EOF;
anything else emits the current code point unchanged. Open exhausted input
suspends through the existing execution/input contract rather than emitting EOF.

Focused tests in `crates/html/tests/data_state.rs` check source effects, context,
consumption, positions, diagnostics, terminal behavior, and all two-chunk splits
plus single-code-point chunks of representative Data input. These include empty
chunks, Unicode scalars, actual surrogate code points, noncharacters, controls,
NUL, CR/CRLF preprocessing, trailing CR, and both Data transitions. This is
Data-specific chunk equivalence, not whole-tokenizer streaming coverage.

## PLAINTEXT coverage

The implementation follows every branch of the pinned **PLAINTEXT state** section:
NUL reports `unexpected-null-character` at the consumed point and emits U+FFFD,
**not the original NUL emitted by Data**. EOF emits one terminal EOF. Anything
else, including `<` and `&`, emits the current code point unchanged and stays in
PLAINTEXT. No branch transitions to another state or schedules reconsumption.
Open exhausted input suspends, never implying EOF.

Select PLAINTEXT explicitly with `Execution::new(State::Plaintext, context)`.
There is no tree builder or automatic handling of a `<plaintext>` start tag;
Data transitions to Tag open on `<`. Existing context fields, including the
pending start tag and temporary buffer, are preserved. PLAINTEXT needs no
execution/input infrastructure or initial-state mapping changes (the runner
already recognized PLAINTEXT).

Focused tests in `crates/html/tests/plaintext_state.rs` check every branch,
consumption/current input, exact tokens, diagnostic codes/positions, Unicode
scalars and actual surrogate/noncharacter/control code points, empty input,
repeated suspension/termination, and preserved context. Every two-chunk split
and single-code-point chunking of representative inputs matches closed-input
source-step traces, including CRLF splits, consecutive/trailing CR, NUL, literal
markup and references, and preprocessing diagnostics. These are state-specific
streaming checks, not whole-tokenizer conformance. Local production-adapter tests
also cover explicit PLAINTEXT selection, repeated independent initial-state runs,
Data defaults, empty input, and double-escaped NUL/surrogate diagnostics.

## Tag open coverage

The implementation follows the pinned **Tag open state** section literally:

| Consumed input | Source effects and intended destination |
| --- | --- |
| `!` | Switch to Markup declaration open; no reconsumption. |
| `/` | Switch to End tag open; no reconsumption. |
| ASCII alpha (`A–Z`, `a–z`) | Create a new pending start tag with an **empty** name; reconsume in Tag name. No lowercasing or name append yet. |
| `?` | Empty the temporary buffer; switch to Processing instruction open, without reconsumption or a Tag open parse error. |
| EOF | Report `eof-before-tag-name` at EOF; emit `<` then one terminal EOF. |
| Anything else | Report `invalid-first-character-of-tag-name` at the consumed point; emit `<`; reconsume in Data. |

The `?` branch uses the pinned processing-instruction transition, **not** an
older unexpected-question-mark/bogus-comment transition. End tag open now executes
on the next step. The other three destinations report their own typed terminal
block on the next step, after the source effects are independently inspectable.
A pending token is creation only, never emitted output or a change to
`last_start_tag`. Other branches preserve
pending-token storage; only `?` clears the temporary buffer.

Focused tests in `crates/html/tests/tag_open_state.rs` cover all 52 ASCII letters,
case distinctions and ASCII-alpha boundaries, punctuation, non-ASCII letters,
NUL, actual surrogates, noncharacters, controls, locations, consumption versus
reconsumption, preserved/replaced context, default creation, terminal stubs, EOF,
and incremental suspension. Both Data's `<` transition and isolated Tag open
entry use the existing execution interface. Recovery executes real Data, including
its original NUL emission/error and transitions on `&` and `<`. All two-chunk
splits and single-code-point chunks of representative inputs match closed-input
source-step traces, including CRLF splits, consecutive/trailing CR, EOF after
`<`, malformed input, and downstream blocks. These are state-specific checks,
not whole-tokenizer streaming conformance. Existing Data/PLAINTEXT tests also
verify preservation of pending-start-tag storage and the temporary buffer.

## End tag open coverage and remaining work

The implementation follows every branch of the pinned **End tag open state**:

| Consumed input | Source effects and intended destination |
| --- | --- |
| ASCII alpha (`A–Z`, `a–z`) | Create a pending end tag with an **empty** name; reconsume in Tag name. No lowercasing or name append yet. |
| `>` | Report `missing-end-tag-name`; switch to Data without reconsumption, token creation, or output. |
| EOF | Report `eof-before-tag-name` at EOF; emit `<`, `/`, then one terminal EOF. |
| Anything else | Report `invalid-first-character-of-tag-name`; create a comment with **empty** data; reconsume in Bogus comment. No character emission, comment append, or NUL replacement here. |

Malformed closing tags are not simply discarded. Comment creation precedes the
terminal Bogus comment block; end-tag creation precedes the terminal Tag name
block. Both targets remain unimplemented, and neither block is an HTML parse
error, EOF, success, or supported suspension. Other context, including pending
start tags, the last emitted start tag, return state, and temporary buffer, is
preserved. End-tag self-closing/attribute defaults are implicit exactly as for
the existing pending start tag; no emitted-token variants or fixture comparisons
were added.

Focused tests in `crates/html/tests/end_tag_open_state.rs` cover all 52 ASCII
letters without case conversion, ASCII boundaries, punctuation, non-ASCII
letters, NUL, actual surrogates, noncharacters, controls, and CR normalization.
They inspect exact source effects, token initialization/replacement and preserved
context, diagnostics and normalized positions, consumption/reconsumption,
unconsumed tails, and terminal stub/EOF behavior. Tests use both isolated states
and real Data → Tag open → End tag open transitions. Empty end tags recover to
real Data, including subsequent literal output, NUL errors, repeated `</>`, and
later transitions. Open exhaustion (including after `</`) repeatedly suspends
without effects; only explicit EOF emits `<`, `/`, EOF. All two-chunk splits and
single-code-point chunks of representative inputs match closed-input source-step
traces, particularly `<` / `/` / next code point, CRLF boundaries, malformed
input, EOF, and blocked destinations. These are state-specific streaming checks,
not full tokenizer conformance. A separate regression checks that every prior
implemented state's branches preserve the new pending storage.

The next state PR is **Tag name**. It will need explicit current-tag selection
(start versus end), real name mutation, and eventually tag emission/field
comparison support. The separate pending storage here deliberately does not
provide that downstream API. Bogus comment, comments, markup declarations,
processing instructions, and character references also remain stubs. Do not
resume an execution already terminated at a stub. No neighboring-state
algorithms, execution model changes, preprocessing changes, dependencies,
tree-building behavior, or optimizations were added here.

Before claiming broader coverage, add actual token variants/field comparisons,
review pending-spec-change expectations, and decide whether XML coercion belongs
in scope. More complex states will justify scratch buffers and context gradually.
Tree-builder control (including foreign-content/CDATA permission), synchronous
emission feedback, insertion points, fuzzing, and whole-tokenizer chunk equivalence
are deferred. Do not present this standalone execution shell as tree-builder-ready.
Tree-construction tests now live in WPT `html/syntax/parsing`; pin that source when
adopting them.

## Provenance and licensing

[docs/sources.toml](sources.toml) records immutable revisions; the existing Git
submodule retains upstream licensing and attribution. The living HTML URL is a
navigation aid, not our baseline. This repository does not relicense or vendor
the standard's algorithm prose under Neptune's license.
