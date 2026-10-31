# Working on Neptune

Read README.md, CONTRIBUTING.md, docs/tokenizer.md, and docs/sources.toml first.

- Correctness and safety before performance. Use safe Rust.
- Prefer one tokenizer state per PR; shared infrastructure gets its own small PR.
- Implement behavior from the pinned specification, not recollection or another engine.
- Keep specification branches and state transitions easy to trace in code.
- A transition into an unimplemented state must report UnimplementedState.
  Never silently skip it, emit EOF, panic, or report a recoverable HTML parse error.
- Test EOF, malformed input, parse errors, and reconsumption where applicable.
- Never modify upstream fixtures to make an implementation pass.
- Report executed/passed/failed/blocked conformance cases separately. Fixture
  inventory tests do not execute tokenizer conformance tests.
- Do not claim full conformance or browser security from passing a partial suite.
- Add abstractions and dependencies only when the current work needs them.
- Run the checks in CONTRIBUTING.md before proposing a commit.
