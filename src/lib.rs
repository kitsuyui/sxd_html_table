use std::collections::HashMap;

use sxd_xpath::{nodeset::Node, Context, Factory, Value};

#[derive(Debug)]
pub enum Error {
    TableNotFound,
    InvalidDocument,
    FailedToConvertToCSV,
}

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

pub fn extract_tables_from_document(html: &str) -> Result<Vec<Table>, Error> {
    let package = sxd_html::parse_html(html);
    let document = package.as_document();
    #[allow(clippy::expect_used)]
    let val = evaluate_xpath_node(document.root(), "//table").expect("XPath evaluation failed");

    let Value::Nodeset(table_nodes) = val else {
        panic!("Expected node set");
    };
    let mut tables = vec![];
    for node in table_nodes.document_order() {
        match extract_table(&node) {
            Ok(table) => tables.push(table),
            Err(e) => return Err(e),
        }
    }
    Ok(tables)
}

fn extract_rowspan_and_colspan(node: &Node) -> (usize, usize) {
    #[allow(clippy::expect_used)]
    let element = node.element().expect("Expected element");
    let rowspan = element
        .attribute_value("rowspan")
        .unwrap_or("1")
        .parse::<usize>()
        .unwrap_or(1);
    let colspan = element
        .attribute_value("colspan")
        .unwrap_or("1")
        .parse::<usize>()
        .unwrap_or(1);
    (rowspan, colspan)
}

fn extract_table(node: &Node) -> Result<Table, Error> {
    let tr_nodes = match evaluate_xpath_node(*node, "./tbody/tr") {
        Ok(Value::Nodeset(tr_nodes)) => tr_nodes,
        _ => return Err(Error::InvalidDocument),
    };
    let tr_nodes = tr_nodes.document_order();

    let mut map: HashMap<(usize, usize), String> = HashMap::new();
    let mut header_map: HashMap<(usize, usize), bool> = HashMap::new();
    for (row_index, tr) in tr_nodes.iter().enumerate() {
        let cell_nodes = match evaluate_xpath_node(*tr, "./td|./th") {
            Ok(Value::Nodeset(td_nodes)) => td_nodes,
            _ => return Err(Error::InvalidDocument),
        };
        let cell_nodes = cell_nodes.document_order();
        let mut col_index = 0;
        for (_, cell_node) in cell_nodes.iter().enumerate() {
            let (row_size, col_size) = extract_rowspan_and_colspan(cell_node);
            let text = &cell_node.string_value();
            #[allow(clippy::expect_used)]
            let is_header = cell_node.element().expect("Expected element").name() == "th".into();
            while map.contains_key(&(row_index, col_index)) {
                col_index += 1;
            }
            for k in 0..row_size {
                for l in 0..col_size {
                    map.insert((row_index + k, col_index + l), text.to_string());
                    header_map.insert((row_index + k, col_index + l), is_header);
                }
            }
        }
    }
    let rows = map.keys().map(|(i, _)| i).max().unwrap_or(&0) + 1;
    let cols = map.keys().map(|(_, j)| j).max().unwrap_or(&0) + 1;
    let mut table = Table::new((rows, cols));
    for ((i, j), text) in map {
        table.cells[i * table.size.1 + j] = Some(text);
    }
    for ((i, j), is_header) in header_map {
        table.headers[i * table.size.1 + j] = is_header;
    }
    Ok(table)
}

