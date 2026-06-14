use std::collections::HashMap;
use std::io::Read;

use anyhow::{Context, anyhow};

use crate::model::TableData;

pub fn parse_csv<R: Read>(mut reader: R, delimiter: u8) -> anyhow::Result<TableData> {
    let mut raw = Vec::new();
    reader.read_to_end(&mut raw).context("failed to read input")?;
    let processed = strip_framing_delimiters(&raw, delimiter);

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .from_reader(processed.as_slice());

    let headers: Vec<String> = rdr
        .headers()
        .context("failed to read CSV headers")?
        .iter()
        .map(str::to_string)
        .collect();

    let rows: Vec<Vec<String>> = rdr
        .records()
        .map(|r| {
            r.context("failed to read CSV record")
                .map(|rec| rec.iter().map(str::to_string).collect())
        })
        .collect::<anyhow::Result<_>>()?;

    Ok(TableData::new(headers, rows))
}

// Strips leading/trailing delimiter chars from each line so that framed rows like
// `| a | b |` parse identically to unframed `a | b`.
fn strip_framing_delimiters(raw: &[u8], delimiter: u8) -> Vec<u8> {
    let mut out = Vec::with_capacity(raw.len());
    for line in raw.split(|&b| b == b'\n') {
        let line = line.strip_suffix(b"\r" as &[u8]).unwrap_or(line);
        let line = trim_bytes(line);
        let line = match line.split_first() {
            Some((&b, rest)) if b == delimiter => rest,
            _ => line,
        };
        let line = match line.split_last() {
            Some((&b, rest)) if b == delimiter => rest,
            _ => line,
        };
        out.extend_from_slice(line);
        out.push(b'\n');
    }
    out
}

fn trim_bytes(s: &[u8]) -> &[u8] {
    let start = s.iter().position(|b| !b.is_ascii_whitespace()).unwrap_or(s.len());
    let end = s.iter().rposition(|b| !b.is_ascii_whitespace()).map_or(0, |i| i + 1);
    if start >= end { &[] } else { &s[start..end] }
}

pub fn parse_json(input: &str) -> anyhow::Result<TableData> {
    let value: serde_json::Value = serde_json::from_str(input).context("failed to parse JSON")?;

    let arr = match &value {
        serde_json::Value::Array(a) => a,
        _ => return Err(anyhow!("JSON input must be an array of objects")),
    };

    if arr.is_empty() {
        return Ok(TableData::new(vec![], vec![]));
    }

    let mut headers: Vec<String> = Vec::new();
    let mut header_idx: HashMap<String, usize> = HashMap::new();

    // First pass: collect all unique keys in insertion order
    for item in arr {
        let obj = match item {
            serde_json::Value::Object(m) => m,
            _ => return Err(anyhow!("JSON array elements must be objects")),
        };
        for key in obj.keys() {
            if !header_idx.contains_key(key) {
                header_idx.insert(key.clone(), headers.len());
                headers.push(key.clone());
            }
        }
    }

    let rows: Vec<Vec<String>> = arr
        .iter()
        .map(|item| {
            let obj = item.as_object().unwrap();
            headers
                .iter()
                .map(|h| obj.get(h).map(json_value_to_string).unwrap_or_default())
                .collect()
        })
        .collect();

    Ok(TableData::new(headers, rows))
}

fn json_value_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

pub fn parse_yaml(input: &str) -> anyhow::Result<TableData> {
    use saphyr::{LoadableYamlNode, Yaml};

    let docs = Yaml::load_from_str(input).map_err(|e| anyhow!("YAML parse error: {e}"))?;

    let doc = docs
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("empty YAML document"))?;

    let sequence = match doc {
        Yaml::Sequence(seq) => seq,
        _ => return Err(anyhow!("YAML input must be a sequence (array) of mappings")),
    };

    if sequence.is_empty() {
        return Ok(TableData::new(vec![], vec![]));
    }

    let mut headers: Vec<String> = Vec::new();
    let mut header_idx: HashMap<String, usize> = HashMap::new();
    let mut row_pairs: Vec<Vec<(String, String)>> = Vec::new();

    for item in &sequence {
        let map = match item {
            Yaml::Mapping(m) => m,
            _ => return Err(anyhow!("each YAML sequence element must be a mapping")),
        };

        let mut pairs: Vec<(String, String)> = Vec::new();
        for (k, v) in map {
            let key = yaml_key_to_string(k)?;
            let val = yaml_value_to_string(v);
            if !header_idx.contains_key(&key) {
                header_idx.insert(key.clone(), headers.len());
                headers.push(key.clone());
            }
            pairs.push((key, val));
        }
        row_pairs.push(pairs);
    }

    let rows: Vec<Vec<String>> = row_pairs
        .into_iter()
        .map(|pairs| {
            let mut row = vec![String::new(); headers.len()];
            for (k, v) in pairs {
                if let Some(&idx) = header_idx.get(&k) {
                    row[idx] = v;
                }
            }
            row
        })
        .collect();

    Ok(TableData::new(headers, rows))
}

