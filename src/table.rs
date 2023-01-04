use crate::Error;

#[derive(Debug, Eq, PartialEq)]
pub struct Table {
    size: (usize, usize),
    cells: Vec<Option<String>>,
    headers: Vec<bool>,
}

impl Table {
    pub fn new(size: (usize, usize)) -> Self {
        Self {
            size,
            cells: vec![None; size.0 * size.1],
            headers: vec![false; size.0 * size.1],
        }
    }

    pub fn is_header(&self, row: usize, col: usize) -> bool {
        self.headers[row * self.size.1 + col]
    }

    pub fn set(&mut self, row: usize, col: usize, text: String) {
        self.cells[row * self.size.1 + col] = Some(text);
    }

    pub fn set_header(&mut self, row: usize, col: usize) {
        self.headers[row * self.size.1 + col] = true;
    }

    pub fn rows(&self) -> Vec<Vec<Option<&str>>> {
        let mut rows = vec![];
        for i in 0..self.size.0 {
            let mut row = vec![];
            for j in 0..self.size.1 {
                row.push(self.cells[i * self.size.1 + j].as_deref());
            }
            rows.push(row);
        }
        rows
    }

    pub fn write_csv(&self, writer: &mut impl std::io::Write) -> Result<(), Error> {
        let mut writer = csv::Writer::from_writer(writer);
        for row in &self.rows() {
            let mut record = csv::StringRecord::new();
            for cell in row {
                if let Some(text) = cell {
                    record.push_field(text);
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
