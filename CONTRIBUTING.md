# Contributing

Neptune is correctness-first and specification-driven. Small, reviewable PRs
are preferred over large generated implementations.

## Setup and checks

Install Rust through rustup, then initialize the pinned fixture checkout:

```sh
git submodule update --init --recursive
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
```

Use `cargo fmt --all` to format changes. The repository toolchain file selects
stable Rust and the required components. CI runs the same checks on Linux and
macOS.

The fixture inventory test fails with setup instructions if the submodule is
missing. It never silently skips missing fixtures. To see its inventory report:

```sh
cargo test --workspace --locked pinned_upstream_fixture_inventory_is_readable -- --nocapture
```

**That report is not a conformance result.** The partial execution runner is separate:

```sh
cargo test --workspace --locked pinned_upstream_runner -- --nocapture
```

It reports one run per initial state, including explicit blocked reasons.
`executed = passed + failed`; dispatch attempts ending at a stub are blocked,
not executed/comparable cases. Synthetic executor tests validate the runner,
not tokenizer conformance. See [supported capabilities](docs/tokenizer.md#fixture-runner).

## Pull requests

Branch from `main`, make a focused change, and open a PR. Prefer one state
per PR; small supporting infrastructure PRs are also appropriate.

For a state implementation:

1. Cite the section in the pinned source recorded in `docs/sources.toml`.
2. Map each specification branch to readable code and focused tests.
3. Cover EOF, parse errors, consumption, and reconsumption where applicable.
4. Test transitions to unimplemented states as explicit blocked execution.
5. Report upstream conformance cases executed, passed, failed, and blocked
   separately once a runner exists. A blocked case is not a passing case.
6. Keep performance changes separate and support them with measurements.

See `docs/tokenizer.md` for the planned infrastructure and dependency boundaries.
Do not implement adjacent states merely to make a state PR appear complete.

### Parallel state work

- Merge prerequisite shared API/storage/adapter changes in small supporting PRs
  **before** starting a wave of states that needs them.
- Assign one state owner per worktree/PR. Keep algorithms and branch evidence in
  state-local files; keep module/dispatch and independent support-inventory edits
  narrow. Use test-local helpers or `..Default::default()` for incidental storage,
  but keep fields under test and creation-default assertions explicit.
- Add reviewed newly passing upstream identities to the
  [pass baseline](crates/html/tests/baselines/README.md); never replace prior
  passes with aggregate counts or auto-bless current execution.
- Rebase after prerequisite/neighbor merges and rerun all checks plus the upstream
  report. A new destination implementation can legitimately move a terminal block;
  review those integration changes rather than hiding them. Shared token types,
  dispatch registration, adapter capabilities, and actual execution paths remain
  shared responsibilities, not a promise of conflict-free development.

## Upstream tests and specification updates

The html5lib submodule is pinned by Git. Do not edit its expected results,
track its moving branch, or run `git submodule update --remote` as routine setup.

Update baselines in a dedicated PR. Review the upstream diff, update both the
submodule gitlink and `docs/sources.toml`, explain changes in expected behavior,
and update the registry deliberately if the specification's states change.

The specification is normative. Fixtures and other browser engines are useful
evidence but can be incomplete or disagree. Document discrepancies rather than
silently adapting the implementation to whichever result is convenient.

## AI-assisted changes

AI-assisted contributions follow exactly the same standard: small scope,
traceable behavior, independent upstream evidence, and human-reviewable diffs.
Generated implementation and generated tests alone are not independent evidence.
Never describe inventory checks, skipped cases, or partial results as full
conformance.

## License

Unless explicitly stated otherwise, contributions intentionally submitted for
inclusion in Neptune are licensed under both MIT and Apache-2.0, allowing
recipients to choose either, without additional terms or conditions.
Preserve upstream licenses and attribution for third-party material.
