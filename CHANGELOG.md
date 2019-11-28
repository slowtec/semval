# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Changed

### Removed

## [0.1.3] - 2019-11-28

### Added

- Added utility trait `IsValid` for boolean validity checks

## [0.1.2] - 2019-10-07

### Changed

- Renamed functions with a *map* parameter:
    - `validate_and_map()` becomes `validate_with()`
    - `map_and_merge_result()` becomes `merge_result_with()`

## [0.1.1] - 2019-09-09

### Added

- Added traits `ValidatedFrom` and `IntoValidated` for type-safe validation

### Changed

- Generalized `validate()` for all `Invalidity` types that implement `Into`

## [0.1.0] - 2019-09-03

### Added

- Initial public release

[Unreleased]: https://github.com/slowtec/semval/compare/v0.1.3...master
[0.1.3]: https://github.com/slowtec/semval/releases/v0.1.3
[0.1.2]: https://github.com/slowtec/semval/releases/v0.1.2
[0.1.1]: https://github.com/slowtec/semval/releases/v0.1.1
[0.1.0]: https://github.com/slowtec/semval/releases/v0.1.0
