# Dev notes

## Releasing new version

### 1. update release docs

First, make sure git work tree is clean - commit or stash all the changes. Then, ensure
`CHANGELOG.md` is up to date:

```sh
cargo xtask update-changelog
```

Edit `CHANGELOG.md` if needed.

⚠️ Do not change `## unreleased` section header, this will be handled automatically by
release process. If `## unreleased` is missing from `CHANGELOG.md`, `cargo xtask
update-changelog` will fail. The heading must be exactly `## unreleased` (lowercase, no
trailing whitespace).

Once satisfied, commit your changes, if there are any.

```sh
git add CHANGELOG.md
git commit -m "build: updated docs for next release."
```

⚠️ `git` message for this commit must be prefixed by `build:`

### 2. Run cargo-release

```sh
cargo release patch             # or: minor / major; this is dry run for preview only
cargo release patch --execute   # or: minor / major; actually updates version
```

This produces a **single commit** that:
- Renames `## unreleased` → `## vX.Y.Z (YYYY-MM-DD)` in `CHANGELOG.md` and re-seeds a
  blank `## unreleased` heading for the next cycle
- Bumps the version in `Cargo.toml` / `Cargo.lock`
- Creates the git tag `vX.Y.Z`

The commit message will be `build: bumped version to vX.Y.Z`.

### 3. Push

```sh
git push && git push --tags
```

This pushes both the commit and the tag. The `GitHub Actions` release workflow fires on
the tag, extracts the `## vX.Y.Z (...)` section from `CHANGELOG.md` as the release body,
and appends a **Full Changelog** comparison link.

`push = false` in the workspace `Cargo.toml` means `cargo-release` never pushes
automatically. Always use `git push --follow-tags`.

If `CHANGELOG.md` does not contain a `## vX.Y.Z (...)` section matching the pushed tag,
the `GitHub` release is created with an empty body. If you'd followed above instructions
correctly, this should never happen.

## Regenerating the demo GIF

The asciinema cast and GIF live in `docs/demo/` (gitignored). To regenerate:

```sh
python3 scripts/gen-demo.py
agg docs/demo/tablefmt-demo.cast docs/demo/tablefmt-demo.gif
```

`gen-demo.py` builds the binary if needed, runs each demo command against the real binary,
and writes `docs/demo/tablefmt-demo.cast`. `agg` then renders it to a GIF.

Example input files used by the script are tracked in `docs/demo/data/`.

## `musl` build locally

```sh
sudo apt install musl-tools
rustup target add x86_64-unknown-linux-musl
```

and add this to `./cargo/config.toml`:

```toml
[target.x86_64-unknown-linux-musl]
# linker = "x86_64-linux-musl-gcc" or just "musl-gcc" on Linux hosts
linker = "musl-gcc"
```

Finally:

```sh
cargo build --release --target x86_64-unknown-linux-musl
```