fn yaml_key_to_string(yaml: &saphyr::Yaml) -> anyhow::Result<String> {
    use saphyr::Yaml;
    match yaml {
        Yaml::Value(scalar) => {
            match scalar {
                saphyr::Scalar::String(s) => Ok(s.to_string()),
                saphyr::Scalar::Integer(n) => Ok(n.to_string()),
                _ => Err(anyhow!("YAML mapping key must be a string")),
            }
        }
        _ => Err(anyhow!("YAML mapping key must be a string")),
    }
}

fn yaml_value_to_string(yaml: &saphyr::Yaml) -> String {
    use saphyr::Yaml;
    match yaml {
        Yaml::Value(scalar) => {
            match scalar {
                saphyr::Scalar::String(s) => s.to_string(),
                saphyr::Scalar::Integer(n) => n.to_string(),
                saphyr::Scalar::FloatingPoint(f) => f.to_string(),
                saphyr::Scalar::Boolean(b) => b.to_string(),
                saphyr::Scalar::Null => String::new(),
            }
        }
        _ => String::new(),
    }
}

pub fn parse_toml(input: &str) -> anyhow::Result<TableData> {
    let value: toml::Value = toml::from_str(input).context("failed to parse TOML")?;

    let root = match value {
        toml::Value::Table(t) => t,
        _ => return Err(anyhow!("TOML input must be a table")),
    };

    // Look for a top-level key whose value is an array-of-tables
    for val in root.values() {
        if let toml::Value::Array(arr) = val {
            if !arr.is_empty() && arr.iter().all(|v| matches!(v, toml::Value::Table(_))) {
                return parse_toml_array(arr);
            }
        }
    }

    // Fall back: treat root as a single flat row (only if all values are scalar)
    let all_scalar = root.values().all(|v| {
        matches!(
            v,
            toml::Value::String(_)
                | toml::Value::Integer(_)
                | toml::Value::Float(_)
                | toml::Value::Boolean(_)
                | toml::Value::Datetime(_)
        )
    });

    if all_scalar && !root.is_empty() {
        let headers: Vec<String> = root.keys().cloned().collect();
        let row: Vec<String> = root.values().map(toml_value_to_string).collect();
        return Ok(TableData::new(headers, vec![row]));
    }

    Err(anyhow!(
        "TOML input must contain an array of tables (e.g. [[items]]) or be a flat key-value table"
    ))
}

fn parse_toml_array(arr: &[toml::Value]) -> anyhow::Result<TableData> {
    let mut headers: Vec<String> = Vec::new();
    let mut header_idx: HashMap<String, usize> = HashMap::new();

    for item in arr {
        if let toml::Value::Table(t) = item {
            for key in t.keys() {
                if !header_idx.contains_key(key) {
                    header_idx.insert(key.clone(), headers.len());
                    headers.push(key.clone());
                }
            }
        }
    }

    let rows: Vec<Vec<String>> = arr
        .iter()
        .map(|item| {
            let t = item.as_table().unwrap();
            headers
                .iter()
                .map(|h| t.get(h).map(toml_value_to_string).unwrap_or_default())
                .collect()
        })
        .collect();

    Ok(TableData::new(headers, rows))
}

