# HTML tokenizer infrastructure

## Status and source

**Only Data, PLAINTEXT, Tag open, End tag open, Tag name, and Bogus comment are
implemented. The other 78 states, including Before attribute name, Self-closing
start tag, ordinary Comment, and Character reference, remain `UnimplementedState` stubs.** This is a partial tokenizer, not a stable parsing API
or a usable browser. No full tokenizer-conformance or browser-security claim is made.

Implementation evidence is the immutable HTML Standard source in
[docs/sources.toml](sources.toml), specifically **Preprocessing the input stream**
and the opening **Tokenization** paragraphs (current/next input, reconsumption,
and token creation/emission), plus **Data state** (`#data-state`), **PLAINTEXT
state** (`#plaintext-state`), **Tag open state** (`#tag-open-state`), **End tag
open state** (`#end-tag-open-state`), **Tag name state** (`#tag-name-state`), and
**Bogus comment state** (`#bogus-comment-state`).
Newline normalization follows the linked Infra
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
- `Context` contains the cursor, emitted tokens, state diagnostics, optional last
  emitted start-tag name, and optional character-reference return state. Data's
  `&` sets the return state to Data even while Character reference remains a stub.
  Tag open creates `pending_start_tag: Option<PendingStartTag>`; End tag open
  creates `pending_end_tag: Option<PendingEndTag>` or
  `pending_comment: Option<PendingComment>`. Tags store code-point names only;
  their self-closing flag is implicitly unset and attributes implicitly empty,
  as required at creation. Comments store code-point data; Bogus comment appends
  to it and takes the pending comment on emission as `Token::Comment { data }`.
  Isolated Bogus comment entry requires initialized pending-comment storage;
  existing data is preserved literally, not preprocessed again.
  Each creation replaces only its own pending storage; unrelated storage remains.
  The code-point `temporary_buffer` is cleared by Tag open on `?`.
- Tag creation selects `current_tag: Option<TagKind>` (Start or End), independently
  of any unrelated retained pending storage. Tag name mutates only the selected
  name. On `>` it takes that pending tag and emits `Token::StartTag { name }` or
  `Token::EndTag { name }`; on EOF it discards the selected tag. Both clear the
  selector. Direct isolated Tag name entry requires a selector and matching
  pending tag; normal entry through Tag open / End tag open establishes this
  invariant. This remains inspectable scaffolding, not a general current-token API.
  The emitted variants also have implicit empty attributes/unset self-closing flag;
  no implemented algorithm can change those fields. Thus neither end-tag emission
  diagnostic (`end-tag-with-attributes`, `end-tag-with-trailing-solidus`) can arise
  here, nor is self-closing acknowledgement applicable. These checks must be added
  when those fields become representable, not simulated now.
- Only start-tag **emission** updates `last_start_tag`, with the emitted normalized
  name; creation, mutation, EOF abandonment, and end-tag emission do not. The
  appropriate-end-tag definition still compares against the last emitted start tag,
  if any. Tag name itself does not filter end tags by appropriateness.
- `Execution::step` performs **one source step**, returning its intended target
  through `Progress::Continue`. Source effects can be inspected before the next
  step attempts the target. A handler schedules reconsumption on the cursor.
  `NeedMoreInput` keeps the same state and permits feeding/resuming.
- An unimplemented source or target is a terminal development error, not EOF,
  a recoverable parse error, or success. Repeated steps return that same error
  without dispatching. `Finished` is terminal too. Context remains inspectable;
  mutation does not clear terminal status. Reconstructing/resuming after a stub
  has **no supported semantics**.

Production dispatch implements only Data, PLAINTEXT, Tag open, End tag open,
Tag name, and Bogus comment and has no synthetic handlers. Unit tests inject private, non-algorithmic
source steps to validate the execution shell separately. The independent support-
inventory test explicitly lists those six and checks that the other 78 states
block without effects at both suspension and EOF; it does not infer support from
dispatch.

## Fixture runner

The runner lives in `crates/html/tests/support` as test-only infrastructure;
`serde_json` remains a dev dependency. It is deliberately limited to the
character/EOF, simple-tag, and comment surface emitted by the implemented states.
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
- Character strings are expanded to individual code points on the expected side,
  equivalent to coalescing adjacent character outputs without crossing tags/comments.
  Tag kind/name and comment kind/data/boundaries/order compare exactly, including
  double-escaped code-point strings.
  Production must finish and emit exactly one terminal EOF, omitted from the fixture comparison. Truncated output,
  missing/duplicate EOF, output after EOF, and need-more-input on closed fixture
  input fail, rather than pass on an output prefix.
- Exact character values and parse-error codes, one-based line/column positions,
  and multiplicities are compared. Diagnostics are compared as a **multiset**,
  not by emission order; cursor and state diagnostics are both included.
  Absent `errors` means no expected errors.

