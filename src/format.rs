use tabled::builder::Builder;
use tabled::settings::object::Columns;
use tabled::settings::style::HorizontalLine;
use tabled::settings::{Alignment, Style};

use crate::cli::{ColorMode, OutputFormat};
use crate::model::TableData;

// Must be defined before use (macro_rules! has textual scoping).
macro_rules! tabled_style {
    ($data:expr, $style:expr, $color:expr, $is_tty:expr) => {{
        let mut table = build_table($data);
        table.with($style);
        finish_table(table, $data, $color, $is_tty)
    }};
}

pub fn render(data: &TableData, fmt: &OutputFormat, color: &ColorMode, is_tty: bool) -> String {
    match fmt {
        // Tier 1: direct tabled presets
        OutputFormat::Github => tabled_style!(data, Style::markdown(), color, is_tty),
        OutputFormat::Psql => tabled_style!(data, Style::psql(), color, is_tty),
        OutputFormat::Rst => tabled_style!(data, Style::re_structured_text(), color, is_tty),
        OutputFormat::Dots => tabled_style!(data, Style::dots(), color, is_tty),
        OutputFormat::Ascii => tabled_style!(data, Style::ascii(), color, is_tty),
        OutputFormat::Modern => tabled_style!(data, Style::modern(), color, is_tty),
        OutputFormat::Sharp => tabled_style!(data, Style::sharp(), color, is_tty),
        OutputFormat::Extended => tabled_style!(data, Style::extended(), color, is_tty),

        // Custom renderers
        OutputFormat::Jira => {
            let widths = col_widths(data);
            render_jira(data, &widths)
        }
        OutputFormat::Asciidoc => {
            let widths = col_widths(data);
            render_asciidoc(data, &widths)
        }
        OutputFormat::Orgtbl => tabled_style!(
            data,
            Style::empty()
                .left('|')
                .right('|')
                .vertical('|')
                .horizontals([(1, HorizontalLine::full('-', '+', '|', '|'))]),
            color,
            is_tty
        ),
        OutputFormat::RstGrid => tabled_style!(
            data,
            Style::empty()
                .top('-')
                .bottom('-')
                .left('|')
                .right('|')
                .vertical('|')
                .horizontal('-')
                .corner_top_left('+')
                .corner_top_right('+')
                .corner_bottom_left('+')
                .corner_bottom_right('+')
                .intersection_top('+')
                .intersection_bottom('+')
                .intersection_left('+')
                .intersection_right('+')
                .intersection('+')
                .horizontals([(1, HorizontalLine::full('=', '+', '+', '+'))]),
            color,
            is_tty
        ),
        OutputFormat::HeavyOutline => tabled_style!(
            data,
            Style::empty()
                .top('━')
                .bottom('━')
                .left('┃')
                .right('┃')
                .vertical('┃')
                .corner_top_left('┏')
                .corner_top_right('┓')
                .corner_bottom_left('┗')
                .corner_bottom_right('┛')
                .intersection_top('┳')
                .intersection_bottom('┻')
                .horizontals([(1, HorizontalLine::full('━', '╋', '┣', '┫'))]),
            color,
            is_tty
        ),
    }
}

// --- tabled-based rendering ---

fn build_table(data: &TableData) -> tabled::Table {
    let mut builder = Builder::new();
    builder.push_record(data.headers.iter());
    for row in &data.rows {
        builder.push_record(row.iter());
    }
    builder.build()
}

fn finish_table(
    mut table: tabled::Table, data: &TableData, color: &ColorMode, is_tty: bool,
) -> String {
    for (i, meta) in data.column_meta.iter().enumerate() {
        if meta.is_numeric {
            table.modify(Columns::one(i), Alignment::right());
        }
    }
    if is_tty {
        crate::color::apply(&mut table, color, data.headers.len(), data.rows.len());
    }
    table.to_string()
}

// --- custom string renderers helper ---

pub fn col_widths(data: &TableData) -> Vec<usize> {
    data.headers
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let header_w = h.chars().count();
            let data_w = data
                .rows
                .iter()
                .map(|row| row[i].chars().count())
                .max()
                .unwrap_or(0);
            header_w.max(data_w)
        })
        .collect()
}

fn fmt_cell(cell: &str, width: usize, right_align: bool) -> String {
    if right_align {
        format!("{cell:>width$}")
    } else {
        format!("{cell:<width$}")
    }
}

// --- fully custom string renderers ---