fn evaluate_xpath_node<'d>(
    node: impl Into<Node<'d>>,
    expr: &str,
) -> Result<Value<'d>, sxd_xpath::Error> {
    let factory = Factory::new();
    let expression = factory.build(expr)?;
    let expression = expression.ok_or(sxd_xpath::Error::NoXPath)?;
    let context = Context::new();
    expression
        .evaluate(&context, node.into())
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_table_from_document() {
        // found 1 table
        let html = r#"
        <html>
            <body>
                <table>
                    <tr>
                        <td>1</td>
                        <td>2</td>
                    </tr>
                </table>
            </body>
        </html>
        "#;
        let result = extract_tables_from_document(html).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].to_csv().unwrap(), "1,2\n");

        // found 2 tables
        let html = r#"
        <html>
            <body>
                <table>
                    <tr>
                        <td>1</td>
                        <td>2</td>
                    </tr>
                </table>
                <table>
                    <tr>
                        <td>3</td>
                        <td>4</td>
                    </tr>
                </table>
            </body>
        </html>
        "#;
        let result = extract_tables_from_document(html).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].to_csv().unwrap(), "1,2\n",);
        assert_eq!(result[1].to_csv().unwrap(), "3,4\n",);

        // found 0 table
        let html = r#"
        <html>
            <body>
                <div>
                    <p>1</p>
                    <p>2</p>
                </div>
            </body>
        </html>
        "#;
        let result = extract_tables_from_document(html).unwrap();
        assert_eq!(result.len(), 0);

        // empty html
        let html = r#""#;
        let result = extract_tables_from_document(html).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_td_and_th() {
        let html = r#"
        <html>
            <body>
                <table>
                    <tr>
                        <th>1</th>
                        <td>2</td>
                    </tr>
                    <tr>
                        <td>3</td>
                        <td>4</td>
                    </tr>
                </table>
            </body>
        </html>
        "#;
        let result = extract_tables_from_document(html).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].to_csv().unwrap(), "1,2\n3,4\n");
    }

    #[test]
    fn test_rowspan_and_colspan() {
        let html = r#"
        <html>
            <body>
                <table>
                    <tr>
                        <td rowspan="2">A</td>
                        <td>B</td>
                    </tr>
                    <tr>
                        <td>C</td>
                    </tr>
                </table>
            </body>
        </html>
        "#;
        let result = extract_tables_from_document(html).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].to_csv().unwrap(), "A,B\nA,C\n");

        let html = r#"
        <html>
            <body>
                <table>
                    <tr>
                        <td colspan="2">A</td>
                        <td>B</td>
                    </tr>
                    <tr>
                        <td>C</td>
                        <td>D</td>
                        <td>E</td>
                    </tr>
                </table>
            </body>
        </html>
        "#;
        let result = extract_tables_from_document(html).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].to_csv().unwrap(), "A,A,B\nC,D,E\n");

        // more complex
        let html = r#"
        <html>
            <body>
                <table>
                    <tr>
                        <td rowspan="2" colspan="2">A</td>
                        <td>B</td>
                    </tr>
                    <tr><td>C</td></tr>
                </table>
                <table>
                    <tr><td>a</td><td>b</td><td>c</td></tr>
                    <tr><td>d</td><td>e</td><td>f</td></tr>
                </table>
                <table>
                    <tr><td>a</td><td>b</td><td>c</td><td rowspan="2">d</td></tr>
                    <tr><td>e</td><td colspan="2">f</td></tr>
                    <tr><td>i</td><td>j</td><td>k</td><td>l</td></tr>
                </table>
                <table>
                    <tr><td>a</td><td>b</td><td rowspan="2">c</td><td>d</td></tr>
                    <tr><td>e</td><td colspan="3">f</td></tr>
                    <tr><td>i</td><td>j</td><td>k</td><td>l</td></tr>
                </table>
                <table>
                    <tr><td>a</td><td>b</td><td>c</td><td>d</td></tr>
                    <tr><td>e</td><td rowspan="2" colspan="2">f</td><td>g</td></tr>
                    <tr><td>h</td><td>i</td></tr>
                </table>

                <!-- invalid rowspan -->
                <table>
                    <tr><td>a</td><td>b</td><td>c</td><td>d</td></tr>
                    <tr><td>e</td><td rowspan="a" colspan="b">f</td><td>g</td></tr>
                    <tr><td>h</td><td>i</td></tr>
                </table>
            </body>
        </html>
        "#;

        let result = extract_tables_from_document(html).unwrap();
        assert_eq!(result.len(), 6);
        assert_eq!(result[0].to_csv().unwrap(), "A,A,B\nA,A,C\n");
        assert_eq!(result[1].to_csv().unwrap(), "a,b,c\nd,e,f\n");
        assert_eq!(result[2].to_csv().unwrap(), "a,b,c,d\ne,f,f,d\ni,j,k,l\n");
        assert_eq!(result[3].to_csv().unwrap(), "a,b,c,d\ne,f,f,f\ni,j,k,l\n");
        assert_eq!(result[4].to_csv().unwrap(), "a,b,c,d\ne,f,f,g\nh,f,f,i\n");
        assert_eq!(result[5].to_csv().unwrap(), "a,b,c,d\ne,f,g,\nh,i,,\n");
    }
}
