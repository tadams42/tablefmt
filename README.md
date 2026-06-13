# tablefmt

[![Release](https://img.shields.io/github/v/release/tadams42/tablefmt)](https://github.com/tadams42/tablefmt/releases/latest)
[![License: MIT](https://img.shields.io/github/license/tadams42/tablefmt)](LICENSE)
[![Build](https://img.shields.io/github/actions/workflow/status/tadams42/tablefmt/release.yml?label=release+build)](https://github.com/tadams42/tablefmt/actions/workflows/release.yml)
[![Built with Rust](https://img.shields.io/badge/built_with-Rust-orange?logo=rust)](https://www.rust-lang.org)

`tablefmt` converts tabular data (CSV, JSON, YAML, TOML, …) into a pretty table in your terminal, with 15+ output styles and optional column/row colorization.

![demo](demo.gif)

## Features

- **Reads CSV, TSV, PSV, JSON, YAML, TOML** — format auto-detected from the file extension
- **15+ output styles** — GitHub Markdown, PostgreSQL, reStructuredText, AsciiDoc, Jira, Reddit, Org-mode, Unicode box-drawing, and more
- **`prettify`** — re-align a misaligned table without touching its content
- **Column or row colorization** in the terminal
- **Decimal normalization** — round all numeric values in a column to a fixed number of decimal places
- **`--max-rows`** — cap output length (default: 20 rows)
- **Shell completions** — bash, zsh, fish, PowerShell, elvish

## Installation

Download a pre-built binary from the [Releases](https://github.com/tadams42/tablefmt/releases/latest) page:

| Platform | Archive |
|---|---|
| Linux x86_64 | `tablefmt-x86_64-unknown-linux-musl.tar.gz` |
| Linux aarch64 | `tablefmt-aarch64-unknown-linux-musl.tar.gz` |
| macOS x86_64 | `tablefmt-x86_64-apple-darwin.tar.gz` |
| macOS aarch64 | `tablefmt-aarch64-apple-darwin.tar.gz` |
| Windows x86_64 | `tablefmt-x86_64-pc-windows-msvc.zip` |

Or build from source:

```sh
cargo install --git https://github.com/tadams42/tablefmt
```

## Usage

### `format` — convert tabular data to a table

```sh
# CSV → GitHub-flavored Markdown (default style)
tablefmt format -i data.csv

# Explicit input format; read from stdin
cat data.json | tablefmt format -s json

# Different output styles
tablefmt format -i data.csv -S psql
tablefmt format -i data.yaml -S rst

# Colorize alternating columns; limit to 50 rows; 2 decimal places
tablefmt format -i data.csv --color columns --max-rows 50 -p 2

# Write to a file
tablefmt format -i data.csv -o table.md
```

### `prettify` — re-align an existing table

```sh
# Re-align a hand-edited Markdown table
tablefmt prettify -S github -i docs/table.md

# Pipe from stdin
cat table.md | tablefmt prettify -S github
```

### `completions` — generate shell completions

```sh
tablefmt completions zsh  > ~/.zfunc/_tablefmt
tablefmt completions bash > /etc/bash_completion.d/tablefmt
tablefmt completions fish > ~/.config/fish/completions/tablefmt.fish
```

## Output styles

### `github` — GitHub-flavored Markdown (default)

```
| Month |  Revenue | Units |
|-------|----------|-------|
| Jan   | 12500.50 |   342 |
| Feb   | 13200.75 |   389 |
| Mar   | 15800.25 |   421 |
```

### `psql` — PostgreSQL `\pset border 2` style

```
 Month |  Revenue | Units 
-------+----------+-------
 Jan   | 12500.50 |   342 
 Feb   | 13200.75 |   389 
 Mar   | 15800.25 |   421 
```

### `asciidoc` — AsciiDoc table

```
[cols="<5,>8,>5",options="header"]
|====
| Month | Revenue  | Units
| Jan   | 12500.50 |   342
| Feb   | 13200.75 |   389
| Mar   | 15800.25 |   421
|====
```

### `jira` — Atlassian Jira wiki table

```
|| Month || Revenue  || Units ||
| Jan   | 12500.50 |   342 |
| Feb   | 13200.75 |   389 |
| Mar   | 15800.25 |   421 |
```

### `rst` — reStructuredText simple table

```
======= ========== =======
 Month    Revenue   Units 
======= ========== =======
 Jan     12500.50     342 
 Feb     13200.75     389 
 Mar     15800.25     421 
======= ========== =======
```

### `rst-grid` — reStructuredText grid table

```
+-------+----------+-------+
| Month | Revenue  | Units |
+=======+==========+=======+
| Jan   | 12500.50 |   342 |
+-------+----------+-------+
| Feb   | 13200.75 |   389 |
+-------+----------+-------+
| Mar   | 15800.25 |   421 |
+-------+----------+-------+
```

### `reddit` — Reddit Markdown table

```
| Month | Revenue  | Units |
| ----- | -------- | ----- |
| Jan   | 12500.50 |   342 |
| Feb   | 13200.75 |   389 |
| Mar   | 15800.25 |   421 |
```

### `table-el` — Emacs `table.el` style

```
+-------+----------+-------+
| Month | Revenue  | Units |
+=======+==========+=======+
| Jan   | 12500.50 |   342 |
+-------+----------+-------+
| Feb   | 13200.75 |   389 |
+-------+----------+-------+
| Mar   | 15800.25 |   421 |
+-------+----------+-------+
```

### `orgtbl` — Emacs Org-mode table

```
| Month | Revenue  | Units |
|-------+----------+-------|
| Jan   | 12500.50 |   342 |
| Feb   | 13200.75 |   389 |
| Mar   | 15800.25 |   421 |
```

### `dots` — Dot-separated ASCII

```
............................
: Month :  Revenue : Units :
:.......:..........:.......:
: Jan   : 12500.50 :   342 :
:.......:..........:.......:
: Feb   : 13200.75 :   389 :
:.......:..........:.......:
: Mar   : 15800.25 :   421 :
:.......:..........:.......:
```

### `ascii` — Plain ASCII

```
+-------+----------+-------+
| Month |  Revenue | Units |
+-------+----------+-------+
| Jan   | 12500.50 |   342 |
+-------+----------+-------+
| Feb   | 13200.75 |   389 |
+-------+----------+-------+
| Mar   | 15800.25 |   421 |
+-------+----------+-------+
```

### `modern` — Unicode box-drawing (thin lines, row separators)

```
┌───────┬──────────┬───────┐
│ Month │  Revenue │ Units │
├───────┼──────────┼───────┤
│ Jan   │ 12500.50 │   342 │
├───────┼──────────┼───────┤
│ Feb   │ 13200.75 │   389 │
├───────┼──────────┼───────┤
│ Mar   │ 15800.25 │   421 │
└───────┴──────────┴───────┘
```

### `sharp` — Unicode box-drawing (thin lines, no row separators)

```
┌───────┬──────────┬───────┐
│ Month │  Revenue │ Units │
├───────┼──────────┼───────┤
│ Jan   │ 12500.50 │   342 │
│ Feb   │ 13200.75 │   389 │
│ Mar   │ 15800.25 │   421 │
└───────┴──────────┴───────┘
```

### `extended` — Unicode box-drawing (double lines)

```
╔═══════╦══════════╦═══════╗
║ Month ║  Revenue ║ Units ║
╠═══════╬══════════╬═══════╣
║ Jan   ║ 12500.50 ║   342 ║
╠═══════╬══════════╬═══════╣
║ Feb   ║ 13200.75 ║   389 ║
╠═══════╬══════════╬═══════╣
║ Mar   ║ 15800.25 ║   421 ║
╚═══════╩══════════╩═══════╝
```

### `heavy-outline` — Unicode box-drawing (heavy outer border)

```
┏━━━━━━━┳━━━━━━━━━━┳━━━━━━━┓
┃ Month ┃ Revenue  ┃ Units ┃
┣━━━━━━━╋━━━━━━━━━━╋━━━━━━━┫
┃ Jan   ┃ 12500.50 ┃   342 ┃
┃ Feb   ┃ 13200.75 ┃   389 ┃
┃ Mar   ┃ 15800.25 ┃   421 ┃
┗━━━━━━━┻━━━━━━━━━━┻━━━━━━━┛
```

## License

MIT — see [LICENSE](LICENSE).
