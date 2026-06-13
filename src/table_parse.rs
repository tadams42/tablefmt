use anyhow::anyhow;

use crate::cli::OutputFormat;
use crate::model::TableData;

pub fn parse_table(lines: &[&str], style: &OutputFormat) -> anyhow::Result<TableData> {
    match style {
        OutputFormat::Rst => parse_rst_table(lines),
        OutputFormat::Asciidoc => {
            let filtered: Vec<&str> = lines
                .iter()
                .filter(|line| !line.trim_start().starts_with("[cols="))
                .copied()
                .collect();
            parse_pipe_table(&filtered)
        }
        _ => parse_pipe_table(lines),
    }
}

// A line is a border/separator line if every character is whitespace, an ASCII
// structural char, or falls in the Unicode box-drawing block (U+2500..=U+257F).
// Content chars (letters, digits, punctuation outside the set) mark data lines.
fn is_border_line(line: &str) -> bool {
    line.chars().all(|c| {
        matches!(c, ' ' | '\t' | '-' | '=' | '+' | '|' | '·')
            || ('\u{2500}'..='\u{257F}').contains(&c)
    })
}

fn detect_separator(line: &str) -> char {
    const CANDIDATES: &[char] = &['│', '┃', '║', '·', '|'];
    CANDIDATES
        .iter()
        .map(|&c| (c, line.chars().filter(|&ch| ch == c).count()))
        .filter(|(_, count)| *count > 0)
        .max_by_key(|(_, count)| *count)
        .map(|(c, _)| c)
        .unwrap_or('|')
}

