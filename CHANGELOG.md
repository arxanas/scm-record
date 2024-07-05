# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## [Unreleased] - ReleaseDate

### Added

- (#58) Pressing question mark '?' pops up a help dialog.

## [0.3.0] - 2024-05-26

### Added

- (#41) When collapsing editable sections, the uneditable context lines between those collapsed editable sections is now also hidden.

### Fixed

- (#31) Console mode settings are now undone in LIFO order, improving Powershell integration.
- (#47) The alternate screen is now cleared before starting, improving Mosh integration.

## [v0.2.0] - 2023-12-25

### Added

- Support invoking a commit editor while selecting changes.

### Changed

- The maximum file view width is now 120 columns.
- (#11) Render a message when there are no changes to view.

### Fixed

- Fixed typo in "next item" menu item.
- Fixed build without `serde` feature.

## [v0.1.0] - 2023-03-01

Initial release as part of git-branchless v0.7.0.
