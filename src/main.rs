mod cli;
mod color;
mod edit;
mod format;
mod locate;
mod model;
mod numeric;
mod parse;
mod prettify;
mod table_parse;

use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::Path;

use anyhow::{Context, anyhow};
use clap::CommandFactory;
use clap_complete::generate;

use cli::{Args, Commands, MaxRows, SourceFormat};

fn main() -> anyhow::Result<()> {
    let args = cli::parse_args();

    match args.command {
        Commands::Completions { shell } => {
            let mut cmd = Args::command();
            let name = cmd.get_name().to_string();
            generate(shell, &mut cmd, name, &mut io::stdout());
            Ok(())
        }
        Commands::Format(fmt_args) => run_format(&fmt_args),
        Commands::Edit(edit_args) => {
            match run_edit(&edit_args) {
                Ok(()) => Ok(()),
                Err(e) => {
                    println!("{}", serde_json::json!({"error": e.to_string()}));
                    std::process::exit(1);
                }
            }
        }
    }
}

fn run_format(args: &cli::FormatArgs) -> anyhow::Result<()> {
    if let Some(d) = args.delimiter {
        if !d.is_ascii() {
            return Err(anyhow!("--delimiter must be a single ASCII character"));
        }
    }

    let source = resolve_source(args)?;

    let input_bytes: Vec<u8> = match &args.input {
        Some(path) => {
            fs::read(path).with_context(|| format!("failed to read '{}'", path.display()))?
        }
        None => {
            let mut buf = Vec::new();
            io::stdin()
                .read_to_end(&mut buf)
                .context("failed to read stdin")?;
            buf
        }
    };

    let delimiter = args.delimiter.map(|c| c as u8);

    let mut data = match source {
        SourceFormat::Csv => parse::parse_csv(input_bytes.as_slice(), delimiter.unwrap_or(b','))?,
        SourceFormat::Tsv => parse::parse_csv(input_bytes.as_slice(), delimiter.unwrap_or(b'\t'))?,
        SourceFormat::Psv => parse::parse_csv(input_bytes.as_slice(), delimiter.unwrap_or(b'|'))?,
        SourceFormat::Json => {
            let text =
                std::str::from_utf8(&input_bytes).context("JSON input is not valid UTF-8")?;
            parse::parse_json(text)?
        }
        SourceFormat::Yaml => {
            let text =
                std::str::from_utf8(&input_bytes).context("YAML input is not valid UTF-8")?;
            parse::parse_yaml(text)?
        }
        SourceFormat::Toml => {
            let text =
                std::str::from_utf8(&input_bytes).context("TOML input is not valid UTF-8")?;
            parse::parse_toml(text)?
        }
    };

    if let MaxRows::Limit(n) = args.max_rows {
        data.truncate_rows(n);
    }

    numeric::populate_column_meta(&mut data);
    if let Some(places) = args.decimal_places {
        numeric::normalize_decimal_places(&mut data, places);
    }

    let is_tty = match &args.output {
        None => io::stdout().is_terminal(),
        Some(_) => false,
    };

    let rendered = format::render(&data, &args.style, &args.color, is_tty);
    let rendered = if rendered.ends_with('\n') {
        rendered
    } else {
        rendered + "\n"
    };

    match &args.output {
        None => {
            print!("{rendered}");
        }
        Some(path) => {
            fs::write(path, rendered.as_bytes())
                .with_context(|| format!("failed to write '{}'", path.display()))?;
        }
    }

    Ok(())
}

fn run_edit(args: &cli::EditArgs) -> anyhow::Result<()> {
    use cli::EditOperation;

    let (file_path, line_num) = match &args.operation {
        EditOperation::Prettify(a) => (&a.file, a.line),
        EditOperation::AddColumnBefore(a)
        | EditOperation::AddColumnAfter(a)
        | EditOperation::RemoveColumn(a) => (&a.file, a.line),
    };

    let file_content = fs::read_to_string(file_path)
        .with_context(|| format!("failed to read '{}'", file_path.display()))?;

    let file_lines: Vec<&str> = file_content.lines().collect();

    let (start, end) = locate::find_table_bounds(&file_lines, line_num)
        .with_context(|| format!("cannot locate table at line {line_num}"))?;

    let table_str = file_lines[start..=end].join("\n");
    let (bare_lines, meta) = prettify::preprocess(&table_str);
    let line_refs: Vec<&str> = bare_lines.iter().map(String::as_str).collect();

    let style = table_parse::detect_style(&line_refs);

    let mut data = table_parse::parse_table(&line_refs, &style).context("failed to parse table")?;

    numeric::populate_column_meta(&mut data);

    match &args.operation {
        EditOperation::Prettify(pargs) => {
            if let Some(places) = pargs.decimal_places {
                numeric::normalize_decimal_places(&mut data, places);
            }
        }
        EditOperation::AddColumnBefore(cargs) => {
            let original_line = file_lines[cargs.line];
            let col_idx = edit::find_column_at_cursor(original_line, cargs.col);
            data = edit::add_column(data, col_idx);
            numeric::populate_column_meta(&mut data);
        }
        EditOperation::AddColumnAfter(cargs) => {
            let original_line = file_lines[cargs.line];
            let col_idx = edit::find_column_at_cursor(original_line, cargs.col);
            data = edit::add_column(data, col_idx + 1);
            numeric::populate_column_meta(&mut data);
        }
        EditOperation::RemoveColumn(cargs) => {
            let original_line = file_lines[cargs.line];
            let col_idx = edit::find_column_at_cursor(original_line, cargs.col);
            data = edit::remove_column(data, col_idx).context("failed to remove column")?;
            numeric::populate_column_meta(&mut data);
        }
    }

    let rendered = format::render(&data, &style, &cli::ColorMode::None, false);
    let rendered = if rendered.ends_with('\n') {
        rendered
    } else {
        rendered + "\n"
    };
    let reformatted = prettify::postprocess(&rendered, &meta);
    let text = reformatted.trim_end_matches('\n');

    println!(
        "{}",
        serde_json::json!({
            "start_line": start,
            "end_line": end,
            "text": text,
            "style": style.to_string(),
        })
    );

    Ok(())
}

fn resolve_source(args: &cli::FormatArgs) -> anyhow::Result<SourceFormat> {
    if let Some(ref s) = args.format {
        return Ok(s.clone());
    }

    if let Some(ref path) = args.input {
        if let Some(ext) = Path::new(path).extension().and_then(|e| e.to_str()) {
            let src = match ext.to_ascii_lowercase().as_str() {
                "csv" => Some(SourceFormat::Csv),
                "tsv" => Some(SourceFormat::Tsv),
                "psv" => Some(SourceFormat::Psv),
                "json" => Some(SourceFormat::Json),
                "yaml" | "yml" => Some(SourceFormat::Yaml),
                "toml" => Some(SourceFormat::Toml),
                _ => None,
            };
            if let Some(s) = src {
                return Ok(s);
            }
            return Err(anyhow!(
                "cannot infer format from extension '.{ext}'; use --format to specify it"
            ));
        }
        return Err(anyhow!(
            "input file has no extension; use --format to specify the format"
        ));
    }

    if args.delimiter.is_none() {
        return Err(anyhow!(
            "reading from stdin requires --format or --delimiter to identify the format"
        ));
    }

    Ok(SourceFormat::Csv)
}
