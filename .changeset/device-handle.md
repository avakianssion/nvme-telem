---
default: minor
---

# Add Device handle API; deprecate path-per-call free functions

Introduces `nvme::Device`, an owned handle to an open NVMe character
device (`Device::open(path)?`), with `smart_log()`, `identity()`,
`error_log()`, and `ocp_smart_log()` methods that reuse the same file
descriptor instead of reopening the device path per call.

The existing free functions (`get_smart_log`, `get_controller_identity`,
`get_error_log`, `get_smart_add_log`) are now thin wrappers around
`Device` and are deprecated in favor of the new handle-based API. They
remain available and behave identically.
