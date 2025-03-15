# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## [Unreleased] - ReleaseDate

## [0.8.0] - 2025-03-15

## [0.7.0] - 2025-03-15

## [0.6.0] - 2025-03-15

### Changed

- BREAKING (#93): File mode changes (including file creation and file deletion) should now always be represented as `Section::FileMode`.
- (#95) Some selections/deselections are now performed automatically to prevent impossible combinations (like trying to delete a file but leave lines in it)

## [0.5.0] - 2025-01-10

### Changed

- BREAKING (#39): Pressing the "left" or "h" keys now folds the current section if it is unfolded instead of moving to the outer item. You can still move to the outer item directly without automatic folding by pressing shift-left or shift-h.

## [0.4.0] - 2024-10-09

### Added

- (#45): ctrl-up and ctrl-down added as keyboard shortcuts to scroll by individual lines.
- (#45): ctrl-page-up and ctrl-page-down added as keyboard shortcuts to scroll by pages.
- (#58): Pressing question mark '?' pops up a help dialog.

### Changed

- BREAKING (#45): page-up and page-down now jump to the next section of the same type. The old behavior can be accessed with ctrl-page-up and ctrl-page-down.
- BREAKING (#52): Minimum supported Rust version (MSRV) is now 1.74.
- BREAKING (#53): `scm-diff-editor` has been extracted to its own crate and can be installed as its own stand-alone binary.
- (#46): Checkmarks changed to Unicode symobls.
- (#61): Control characters are replaced with Unicode symbols for rendering.

### Fixed

- (#37): Fixed redraw issues when rendering tabs and other non-printing characters.

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