Explicit limitations/policies (no silent skips or partial token comparisons):

- Expected StartTag with empty attributes/unset self-closing flag, and EndTag
  with a name, are supported. A nonempty attribute object or set self-closing
  flag blocks the **whole run** as `TokenVariants`, not a partial field comparison.
  Production can only represent the supported creation defaults. DOCTYPE tokens
  (including IDs/force-quirks) and processing instructions also block
  the whole run; they are not treated as equal or defaulted away.
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

Current pinned baseline including Bogus comment: 14 files, 6,827 cases,
**7,053 runs; 5,818 dispatch-attempted; 547 executed, 547 passed, 0 failed,
6,506 blocked**:

- 4,659 blocked at Character reference (including one after Tag open recovery)
- 24 blocked at Before attribute name
- 6 blocked at Self-closing start tag
- 52 blocked at Processing instruction open
- 240 blocked at Markup declaration open
- 290 blocked at other initial-state stubs (56 CDATA section, 71 RAWTEXT,
  74 RCDATA, 89 Script data)
- 1,230 unsupported-token runs
- 4 XML-coercion runs
- 1 pending-spec-review run

At the preceding Tag name milestone, all **466 prior passes were retained**: 385 original Data, 52 PLAINTEXT, 24 Tag
open additions, and five End tag open additions. The runner asserts these groups
independently of dispatch, including explicit identities for the five End tag
open cases. Of the 11 former Tag-name-blocked runs (also explicitly identified),
only `test4.test` “EOF in tag name state ” now passes; nine block at Before
attribute name and one at Self-closing start tag. Thus 30 of the original 66
Tag-open-blocked runs now pass; 36 still block (9 Before attribute name, 1 Self-
closing start tag, 25 Processing instruction open, 1 Character reference).

That milestone's adapter admitted **101 additional runs** with supported tag outputs: 55 pass,
20 reach missing attribute/self-closing states, and 26 reach still-missing initial
states. Together with the EOF-in-name pass this adds **56 passes**, all Data-
initial: 470 Data and 52 PLAINTEXT passes in total. There are 5,184 Data-initial
dispatch attempts, of which 4,714 remain blocked. No failed run was excluded to
obtain these counts; policy precedence and existing exclusions are unchanged.
Unsupported-token runs decrease from 1,625 to 1,524, while the admitted runs can
still block at dispatch. Fixtures and pinned revisions remain unchanged.

Bogus comment retains **all 522 Tag name baseline passes** and adds **25 Data-
initial passes**, explicitly identified in the runner (one `test2.test` case and
24 `test3.test` cases). Current totals are 495 Data and 52 PLAINTEXT passes.
Exact comment output support admits 294 formerly unsupported-token runs: 25 pass,
240 block at Markup declaration open, 27 at Processing instruction open, and one
each at RAWTEXT/RCDATA. These are additional dispatch attempts, not 294 passes.
No failed case was excluded, and fixtures, revisions, and policy precedence are
unchanged. Processing-instruction support has not been added.

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
older unexpected-question-mark/bogus-comment transition. End tag open and Tag
name now execute on the next step. The other two destinations report their own typed terminal
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

## End tag open coverage

The implementation follows every branch of the pinned **End tag open state**:

| Consumed input | Source effects and intended destination |
| --- | --- |
| ASCII alpha (`A–Z`, `a–z`) | Create a pending end tag with an **empty** name; reconsume in Tag name. No lowercasing or name append yet. |
| `>` | Report `missing-end-tag-name`; switch to Data without reconsumption, token creation, or output. |
| EOF | Report `eof-before-tag-name` at EOF; emit `<`, `/`, then one terminal EOF. |
| Anything else | Report `invalid-first-character-of-tag-name`; create a comment with **empty** data; reconsume in Bogus comment. No character emission, comment append, or NUL replacement here. |

Malformed closing tags are not simply discarded. Comment creation precedes
reconsumption in the now implemented Bogus comment; end-tag creation precedes
reconsumption in Tag name. End tag open itself neither appends to nor emits the
comment; Bogus comment handles the reconsumed point, including NUL replacement. Other context, including pending start tags,
the last emitted start tag, return state, and temporary buffer, is preserved.
End-tag self-closing/attribute defaults remain implicit exactly as for start tags.

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

## Tag name coverage and simple-tag milestone

The implementation follows every branch of the pinned **Tag name state**:

| Consumed input | Source effects and intended destination |
| --- | --- |
| TAB U+0009, LF U+000A, FF U+000C, SPACE U+0020 | Switch to Before attribute name. CR is normalized to LF by the cursor, not separately matched here. |
| `/` U+002F | Switch to Self-closing start tag. |
| `>` U+003E | Switch to Data; emit the current tag token. |
| ASCII upper alpha U+0041–U+005A | Append code point + 0x20 to the current name; stay in Tag name. |
| NUL U+0000 | Report `unexpected-null-character`; append U+FFFD to the name, not the original NUL. |
| EOF | Report `eof-in-tag`; abandon the unfinished tag and emit one terminal EOF, without emitting that tag. |
| Anything else | Append the code point unchanged; stay in Tag name. No Unicode case folding. |

