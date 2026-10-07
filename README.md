### diml

[![crates.io][crates-badge]][crates-url]
[![coverage][cov-badge]][cov-url]
![build-linux-gnu][build-badge-linux-gnu]
![build-linux-musl][build-badge-linux-musl]
![build-macos][build-badge-macos]
![build-macos-aarch64][build-badge-macos-aarch64]
![build-windows][build-badge-windows]
[![mit-license][mit-badge]][mit-license-url]
[![apache-license][apache-badge]][apache-license-url]
[![cc][cc-badge]][cc-url]
[![mbh][mbh-badge]][mbh-url]
[![es][es-badge]][es-url]

[crates-badge]: https://img.shields.io/crates/v/diml.svg
[crates-url]: https://crates.io/crates/diml
[cov-badge]: https://img.shields.io/badge/cov-100%25-21b577.svg
[cov-url]: https://crates.io/crates/coverio
[build-badge-linux-gnu]: https://github.com/EngosSoftware/diml/actions/workflows/build-linux-gnu.yml/badge.svg
[build-badge-linux-musl]: https://github.com/EngosSoftware/diml/actions/workflows/build-linux-musl.yml/badge.svg
[build-badge-macos]: https://github.com/EngosSoftware/diml/actions/workflows/build-macos.yml/badge.svg
[build-badge-macos-aarch64]: https://github.com/EngosSoftware/diml/actions/workflows/build-macos-aarch64.yml/badge.svg
[build-badge-windows]: https://github.com/EngosSoftware/diml/actions/workflows/build-windows.yml/badge.svg
[mit-badge]: https://img.shields.io/badge/License-MIT-9370DB.svg
[mit-url]: https://opensource.org/licenses/MIT
[mit-license-url]: https://github.com/EngosSoftware/diml/blob/main/LICENSE-MIT
[apache-badge]: https://img.shields.io/badge/License-Apache%202.0-9370DB.svg
[apache-url]: https://www.apache.org/licenses/LICENSE-2.0
[apache-license-url]: https://github.com/EngosSoftware/diml/blob/main/LICENSE
[apache-notice-url]: https://github.com/EngosSoftware/diml/blob/main/NOTICE
[cc-badge]: https://img.shields.io/badge/Contributor%20Covenant-2.1-9370DB.svg
[cc-url]: https://github.com/EngosSoftware/diml/blob/main/CODE_OF_CONDUCT.md
[mbh-badge]: https://img.shields.io/badge/Made_by-HUMAN-F5DEB3.svg
[mbh-url]: https://github.com/DariuszDepta
[es-badge]: https://img.shields.io/badge/at-Engos_Software-ADD8E6.svg
[es-url]: https://engos.de
[repository-url]: https://github.com/EngosSoftware/diml

# Delimiter-Indented Markup Language (DIML) parser

## Overview

Delimiter-Indented Markup Language (DIML) is a minimal, line-based markup language
for writing hierarchical text.

Each node starts with a delimiter character, followed by an optional name and free-form content.
Indentation determines nesting, so the structure is visible at a glance.

The document itself sets the rules:
- its first character becomes the delimiter,
- and its first indentation sets the indentation unit.

There are no closing tags, no quoting and no escaping.
Free-form content runs until the next node.
Every line, including the last one, must end with a newline.

## License

Licensed under either of

- [MIT license][mit-url] (see [LICENSE-MIT][mit-license-url]) or
- [Apache License, Version 2.0][apache-url] (see [LICENSE][apache-license-url] and [NOTICE][apache-notice-url])

at your option.

## Contribution

Any contributions to [diml][repository-url] are greatly appreciated.
All contributions intentionally submitted for inclusion in the work by you,
shall be dual licensed as above, without any additional terms or conditions.
