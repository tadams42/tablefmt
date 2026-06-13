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
}
