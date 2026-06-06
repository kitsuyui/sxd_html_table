use std::borrow::Cow;

use crate::Error;

fn sanitize_formula_injection(s: &str) -> Cow<'_, str> {
    if s.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        Cow::Owned(format!("\t{s}"))
    } else {
        Cow::Borrowed(s)
    }
}

pub struct Table<T> {
    size: (usize, usize),
    cells: Vec<Option<T>>,
}

impl<T> Table<T> {
    /// Sets the cell at the given `(row, col)` position.
    ///
    /// Rows are zero-indexed from top; columns are zero-indexed from left.
    /// The row index must come before the column index.
    ///
    /// # Panics
    ///
    /// Panics if `row` is greater than or equal to the table row count, or
    /// if `col` is greater than or equal to the table column count.
    pub fn set(&mut self, row: usize, col: usize, item: T) {
        assert!(
            row < self.size.0,
            "row index {row} out of bounds for table with {} rows",
            self.size.0
        );
        assert!(
            col < self.size.1,
            "column index {col} out of bounds for table with {} columns",
            self.size.1
        );
        self.cells[row * self.size.1 + col] = Some(item);
    }

    pub fn rows(&self) -> Vec<Vec<Option<&T>>> {
        let mut rows = vec![];
        for i in 0..self.size.0 {
            let mut row = vec![];
            for j in 0..self.size.1 {
                row.push(self.cells[i * self.size.1 + j].as_ref());
            }
            rows.push(row);
        }
        rows
    }
}

impl<T> Table<T>
where
    T: Clone,
{
    /// Creates a new table with the given size.
    ///
    /// `size` is `(rows, cols)`: the first element is the row count and the
    /// second is the column count.
    pub fn new(size: (usize, usize)) -> Self {
        Self {
            size,
            cells: vec![None; size.0 * size.1],
        }
    }

    pub fn map<T2>(&self, f: impl Fn(usize, usize, &T) -> T2) -> Table<T2>
    where
        T2: Clone,
    {
        map_table(self, f)
    }
}

fn map_table<S, T, F>(table: &Table<T>, f: F) -> Table<S>
where
    F: Fn(usize, usize, &T) -> S,
    S: Clone,
{
    let mut new_table = Table::new(table.size);
    for i in 0..table.size.0 {
        for j in 0..table.size.1 {
            if let Some(item) = &table.cells[i * table.size.1 + j] {
                new_table.set(i, j, f(i, j, item));
            }
        }
    }
    new_table
}

impl<T> Table<T>
where
    T: std::fmt::Display,
{
    pub fn write_csv(&self, writer: &mut impl std::io::Write) -> Result<(), Error> {
        let mut writer = csv::Writer::from_writer(writer);
        for row in &self.rows() {
            let mut record = csv::StringRecord::new();
            for cell in row {
                if let Some(item) = cell {
                    record.push_field(&sanitize_formula_injection(&item.to_string()));
                } else {
                    record.push_field("");
                }
            }
            writer
                .write_record(&record)
                .map_err(|_| Error::FailedToConvertToCSV)?;
        }
        writer.flush().map_err(|_| Error::FailedToConvertToCSV)?;
        Ok(())
    }

    pub fn to_csv(&self) -> Result<String, Error> {
        let mut buf = std::io::BufWriter::new(Vec::new());
        self.write_csv(&mut buf)?;
        let bytes = buf.into_inner().map_err(|_| Error::FailedToConvertToCSV)?;
        String::from_utf8(bytes).map_err(|_| Error::FailedToConvertToCSV)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_updates_cell_when_index_is_in_bounds() {
        let mut table = Table::new((2, 2));

        table.set(1, 1, 42);

        assert_eq!(table.rows()[1][1], Some(&42));
    }

    #[test]
    #[should_panic(expected = "row index 2 out of bounds for table with 2 rows")]
    fn set_panics_when_row_is_out_of_bounds() {
        let mut table = Table::new((2, 2));

        table.set(2, 0, 42);
    }

    #[test]
    #[should_panic(expected = "column index 2 out of bounds for table with 2 columns")]
    fn set_panics_when_column_is_out_of_bounds() {
        let mut table = Table::new((2, 2));

        table.set(0, 2, 42);
    }
}
