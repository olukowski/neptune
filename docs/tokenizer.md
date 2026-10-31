# HTML tokenizer infrastructure

## Status and source

**Only Data, RCDATA, RCDATA less-than sign, RCDATA end tag open, RCDATA end tag name,
PLAINTEXT, RAWTEXT, RAWTEXT less-than sign, RAWTEXT end tag open, RAWTEXT end tag name,
Script data ([state-local evidence](tokenizer-script-data.md)),
[Script data less-than sign](tokenizer-script-data-less-than-sign.md),
Tag open, End tag open, Tag name, Before attribute name, Attribute name,
After attribute name, Before attribute value, Attribute value
(double-quoted), Attribute value (single-quoted), Attribute value (unquoted),
After attribute value (quoted), Self-closing start tag, Bogus comment,
[Script data end tag open](tokenizer-script-data-end-tag-open.md),
[Script data end tag name](tokenizer-script-data-end-tag-name.md),
[Script data escape start](tokenizer-script-data-escape-start.md),
[Script data escape start dash](tokenizer-script-data-escape-start-dash.md),
[Script data escaped](tokenizer-script-data-escaped.md),
[Script data escaped dash](tokenizer-script-data-escaped-dash.md),
[Script data escaped dash dash](tokenizer-script-data-escaped-dash-dash.md), isolated-entry
[Script data double escaped less-than sign](tokenizer-script-data-double-escaped-less-than-sign.md), and
[Script data double escape end](tokenizer-script-data-double-escape-end.md) are implemented.
All other states, including Script data escaped less-than sign, ordinary Comment and
Character reference, remain `UnimplementedState` stubs.**
This is a partial tokenizer, not a stable parsing API or a usable browser. No full tokenizer-conformance or browser-security claim is made.

Explicit Script data entry now reaches basic comment-like text through
`<!` → escape start → escape start dash → escaped dash dash. The escaped trio
handles literal characters, hyphen cycles, NUL replacement and
`eof-in-script-html-comment-like-text`; `>` in escaped dash dash returns to
Script data. This emits character tokens, not Comment tokens. An escaped `<`
still reaches terminal Script data escaped less-than sign: escaped end tags and
end-to-end double escaping are not reachable from Script data entry.
Isolated double escaped less-than sign entry now reaches Double escape end on
`/`; the double-escaped predecessors and fallback destination remain terminal
stubs. Double escape end uses explicit temporary-buffer context and can return
to real Escaped on a matching `script` delimiter. Ordinary Script data closing-tag
support remains separate. No tree builder or automatic element-based
mode switching is implied. See the [integrated branch/fixture audit](tokenizer-script-data-escape-start.md).

