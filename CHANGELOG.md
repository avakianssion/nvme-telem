# Changelog

All notable changes to nvme-telem are documented here, generated
automatically from change files at release time.
## 0.3.1 (2026-07-12)

### Fixes

#### Add automated release workflow

Versioning and changelogs are now driven by change files in `.changeset/`
(one per MR). Releases are prepared with knope on the `rc` branch and
published to crates.io automatically when `rc` merges to `main`.
See CONTRIBUTING.md and RELEASING.md.

#### Add unit tests for ocp and types

New unit tests were added to ocp.rs and types.rs help with automated testing.

#### Update main readme with more useful information

Useful build and quick start guide information in the main readme.
