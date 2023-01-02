use sxd_xpath::{nodeset::Node, Context, Factory, Value};

#[derive(Debug)]
pub enum Error {
    TableNotFound,
}

pub fn find_table_from_document(html: &str) -> Result<Vec<String>, Error> {
    let package = sxd_html::parse_html(html);
    let document = package.as_document();
    #[allow(clippy::expect_used)]
    let val = evaluate_xpath_node(document.root(), "//table").expect("XPath evaluation failed");
    let Value::Nodeset(set) = val else {
        panic!("Expected node set");
    };

    let mut tables = vec![];
    for node in set.document_order() {
        tables.push(node.string_value());
    }
    Ok(tables)
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
        let result = find_table_from_document(html).unwrap();
        assert_eq!(result.len(), 1);

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
                        <td>1</td>
                        <td>2</td>
                    </tr>
                </table>
            </body>
        </html>
        "#;
        let result = find_table_from_document(html).unwrap();
        assert_eq!(result.len(), 2);

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
        let result = find_table_from_document(html).unwrap();
        assert_eq!(result.len(), 0);

        // empty html
        let html = r#""#;
        let result = find_table_from_document(html).unwrap();
        assert_eq!(result.len(), 0);
    }
}
