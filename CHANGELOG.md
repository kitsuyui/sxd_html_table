# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-10-26

### Breaking

- `Error` is non-exhaustive. Include a wildcard arm when matching errors.
- `Error::InvalidDocument` now carries a static description; match it as `InvalidDocument(_)`.
- `Error::XPathEvaluationError` now wraps a boxed error rather than `sxd_xpath::Error` directly. Use `std::error::Error::source` or downcast the boxed value when needed.

### Changed

- Add descriptive error messages and implement `std::error::Error`.
- Improve nested-table handling and span validation.

## [0.1.0] - Initial release

### Added

- `extract_table_nodes_to_table`: extract HTML table nodes from an SXD document into a structured `Table`
- `Table<T>` with support for `colspans`, `rowspans`, `<th>`/`<td>` distinction
- CSV output via `Table::to_csv`
- String table conversion via `Table::to_string_table`

[Unreleased]: https://github.com/kitsuyui/sxd_html_table/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/kitsuyui/sxd_html_table/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/kitsuyui/sxd_html_table/releases/tag/v0.1.0
