use sxd_xpath::{nodeset::Node, Context, Factory, Value};

#[derive(Debug)]
pub enum Error {
    TableNotFound,
    InvalidDocument,
}

#[derive(Debug, Eq, PartialEq)]
pub struct TableCell {
    header: bool,
    text: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Table {
    rows: Vec<Vec<TableCell>>,
}

impl TableCell {
    pub fn new(header: bool) -> Self {
        Self { header, text: None }
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl Table {
    pub fn new() -> Self {
        Self { rows: vec![] }
    }

    pub fn expand_size(&mut self, row_size: usize, col_size: usize) {
        if self.rows.len() < row_size {
            self.rows.push(vec![]);
        }
        for row in &mut self.rows {
            if row.len() < col_size {
                row.push(TableCell::new(false));
            }
        }
    }

    pub fn set_cell(&mut self, text: &str, header: bool, row_index: usize, col_index: usize) {
        self.expand_size(row_index + 1, col_index + 1);
        self.rows[row_index][col_index].text = Some(text.to_string());
        self.rows[row_index][col_index].header = header;
    }

    pub fn rows(&self) -> &Vec<Vec<TableCell>> {
        &self.rows
    }

    pub fn to_csv(&self) -> String {
        let mut csv = String::new();
        for row in &self.rows {
            let mut first = true;
            for cell in row {
                if first {
                    first = false;
                } else {
                    csv.push(',');
                }
                if let Some(text) = &cell.text {
                    csv.push_str(text);
                }
            }
            csv.push('\n');
        }
        csv
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

fn extract_table(node: &Node) -> Result<Table, Error> {
    let mut table = Table::new();
    let tr_nodes = match evaluate_xpath_node(*node, "./tbody/tr") {
        Ok(Value::Nodeset(tr_nodes)) => tr_nodes,
        _ => return Err(Error::InvalidDocument),
    };
    let tr_nodes = tr_nodes.document_order();
    for (i, tr) in tr_nodes.iter().enumerate() {
        let td_nodes = match evaluate_xpath_node(*tr, "./td") {
            Ok(Value::Nodeset(td_nodes)) => td_nodes,
            _ => return Err(Error::InvalidDocument),
        };
        let td_nodes = td_nodes.document_order();
        for (j, td) in td_nodes.iter().enumerate() {
            let header = false;
            table.set_cell(&td.string_value(), header, i, j);
        }
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
        assert_eq!(result[0].to_csv(), "1,2\n",);

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
        assert_eq!(result[0].to_csv(), "1,2\n",);
        assert_eq!(result[1].to_csv(), "3,4\n",);

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
}
