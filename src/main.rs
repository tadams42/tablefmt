mod cli;
mod color;
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
        Commands::Prettify(pfy_args) => run_prettify(&pfy_args),
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

fn run_prettify(args: &cli::PrettifyArgs) -> anyhow::Result<()> {
    if args.line.is_none() && args.style.is_none() {
        return Err(anyhow!("--style is required when --line is not given"));
    }

    // ── Path B: locate table by line number inside a file ───────────────────
    if let Some(line_num) = args.line {
        let path = args
            .input
            .as_ref()
            .ok_or_else(|| anyhow!("--input must be a file path when --line is given"))?;

        let file_content = fs::read_to_string(path)
            .with_context(|| format!("failed to read '{}'", path.display()))?;

        let file_lines: Vec<&str> = file_content.lines().collect();

        let (start, end) = locate::find_table_bounds(&file_lines, line_num)
            .with_context(|| format!("cannot locate table at line {line_num}"))?;

        let table_str = file_lines[start..=end].join("\n");
        let (bare_lines, meta) = prettify::preprocess(&table_str);
        let line_refs: Vec<&str> = bare_lines.iter().map(String::as_str).collect();

        let style = args
            .style
            .clone()
            .unwrap_or_else(|| table_parse::detect_style(&line_refs));

        let mut data = table_parse::parse_table(&line_refs, &style)
            .context("failed to parse table for prettify")?;

        numeric::populate_column_meta(&mut data);
        if let Some(places) = args.decimal_places {
            numeric::normalize_decimal_places(&mut data, places);
        }

        let rendered = format::render(&data, &style, &cli::ColorMode::None, false);
        let rendered = if rendered.ends_with('\n') {
            rendered
        } else {
            rendered + "\n"
        };
        let reformatted = prettify::postprocess(&rendered, &meta);

        // Splice reformatted table back into the file
        let new_table_lines: Vec<&str> = reformatted.trim_end_matches('\n').lines().collect();
        let mut result_lines: Vec<&str> = Vec::with_capacity(file_lines.len());
        result_lines.extend_from_slice(&file_lines[..start]);
        result_lines.extend_from_slice(&new_table_lines);
        if end + 1 < file_lines.len() {
            result_lines.extend_from_slice(&file_lines[end + 1..]);
        }

        let mut result = result_lines.join("\n");
        if file_content.ends_with('\n') {
            result.push('\n');
        }

        let out_path = args.output.as_deref().unwrap_or(path.as_path());
        fs::write(out_path, result.as_bytes())
            .with_context(|| format!("failed to write '{}'", out_path.display()))?;

        return Ok(());
    }

    // ── Path A: existing behaviour — pipe the entire table ───────────────────
    let style = args.style.as_ref().unwrap(); // validated above

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

    let input_str =
        std::str::from_utf8(&input_bytes).context("prettify input is not valid UTF-8")?;

    let (bare_lines, meta) = prettify::preprocess(input_str);
    let line_refs: Vec<&str> = bare_lines.iter().map(String::as_str).collect();

    let mut data = table_parse::parse_table(&line_refs, style)
        .context("failed to parse table for prettify")?;

    numeric::populate_column_meta(&mut data);
    if let Some(places) = args.decimal_places {
        numeric::normalize_decimal_places(&mut data, places);
    }

    let is_tty = match &args.output {
        None => io::stdout().is_terminal(),
        Some(_) => false,
    };

    let rendered = format::render(&data, style, &cli::ColorMode::None, is_tty);
    let rendered = if rendered.ends_with('\n') {
        rendered
    } else {
        rendered + "\n"
    };
    let final_out = prettify::postprocess(&rendered, &meta);
    let final_out = if final_out.ends_with('\n') {
        final_out
    } else {
        final_out + "\n"
    };

    match &args.output {
        None => print!("{final_out}"),
        Some(path) => {
            fs::write(path, final_out.as_bytes())
                .with_context(|| format!("failed to write '{}'", path.display()))?
        }
    }

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
