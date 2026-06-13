use anyhow::anyhow;

use crate::model::TableData;
use crate::{prettify, table_parse};

/// Returns the 0-based table column index for the given cursor character offset
/// in the original (unstripped) file line.
///
/// Works with any pipe-family separator (|, │, ┃, ║, ·). RST simple tables
/// (space-separated columns) are not supported; the function returns 0 for them.
pub fn find_column_at_cursor(original_line: &str, raw_cursor: usize) -> usize {
    let bare = prettify::bare_line(original_line);
    // Prefix is always ASCII (spaces, tabs, comment chars), so byte offset = char offset.
    let prefix_char_len = original_line.len() - bare.len();
    let cursor_in_bare = raw_cursor.saturating_sub(prefix_char_len);
    let sep = table_parse::detect_separator(bare);
    bare.chars()
        .take(cursor_in_bare)
        .filter(|&c| c == sep)
        .count()
        .saturating_sub(1)
}

/// Inserts an empty column at `col_idx` (0-based). Clamps to the column count.
pub fn add_column(data: TableData, col_idx: usize) -> TableData {
    let idx = col_idx.min(data.headers.len());
    let mut headers = data.headers;
    headers.insert(idx, String::new());
    let rows = data
        .rows
        .into_iter()
        .map(|mut row| {
            let insert_at = idx.min(row.len());
            row.insert(insert_at, String::new());
            row
        })
        .collect();
    TableData::new(headers, rows)
}