fn toml_value_to_string(v: &toml::Value) -> String {
    match v {
        toml::Value::String(s) => s.clone(),
        toml::Value::Integer(n) => n.to_string(),
        toml::Value::Float(f) => f.to_string(),
        toml::Value::Boolean(b) => b.to_string(),
        toml::Value::Datetime(dt) => dt.to_string(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn csv(src: &str) -> TableData { parse_csv(src.as_bytes(), b',').unwrap() }

    #[test]
    fn parse_csv_basic_two_columns() {
        let data = csv("item,qty\nspam,42\neggs,451");
        assert_eq!(data.headers, ["item", "qty"]);
        assert_eq!(data.rows[0], ["spam", "42"]);
        assert_eq!(data.rows[1], ["eggs", "451"]);
    }

    #[test]
    fn parse_tsv_tab_delimiter() {
        let data = parse_csv("item\tqty\nspam\t42".as_bytes(), b'\t').unwrap();
        assert_eq!(data.headers, ["item", "qty"]);
        assert_eq!(data.rows[0], ["spam", "42"]);
    }

    #[test]
    fn parse_psv_pipe_delimiter() {
        let data = parse_csv("item|qty\nspam|42".as_bytes(), b'|').unwrap();
        assert_eq!(data.headers, ["item", "qty"]);
    }

    #[test]
    fn parse_psv_framed_rows_strips_outer_pipes() {
        let data = parse_csv("| item | qty |\n| spam | 42 |".as_bytes(), b'|').unwrap();
        assert_eq!(data.headers, [" item ", " qty "]);
        assert_eq!(data.rows[0], [" spam ", " 42 "]);
    }

    #[test]
    fn parse_psv_framed_rows_no_extra_empty_columns() {
        let data = parse_csv("| a | b |\n| 1 | 2 |".as_bytes(), b'|').unwrap();
        assert_eq!(data.headers.len(), 2);
        assert_eq!(data.rows[0].len(), 2);
    }

    #[test]
    fn parse_csv_delimiter_override_semicolon() {
        let data = parse_csv("item;qty\nspam;42".as_bytes(), b';').unwrap();
        assert_eq!(data.headers, ["item", "qty"]);
    }

    #[test]
    fn parse_csv_header_only_produces_no_data_rows() {
        let data = csv("col1,col2");
        assert_eq!(data.headers, ["col1", "col2"]);
        assert!(data.rows.is_empty());
    }

    #[test]
    fn parse_json_array_of_objects() {
        let data = parse_json(r#"[{"item":"spam","qty":42}]"#).unwrap();
        assert_eq!(data.headers, ["item", "qty"]);
        assert_eq!(data.rows[0], ["spam", "42"]);
    }

    #[test]
    fn parse_json_non_array_returns_error() {
        assert!(parse_json(r#"{"key":"val"}"#).is_err());
    }

    #[test]
    fn parse_json_array_with_non_object_element_returns_error() {
        assert!(parse_json(r#"["string","element"]"#).is_err());
    }

    #[test]
    fn parse_json_heterogeneous_keys_fills_missing_with_empty() {
        let data = parse_json(r#"[{"a":"1"},{"b":"2"}]"#).unwrap();
        assert_eq!(data.headers, ["a", "b"]);
        assert_eq!(data.rows[0], ["1", ""]);
        assert_eq!(data.rows[1], ["", "2"]);
    }

    #[test]
    fn parse_json_null_value_becomes_empty_string() {
        let data = parse_json(r#"[{"a":null}]"#).unwrap();
        assert_eq!(data.rows[0][0], "");
    }

    #[test]
    fn parse_yaml_array_of_mappings() {
        let input = "- item: spam\n  qty: 42\n- item: eggs\n  qty: 451\n";
        let data = parse_yaml(input).unwrap();
        assert_eq!(data.headers, ["item", "qty"]);
        assert_eq!(data.rows[0], ["spam", "42"]);
    }

    #[test]
    fn parse_yaml_non_sequence_returns_error() {
        assert!(parse_yaml("key: value\n").is_err());
    }

    #[test]
    fn parse_toml_array_of_tables() {
        let input =
            "[[items]]\nname = \"spam\"\nqty = 42\n\n[[items]]\nname = \"eggs\"\nqty = 451\n";
        let data = parse_toml(input).unwrap();
        assert_eq!(data.headers, ["name", "qty"]);
        assert_eq!(data.rows[0], ["spam", "42"]);
    }

    #[test]
    fn parse_toml_flat_becomes_single_row() {
        let input = "name = \"spam\"\nqty = 42\n";
        let data = parse_toml(input).unwrap();
        assert_eq!(data.headers, ["name", "qty"]);
        assert_eq!(data.rows.len(), 1);
    }

    #[test]
    fn parse_toml_invalid_shape_returns_error() {
        let input = "nested = { a = { b = 1 } }\n";
        assert!(parse_toml(input).is_err());
    }
}
