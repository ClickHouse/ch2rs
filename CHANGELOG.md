# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased] - ReleaseDate
### Added
- `rustls-tls` feature is enabled by default so that the library can work with HTTPS. Optionally, it is also possible to use `native-tls` instead.

## [0.1.7] - 2024-08-08
### Added
- Option `--derive` to add custom `#[derive]` attributes to generated structs ([#7]).
- Use column comment as a doc for the generated fields ([#8]).

[#8]: https://github.com/ClickHouse/ch2rs/pull/8
[#7]: https://github.com/ClickHouse/ch2rs/pull/7

## [0.1.6] - 2022-06-09
### Added
- Support `Bool`, `IPv4`, `IPv6`, `UUID`, `i128`, `u128` types.

## [0.1.5] - 2022-01-25
### Added
- `SimpleAggregateFunctions` support ([#2], [#3]).

[#3]: https://github.com/ClickHouse/ch2rs/pull/3
[#2]: https://github.com/ClickHouse/ch2rs/pull/2

## [0.1.4] - 2021-12-16
### Added
- Option `-I` to ignore specified columns.
- The package's version in generated code.

## [0.1.3] - 2021-07-29
### Added
- Support `Map(K, V)` types ([#1]).

[#1]: https://github.com/ClickHouse/ch2rs/pull/1

## [0.1.2] - 2021-06-01
### Fixed
- Format `DateTime64` right way.

## [0.1.1] - 2021-05-31
### Changed
- Use clickhouse v0.7

## [0.1.0] - 2021-05-30

<!-- next-url -->
[Unreleased]: https://github.com/ClickHouse/ch2rs/compare/v0.1.7...HEAD
[0.1.7]: https://github.com/ClickHouse/ch2rs/compare/v0.1.6...v0.1.7
[0.1.6]: https://github.com/ClickHouse/ch2rs/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/ClickHouse/ch2rs/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/ClickHouse/ch2rs/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/ClickHouse/ch2rs/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/ClickHouse/ch2rs/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/ClickHouse/ch2rs/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/ClickHouse/ch2rs/releases/tag/v0.1.0
