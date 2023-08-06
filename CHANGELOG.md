<!-- SPDX-FileCopyrightText: slowtec GmbH -->
<!-- SPDX-License-Identifier: MPL-2.0 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2022-11-24

- Add `as_ref()` to `Validated`

### BREAKING CHANGES

- As `Validated` now has an inherent `as_ref()` method, code that used to call `AsRef::as_ref` on it
  may now behave differently. `AsRef` impl is still available and can be called explicitly, or
  simply use `Deref` instead.

## [0.4.1] - 2022-10-21

- Add support for `std::borrow::Cow`

## [0.4.0] - 2022-07-12

- Switch license from Apache/MIT to MPL-2.0

## [0.3.0] - 2022-04-14

- Tag validated values by wrapping them into `Validated`
- Rename `semval::Result` into `semval::ValidationResult` for disambiguation and consistency with
  `prelude`.

## [0.2.0] - 2022-01-29

- Upgraded to Rust 2021 Edition
- Added `#[must_use]` attribute to `Context` functions
- Removed deprecated functions

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

**_Unreleased_**

## 0.1.3 - 2019-11-28

### Added

- Added utility trait `IsValid` for boolean validity checks

## [0.1.2] - 2019-10-07

### Changed

- Renamed functions with a _map_ parameter:
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
