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
  [Before attribute value](docs/tokenizer-before-attribute-value.md) connects the
  merged attribute states: Data entry now completes boolean, double-/single-quoted,
  and unquoted attributes, including duplicate removal and malformed recovery.
  Character reference remains a terminal stub, including inside attribute values.
  Shared tag emission and the test adapter preserve/compare attributes and diagnostics.
- Explicit RCDATA and RAWTEXT entry now recognizes appropriate closing tags,
  handles literal mismatches/EOF, and returns to Data through existing tag support.
  RAWTEXT ampersands stay literal. No automatic element-based mode switching or
  tree-builder behavior is implied; see [RAWTEXT integration evidence](docs/tokenizer-rawtext-end-tag-name.md).
- Explicit Script data entry now recognizes appropriate closing tags and returns
  to Data, with literal mismatch/EOF recovery and existing attribute/tag support.
  Basic comment-like script text (`<!--text-->`) now traverses the escape-start
  and escaped/dash/dash-dash states, including NUL recovery and escaped EOF
  diagnostics; `-->` returns to Script data. An escaped `<` still stops terminally
  at unimplemented Script data escaped less-than sign; escaped end tags and
  double escaping are not reachable from Script data entry
  ([integration evidence](docs/tokenizer-script-data-escape-start.md)).
  Isolated [Double escape start](docs/tokenizer-script-data-double-escape-start.md)
  entry now accumulates ASCII-normalized scratch and selects the real double-escaped
  loop at exact `script` delimiters, or Escaped otherwise.
  Isolated [double escaped](docs/tokenizer-script-data-double-escaped.md)
  entry now completes the character/dash/dash-dash loop and reaches
  [double escaped less-than sign](docs/tokenizer-script-data-double-escaped-less-than-sign.md),
  then [Double escape end](docs/tokenizer-script-data-double-escape-end.md).
  Matching `script` scratch returns to Escaped; `>` in double escaped dash dash
  returns to Script data.
  Isolated [escaped end tag name](docs/tokenizer-script-data-escaped-end-tag-name.md)
  entry supports appropriate closing tags with initialized End/scratch context;
  its escaped-less-than-sign and escaped-end-tag-open predecessors remain stubs. No
  automatic mode switching, comment-token emission, or JavaScript parsing is implied; see
  [Script data integration evidence](docs/tokenizer-script-data-end-tag-name.md).
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
