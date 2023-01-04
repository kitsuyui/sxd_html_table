use std::collections::HashMap;

use sxd_xpath::{nodeset::Node, Context, Factory, Value};
pub mod element_utils;
pub mod table;
use crate::table::Table;

#[derive(Debug)]
pub enum Error {
    TableNotFound,
    InvalidDocument,
    FailedToConvertToCSV,
}

pub fn extract_table_texts_from_document(html: &str) -> Result<Vec<Table>, Error> {
    let package = sxd_html::parse_html(html);
    let document = package.as_document();
    #[allow(clippy::expect_used)]
    let val = evaluate_xpath_node(document.root(), "//table").expect("XPath evaluation failed");

    let Value::Nodeset(table_nodes) = val else {
        panic!("Expected node set");
    };
    let mut tables = vec![];
    for node in table_nodes.document_order() {
        match extract_table_texts(&node) {
            Ok(table) => tables.push(table),
            Err(e) => return Err(e),
        }
    }
    Ok(tables)
}

pub fn extract_table_elements_from_document(html: &str) -> Result<Vec<Table>, Error> {
    let package = sxd_html::parse_html(html);
    let document = package.as_document();
    #[allow(clippy::expect_used)]
    let val = evaluate_xpath_node(document.root(), "//table").expect("XPath evaluation failed");

    let Value::Nodeset(table_nodes) = val else {
        panic!("Expected node set");
    };
    let mut tables = vec![];
    for node in table_nodes.document_order() {
        match extract_table_elements(&node) {
            Ok(table) => tables.push(table),
            Err(e) => return Err(e),
        }
    }
    Ok(tables)
}

pub fn map_table_cell(node: &Node, f: fn(&Node) -> String) -> Result<Table, Error> {
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
            #[allow(clippy::expect_used)]
            let element = cell_node.element().expect("Expected element");
            let (row_size, col_size) = element_utils::extract_rowspan_and_colspan(element);
            let text = f(cell_node);
            #[allow(clippy::expect_used)]
            let is_header = element.name() == "th".into();
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
    let mut table = map_to_table(&map);
    for ((i, j), is_header) in header_map {
        if is_header {
            table.set_header(i, j);
        }
    }
    Ok(table)
}

fn map_to_table(map: &HashMap<(usize, usize), String>) -> Table {
    let rows = map.keys().map(|(i, _)| i).max().unwrap_or(&0) + 1;
    let cols = map.keys().map(|(_, j)| j).max().unwrap_or(&0) + 1;
    let mut table = Table::new((rows, cols));
    for ((i, j), text) in map {
        table.set(*i, *j, text.to_string());
    }
    table
}

fn extract_table_texts(node: &Node) -> Result<Table, Error> {
    map_table_cell(node, |node| node.string_value())
}

fn extract_table_elements(node: &Node) -> Result<Table, Error> {
    map_table_cell(node, element_to_html)
}

fn element_to_html(node: &Node) -> String {
    let mut buf = Vec::new();
    let package = sxd_document::Package::new();
    let doc = package.as_document();
    let root = doc.root();
    if let Some(element) = node.element() {
        root.append_child(element);
    }
    #[allow(clippy::expect_used)]
    sxd_document::writer::format_document(&doc, &mut buf).expect("Failed to format document");
    #[allow(clippy::expect_used)]
    String::from_utf8(buf).expect("Failed to convert to UTF-8")
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
        let result = extract_table_texts_from_document(html).unwrap();
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
        let result = extract_table_texts_from_document(html).unwrap();
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
        let result = extract_table_texts_from_document(html).unwrap();
        assert_eq!(result.len(), 0);

        // empty html
        let html = r#""#;
        let result = extract_table_texts_from_document(html).unwrap();
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
        let result = extract_table_texts_from_document(html).unwrap();
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
        let result = extract_table_texts_from_document(html).unwrap();
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
        let result = extract_table_texts_from_document(html).unwrap();
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

        let result = extract_table_texts_from_document(html).unwrap();
        assert_eq!(result.len(), 6);
        assert_eq!(result[0].to_csv().unwrap(), "A,A,B\nA,A,C\n");
        assert_eq!(result[1].to_csv().unwrap(), "a,b,c\nd,e,f\n");
        assert_eq!(result[2].to_csv().unwrap(), "a,b,c,d\ne,f,f,d\ni,j,k,l\n");
        assert_eq!(result[3].to_csv().unwrap(), "a,b,c,d\ne,f,f,f\ni,j,k,l\n");
        assert_eq!(result[4].to_csv().unwrap(), "a,b,c,d\ne,f,f,g\nh,f,f,i\n");
        assert_eq!(result[5].to_csv().unwrap(), "a,b,c,d\ne,f,g,\nh,i,,\n");
    }
}
