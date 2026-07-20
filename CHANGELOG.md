# Changelog

All notable changes to nvme-telem are documented here, generated
automatically from change files at release time.
## 0.3.2 (2026-07-14)

### Features

#### Add Device handle API; deprecate path-per-call free functions

Introduces `nvme::Device`, an owned handle to an open NVMe character
device (`Device::open(path)?`), with `smart_log()`, `identity()`,
`error_log()`, and `ocp_smart_log()` methods that reuse the same file
descriptor instead of reopening the device path per call.

The existing free functions (`get_smart_log`, `get_controller_identity`,
`get_error_log`, `get_smart_add_log`) are now thin wrappers around
`Device` and are deprecated in favor of the new handle-based API. They
remain available and behave identically.

### Fixes

#### Update nvme-cli-sys to 1.0

Fixes docs.rs documentation builds, which previously failed for every
published version due to a bug in the older dependency. API docs are
now available at docs.rs/nvme-telem. No changes to this crate's API
or behavior.

## 0.3.1 (2026-07-12)

### Fixes

#### Add automated release workflow

Versioning and changelogs are now driven by change files in `.changeset/`
(one per MR). Releases are prepared with knope on the `rc` branch and
published to crates.io automatically when `rc` merges to `main`.
See CONTRIBUTING.md.

#### Add unit tests for ocp and types

New unit tests were added to ocp.rs and types.rs help with automated testing.

#### Update main readme with more useful information

Useful build and quick start guide information in the main readme.
