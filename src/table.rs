use itertools::Itertools;
use parser::Parser;
use std::fmt;
use utils::StringUtils;

lazy_static! {
    static ref DEFAULT_TABLE_CELL: TableCell = TableCell::default();
}

#[derive(Serialize, Deserialize)]
pub struct Table {
    rows: Vec<TableEntry>,
}

impl Table {
    pub fn parse(line: &str, parser: &mut Parser) -> Option<Table> {
        if let Some(row) = TableEntry::parse(line) {
            let mut table = Table { rows: vec![row] };

            while let Some(row) = parser.peek().and_then(|line| TableEntry::parse(line)) {
                parser.next();
                table.rows.push(row);
            }

            Some(table)
        } else {
            None
        }
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn column_count(&self) -> usize {
        self.rows
            .iter()
            .map(|row| row.column_count())
            .max()
            .unwrap_or(0)
    }

    pub fn column_width(&self, index: usize) -> usize {
        self.rows
            .iter()
            .map(|row| row.column_width(index))
            .max()
            .unwrap_or(0)
    }

    pub fn cell_mut(&self, row: usize, column: usize) -> Option<&TableCell> {
        if row < self.row_count() && column < self.column_count() {
            if let TableEntry::Row(ref row) = self.rows[row] {
                Some(row.cell(column))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn cell(&mut self, row: usize, column: usize) -> Option<&TableCell> {
        if row < self.row_count() && column < self.column_count() {
            if let TableEntry::Row(ref mut row) = self.rows[row] {
                Some(row.cell_mut(column))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let column_widths: Vec<usize> = (0..self.column_count())
            .map(|index| self.column_width(index))
            .collect();

        let rows = self
            .rows
            .iter()
            .map(|row| row.format(&column_widths))
            .join("\n");

        write!(f, "{}", rows)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TableEntry {
    Rule,
    Row(TableRow),
}

impl TableEntry {
    fn parse(line: &str) -> Option<TableEntry> {
        let line = line.trim();

        if line.starts_with("|-") {
            Some(TableEntry::Rule)
        } else if line.starts_with("|") {
            let cells = line
                .trim_matches('|')
                .split('|')
                .map(|cell| TableCell {
                    text: cell.trim().to_string(),
                }).collect();
            Some(TableEntry::Row(TableRow { cells }))
        } else {
            None
        }
    }

    pub fn column_count(&self) -> usize {
        match self {
            TableEntry::Rule => 1,
            TableEntry::Row(row) => row.cells.len(),
        }
    }

    pub fn column_width(&self, index: usize) -> usize {
        match self {
            TableEntry::Rule => 0,
            TableEntry::Row(row) => row
                .cells
                .get(index)
                .map(|cell| cell.text.len())
                .unwrap_or(0),
        }
    }

    pub fn format(&self, column_widths: &[usize]) -> String {
        match self {
            TableEntry::Rule => column_widths
                .iter()
                .map(|width| "-".repeat(width + 2))
                .join("+")
                .capped("|"),
            TableEntry::Row(row) => column_widths
                .iter()
                .enumerate()
                .map(|(index, width)| format!(" {:1$} ", row.cell_text(index), width))
                .join("|")
                .capped("|"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TableRow {
    cells: Vec<TableCell>,
}

impl TableRow {
    pub fn cell(&self, index: usize) -> &TableCell {
        if self.cells.len() > index {
            &self.cells[index]
        } else {
            &DEFAULT_TABLE_CELL
        }
    }

    pub fn cell_mut(&mut self, index: usize) -> &mut TableCell {
        if self.cells.len() <= index {
            self.cells.resize(index + 1, TableCell::default());
        }

        &mut self.cells[index]
    }

    pub fn cell_text(&self, index: usize) -> &str {
        &self.cell(index).text
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct TableCell {
    text: String,
}