fn render_jira(data: &TableData, widths: &[usize]) -> String {
    let mut out = String::new();
    let aligns: Vec<bool> = data.column_meta.iter().map(|m| m.is_numeric).collect();

    // Header row: || col || col ||
    out.push_str("||");
    for (i, (h, &w)) in data.headers.iter().zip(widths.iter()).enumerate() {
        out.push(' ');
        // headers always left-aligned
        out.push_str(&fmt_cell(h, w, false));
        out.push_str(" ||");
        let _ = (i, aligns[i]); // suppress unused warning
    }
    out.push('\n');

    // Data rows: | cell | cell |
    for row in &data.rows {
        out.push('|');
        for (i, (cell, &w)) in row.iter().zip(widths.iter()).enumerate() {
            out.push(' ');
            out.push_str(&fmt_cell(cell, w, aligns[i]));
            out.push_str(" |");
        }
        out.push('\n');
    }

    out
}

fn render_asciidoc(data: &TableData, widths: &[usize]) -> String {
    let aligns: Vec<bool> = data.column_meta.iter().map(|m| m.is_numeric).collect();
    let mut out = String::new();

    // [cols="<w,>w",options="header"]
    let cols_spec: Vec<String> = widths
        .iter()
        .enumerate()
        .map(|(i, &w)| {
            let a = if aligns[i] { '>' } else { '<' };
            format!("{a}{w}")
        })
        .collect();
    out.push_str(&format!("[cols=\"{}\",options=\"header\"]\n", cols_spec.join(",")));
    out.push_str("|====\n");

    // Header row
    out.push_str(&asciidoc_row(&data.headers, widths, &vec![false; widths.len()]));

    // Data rows
    for row in &data.rows {
        out.push_str(&asciidoc_row(row, widths, &aligns));
    }

    out.push_str("|====\n");
    out
}

fn asciidoc_row(cells: &[String], widths: &[usize], aligns: &[bool]) -> String {
    let mut s = String::new();
    for (i, (cell, &w)) in cells.iter().zip(widths.iter()).enumerate() {
        s.push_str("| ");
        s.push_str(&fmt_cell(cell, w, aligns[i]));
        if i + 1 < cells.len() {
            s.push(' ');
        }
    }
    s.push('\n');
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ColumnMeta, TableData};

    fn make_data(headers: &[&str], rows: &[&[&str]], numeric: &[bool]) -> TableData {
        let mut data = TableData::new(
            headers.iter().map(|s| s.to_string()).collect(),
            rows.iter()
                .map(|r| r.iter().map(|s| s.to_string()).collect())
                .collect(),
        );
        for (i, &is_num) in numeric.iter().enumerate() {
            data.column_meta[i] = ColumnMeta {
                is_numeric:         is_num,
                max_decimal_places: 0,
            };
        }
        data
    }

    #[test]
    fn render_jira_header_uses_double_pipe_delimiter() {
        let data = make_data(&["item", "qty"], &[&["spam", "42"]], &[false, true]);
        let widths = col_widths(&data);
        let out = render_jira(&data, &widths);
        let first_line = out.lines().next().unwrap();
        assert!(first_line.starts_with("||"));
        assert!(first_line.ends_with("||"));
    }

    #[test]
    fn render_jira_data_row_uses_single_pipe() {
        let data = make_data(&["item", "qty"], &[&["spam", "42"]], &[false, true]);
        let widths = col_widths(&data);
        let out = render_jira(&data, &widths);
        let second_line = out.lines().nth(1).unwrap();
        assert!(second_line.starts_with('|'));
        assert!(!second_line.starts_with("||"));
    }

    #[test]
    fn render_asciidoc_uses_four_equals_delimiter() {
        let data = make_data(&["item", "qty"], &[&["spam", "42"]], &[false, true]);
        let widths = col_widths(&data);
        let out = render_asciidoc(&data, &widths);
        assert!(out.contains("|====\n"));
    }

    #[test]
    fn render_asciidoc_cols_line_present() {
        let data = make_data(&["item", "qty"], &[&["spam", "42"]], &[false, true]);
        let widths = col_widths(&data);
        let out = render_asciidoc(&data, &widths);
        let first_line = out.lines().next().unwrap();
        assert!(first_line.starts_with("[cols="));
        assert!(first_line.contains("options=\"header\""));
    }

    #[test]
    fn render_asciidoc_no_blank_lines_between_rows() {
        let data = make_data(&["a", "b"], &[&["1", "2"], &["3", "4"]], &[true, true]);
        let widths = col_widths(&data);
        let out = render_asciidoc(&data, &widths);
        // No consecutive newlines inside the |==== block
        assert!(!out.contains("\n\n"));
    }
}
