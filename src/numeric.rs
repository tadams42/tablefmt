use crate::model::TableData;

pub fn populate_column_meta(data: &mut TableData) {
    for col_idx in 0..data.headers.len() {
        let (is_numeric, max_dec) = analyze_column(data, col_idx);
        data.column_meta[col_idx].is_numeric = is_numeric;
        data.column_meta[col_idx].max_decimal_places = max_dec;
    }
}

fn analyze_column(data: &TableData, col_idx: usize) -> (bool, usize) {
    let mut all_numeric = true;
    let mut max_dec = 0usize;
    let mut any_non_empty = false;

    for row in &data.rows {
        let cell = row[col_idx].trim();
        if cell.is_empty() {
            continue;
        }
        any_non_empty = true;
        if cell.parse::<f64>().is_ok() {
            max_dec = max_dec.max(decimal_places(cell));
        } else {
            all_numeric = false;
            break;
        }
    }

    (all_numeric && any_non_empty, max_dec)
}

fn decimal_places(s: &str) -> usize {
    if let Some(dot_pos) = s.find('.') {
        let after_dot = &s[dot_pos + 1..];
        after_dot.find(['e', 'E']).unwrap_or(after_dot.len())
    } else {
        0
    }
}

pub fn normalize_decimal_places(data: &mut TableData, places: usize) {
    for col_idx in 0..data.headers.len() {
        if !data.column_meta[col_idx].is_numeric {
            continue;
        }
        for row in &mut data.rows {
            let cell = row[col_idx].trim().to_string();
            if cell.is_empty() {
                continue;
            }
            if let Ok(val) = cell.parse::<f64>() {
                row[col_idx] = format!("{val:.places$}");
            }
        }
        data.column_meta[col_idx].max_decimal_places = places;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::TableData;

    fn make_data(headers: &[&str], rows: &[&[&str]]) -> TableData {
        TableData::new(
            headers.iter().map(|s| s.to_string()).collect(),
            rows.iter()
                .map(|r| r.iter().map(|s| s.to_string()).collect())
                .collect(),
        )
    }

    #[test]
    fn detect_numeric_column_marks_is_numeric_true() {
        let mut data = make_data(&["n"], &[&["42"], &["3.14"], &["0"]]);
        populate_column_meta(&mut data);
        assert!(data.column_meta[0].is_numeric);
    }

    #[test]
    fn detect_mixed_column_marks_is_numeric_false() {
        let mut data = make_data(&["n"], &[&["42"], &["hello"], &["0"]]);
        populate_column_meta(&mut data);
        assert!(!data.column_meta[0].is_numeric);
    }

    #[test]
    fn detect_empty_cells_do_not_disqualify_numeric() {
        let mut data = make_data(&["n"], &[&["42"], &[""], &["7"]]);
        populate_column_meta(&mut data);
        assert!(data.column_meta[0].is_numeric);
    }

    #[test]
    fn max_decimal_places_computes_correctly() {
        let mut data = make_data(&["n"], &[&["1"], &["3.14"], &["2.5"]]);
        populate_column_meta(&mut data);
        assert_eq!(data.column_meta[0].max_decimal_places, 2);
    }

    #[test]
    fn normalize_decimal_places_rewrites_cells() {
        let mut data = make_data(&["n"], &[&["42"], &["3.1"], &["0.125"]]);
        populate_column_meta(&mut data);
        normalize_decimal_places(&mut data, 2);
        assert_eq!(data.rows[0][0], "42.00");
        assert_eq!(data.rows[1][0], "3.10");
        assert_eq!(data.rows[2][0], "0.12");
    }

    #[test]
    fn all_empty_column_is_not_numeric() {
        let mut data = make_data(&["n"], &[&[""], &[""]]);
        populate_column_meta(&mut data);
        assert!(!data.column_meta[0].is_numeric);
    }
}
