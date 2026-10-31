# Neptune

An independent, open-source web browser written in Rust.

**Correctness and safety first; performance follows measured, tested behavior.**

Neptune is at the very beginning of development. It is not a usable browser.

## Current status

The first component is a specification-first HTML tokenizer.

- Rust workspace and CI on Linux/macOS.
- Registry of all 84 tokenizer states in the pinned HTML Standard.
- [Partial tokenizer scope](docs/tokenizer.md#status-and-source), with typed
  `UnimplementedState` stubs for unsupported states.
  After attribute name and attribute-value states are still stubs;
  tags with whitespace/attributes cannot complete. Shared tag emission and the
  test adapter preserve/compare attributes, but those paths are not yet reachable.
- Simple-tag milestone: `<p>Hello</p>` and `<DIV>world</DIV>` tokenize completely
  (standalone tokenizer only, not tree construction).
- Pinned upstream html5lib tokenizer fixtures, retaining their original license.
- Tested incremental code-point cursor, preprocessing, and execution scaffolding.
- Partial upstream runner with case-level pass regression protection. Later-state
  blocks are not passes; this is not full tokenizer conformance.
  See [runner support, reporting command, and count definitions](docs/tokenizer.md#fixture-runner)
  for reproducible current results.

The public Rust surface is temporary scaffolding, not a stable parsing API.

## Getting started

Install [Rust through rustup](https://rustup.rs/), then:

```sh
git clone --recurse-submodules https://github.com/olukowski/neptune.git
cd neptune
cargo test --workspace --locked
```

For an existing checkout:

```sh
git submodule update --init --recursive
```

The workspace uses stable Rust with rustfmt and Clippy. No minimum supported
Rust version is promised yet. `Cargo.lock` is committed for reproducible dependency
resolution.

## Development

Use small pull requests, normally one tokenizer state per PR. Transitions to
states not implemented yet must stop explicitly, never silently succeed.

- [Contributing and local checks](CONTRIBUTING.md)
- [Tokenizer design boundaries and first PRs](docs/tokenizer.md)
- [Pinned specification and test sources](docs/sources.toml)

## License

Neptune is licensed under either the [MIT license](LICENSE-MIT) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option.

Third-party material retains its own license and attribution. In particular,
the html5lib fixture submodule is MIT-licensed by its upstream authors; see
[its license](tests/html5lib-tests/LICENSE).
