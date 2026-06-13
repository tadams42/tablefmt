mod cli;
mod color;
mod format;
mod model;
mod numeric;
mod parse;

use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::Path;

use anyhow::{anyhow, Context};
use clap::{CommandFactory, Parser};
use clap_complete::generate;

use cli::{Args, Commands, MaxRows, SourceFormat};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Completions subcommand bypasses all other logic
    if let Some(Commands::Completions { shell }) = args.command {
        let mut cmd = Args::command();
        let name = cmd.get_name().to_string();
        generate(shell, &mut cmd, name, &mut io::stdout());
        return Ok(());
    }

    // Validate delimiter
    if let Some(d) = args.delimiter {
        if !d.is_ascii() {
            return Err(anyhow!("--delimiter must be a single ASCII character"));
        }
    }

    // Resolve source format
    let source = resolve_source(&args)?;

    // Open input
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

    // Parse input into TableData
    let mut data = match source {
        SourceFormat::Csv => {
            parse::parse_csv(input_bytes.as_slice(), delimiter.unwrap_or(b','))?
        }
        SourceFormat::Tsv => {
            parse::parse_csv(input_bytes.as_slice(), delimiter.unwrap_or(b'\t'))?
        }
        SourceFormat::Psv => {
            parse::parse_csv(input_bytes.as_slice(), delimiter.unwrap_or(b'|'))?
        }
        SourceFormat::Json => {
            let text = std::str::from_utf8(&input_bytes).context("JSON input is not valid UTF-8")?;
            parse::parse_json(text)?
        }
        SourceFormat::Yaml => {
            let text = std::str::from_utf8(&input_bytes).context("YAML input is not valid UTF-8")?;
            parse::parse_yaml(text)?
        }
        SourceFormat::Toml => {
            let text = std::str::from_utf8(&input_bytes).context("TOML input is not valid UTF-8")?;
            parse::parse_toml(text)?
        }
    };

    // Truncate rows before computing column metadata
    if let MaxRows::Limit(n) = args.max_rows {
        data.truncate_rows(n);
    }

    // Numeric detection and optional normalization
    numeric::populate_column_meta(&mut data);
    if let Some(places) = args.decimal_places {
        numeric::normalize_decimal_places(&mut data, places);
    }

    // Determine whether output is a TTY
    let is_tty = match &args.output {
        None => io::stdout().is_terminal(),
        Some(_) => false,
    };

    // Render
    let rendered = format::render(&data, &args.style, &args.color, is_tty);

    // Write output (always end with a newline)
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

fn resolve_source(args: &Args) -> anyhow::Result<SourceFormat> {
    if let Some(ref s) = args.source {
        return Ok(s.clone());
    }

    // Infer from file extension when --input is given
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
                "cannot infer format from extension '.{ext}'; use --source to specify it"
            ));
        }
        return Err(anyhow!(
            "input file has no extension; use --source to specify the format"
        ));
    }

    // stdin without --source or --delimiter is not allowed
    if args.delimiter.is_none() {
        return Err(anyhow!(
            "reading from stdin requires --source or --delimiter to identify the format"
        ));
    }

    // Delimiter provided but no source: default to CSV-like parsing
    Ok(SourceFormat::Csv)
}