fn split_cells(line: &str, sep: char) -> Vec<String> {
    line.split(sep)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

fn parse_pipe_table(lines: &[&str]) -> anyhow::Result<TableData> {
    let data_lines: Vec<&str> = lines
        .iter()
        .copied()
        .filter(|line| !line.trim().is_empty() && !is_border_line(line))
        .collect();

    let Some(&first) = data_lines.first() else {
        return Err(anyhow!("no data rows found in table"));
    };

    let sep = detect_separator(first);
    let headers = split_cells(first, sep);

    if headers.is_empty() {
        return Err(anyhow!("table header row is empty"));
    }

    let n_cols = headers.len();
    let rows: Vec<Vec<String>> = data_lines[1..]
        .iter()
        .map(|line| {
            let mut cells = split_cells(line, sep);
            cells.resize(n_cols, String::new());
            cells
        })
        .collect();

    Ok(TableData::new(headers, rows))
}

fn parse_rst_table(lines: &[&str]) -> anyhow::Result<TableData> {
    let is_rst_sep = |line: &str| -> bool {
        !line.trim().is_empty() && line.chars().all(|c| c == '=' || c == ' ')
    };

    let sep_indices: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| is_rst_sep(line))
        .map(|(i, _)| i)
        .collect();

    if sep_indices.len() < 2 {
        return Err(anyhow!(
            "RST table requires at least 2 separator lines, found {}",
            sep_indices.len()
        ));
    }

    // Compute column char-ranges from the first separator line.
    // char-indexed (not byte-indexed) to handle multi-byte content.
    let first_sep_chars: Vec<char> = lines[sep_indices[0]].chars().collect();
    let mut col_ranges: Vec<(usize, usize)> = Vec::new();
    let mut i = 0;
    while i < first_sep_chars.len() {
        if first_sep_chars[i] == '=' {
            let start = i;
            while i < first_sep_chars.len() && first_sep_chars[i] == '=' {
                i += 1;
            }
            col_ranges.push((start, i));
        } else {
            i += 1;
        }
    }

    if col_ranges.is_empty() {
        return Err(anyhow!("no columns found in RST separator line"));
    }

    let extract = |line: &str| -> Vec<String> {
        let chars: Vec<char> = line.chars().collect();
        col_ranges
            .iter()
            .map(|&(start, end)| {
                // Clamp to actual length — preprocess may strip leading whitespace
                // that the RST format uses as cell padding, shortening data lines
                // relative to the separator-derived column positions.
                let s = start.min(chars.len());
                let e = end.min(chars.len());
                chars[s..e].iter().collect::<String>().trim().to_string()
            })
            .collect()
    };

    let header_line = lines
        .get((sep_indices[0] + 1)..sep_indices[1])
        .unwrap_or(&[])
        .iter()
        .find(|line| !line.trim().is_empty())
        .copied();

    let Some(header_line) = header_line else {
        return Err(anyhow!("no header row found in RST table"));
    };

    let headers = extract(header_line);
    let n_cols = headers.len();
    let last_sep = *sep_indices.last().unwrap();

    let rows: Vec<Vec<String>> = lines
        .get((sep_indices[1] + 1)..last_sep)
        .unwrap_or(&[])
        .iter()
        .copied()
        .filter(|line| !line.trim().is_empty() && !is_rst_sep(line))
        .map(|line| {
            let mut cells = extract(line);
            cells.resize(n_cols, String::new());
            cells
        })
        .collect();

    Ok(TableData::new(headers, rows))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── border-line detection ────────────────────────────────────────────────

    #[test]
    fn is_border_github_sep() {
        assert!(is_border_line("|---|---|"));
    }

    #[test]
    fn is_border_ascii_sep() {
        assert!(is_border_line("+-----+-----+"));
    }

    #[test]
    fn is_border_box_drawing() {
        assert!(is_border_line("├───┼───┤"));
    }

    #[test]
    fn is_border_rst_sep() {
        assert!(is_border_line("=====  ====="));
    }

    #[test]
    fn is_border_psql_sep() {
        assert!(is_border_line("------+------"));
    }

    #[test]
    fn is_border_heavy_outline_sep() {
        assert!(is_border_line("┣━━━━━━━╋━━━━━━━┫"));
    }

    #[test]
    fn not_border_has_content() {
        assert!(!is_border_line("| spam | 42 |"));
    }

    #[test]
    fn not_border_box_drawing_with_content() {
        assert!(!is_border_line("│ spam │ 42 │"));
    }

    // ── generic pipe-table parser ────────────────────────────────────────────

    fn lines(s: &str) -> Vec<&str> { s.lines().collect() }

    #[test]
    fn parse_github_two_cols() {
        let input = lines("| item | qty |\n|------|-----|\n| spam | 42 |\n| eggs | 451 |");
        let data = parse_pipe_table(&input).unwrap();
        assert_eq!(data.headers, ["item", "qty"]);
        assert_eq!(data.rows[0], ["spam", "42"]);
        assert_eq!(data.rows[1], ["eggs", "451"]);
    }

    #[test]
    fn parse_psql_no_leading_pipe() {
        let input = lines(" item | qty \n------+-----\n spam |  42 \n eggs | 451 ");
        let data = parse_pipe_table(&input).unwrap();
        assert_eq!(data.headers, ["item", "qty"]);
        assert_eq!(data.rows[0], ["spam", "42"]);
    }

    #[test]
    fn parse_jira_double_pipe_header_collapses_to_cells() {
        let input = lines("|| item || qty ||\n| spam |  42 |");
        let data = parse_pipe_table(&input).unwrap();
        assert_eq!(data.headers, ["item", "qty"]);
        assert_eq!(data.rows[0], ["spam", "42"]);
    }

    #[test]
    fn parse_orgtbl_cross_in_separator_is_skipped() {
        let input = lines("| item | qty |\n|------+------|\n| spam | 42 |");
        let data = parse_pipe_table(&input).unwrap();
        assert_eq!(data.headers, ["item", "qty"]);
        assert_eq!(data.rows.len(), 1);
    }

    #[test]
    fn parse_extra_whitespace_trimmed() {
        let input = lines("|  item   |   qty  |\n|---------|--------|\n|spam   |42|");
        let data = parse_pipe_table(&input).unwrap();
        assert_eq!(data.headers, ["item", "qty"]);
        assert_eq!(data.rows[0], ["spam", "42"]);
    }

    #[test]
    fn parse_box_drawing_modern() {
        let input = lines("│ item │ qty │\n├──────┼─────┤\n│ spam │  42 │");
        let data = parse_pipe_table(&input).unwrap();
        assert_eq!(data.headers, ["item", "qty"]);
        assert_eq!(data.rows[0], ["spam", "42"]);
    }

    // ── RST simple-table parser ──────────────────────────────────────────────

    #[test]
    fn parse_rst_two_cols() {
        let input = lines("=====  ===\nitem   qty\n=====  ===\nspam    42\neggs   451\n=====  ===");
        let data = parse_rst_table(&input).unwrap();
        assert_eq!(data.headers, ["item", "qty"]);
        assert_eq!(data.rows[0], ["spam", "42"]);
        assert_eq!(data.rows[1], ["eggs", "451"]);
    }

    #[test]
    fn parse_rst_right_aligned_content_trimmed() {
        let input = lines("======  ===\n  item  qty\n======  ===\n  spam   42\n======  ===");
        let data = parse_rst_table(&input).unwrap();
        assert_eq!(data.headers, ["item", "qty"]);
        assert_eq!(data.rows[0], ["spam", "42"]);
    }

    #[test]
    fn parse_rst_leading_space_stripped_by_preprocess() {
        // RST data lines have a 1-char RST-inherent leading space that preprocess
        // strips; the clamped slice must still recover the column content.
        let input = lines(
            "====== =====\nitem   qty \n====== =====\nspam    42 \neggs   451 \n====== =====",
        );
        let data = parse_rst_table(&input).unwrap();
        assert_eq!(data.headers, ["item", "qty"]);
        assert_eq!(data.rows[0], ["spam", "42"]);
    }

    #[test]
    fn parse_rst_insufficient_separators_returns_error() {
        let input = lines("=====\nitem\n");
        assert!(parse_rst_table(&input).is_err());
    }

    // ── AsciiDoc parser ──────────────────────────────────────────────────────

    #[test]
    fn parse_asciidoc_cols_line_is_not_phantom_header() {
        let input =
            lines("[cols=\"<4,>3\",options=\"header\"]\n|====\n| item | qty\n| spam |  42\n|====");
        let data = parse_table(&input, &OutputFormat::Asciidoc).unwrap();
        assert_eq!(data.headers, ["item", "qty"]);
    }

    #[test]
    fn parse_asciidoc_data_rows_correct() {
        let input = lines(
            "[cols=\"<4,>3\",options=\"header\"]\n|====\n| item | qty\n| spam |  42\n| eggs | 451\n|====",
        );
        let data = parse_table(&input, &OutputFormat::Asciidoc).unwrap();
        assert_eq!(data.headers, ["item", "qty"]);
        assert_eq!(data.rows[0], ["spam", "42"]);
        assert_eq!(data.rows[1], ["eggs", "451"]);
    }
}
