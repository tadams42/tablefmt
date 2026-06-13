# tabulate-rs

## Build & run

```sh
cargo build
cargo run -- --help
cargo run -- completions zsh
```

## Code conventions

1. format and lint the code

```sh
cargo +nightly fmt
cargo check --workspace
cargo clippy --no-deps
```

2. `git` commit messages should use past tense (`added foobar` instead of `add foobar`,
  `adding foobar` or `adds foobar`)

3. `git` commit messages should be prefixed by short category like `refact:`, `build:`,
  `ci:`, `feat:`, `docs:`, `chore:` and similar

4. after making new `git` commit, run `cargo xtask update-changelog` and then amend that
   last commit with changes created in `CHANGELOG.md` (if there are any)
