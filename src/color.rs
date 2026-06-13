use tabled::{
    settings::{
        object::{Columns, Rows},
        Color,
    },
    Table,
};

use crate::cli::ColorMode;

const COLUMN_COLORS: [Color; 6] = [
    Color::FG_CYAN,
    Color::FG_YELLOW,
    Color::FG_GREEN,
    Color::FG_MAGENTA,
    Color::FG_RED,
    Color::FG_BLUE,
];

const ROW_COLOR_A: Color = Color::FG_CYAN;
const ROW_COLOR_B: Color = Color::FG_WHITE;

pub fn apply(table: &mut Table, mode: &ColorMode, n_cols: usize, n_data_rows: usize) {
    match mode {
        ColorMode::None => {}
        ColorMode::Columns => {
            for i in 0..n_cols {
                table.modify(Columns::one(i), COLUMN_COLORS[i % COLUMN_COLORS.len()].clone());
            }
        }
        ColorMode::Rows => {
            for i in 0..n_data_rows {
                // Row 0 is the header; data rows start at index 1
                let color = if i % 2 == 0 { ROW_COLOR_A.clone() } else { ROW_COLOR_B.clone() };
                table.modify(Rows::one(i + 1), color);
            }
        }
    }
}
