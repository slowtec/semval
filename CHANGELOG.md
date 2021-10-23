# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Changed

### Removed

## [0.1.8] - 2021-10-23

No (notable) API changes.

- Use GitHub Actions for CI
- Update Rust edition from 2018 to 2021

## [0.1.7] - 2021-01-12

### Added

- Added implicit implementation of `Validate` for `Vec<T>` if feature `std` is enabled

## [0.1.6] - 2020-05-25

### Added

- Added implicit implementation of `Validate` for slices

## [0.1.5] - 2020-05-23

### Added

- Added implicit implementation of `Validate` trait reference types

## 0.1.4 - 2020-05-23

***Unreleased***

## 0.1.3 - 2019-11-28

### Added

- Added utility trait `IsValid` for boolean validity checks

## [0.1.2] - 2019-10-07

### Changed

- Renamed functions with a *map* parameter:
  - `validate_and_map()` becomes `validate_with()`
  - `map_and_merge_result()` becomes `merge_result_with()`

## 0.1.1 - 2019-09-09

### Added

- Added traits `ValidatedFrom` and `IntoValidated` for type-safe validation

### Changed

- Generalized `validate()` for all `Invalidity` types that implement `Into`

## 0.1.0 - 2019-09-03

### Added

- Initial public release

[Unreleased]: https://github.com/slowtec/semval/compare/v0.1.8...main
[0.1.8]: https://github.com/slowtec/semval/releases/v0.1.8
[0.1.7]: https://github.com/slowtec/semval/releases/v0.1.7
[0.1.6]: https://github.com/slowtec/semval/releases/v0.1.6
[0.1.5]: https://github.com/slowtec/semval/releases/v0.1.5
