# HTML tokenizer foundation

## Scope of this scaffold

The `neptune-html` crate currently contains only:

- A state registry with specification names and link fragments.
- A typed `UnimplementedState` error.
- A temporary `dispatch(State)` seam that returns that error for every state.
- Tests for the registry, stubs, and availability/shape of upstream fixtures.

There is **no tokenizer algorithm, input processing, token emission, or
conformance runner yet**. The dispatcher takes no input and must not be mistaken
for a usable tokenizer. Replace its signature when real execution needs context;
do not preserve a placeholder API at the expense of a sound design.

All 84 state names match the pinned specification source in order. This baseline
includes processing-instruction states and has no separate decimal-character-
reference-start state. Use the pinned source rather than older state lists.

## Boundaries to preserve

- Tokenization operates after byte decoding. Networking, encoding sniffing, and
  byte decoders are separate concerns.
- Input preprocessing (including CRLF normalization) is explicit and tested,
  not accidentally delegated to string or line helpers.
- Preserve the distinction between a Unicode code point and a Rust `char`.
  Upstream `doubleEscaped` fixtures can describe lone surrogates; a runner must
  represent them deliberately, not silently replace them or discard tests.
- Design for incremental input: end-of-chunk/need-more-input is not EOF.
  Results must be invariant under chunk boundaries.
- Consumption, reconsumption, and state switching need explicit semantics.
- The tree builder will supply tokenizer context: text/script modes, the last
  start tag for appropriate-end-tag checks, and foreign-content/CDATA context.
  Do not assume the tokenizer is forever a context-free iterator.
- Recoverable HTML parse errors are output diagnostics. An unimplemented state
  is a development limitation, not one of those errors.
- Use safe Rust. Do not optimize storage or dispatch before measuring it.

These are constraints, not a demand to build every abstraction upfront.

## First follow-up PRs

### 1. Execution and test-harness infrastructure

Introduce only the interfaces needed to test the first state:

- Input events/cursor, with EOF distinct from temporarily unavailable input.
- A tested way to retain/reconsume the current character.
- State/context storage and minimal token/parse-error output types.
- Dispatch that errors on entering an unimplemented target. The error records
  that target; it must not imply execution can safely resume after the stub.
- A fixture adapter with explicit supported coverage and blocked-case reporting.

Use synthetic infrastructure tests without implementing any state algorithms.
Keep tree-builder integration and broader token variants for when needed.

### 2. Data state

Implement the pinned Data state literally and test each branch. Keep Tag open
and Character reference as stubs until their own PRs. Tests should distinguish
the source state's completed effects from the target state's blocked execution.

Update the scaffold's blanket all-states-unimplemented assertion deliberately.
Keep an explicit inventory of which states remain stubs; do not derive expected
coverage from the same dispatch behavior being tested.

### 3. Additional states

Choose a small dependency-aware order, such as Tag open, End tag open, and
Tag name. Character-reference support can be its own sequence, with entity
tables generated reproducibly and reviewed separately.

Each state PR should stand on its own tests even when full-document fixtures
cannot finish because a later state is still missing.

## Conformance runner requirements

Before claiming upstream results, implement the conventions in
`tests/html5lib-tests/tokenizer/README.md`, including:

- Defaults and multiple runs for `initialStates`.
- `lastStartTag`, input preprocessing, and `doubleEscaped` handling.
- Character-token coalescing and token field conventions.
- Parse-error codes and one-based line/column positions.
- Separate treatment of `xmlViolationTests`, which use infoset-coercion rules.
- A documented policy for `pendingSpecChanges.test` relative to our pinned spec.

Inventory currently reads all 14 fixture files (6,827 cases at the pinned
revision), including the separately shaped XML-violation data, but executes
**zero** conformance cases. No test is labeled as passing tokenizer conformance.

Once execution exists, print executed/passed/failed/blocked totals and make
regressions in the explicitly supported subset fail CI. Never hide failures by
turning them into skips or expanding a blocklist without review. Later add
chunk-boundary equivalence, fuzzing, and differential checks.

Tree-construction tests are now maintained in WPT's `html/syntax/parsing`;
pin that source when the tree builder is introduced rather than assuming those
fixtures still live in html5lib-tests.

## Provenance and licensing

`docs/sources.toml` records immutable source revisions. The Git submodule pins
the test checkout and retains upstream attribution and licensing.

The HTML Standard is normative; its living URL is a convenience link, not our
revision pin. Consult the immutable GitHub source when reviewing behavior.
This scaffold references the standard and lists state names; it does not vendor
the standard's algorithm prose under Neptune's license.