Implementation evidence is the immutable HTML Standard source in
[docs/sources.toml](sources.toml), specifically **Preprocessing the input stream**
and the opening **Tokenization** paragraphs (current/next input, reconsumption,
and token creation/emission), plus **Data state** (`#data-state`), **PLAINTEXT
state** (`#plaintext-state`), **RAWTEXT state** (`#rawtext-state`),
**RAWTEXT less-than sign state** (`#rawtext-less-than-sign-state`;
[state-local evidence](tokenizer-rawtext-less-than-sign.md)),
**RAWTEXT end tag open state** (`#rawtext-end-tag-open-state`;
[state-local branch and integrated fixture evidence](tokenizer-rawtext-end-tag-open.md)),
**RAWTEXT end tag name state** (`#rawtext-end-tag-name-state`;
[state-local branch and integrated fixture evidence](tokenizer-rawtext-end-tag-name.md)),
**Tag open state** (`#tag-open-state`), **End tag open state** (`#end-tag-open-state`),
**Tag name state** (`#tag-name-state`),
**Before attribute name state** (`#before-attribute-name-state`),
**Attribute name state** (`#attribute-name-state`),
**After attribute name state** (`#after-attribute-name-state`;
[state-local branch and fixture evidence](tokenizer-after-attribute-name.md)),
**Attribute value (double-quoted) state** (`#attribute-value-(double-quoted)-state`;
[state-local coverage](tokenizer-attribute-value-double-quoted.md)),
**After attribute value (quoted) state** (`#after-attribute-value-quoted-state`;
[state-local evidence](tokenizer-after-attribute-value-quoted.md)),
**Attribute value (unquoted) state** (`#attribute-value-(unquoted)-state`;
[state-local coverage](tokenizer-attribute-value-unquoted.md)),
**Before attribute value state** (`#before-attribute-value-state`;
[state-local evidence](tokenizer-before-attribute-value.md)),
**Self-closing start tag state** (`#self-closing-start-tag-state`),
**Bogus comment state** (`#bogus-comment-state`), **RCDATA state**
(`#rcdata-state`; [state-local coverage](tokenizer-rcdata.md)),
**RCDATA less-than sign state** (`#rcdata-less-than-sign-state`;
[state-local branch evidence](tokenizer-rcdata-less-than-sign.md)),
**RCDATA end tag open state** (`#rcdata-end-tag-open-state`;
[state-local branch evidence](tokenizer-rcdata-end-tag-open.md)), and
**RCDATA end tag name state** (`#rcdata-end-tag-name-state`;
[state-local branch and integrated fixture evidence](tokenizer-rcdata-end-tag-name.md)).
[Attribute value (single-quoted) branch evidence](tokenizer-attribute-value-single-quoted.md)
cites its exact pinned source lines.
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
  `pending_comment: Option<PendingComment>`. RCDATA/RAWTEXT/Script data end tag
  open also create empty selected end tags. Their name states accumulate ASCII-normalized
  names and original spelling in the temporary buffer, testing appropriateness
  only at delimiters. Tags store code-point names and an
  ordered `Vec<Attribute>` with code-point names/values. The current attribute is
  the per-tag `discarded_current_attribute` when present, otherwise the last
  retained entry. Attribute name removes a duplicate from the retained list but
  keeps it in that detached slot; later value states must use the shared
  `tag::current_attribute_mut` helper, NEVER fall back to its retained predecessor.
  Starting another attribute clears the detached slot before appending the new
  current entry. Creation defaults it to `None`; taking/abandoning the selected
  tag drops it along with that tag, preserving unrelated storage.
  Their explicit `self_closing: bool` defaults to false and
  attribute lists start empty, as required at creation. Comments store code-point data;
  Bogus comment appends to it and takes the pending comment on emission as
  `Token::Comment { data }`. Isolated Bogus comment entry requires initialized
  pending-comment storage; existing data is preserved literally, not preprocessed again.
  Each creation replaces only its own pending storage; unrelated storage remains.
  The code-point `temporary_buffer` is cleared by Tag open on `?` and by RCDATA,
  RAWTEXT, Script data less-than sign, and Script data double escaped less-than
  sign on `/`.
