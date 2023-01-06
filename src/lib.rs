pub mod element_utils;
pub mod node_utils;
pub mod table;
pub use crate::node_utils::extract_table_nodes_to_table;
pub use crate::table::Table;

#[derive(Debug)]
pub enum Error {
    TableNotFound,
    InvalidDocument,
    FailedToConvertToCSV,
    XPathEvaluationError(sxd_xpath::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extract_table_texts_from_document(html: &str) -> Result<Vec<Table<String>>, Error> {
        let package = sxd_html::parse_html(html);
        let document = package.as_document();
        let tables = extract_table_nodes_to_table(document.root())?;
        let tables = tables
            .into_iter()
            .map(|table| table.to_string_table())
            .collect();
        Ok(tables)
    }

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
    fn test_table_item_xpath() {
        let html = r#"
        <html>
            <body>
                <table>
                    <tr>
                        <th class="aaa">1</th>
                        <td><a href="https:://example.com/">Hello, World!</a></td>
                    </tr>
                    <tr>
                        <td>
                            <div>
                                <p>3</p>
                                <p>4</p>
                            </div>
                        </td>
                        <td>4</td>
                    </tr>
                </table>
            </body>
        </html>
        "#;
        let package = sxd_html::parse_html(html);
        let document = package.as_document();
        let tables = extract_table_nodes_to_table(document.root()).unwrap();
        assert_eq!(tables.len(), 1);
        let csv1 = tables[0]
            .map(|_, _, node| match node.element() {
                Some(element) => {
                    if let Some(cls) = element.attribute_value("class") {
                        return cls;
                    }
                    "empty"
                }
                None => "empty",
            })
            .to_csv();
        assert_eq!(csv1.unwrap(), "aaa,empty\nempty,empty\n");

        let csv2 = tables[0]
            .map(|_, _, node| {
                for node in node.children().iter() {
                    if let Some(element) = node.element() {
                        if let Some(href) = element.attribute_value("href") {
                            return href.to_string();
                        }
                    }
                }
                "empty".to_string()
            })
            .to_csv();
        assert_eq!(csv2.unwrap(), "empty,https:://example.com/\nempty,empty\n");

        let csv3 = tables[0]
            .map(|_, _, node| node.string_value().trim().to_string())
            .to_csv();
        assert_eq!(
            csv3.unwrap(),
            "1,\"Hello, World!\"\n\"3\n                                4\",4\n"
        );
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