/// Removes the column at `col_idx` (0-based). Returns an error if out of range.
pub fn remove_column(data: TableData, col_idx: usize) -> anyhow::Result<TableData> {
    if col_idx >= data.headers.len() {
        return Err(anyhow!(
            "column index {} is out of range (table has {} columns)",
            col_idx,
            data.headers.len()
        ));
    }
    let mut headers = data.headers;
    headers.remove(col_idx);
    let rows = data
        .rows
        .into_iter()
        .map(|mut row| {
            if col_idx < row.len() {
                row.remove(col_idx);
            }
            row
        })
        .collect();
    Ok(TableData::new(headers, rows))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::TableData;

    // ── find_column_at_cursor ────────────────────────────────────────────────

    #[test]
    fn cursor_in_first_cell() {
        // "| col1 | col2 |", cursor at 3 → inside col1 → column 0
        assert_eq!(find_column_at_cursor("| col1 | col2 |", 3), 0);
    }

    #[test]
    fn cursor_in_second_cell() {
        // "| col1 | col2 |", cursor at 10 → inside col2 → column 1
        assert_eq!(find_column_at_cursor("| col1 | col2 |", 10), 1);
    }

    #[test]
    fn cursor_on_separator() {
        // "| col1 | col2 |", cursor at 7 → on the | between col1 and col2 → column 0
        assert_eq!(find_column_at_cursor("| col1 | col2 |", 7), 0);
    }

    #[test]
    fn cursor_before_table_clamps_to_zero() {
        assert_eq!(find_column_at_cursor("| a | b |", 0), 0);
    }

    #[test]
    fn cursor_past_trailing_pipe_returns_out_of_range() {
        // "| a | b |" has 3 pipes; past the trailing | gives count 3 → 3-1 = 2.
        // Callers (add_column) clamp this to the column count; remove_column returns Err.
        assert_eq!(find_column_at_cursor("| a | b |", 999), 2);
    }

    #[test]
    fn cursor_with_comment_prefix() {
        // "// | col1 | col2 |", prefix "// " is 3 bytes
        // cursor at 6 → inside col1 after prefix → column 0
        assert_eq!(find_column_at_cursor("// | col1 | col2 |", 6), 0);
    }

    #[test]
    fn cursor_with_comment_prefix_second_col() {
        // "// | col1 | col2 |", cursor at 14 → inside col2 → column 1
        assert_eq!(find_column_at_cursor("// | col1 | col2 |", 14), 1);
    }

    #[test]
    fn cursor_with_box_drawing_separator() {
        // "│ col1 │ col2 │" — │ is U+2502 (3 bytes, 1 char/UTF-16 unit)
        // cursor at char 3 → inside col1 → column 0
        assert_eq!(find_column_at_cursor("│ col1 │ col2 │", 3), 0);
    }

    // ── add_column ──────────────────────────────────────────────────────────

    fn make_table() -> TableData {
        TableData::new(
            vec!["a".into(), "b".into(), "c".into()],
            vec![
                vec!["1".into(), "2".into(), "3".into()],
                vec!["4".into(), "5".into(), "6".into()],
            ],
        )
    }

    #[test]
    fn add_column_at_start() {
        let t = add_column(make_table(), 0);
        assert_eq!(t.headers, vec!["", "a", "b", "c"]);
        assert_eq!(t.rows[0], vec!["", "1", "2", "3"]);
        assert_eq!(t.rows[1], vec!["", "4", "5", "6"]);
    }

    #[test]
    fn add_column_in_middle() {
        let t = add_column(make_table(), 1);
        assert_eq!(t.headers, vec!["a", "", "b", "c"]);
        assert_eq!(t.rows[0], vec!["1", "", "2", "3"]);
    }

    #[test]
    fn add_column_at_end() {
        let t = add_column(make_table(), 3);
        assert_eq!(t.headers, vec!["a", "b", "c", ""]);
        assert_eq!(t.rows[0], vec!["1", "2", "3", ""]);
    }

    #[test]
    fn add_column_beyond_end_clamps() {
        let t = add_column(make_table(), 999);
        assert_eq!(t.headers.len(), 4);
        assert_eq!(t.headers.last().unwrap(), "");
    }

    // ── remove_column ────────────────────────────────────────────────────────

    #[test]
    fn remove_column_first() {
        let t = remove_column(make_table(), 0).unwrap();
        assert_eq!(t.headers, vec!["b", "c"]);
        assert_eq!(t.rows[0], vec!["2", "3"]);
    }

    #[test]
    fn remove_column_last() {
        let t = remove_column(make_table(), 2).unwrap();
        assert_eq!(t.headers, vec!["a", "b"]);
        assert_eq!(t.rows[0], vec!["1", "2"]);
    }

    #[test]
    fn remove_column_out_of_range_errors() {
        assert!(remove_column(make_table(), 3).is_err());
        assert!(remove_column(make_table(), 99).is_err());
    }

    // ── integration: full prettify pipeline ──────────────────────────────────

    fn run_prettify_pipeline(file_content: &str, line: usize) -> (usize, usize, String, String) {
        use crate::{format, locate, numeric, prettify, table_parse};

        let file_lines: Vec<&str> = file_content.lines().collect();
        let (start, end) = locate::find_table_bounds(&file_lines, line).unwrap();
        let table_str = file_lines[start..=end].join("\n");
        let (bare_lines, meta) = prettify::preprocess(&table_str);
        let line_refs: Vec<&str> = bare_lines.iter().map(String::as_str).collect();
        let style = table_parse::detect_style(&line_refs);
        let mut data = table_parse::parse_table(&line_refs, &style).unwrap();
        numeric::populate_column_meta(&mut data);
        let rendered = format::render(&data, &style, &crate::cli::ColorMode::None, false);
        let rendered = if rendered.ends_with('\n') {
            rendered
        } else {
            rendered + "\n"
        };
        let reformatted = prettify::postprocess(&rendered, &meta);
        let text = reformatted.trim_end_matches('\n').to_string();
        (start, end, text, style.to_string())
    }

    #[test]
    fn pipeline_github_table_in_prose() {
        let file = "# Heading\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\nSome prose";
        let (start, end, text, style) = run_prettify_pipeline(file, 2);
        assert_eq!(start, 2);
        assert_eq!(end, 4);
        assert_eq!(style, "github");
        assert!(text.contains("| a | b |"));
        assert!(text.contains("|---|---|") || text.contains("| - | - |") || text.contains("|--"));
    }

    #[test]
    fn pipeline_commented_github_table() {
        let file = "code:\n// | x  | y |\n// |----|----|";
        let (start, end, text, style) = run_prettify_pipeline(file, 1);
        assert_eq!(start, 1);
        assert_eq!(end, 2);
        assert_eq!(style, "github");
        assert!(text.starts_with("// "), "expected comment prefix, got: {text:?}");
    }

    #[test]
    fn pipeline_rst_table() {
        // Target a data line (line 3 = "spam    42") so find_rst_table_around fires.
        let file = "=====  ===\nitem   qty\n=====  ===\nspam    42\n=====  ===";
        let (start, end, _text, style) = run_prettify_pipeline(file, 3);
        assert_eq!(start, 0);
        assert_eq!(end, 4);
        assert_eq!(style, "rst");
    }
}
