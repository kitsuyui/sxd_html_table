/// Returns `(rowspan, colspan)` for an HTML element.
///
/// The first tuple element is the row span and the second is the column span.
/// Missing or non-numeric attributes default to `1`.
pub fn extract_rowspan_and_colspan(element: sxd_document::dom::Element) -> (usize, usize) {
    let rowspan = extract_span(element, "rowspan");
    let colspan = extract_colspan(element);
    (rowspan, colspan)
}

fn extract_span(element: sxd_document::dom::Element, name: &str) -> usize {
    element
        .attribute_value(name)
        .unwrap_or("1")
        .parse::<usize>()
        .unwrap_or(1)
}

fn extract_colspan(element: sxd_document::dom::Element) -> usize {
    extract_span(element, "colspan").max(1)
}
