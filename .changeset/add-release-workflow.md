---
default: patch
---

# Add automated release workflow

Versioning and changelogs are now driven by change files in `.changeset/`
(one per MR). Releases are prepared with knope on the `rc` branch and
published to crates.io automatically when `rc` merges to `main`.
See CONTRIBUTING.md and RELEASING.md.
