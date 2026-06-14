#[derive(Debug, Default)]
pub struct ColumnMeta {
    pub is_numeric:         bool,
    pub max_decimal_places: usize,
}

#[derive(Debug)]
pub struct TableData {
    pub headers:     Vec<String>,
    pub rows:        Vec<Vec<String>>,
    pub column_meta: Vec<ColumnMeta>,
}

impl TableData {
    pub fn new(headers: Vec<String>, rows: Vec<Vec<String>>) -> Self {
        let n = headers.len();
        Self {
            headers,
            rows,
            column_meta: (0..n).map(|_| ColumnMeta::default()).collect(),
        }
    }

    pub fn truncate_rows(&mut self, max_rows: usize) { self.rows.truncate(max_rows); }

    pub fn trim_values(&mut self) {
        for h in &mut self.headers {
            *h = h.trim().to_string();
        }
        for row in &mut self.rows {
            for cell in row {
                *cell = cell.trim().to_string();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_data_truncate_limits_rows() {
        let mut data = TableData::new(
            vec!["a".to_string()],
            vec![
                vec!["1".to_string()],
                vec!["2".to_string()],
                vec!["3".to_string()],
            ],
        );
        data.truncate_rows(2);
        assert_eq!(data.rows.len(), 2);
    }

    #[test]
    fn table_data_truncate_no_op_when_fewer_rows() {
        let mut data = TableData::new(vec!["a".to_string()], vec![vec!["1".to_string()]]);
        data.truncate_rows(10);
        assert_eq!(data.rows.len(), 1);
    }

    #[test]
    fn trim_values_removes_surrounding_whitespace() {
        let mut data = TableData::new(
            vec![" col1 ".to_string(), "\tcol2\t".to_string()],
            vec![vec![" val1 ".to_string(), "  val2  ".to_string()]],
        );
        data.trim_values();
        assert_eq!(data.headers, ["col1", "col2"]);
        assert_eq!(data.rows[0], ["val1", "val2"]);
    }
}