- Tag creation selects `current_tag: Option<TagKind>` (Start or End), independently
  of any unrelated retained pending storage. Tag name mutates only the selected
  name. Before attribute name appends only to the selected tag's attribute list.
  Tag name and Self-closing start tag reuse the selected-token take/emission path.
  Emission preserves the name, ordered retained attributes, and flag in
  `Token::StartTag` / `Token::EndTag`; the detached duplicate is never emitted;
  EOF abandons only the selected tag. Both clear the selector. Direct isolated
  Tag name / Before attribute name / After attribute name /
  After attribute value (quoted) /
  Self-closing start tag entry requires a selector and matching pending tag;
  RCDATA/RAWTEXT/Script data end tag name require selected End storage plus original-spelling
  temporary-buffer context;
  Attribute name, Before attribute value, and Attribute value (double-quoted) / (single-quoted) /
  (unquoted) also require an initialized current attribute. This is inspectable
  scaffolding, not a general current-token API. Self-closing start tag sets the
  flag only on `>`.
  Shared emission reports `end-tag-with-attributes` once for a nonempty retained
  end-tag attribute list and independently `end-tag-with-trailing-solidus` once
  for a flagged end tag. Both use the emitting state's current `>` position,
  under our normalized code-point diagnostic convention. Detached-only storage
  does not count as attributes. Abandonment reports neither emission error.
  These independent rules follow the pinned opening
  [Tokenization paragraphs](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L143174-L143210);
  the [Attribute name removal rule](https://github.com/whatwg/html/blob/404b3eb14a7a936dddefb2d78e224dbeb01d2358/source#L144218-L144229)
  excludes detached duplicates from the token.
  Data entry now reaches boolean, quoted, and unquoted attribute parsing.
  Before attribute value consumes opening quotes, emits on missing-value `>`,
  or reconsumes anything else (including EOF) in unquoted value. That destination,
  not Before attribute value, reports `eof-in-tag` and abandons the tag.
  Character reference still blocks terminally after `&` in any value state.
  No tree-builder acknowledgement or
  `non-void-html-element-start-tag-with-trailing-solidus` behavior is simulated.
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

The [independent support inventory](../crates/html/src/tokenizer/support_inventory.rs)
explicitly lists supported states separately from production dispatch. It checks
both supported entry and effect-free terminal stubs at suspension and EOF. Unit
tests inject private, non-algorithmic source steps to validate the execution shell
separately.

## Fixture runner

The runner lives in `crates/html/tests/support` as test-only infrastructure;
`serde_json` remains a dev dependency. It is deliberately limited to the
character/EOF, tag (including attributes and self-closing flags), and comment
surface. Boolean, quoted, and unquoted attribute emission is reachable from
Data; ampersands still block at Character reference, never compare a prefix.
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

- Expected StartTag attribute objects may be nonempty; names and values support
  empty strings and double-escaped code points in both keys and values. Attributes
  compare by exact code-point name/value, independent of object/list order; token
  order and the self-closing flag still compare exactly. Production retains list
  order (asserted separately by local support tests). Missing/extra/different
  attributes fail. Actual duplicate names fail even with identical values: the
  adapter never silently overwrites them or applies a second duplicate-removal
  policy. Distinct fixture keys decoding to the same name block the whole run as
  `FixtureCapability`, since that object has no unambiguous unique-name meaning.
  Exact duplicate raw JSON members are not representable by the imported fixture
  object; this is not a duplicate-preserving JSON reader.
- EndTag fixtures are name-only, so the adapter explicitly projects away both
  production attributes and the self-closing flag, but compares required
  `end-tag-with-attributes` and `end-tag-with-trailing-solidus` diagnostics exactly.
  Local support tests assert both stored fields directly. DOCTYPE tokens
  (including IDs/force-quirks) and processing instructions still block the
  **whole run** as `TokenVariants`.
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

The runner prints current totals and blocked-reason distribution centrally; keep
exact run evidence in PR reports, not repeated hand-maintained documentation.
Every comparable mismatch fails CI, including newly admitted runs.
The [passing-run baseline](../crates/html/tests/baselines/README.md) independently
records previously verified passes, including fixture identity and initial state.
Each must still be present and pass exact output/diagnostic comparison; becoming
blocked, failed, or missing fails CI even if other cases now pass. There is no
automatic baseline-update mode. Add reviewed new pass identities without removing
old ones; fixture/specification migrations require explicit review.

Policy/adapter limitations above remain explicit classifications, never skips.
Unknown capabilities/states require review. Focused state tests retain meaningful
destination-block assertions: implementing a destination can legitimately change
an integration test's terminal block, so rebase and rerun after merges.
Synthetic runner tests exercise rejection/classification and pass-regression
tracking separately; their passes are **not** upstream conformance runs.

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

## RAWTEXT coverage

The implementation follows the pinned **RAWTEXT state** (`#rawtext-state`) in
source order: `<` switches to RAWTEXT less-than sign; NUL reports
`unexpected-null-character` at the consumed point and emits U+FFFD; EOF emits one
terminal EOF; anything else emits the current code point unchanged. In particular,
`&` is literal, not a character reference, and neither sets nor consumes the
return state. No branch schedules reconsumption. Open exhausted input suspends.

Select RAWTEXT explicitly with `Execution::new(State::Rawtext, context)`; the
fixture runner already maps `RAWTEXT state` to that initial state. No tree builder
or automatic switching on `<style>` is added. [RAWTEXT less-than sign](tokenizer-rawtext-less-than-sign.md),
[RAWTEXT end tag open](tokenizer-rawtext-end-tag-open.md), and
[RAWTEXT end tag name](tokenizer-rawtext-end-tag-name.md) are implemented and
reachable from RAWTEXT entry. Appropriate end tags can emit through existing
attribute/tag support and switch to Data. Mismatches, non-alpha `</` fallback,
and name-state EOF recover literally in RAWTEXT; `&` stays literal there.
Character references still block in attributes and in Data after a matching close.
A matching output prefix before a stub is blocked, never EOF or a passing run;
blocked execution must not be reconstructed/resumed.
[RCDATA is implemented separately](tokenizer-rcdata.md).

Focused tests in `crates/html/tests/rawtext_state.rs` check literal ampersands and
markup-like punctuation, NUL replacement/errors, exact normalized positions,
Unicode and actual surrogate/noncharacter/control points without silent
replacement or duplicate preprocessing errors, intended transitions, consumption,
retained context/return state, empty input, and repeated suspension/termination.
All two-chunk splits and single-code-point chunks of representative inputs compare
every source-step snapshot, including `<`, CRLF, consecutive/trailing CR, NUL,
and terminal blocks. State-local adapter tests in
`crates/html/tests/support/rawtext.rs` check explicit initial-state mapping,
Data defaults, no automatic style handling,
exact output/diagnostics, matching/mismatching end tags and EOF fallback. Integrated
name-state tests preserve source-step evidence through real RAWTEXT entry, with
all chunk splits and explicit terminal Character reference / Markup declaration
open blocks after matching closing tags.
These are state-specific checks, not full tokenizer streaming conformance.

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
End-tag creation explicitly unsets its self-closing flag and initializes an empty
attribute list, as for start tags.

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
inspectable before the next step executes Before attribute name or Self-closing
start tag. Before attribute name now reaches boolean and valued attributes,
or destination-state EOF abandonment. Character references in values still block.
Do not resume a blocked execution.
VT and NBSP are name characters, not separators; VT also gets the cursor's
preprocessing diagnostic. Surrogates/noncharacters are preserved as code points,
with their original preprocessing errors, not converted through scalar strings.

**Milestone:** real Data → Tag open / End tag open → Tag name → Data execution
now completes `<p>Hello</p>` and `<DIV>world</DIV>`, emitting start tag, character
data, end tag, and EOF. There is no tree builder, special-element mode selection,
or DOM construction; even an unmatched simple end tag emits without tree-builder
errors. The initial milestone used the explicit selector, two emitted tag variants,
selected-token take/discard, and last-start-tag bookkeeping described above.
Self-closing start tag minimally extends these variants with a flag and reuses
the selected-token emission path; that milestone added no general emission
framework, attribute-value algorithm, or dependency. The fixture adapter compares the
representable fields without suppressing destination stub blocks.

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
matching comment prefix. Only upstream runs contribute to the conformance report.

## Before attribute name coverage

The implementation follows every branch of the pinned **Before attribute name
state** (`#before-attribute-name-state`):

| Consumed input | Source effects and intended destination |
| --- | --- |
| TAB U+0009, LF U+000A, FF U+000C, SPACE U+0020 | Ignore; stay in Before attribute name. CR is normalized to LF by the cursor. |
| `/`, `>`, EOF | Reconsume in After attribute name; no attribute creation, emission, or state diagnostic. |
| `=` U+003D | Report `unexpected-equals-sign-before-attribute-name`; append a new attribute with name `=` and empty value to the current tag; switch to Attribute name **without** reconsumption. |
| Anything else | Append a new attribute with empty name/value to the current tag; reconsume in Attribute name. |

Attribute name and After attribute name now execute on the next step. In
particular, `>` does not emit a tag in Before attribute name itself; the
reconsuming After attribute name emits it. EOF similarly reports `eof-in-tag`,
abandons the selected tag, and emits EOF only on that destination step. NUL,
quotes, uppercase, non-ASCII whitespace, and non-scalars all take the
empty-attribute/reconsume branch; replacement, lowercasing, and attribute-name
diagnostics belong to Attribute name on the next step.
Preprocessing errors occur once on initial consumption, at their original positions.
Open exhaustion repeatedly suspends without effects, never implying EOF.

Focused tests in `crates/html/tests/before_attribute_name_state.rs` inspect both
tag kinds, existing attribute preservation and append order, empty value/name
initialization, exact diagnostics/locations, unchanged unrelated context,
consumption/reconsumption (including conceptual EOF), intended targets and
terminal blocks. They cover all ASCII branches plus Unicode whitespace/letters,
actual surrogates/noncharacters/controls, real Tag name entry, whitespace runs,
and all two-chunk splits plus single-code-point chunks of representative inputs,
including CRLF splits, trailing/consecutive CR, NUL, and downstream blocks.
These local source-step checks are not full tokenizer streaming conformance.

Before attribute name creates pending attribute storage, not emitted tokens.
After attribute name permits boolean-attribute emission; Before attribute value
connects all three value states and After attribute value (quoted) from Data.
Ordinary comments, markup declarations, processing instructions, and character
references remain stubs.

## Attribute name coverage

The implementation follows **Attribute name state** (`#attribute-name-state`)
in the pinned source, including the on-leaving duplicate rule and its
removed-but-still-current note:

| Consumed input | Source effects and intended destination |
| --- | --- |
| TAB, LF, FF, SPACE, `/`, `>`, EOF | Reconsume in After attribute name. CR is normalized to LF by the cursor. |
| `=` | Switch to Before attribute value without appending or reconsumption. |
| ASCII upper alpha | Append code point + 0x20 to the current name; no Unicode folding. |
| NUL | Report `unexpected-null-character`; append U+FFFD. |
| `"`, `'`, `<` | Report `unexpected-character-in-attribute-name`; append the original point. |
| Anything else | Append the point unchanged, including non-scalars. |

Every departure compares the completed name to other retained attributes on the
selected tag using exact code-point equality. A duplicate reports
`duplicate-attribute` at the consumed delimiter/EOF, removes only the new entry,
and retains its identity/name/value in the detached-current slot described above.
This happens before the destination attempt. Both Before attribute value and
After attribute name now execute. NeedMoreInput is not departure and never
checks a prefix for duplicates. No emission, EOF output, `eof-in-tag`, or
`end-tag-with-attributes` diagnostic is supplied by this state.

`crates/html/tests/attribute_name_state.rs` covers every ASCII branch, literal
Unicode/case/normalization distinctions, NUL and punctuation diagnostics,
non-scalars and preprocessing locations, retained versus discarded identity and
values, both tag kinds, and unrelated context preservation. Seeded duplicates
independently inspect detached identity and non-scalar names/values. Shared helper
unit tests independently check duplicate detachment, later current-value mutation,
next-attribute creation, repeated removal protection, tag replacement and take.
Real Data → Tag name → Before attribute name → Attribute name tests inspect
source effects before destination processing and exact EOF abandonment. Every two-chunk split and single-point chunks
match closed-input source-step traces, including name boundaries, duplicate
prefix suspension versus final EOF, CRLF, malformed input, non-scalars, and
blocked targets. These are local state checks, not whole-tokenizer conformance.
Attribute name alone added no upstream passes. After attribute name now makes
boolean-attribute paths comparable; its reviewed identities supplement, never
replace, all prior case-level baselines (including RCDATA and RAWTEXT).
Before attribute value now connects valued-attribute paths from Data.

## Self-closing start tag coverage

The implementation follows every branch of the pinned **Self-closing start tag
state** (`#self-closing-start-tag-state`):

| Consumed input | Source effects and intended destination |
| --- | --- |
| `>` | Set the current tag's self-closing flag; switch to Data and emit it. End tags also report `end-tag-with-trailing-solidus` under the shared Tokenization emission rule. |
| EOF | Report `eof-in-tag`; abandon the unfinished tag and emit one terminal EOF, never the tag. |
| Anything else | Report `unexpected-solidus-in-tag` at the consumed point; leave the flag and pending storage unchanged; reconsume in Before attribute name. |

Open exhausted input suspends without setting the flag or emitting EOF. Recovery
executes Before attribute name and After attribute name. Boolean and valued
attributes can complete; ampersands in values still block at Character reference.
A destination stub is not an HTML parse error or supported resumption.
NUL and Unicode follow the anything-else branch with no replacement or extra
state diagnostic here; cursor preprocessing diagnostics remain intact.

Focused tests in `crates/html/tests/self_closing_start_tag_state.rs` cover both
tag kinds, initially set/unset flags, unrelated context preservation, creation
defaults, shared Tag name emission, exact tokens/diagnostics/normalized locations,
consumption and reconsumption, EOF abandonment, repeated suspension/termination,
and real `<br/>`, non-void start tags, end tags, and malformed-input paths.
Every two-chunk split and single-code-point chunks compare source-step snapshots,
especially around `/` and `>`, CRLF, NUL, Unicode/non-scalars, and destination
blocks. These are state-specific streaming tests, not full conformance.

There is no tree builder or flag acknowledgement. A self-closing start-tag token
does not make an arbitrary HTML element void. Integration tests execute Before
attribute name on recovery and retain explicit destination-block assertions.

## Attribute emission prerequisite coverage

`crates/html/tests/tag_emission_support.rs` initializes local Tag name and
Self-closing start tag contexts to exercise both tag kinds, ordered retained
attributes, detached duplicate exclusion (including detached-only storage),
Unicode/non-scalars, unrelated storage, last-start-tag bookkeeping, flags,
normalized emission positions, error multiplicity/coexistence, suspension, and
EOF abandonment. Shared helper tests check take versus emission and current-value
mutation after duplicate detachment. Test-only adapter regressions in
`crates/html/tests/support/attribute_emission.rs` check attribute-object semantics,
malformed values, duplicate rejection, double escaping, end-tag projection and
exact diagnostics, completion, and continued destination blocks. These are local
support checks, not new upstream conformance executions.

The original prerequisite added no states or dispatch registrations and changed no
upstream fixtures or passing-run baselines. It supplies shared support for six subsequent,
separately reviewed states: After attribute name, Before attribute value,
Attribute value (double-quoted), Attribute value (single-quoted), Attribute value
(unquoted), and After attribute value (quoted). All six are now implemented
separately, and Before attribute value connects Data to the merged value states.
See the state-local evidence for branch mapping and entry limitations:

- [After attribute name](tokenizer-after-attribute-name.md)
- [Before attribute value](tokenizer-before-attribute-value.md)
- [Double-quoted value](tokenizer-attribute-value-double-quoted.md)
- [Single-quoted value](tokenizer-attribute-value-single-quoted.md)
- [Unquoted value](tokenizer-attribute-value-unquoted.md)
- [After quoted value](tokenizer-after-attribute-value-quoted.md)

## Before attribute value and combined attribute coverage

See [branch mapping, EOF boundary, and reviewed fixture identities](tokenizer-before-attribute-value.md).
No value mutation occurs in Before attribute value. EOF is reconsumed in unquoted
value; only that destination abandons the selected tag and emits EOF.
`attribute_value_integration.rs` checks Data-entry values/booleans, retained versus
detached duplicates followed by values/new attributes, no-whitespace recovery,
missing values, literal versus structural solidus, both tag kinds, exact diagnostic
positions/multiplicity, last-start-tag bookkeeping, and terminal ampersand blocks.
The state-local suites preserve source-step snapshots and chunk-splitting traces
through implemented destinations, without resuming stubs. Prior PR-local reports
describe their historical scope; the current combined support is reachable from Data.

Before claiming broader coverage, implement the remaining states,
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
