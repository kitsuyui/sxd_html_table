/// Maximum rowspan per HTML5 §4.9.11 (non-zero positive limit).
const MAX_ROWSPAN: usize = 65534;
/// Maximum colspan per HTML5 §4.9.11.
const MAX_COLSPAN: usize = 1000;

pub fn extract_rowspan_and_colspan(element: sxd_document::dom::Element) -> (usize, usize) {
    let rowspan = extract_span(element, "rowspan", MAX_ROWSPAN);
    let colspan = extract_span(element, "colspan", MAX_COLSPAN);
    (rowspan, colspan)
}

fn extract_span(element: sxd_document::dom::Element, name: &str, max: usize) -> usize {
    let raw = element
        .attribute_value(name)
        .unwrap_or("1")
        .parse::<usize>()
        .unwrap_or(1);
    raw.min(max)
}
