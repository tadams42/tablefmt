# tablefmt

[![Release](https://img.shields.io/github/v/release/tadams42/tablefmt)](https://github.com/tadams42/tablefmt/releases/latest)
[![License: MIT](https://img.shields.io/github/license/tadams42/tablefmt)](LICENSE)
[![Build](https://img.shields.io/github/actions/workflow/status/tadams42/tablefmt/release.yml?label=release+build)](https://github.com/tadams42/tablefmt/actions/workflows/release.yml)
[![Built with Rust](https://img.shields.io/badge/built_with-Rust-orange?logo=rust)](https://www.rust-lang.org)

`tablefmt` converts tabular data (CSV, JSON, YAML, TOML, …) into a pretty table in your terminal, with 15+ output styles and optional column/row colorization.

![demo](demo.gif)

> Built with [tabled](https://github.com/zhiburt/tabled)

## Features

- **Reads CSV, TSV, PSV, JSON, YAML, TOML** — format auto-detected from the file extension
- **15+ output styles** — GitHub Markdown, PostgreSQL, reStructuredText, AsciiDoc, Jira, Org-mode, Unicode box-drawing, and more
- **`edit`** — editor API: re-align a table or add/remove columns; outputs a JSON replacement payload for buffer-precise edits
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
cat data.json | tablefmt format -f json

# Different output styles
tablefmt format -i data.csv -s psql
tablefmt format -i data.yaml -s rst

# Colorize alternating columns; limit to 50 rows; 2 decimal places
tablefmt format -i data.csv --color columns --max-rows 50 -N 2

# Write to a file
tablefmt format -i data.csv -o table.md
```

### `edit` — table editing for editor integration

Outputs a JSON payload `{"start_line", "end_line", "text", "style"}` so that editors can replace the table lines in their own buffer without tablefmt touching the file. Line numbers are 0-based; `--col` is the cursor's character offset within that line.

```sh
# Re-align the table that contains line 3
tablefmt edit prettify docs/table.md --line 3

# Insert an empty column before/after the column under the cursor
tablefmt edit add-column-before docs/table.md --line 3 --col 12
tablefmt edit add-column-after  docs/table.md --line 3 --col 12

# Remove the column under the cursor
tablefmt edit remove-column docs/table.md --line 3 --col 12
```

The style is always auto-detected from the table content. On error (e.g. no table found at the given line) exit code is 1 and stdout is `{"error": "…"}`.

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

### `orgtbl` — Emacs Org-mode table

Like `github`, but the separator row uses `+` at column junctions instead of `-`.
Emacs org-mode requires the `+` to recognize column boundaries when re-aligning
a table with `TAB`.

```
| Month |  Revenue | Units |
|-------+----------+-------|   ← + at column crossings (not -)
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
