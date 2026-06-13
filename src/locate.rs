use anyhow::anyhow;

use crate::prettify::bare_line;
use crate::table_parse::is_border_line;

/// Returns true if `bare` (a comment-stripped, whitespace-stripped line) looks
/// like part of a table (border or content row with a recognised separator char).
pub fn is_table_line(bare: &str) -> bool {
    if bare.is_empty() {
        return false;
    }
    is_border_line(bare) || bare.contains(['|', '│', '║', '┃', '·'])
}

/// Returns true if `bare` is an RST simple-table separator (only '=' and spaces).
fn is_rst_sep(bare: &str) -> bool {
    !bare.trim().is_empty() && bare.chars().all(|c| c == '=' || c == ' ')
}

/// When the target line is plain text inside an RST table (no '|' chars),
/// search outward for the enclosing '=====' separators and return the full table range.
fn find_rst_table_around(all_lines: &[&str], target: usize) -> Option<(usize, usize)> {
    let sep = |i: usize| is_rst_sep(bare_line(all_lines[i]));

    // Nearest RST separator above and below the target (search up to 20 lines each way)
    let sep_above = (0..target).rev().take(20).find(|&i| sep(i))?;
    let sep_below = (target + 1..all_lines.len()).take(20).find(|&i| sep(i))?;

    // RST simple tables have three separators: top, after-header, bottom.
    // Walk outward from sep_above/sep_below to capture the full range.
    let start = (0..sep_above)
        .rev()
        .take(10)
        .find(|&i| sep(i))
        .unwrap_or(sep_above);
    let end = (sep_below + 1..all_lines.len())
        .take(10)
        .find(|&i| sep(i))
        .unwrap_or(sep_below);

    Some((start, end))
}

/// Given the raw file lines and a 0-based target line number, return the
/// `(start, end)` line indices (inclusive) of the table that contains that line.
///
/// Handles commented tables (comment prefixes are stripped before classification).
/// AsciiDoc `[cols=...]` attribute lines immediately above `|====` are included.
pub fn find_table_bounds(all_lines: &[&str], target: usize) -> anyhow::Result<(usize, usize)> {
    if target >= all_lines.len() {
        return Err(anyhow!(
            "line {} is out of range (file has {} lines)",
            target,
            all_lines.len()
        ));
    }

    let bare = |i: usize| bare_line(all_lines[i]);

    if !is_table_line(bare(target)) {
        // Fallback: check whether the target sits inside an RST simple-table
        return find_rst_table_around(all_lines, target)
            .ok_or_else(|| anyhow!("line {} is not part of a table", target));
    }

    let mut start = target;
    while start > 0 && is_table_line(bare(start - 1)) {
        start -= 1;
    }

    let mut end = target;
    while end + 1 < all_lines.len() && is_table_line(bare(end + 1)) {
        end += 1;
    }

    // Include an AsciiDoc [cols=...] attribute line immediately before the table
    if start > 0 && bare(start - 1).starts_with("[cols=") {
        start -= 1;
    }

    Ok((start, end))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── is_table_line ────────────────────────────────────────────────────────

    #[test]
    fn table_line_github_data_row() {
        assert!(is_table_line("| spam | 42 |"));
    }

    #[test]
    fn table_line_github_separator() {
        assert!(is_table_line("|---|---|"));
    }

    #[test]
    fn table_line_box_drawing_data() {
        assert!(is_table_line("│ spam │ 42 │"));
    }

    #[test]
    fn table_line_box_drawing_border() {
        assert!(is_table_line("├───┼───┤"));
    }

    #[test]
    fn table_line_asciidoc_delimiter() {
        assert!(is_table_line("|===="));
    }

    #[test]
    fn table_line_rst_separator() {
        assert!(is_table_line("=====  ==="));
    }

    #[test]
    fn not_table_line_empty() {
        assert!(!is_table_line(""));
    }

    #[test]
    fn not_table_line_prose() {
        assert!(!is_table_line("Some ordinary prose text here."));
    }

    #[test]
    fn not_table_line_heading() {
        assert!(!is_table_line("# My Heading"));
    }

    // ── find_table_bounds ────────────────────────────────────────────────────

    fn bounds(file: &str, target: usize) -> (usize, usize) {
        let lines: Vec<&str> = file.lines().collect();
        find_table_bounds(&lines, target).unwrap()
    }

    #[test]
    fn bounds_github_table_in_prose_file() {
        let file = "# Heading\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\nSome prose";
        // target on the header row (line 2, 0-based)
        assert_eq!(bounds(file, 2), (2, 4));
    }

    #[test]
    fn bounds_target_on_separator_row() {
        let file = "# Heading\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\nSome prose";
        assert_eq!(bounds(file, 3), (2, 4));
    }

    #[test]
    fn bounds_target_on_last_data_row() {
        let file = "# Heading\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\nSome prose";
        assert_eq!(bounds(file, 4), (2, 4));
    }

    #[test]
    fn bounds_table_at_start_of_file() {
        let file = "| a | b |\n|---|---|\n| 1 | 2 |\n\nSome prose";
        assert_eq!(bounds(file, 0), (0, 2));
    }

    #[test]
    fn bounds_table_at_end_of_file() {
        let file = "Prose\n\n| a | b |\n|---|---|\n| 1 | 2 |";
        assert_eq!(bounds(file, 4), (2, 4));
    }

    #[test]
    fn bounds_error_on_prose_line() {
        let file = "# Heading\n\n| a | b |\n\nSome prose";
        let lines: Vec<&str> = file.lines().collect();
        assert!(find_table_bounds(&lines, 0).is_err()); // heading
        assert!(find_table_bounds(&lines, 4).is_err()); // prose
    }

    #[test]
    fn bounds_error_out_of_range() {
        let lines = vec!["| a |"];
        assert!(find_table_bounds(&lines, 99).is_err());
    }

    #[test]
    fn bounds_rst_content_line_expands_to_full_table() {
        let file = "Prose\n\n=====  ===\nitem   qty\n=====  ===\nspam    42\n=====  ===\n\nMore";
        // target on a content line (line 3 = "item   qty") that has no '|'
        assert_eq!(bounds(file, 3), (2, 6));
    }

    #[test]
    fn bounds_rst_data_line_expands_to_full_table() {
        let file = "=====  ===\nitem   qty\n=====  ===\nspam    42\n=====  ===";
        // target on data line "spam    42"
        assert_eq!(bounds(file, 3), (0, 4));
    }

    #[test]
    fn bounds_asciidoc_cols_line_included() {
        let file = "Text\n\n[cols=\"<4,>3\",options=\"header\"]\n|====\n| item | qty\n| spam |  42\n|====\n\nMore";
        // target on data row (line 4)
        assert_eq!(bounds(file, 4), (2, 6)); // includes [cols=...] at line 2
    }

    #[test]
    fn bounds_commented_table() {
        let file = "// | a | b |\n// |---|---|\n// | 1 | 2 |";
        assert_eq!(bounds(file, 1), (0, 2));
    }
}
