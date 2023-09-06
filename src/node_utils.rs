use std::collections::HashMap;

use sxd_xpath::{nodeset::Node, Context, Factory, Value};

use crate::{element_utils, table::Table, Error};

struct TableSupport<'a>(Node<'a>);

impl<'a> TableSupport<'a> {
    fn tr_nodes(&self) -> Result<Vec<Node<'a>>, Error> {
        let tr_nodes = match evaluate_xpath_node(self.0, "./tbody/tr") {
            Ok(Value::Nodeset(tr_nodes)) => tr_nodes,
            _ => return Err(Error::InvalidDocument),
        };
        Ok(tr_nodes.document_order())
    }
    fn td_nodes(&self, tr: Node<'a>) -> Result<Vec<Node<'a>>, Error> {
        let td_nodes = match evaluate_xpath_node(tr, "./td|./th") {
            Ok(Value::Nodeset(td_nodes)) => td_nodes,
            _ => return Err(Error::InvalidDocument),
        };
        Ok(td_nodes.document_order())
    }
}

pub fn evaluate_xpath_node<'a>(
    node: impl Into<Node<'a>>,
    expr: &str,
) -> Result<Value<'a>, sxd_xpath::Error> {
    let factory = Factory::new();
    let expression = factory.build(expr)?;
    let expression = expression.ok_or(sxd_xpath::Error::NoXPath)?;
    let context = Context::new();
    expression
        .evaluate(&context, node.into())
        .map_err(Into::into)
}

fn extract_table_nodes<'a>(node: impl Into<Node<'a>>) -> Result<Vec<Node<'a>>, Error> {
    let val = evaluate_xpath_node(node, "//table").map_err(Error::XPathEvaluationError)?;
    let Value::Nodeset(table_nodes) = val else {
        return Err(Error::TableNotFound);
    };
    Ok(table_nodes.document_order())
}

pub fn extract_table_nodes_to_table<'a>(
    node: impl Into<Node<'a>>,
) -> Result<Vec<Table<Node<'a>>>, Error> {
    let mut tables = vec![];
    for node in extract_table_nodes(node)? {
        tables.push(node_to_table(node)?);
    }
    Ok(tables)
}

fn node_to_table<'a>(node: impl Into<Node<'a>>) -> Result<Table<Node<'a>>, Error> {
    let mut map: HashMap<(usize, usize), Node> = HashMap::new();
    let t = TableSupport(node.into());
    for (row_index, tr_node) in t.tr_nodes()?.iter().enumerate() {
        for td_node in t.td_nodes(*tr_node)? {
            let mut col_index = 0;
            let Some(element) = td_node.element() else {
                return Err(Error::InvalidDocument);
            };
            let (row_size, col_size) = element_utils::extract_rowspan_and_colspan(element);
            while map.contains_key(&(row_index, col_index)) {
                col_index += 1;
            }
            for k in 0..row_size {
                for l in 0..col_size {
                    let row = row_index + k;
                    let col = col_index + l;
                    map.insert((row, col), td_node);
                }
            }
        }
    }
    let rows = map.keys().map(|(i, _)| i).max().unwrap_or(&0) + 1;
    let cols = map.keys().map(|(_, j)| j).max().unwrap_or(&0) + 1;
    let mut table = Table::new((rows, cols));
    for ((i, j), item) in map {
        table.set(i, j, item);
    }
    Ok(table)
}