Every branch consumes once and none reconsumes. Open exhausted input preserves
pending state and returns NeedMoreInput, not EOF. Whitespace and `/` consume but
do not append, emit, or report a state parse error. Their source effects remain
inspectable before the next step terminates with `UnimplementedState` identifying
Before attribute name or Self-closing start tag. Do not resume a blocked execution.
VT and NBSP are name characters, not separators; VT also gets the cursor's
preprocessing diagnostic. Surrogates/noncharacters are preserved as code points,
with their original preprocessing errors, not converted through scalar strings.

**Milestone:** real Data → Tag open / End tag open → Tag name → Data execution
now completes `<p>Hello</p>` and `<DIV>world</DIV>`, emitting start tag, character
data, end tag, and EOF. There is no tree builder, special-element mode selection,
or DOM construction; even an unmatched simple end tag emits without tree-builder
errors. Minimal support is the explicit selector, two name-only emitted variants,
selected-token take/discard, and last-start-tag bookkeeping described above.
No general emission abstraction, token-field mutation API, or dependencies were
needed. The fixture adapter compares only representable tag defaults, without
implementing attribute/self-closing algorithms or suppressing their stub blocks.

Focused tests in `crates/html/tests/tag_name_state.rs` cover both tag kinds,
all ASCII letters, ASCII boundaries, Unicode uppercase without folding, malformed
name punctuation, NUL replacement/errors, actual surrogates/noncharacters/controls,
EOF abandonment, exact emission/consumption, last-start-tag updates/preservation,
and untouched unrelated context. All two-chunk splits plus single-code-point
chunks compare every source-step snapshot, including name accumulation, `</` / name,
name / `>`, CRLF and consecutive/trailing CR, NUL, final EOF, and terminal stub
transitions. Local adapter tests deliberately mismatch tag kind/name/order,
exercise double-escaped non-scalar tag names, and reject matching output prefixes
at later stubs. These local passes are not additional upstream conformance runs.

## Bogus comment coverage

The implementation follows every branch of the pinned **Bogus comment state**:

| Consumed input | Source effects and intended destination |
| --- | --- |
| `>` U+003E | Switch to Data; emit the current comment token. The delimiter is not data. |
| EOF | Emit the current comment token, then one terminal EOF. No `eof-in-comment` diagnostic or switch to Data. |
| NUL U+0000 | Report `unexpected-null-character` at the consumed point; append U+FFFD to comment data. |
| Anything else | Append the current code point unchanged; stay in Bogus comment. |

Every branch consumes once and none reconsumes. `<` and `&` are literal comment
data, not transitions or character references. Existing pending-comment data is
retained; unrelated context (including current-tag storage and last emitted start
tag) is untouched. Emission takes only the pending comment, using the existing
output-vector convention and a state-local helper, not a new general emission API.
Open exhaustion suspends without emitting a comment or EOF; explicit EOF is required.

Focused tests in `crates/html/tests/bogus_comment_state.rs` cover all branches,
empty/seeded data, exact token ordering and diagnostics/normalized locations,
NUL replacement versus subsequent Data's original NUL output, Unicode scalars,
actual surrogates/noncharacters/controls, consumed delimiters and continuation.
End tag open tests inspect source diagnostics/empty creation separately from
Bogus comment's reconsumption/append/error effects; preprocessing errors occur
only once. Every two-chunk split and single-code-point chunking of representative
isolated/recovery inputs matches closed-input source-step traces, including CRLF,
consecutive/trailing CR, NUL, EOF, and later terminal stubs. Blocked executions are
never reconstructed or resumed. These are state-specific streaming checks, not
whole-tokenizer conformance. Local adapter tests compare comment data/kind/order,
empty-token boundaries, and double-escaped non-scalar data, deliberately mismatch
outputs, reject malformed shapes, and retain downstream stub blocks after a
matching comment prefix. Only upstream runs contribute to the counts above.

## Before attribute-state work

Before attribute name and Self-closing start tag remain stubs; neither whitespace
nor `/` can currently complete a tag. Attribute/flag storage, end-tag emission
diagnostics for nondefault fields, and expanded fixture comparisons will be
needed when implementing the relevant states. Ordinary comments, markup
declarations, processing instructions, and character references also remain stubs.
Before attribute name and Self-closing start tag are parallel, separate PRs;
integration checks/counts must be rerun after rebasing onto their merges.
No neighboring-state algorithms, execution/input model changes, tree-building
behavior, or optimizations were added.

Before claiming broader coverage, extend token fields and comparisons,
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
