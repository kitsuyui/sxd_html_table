pub mod element_utils;
pub mod node_utils;
pub mod table;
pub use crate::node_utils::extract_table_nodes_to_table;
pub use crate::table::Table;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    TableNotFound,
    InvalidDocument(&'static str),
    FailedToConvertToCSV,
    XPathEvaluationError(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TableNotFound => f.write_str("no table found in document"),
            Self::InvalidDocument(ctx) => write!(f, "invalid document: {ctx}"),
            Self::FailedToConvertToCSV => f.write_str("failed to convert table to CSV"),
            Self::XPathEvaluationError(err)
                if matches!(
                    err.downcast_ref::<sxd_xpath::Error>(),
                    Some(sxd_xpath::Error::NoXPath)
                ) =>
            {
                f.write_str("failed to evaluate XPath: XPath was empty")
            }
            Self::XPathEvaluationError(err) => write!(f, "failed to evaluate XPath: {err}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::XPathEvaluationError(err) => Some(err.as_ref()),
            Self::TableNotFound | Self::InvalidDocument(_) | Self::FailedToConvertToCSV => None,
        }
    }
}

impl From<sxd_xpath::Error> for Error {
    fn from(err: sxd_xpath::Error) -> Self {
        Self::XPathEvaluationError(Box::new(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as StdError;

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
    fn test_empty_table() {
        let html = r#"<html><body><table></table></body></html>"#;
        let result = extract_table_texts_from_document(html).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].rows().len(), 0);
        assert_eq!(result[0].to_csv().unwrap(), "");

        let html = r#"<html><body><table><tbody></tbody></table></body></html>"#;
        let result = extract_table_texts_from_document(html).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].rows().len(), 0);
        assert_eq!(result[0].to_csv().unwrap(), "");
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
        let result = extract_table_texts_from_document(html);
        assert!(matches!(result, Err(Error::TableNotFound)));

        // empty html
        let html = r#""#;
        let result = extract_table_texts_from_document(html);
        assert!(matches!(result, Err(Error::TableNotFound)));
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
    fn test_csv_formula_injection_sanitization() {
        let html = r#"
        <html>
            <body>
                <table>
                    <tr>
                        <td>=SUM(A1:A2)</td>
                        <td>+1</td>
                        <td>-1</td>
                        <td>@SUM</td>
                        <td>normal</td>
                    </tr>
                </table>
            </body>
        </html>
        "#;
        let result = extract_table_texts_from_document(html).unwrap();
        assert_eq!(result.len(), 1);
        let csv = result[0].to_csv().unwrap();
        assert_eq!(csv, "\t=SUM(A1:A2),\t+1,\t-1,\t@SUM,normal\n");
    }

    #[test]
    fn test_nested_table_excluded() {
        // A nested <table> is not returned as a separate entry.
        // Only the outermost table is returned; the inner table is accessible
        // as a node within the outer table's cells.
        let html = r#"
        <html>
            <body>
                <table>
                    <tr>
                        <td>outer</td>
                        <td>
                            <table>
                                <tr><td>inner</td></tr>
                            </table>
                        </td>
                    </tr>
                </table>
            </body>
        </html>
        "#;
        let result = extract_table_texts_from_document(html).unwrap();
        assert_eq!(
            result.len(),
            1,
            "nested table must not appear as a separate entry"
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

        let html = r#"
        <html>
            <body>
                <table>
                    <tr>
                        <td colspan="0">A</td>
                        <td>B</td>
                    </tr>
                    <tr>
                        <td>C</td>
                        <td>D</td>
                    </tr>
                </table>
            </body>
        </html>
        "#;
        let result = extract_table_texts_from_document(html).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].to_csv().unwrap(), "A,B\nC,D\n");

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

                <!-- rowspan="0" spans to the end of the row group -->
                <table>
                    <tr><td rowspan="0">a</td><td>b</td></tr>
                    <tr><td>c</td></tr>
                    <tr><td>d</td></tr>
                </table>
            </body>
        </html>
        "#;

