use std::path::PathBuf;

use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

fn styles() -> Styles {
    Styles::styled()
        .header(
            AnsiColor::Green
                .on_default()
                .effects(Effects::BOLD | Effects::UNDERLINE),
        )
        .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
        .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
        .placeholder(AnsiColor::Cyan.on_default())
        .error(AnsiColor::Red.on_default().effects(Effects::BOLD))
        .valid(AnsiColor::Green.on_default().effects(Effects::BOLD))
        .invalid(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
}

pub fn parse_args() -> Args {
    let matches = Args::command().styles(styles()).get_matches();
    Args::from_arg_matches(&matches).unwrap_or_else(|e| e.exit())
}

#[derive(Parser, Debug)]
#[command(
    name = "tablefmt",
    about = "Format tabular data as a table",
    version,
    arg_required_else_help = true
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Parser, Debug)]
pub struct FormatArgs {
    /// Input file (default: stdin)
    #[arg(short = 'i', long)]
    pub input: Option<PathBuf>,

    /// Output file (default: stdout)
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,

    /// Input format
    #[arg(short = 'f', long = "format", value_enum)]
    pub format: Option<SourceFormat>,

    /// Delimiter override for CSV/TSV/PSV (must be ASCII)
    #[arg(short = 'd', long)]
    pub delimiter: Option<char>,

    /// Output table style
    #[arg(short = 's', long = "style", value_enum, default_value = "github")]
    pub style: OutputFormat,

    /// Column/row coloring (columns/c, rows/r, none). Not all styles support color; unsupported
    /// styles silently ignore this option.
    #[arg(long, value_enum, default_value = "none")]
    pub color: ColorMode,

    /// Max data rows to output (0 or null = all rows)
    #[arg(long, value_parser = parse_max_rows, default_value = "20")]
    pub max_rows: MaxRows,

    /// Normalize numeric columns to N decimal places
    #[arg(short = 'N', long)]
    pub decimal_places: Option<usize>,
}

#[derive(Parser, Debug)]
pub struct EditArgs {
    #[command(subcommand)]
    pub operation: EditOperation,
}

#[derive(Subcommand, Debug)]
pub enum EditOperation {
    /// Re-format the table at the given line (auto-detects style)
    Prettify(EditPrettifyArgs),
    /// Insert an empty column before the column under the cursor
    AddColumnBefore(EditColumnArgs),
    /// Insert an empty column after the column under the cursor
    AddColumnAfter(EditColumnArgs),
    /// Remove the column under the cursor
    RemoveColumn(EditColumnArgs),
}

#[derive(Parser, Debug)]
pub struct EditPrettifyArgs {
    /// File to edit
    pub file: PathBuf,

    /// 0-based line number of any line inside the table
    #[arg(long)]
    pub line: usize,

    /// Normalize numeric columns to N decimal places
    #[arg(short = 'N', long)]
    pub decimal_places: Option<usize>,
}

#[derive(Parser, Debug)]
pub struct EditColumnArgs {
    /// File to edit
    pub file: PathBuf,

    /// 0-based line number of any line inside the table
    #[arg(long)]
    pub line: usize,

    /// 0-based character offset of the cursor within that line
    #[arg(long)]
    pub col: usize,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Convert tabular data (CSV, JSON, etc.) to a formatted table
    Format(FormatArgs),
    /// Edit a table in a file and return a JSON replacement payload
    Edit(EditArgs),
    /// Generate shell completion definitions
    Completions {
        /// Target shell
        shell: Shell,
    },
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use clap::ValueEnum;
        f.write_str(self.to_possible_value().unwrap().get_name())
    }
}

#[derive(Clone, Debug, ValueEnum)]
pub enum SourceFormat {
    Csv,
    Tsv,
    Psv,
    Json,
    Yaml,
    Toml,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    Github,
    Psql,
    Asciidoc,
    Jira,
    Rst,
    #[value(name = "rst-grid")]
    RstGrid,
    Orgtbl,
    Dots,
    Ascii,
    Modern,
    Sharp,
    Extended,
    #[value(name = "heavy-outline")]
    HeavyOutline,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum ColorMode {
    #[value(alias = "c")]
    Columns,
    #[value(alias = "r")]
    Rows,
    None,
}

#[derive(Clone, Debug)]
pub enum MaxRows {
    All,
    Limit(usize),
}

pub fn parse_max_rows(s: &str) -> Result<MaxRows, String> {
    match s {
        "0" | "null" => Ok(MaxRows::All),
        s => {
            s.parse::<usize>().map(MaxRows::Limit).map_err(|_| {
                format!("invalid value '{s}': expected a positive integer, 0, or null")
            })
        }
    }
}
