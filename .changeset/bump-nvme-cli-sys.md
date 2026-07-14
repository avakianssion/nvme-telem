---
default: patch
---

# Update nvme-cli-sys to 1.0

Fixes docs.rs documentation builds, which previously failed for every
published version due to a bug in the older dependency. API docs are
now available at docs.rs/nvme-telem. No changes to this crate's API
or behavior.
