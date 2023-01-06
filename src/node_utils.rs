use std::collections::HashSet;

use sxd_xpath::{nodeset::Node, Context, Factory, Value};

use crate::{element_utils, Error};

struct TableSupport<'a>(&'a Node<'a>);

impl<'a> TableSupport<'a> {
    fn tr_nodes(&self) -> Result<Vec<Node<'a>>, Error> {
        let tr_nodes = match evaluate_xpath_node(*self.0, "./tbody/tr") {
            Ok(Value::Nodeset(tr_nodes)) => tr_nodes,
            _ => return Err(Error::InvalidDocument),
        };
        Ok(tr_nodes.document_order())
    }
    fn td_nodes(&self, tr: &Node<'a>) -> Result<Vec<Node<'a>>, Error> {
        let td_nodes = match evaluate_xpath_node(*tr, "./td|./th") {
            Ok(Value::Nodeset(td_nodes)) => td_nodes,
            _ => return Err(Error::InvalidDocument),
        };
        Ok(td_nodes.document_order())
    }
}

pub fn map_table_cell<T, F>(node: Node, mut f: F) -> Result<(), Error>
where
    F: FnMut(&Node, usize, usize) -> T,
{
    let t = TableSupport(&node);
    let mut set: HashSet<(usize, usize)> = HashSet::new();
    for (row_index, tr_node) in t.tr_nodes()?.iter().enumerate() {
        for td_node in t.td_nodes(tr_node)? {
            let mut col_index = 0;
            let Some(element) = td_node.element() else {
                return Err(Error::InvalidDocument);
            };
            let (row_size, col_size) = element_utils::extract_rowspan_and_colspan(element);
            while set.contains(&(row_index, col_index)) {
                col_index += 1;
            }
            for k in 0..row_size {
                for l in 0..col_size {
                    let row = row_index + k;
                    let col = col_index + l;
                    set.insert((row, col));
                    f(&td_node, row, col);
                }
            }
        }
    }
    Ok(())
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
