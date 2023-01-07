# sxd_html_table

[![crates.io](https://img.shields.io/crates/d/sxd_html_table)](https://crates.io/crates/sxd_html_table)
[![tests](https://github.com/kitsuyui/sxd_html_table/actions/workflows/test.yml/badge.svg)](https://github.com/kitsuyui/sxd_html_table/actions/workflows/test.yml)
[![codecov](https://codecov.io/gh/kitsuyui/sxd_html_table/branch/main/graph/badge.svg?token=CG4FPNVJI3)](https://codecov.io/gh/kitsuyui/sxd_html_table)
[![license](https://img.shields.io/crates/l/sxd_html_table)](https://github.com/kitsuyui/sxd_html_table#license)

# Provide features related to HTML tables

There are some complexities to deal with when dealing with HTML tables.

- There are colspans and rowspans, and the number of rows and columns is indeterminate.
- There are th and td, and the type of cell requires attention.

This library hides these complexities and makes it easy to deal with the structure of the table.
For example, you can convert an HTML table tag to a CSV file.

## Usage

```rust
use sxd_html_table::Table;

let html = r#"
<table>
  <tr>
    <th>header1</th>
    <th>header2</th>
  </tr>
  <tr>
    <td>data1</td>
    <td>data2</td>
  </tr>
</table>
"#;

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

let table = extract_table_texts_from_document(html).unwrap();
let csv = table.to_csv().unwrap();
assert_eq!(csv, "header1,header2\ndata1,data2\n");
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
