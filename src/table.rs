use crate::Error;

pub struct Table<T> {
    size: (usize, usize),
    cells: Vec<Option<T>>,
    headers: Vec<bool>,
}

impl<T> Table<T> {
    pub fn set(&mut self, row: usize, col: usize, item: T) {
        self.cells[row * self.size.1 + col] = Some(item);
    }

    pub fn set_header(&mut self, row: usize, col: usize) {
        self.headers[row * self.size.1 + col] = true;
    }

    pub fn is_header(&self, row: usize, col: usize) -> bool {
        self.headers[row * self.size.1 + col]
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
    pub fn new(size: (usize, usize)) -> Self {
        Self {
            size,
            cells: vec![None; size.0 * size.1],
            headers: vec![false; size.0 * size.1],
        }
    }

    pub fn map<T2>(&self, f: impl Fn(&T) -> T2) -> Table<T2>
    where
        T2: Clone,
    {
        map_table(self, f)
    }
}

fn map_table<S, T, F>(table: &Table<T>, f: F) -> Table<S>
where
    F: Fn(&T) -> S,
    S: Clone,
{
    let mut new_table = Table::new(table.size);
    for i in 0..table.size.0 {
        for j in 0..table.size.1 {
            if let Some(item) = &table.cells[i * table.size.1 + j] {
                new_table.set(i, j, f(item));
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
                    record.push_field(&item.to_string());
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