        let result = extract_table_texts_from_document(html).unwrap();
        assert_eq!(result.len(), 7);
        assert_eq!(result[0].to_csv().unwrap(), "A,A,B\nA,A,C\n");
        assert_eq!(result[1].to_csv().unwrap(), "a,b,c\nd,e,f\n");
        assert_eq!(result[2].to_csv().unwrap(), "a,b,c,d\ne,f,f,d\ni,j,k,l\n");
        assert_eq!(result[3].to_csv().unwrap(), "a,b,c,d\ne,f,f,f\ni,j,k,l\n");
        assert_eq!(result[4].to_csv().unwrap(), "a,b,c,d\ne,f,f,g\nh,f,f,i\n");
        assert_eq!(result[5].to_csv().unwrap(), "a,b,c,d\ne,f,g,\nh,i,,\n");
        assert_eq!(result[6].to_csv().unwrap(), "a,b\na,c\na,d\n");
    }

    #[test]
    fn test_rejects_when_rowspan_fills_column_limit() {
        let html = format!(
            r#"
        <html>
            <body>
                <table>
                    <tr><td rowspan="0" colspan="{}">A</td></tr>
                    <tr><td>B</td></tr>
                </table>
            </body>
        </html>
        "#,
            crate::node_utils::MAX_TABLE_COLUMNS
        );
        match extract_table_texts_from_document(&html) {
            Err(Error::InvalidDocument(_)) => {}
            Err(_) => panic!("expected InvalidDocument"),
            Ok(_) => panic!("expected table extraction to fail"),
        }
    }

    #[test]
    fn test_rejects_colspan_larger_than_column_limit() {
        let html = format!(
            r#"
        <html>
            <body>
                <table>
                    <tr><td colspan="{}">A</td></tr>
                </table>
            </body>
        </html>
        "#,
            crate::node_utils::MAX_TABLE_COLUMNS + 1
        );
        match extract_table_texts_from_document(&html) {
            Err(Error::InvalidDocument(_)) => {}
            Err(_) => panic!("expected InvalidDocument"),
            Ok(_) => panic!("expected table extraction to fail"),
        }
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            Error::TableNotFound.to_string(),
            "no table found in document"
        );
        assert_eq!(
            Error::InvalidDocument("XPath ./tbody/tr did not return a nodeset").to_string(),
            "invalid document: XPath ./tbody/tr did not return a nodeset"
        );
        assert_eq!(
            Error::FailedToConvertToCSV.to_string(),
            "failed to convert table to CSV"
        );
        assert_eq!(
            Error::from(sxd_xpath::Error::NoXPath).to_string(),
            "failed to evaluate XPath: XPath was empty"
        );
    }

    #[test]
    fn error_implements_std_error() {
        fn assert_std_error<E: StdError>() {}

        assert_std_error::<Error>();
    }

    #[test]
    fn xpath_error_is_exposed_as_source() {
        let err = Error::from(sxd_xpath::Error::NoXPath);

        assert_eq!(err.source().unwrap().to_string(), "NoXPath");
    }

    #[test]
    fn error_converts_to_boxed_error_with_question_mark() {
        fn use_question_mark() -> Result<(), Box<dyn StdError>> {
            Err(Error::InvalidDocument("context"))?;
            Ok(())
        }

        assert_eq!(
            use_question_mark().unwrap_err().to_string(),
            "invalid document: context"
        );
    }

    #[test]
    fn test_thead_rows_included() {
        let html = r#"
        <html>
            <body>
                <table>
                    <thead>
                        <tr>
                            <th>Name</th>
                            <th>Age</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>Alice</td>
                            <td>30</td>
                        </tr>
                        <tr>
                            <td>Bob</td>
                            <td>25</td>
                        </tr>
                    </tbody>
                </table>
            </body>
        </html>
        "#;
        let result = extract_table_texts_from_document(html).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].to_csv().unwrap(), "Name,Age\nAlice,30\nBob,25\n");
    }

    #[test]
    fn test_tfoot_rows_included() {
        let html = r#"
        <html>
            <body>
                <table>
                    <tbody>
                        <tr>
                            <td>Alice</td>
                            <td>30</td>
                        </tr>
                    </tbody>
                    <tfoot>
                        <tr>
                            <td>Total</td>
                            <td>1</td>
                        </tr>
                    </tfoot>
                </table>
            </body>
        </html>
        "#;
        let result = extract_table_texts_from_document(html).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].to_csv().unwrap(), "Alice,30\nTotal,1\n");
    }

    #[test]
    fn test_to_string_table_with_header_thead() {
        let html = r#"
        <html>
            <body>
                <table>
                    <thead>
                        <tr>
                            <th>Name</th>
                            <th>Score</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>Alice</td>
                            <td>100</td>
                        </tr>
                    </tbody>
                </table>
            </body>
        </html>
        "#;
        let package = sxd_html::parse_html(html);
        let document = package.as_document();
        let tables = extract_table_nodes_to_table(document.root()).unwrap();
        assert_eq!(tables.len(), 1);
        let with_header = tables[0].to_string_table_with_header();
        let rows = with_header.rows();
        // THEAD row: both cells should be flagged as headers (th elements)
        assert_eq!(rows[0][0], Some(&("Name".to_string(), true)));
        assert_eq!(rows[0][1], Some(&("Score".to_string(), true)));
        // TBODY row: cells should not be flagged as headers (td elements)
        assert_eq!(rows[1][0], Some(&("Alice".to_string(), false)));
        assert_eq!(rows[1][1], Some(&("100".to_string(), false)));
    }

    #[test]
    fn test_rowspan_exceeds_actual_rows() {
        // rowspan larger than the actual row count must be clamped to the remaining rows
        // per HTML5 §4.9.11, not produce phantom rows
        let html = r#"
        <html>
            <body>
                <table>
                    <tr><td rowspan="99999">a</td><td>b</td></tr>
                    <tr><td>c</td></tr>
                    <tr><td>d</td></tr>
                </table>
            </body>
        </html>
        "#;
        let result = extract_table_texts_from_document(html).unwrap();
        assert_eq!(result.len(), 1);
        // table has exactly 3 rows, not 99999
        assert_eq!(result[0].to_csv().unwrap(), "a,b\na,c\na,d\n");
    }

    #[test]
    fn test_rowspan_zero_multiple_in_same_row() {
        // When a row has multiple rowspan=0 cells, only the first one spans to the
        // end of the row group. Subsequent rowspan=0 cells are treated as rowspan=1
        // to prevent state-machine ambiguity (multiple cells competing for the same
        // row-group span).
        let html = r#"
        <html>
            <body>
                <table>
                    <tr><td rowspan="0">a</td><td rowspan="0">b</td></tr>
                    <tr><td>c</td></tr>
                </table>
            </body>
        </html>
        "#;
        let result = extract_table_texts_from_document(html).unwrap();
        assert_eq!(result.len(), 1);
        // "a" spans both rows (first rowspan=0 in the row).
        // "b" appears only in row 0 (second rowspan=0 treated as rowspan=1).
        // "c" fills row 1 at col 1.
        assert_eq!(result[0].to_csv().unwrap(), "a,b\na,c\n");
    }
}
