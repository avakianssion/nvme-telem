# Changelog

All notable changes to nvme-telem are documented here, generated
automatically from change files at release time.
## 0.3.3 (2026-08-04)

### Features

#### Add `serial_number` to all Identify Controller sub-structs

`CtrlCapacity`, `CtrlCapabilities`, `CtrlLimits`, `CtrlThermals`,
`CtrlFirmware`, `CtrlPowerStates`, `CtrlHostMemory`, `CtrlArbitration`,
`CtrlDiagnostics`, `CtrlAdvanced`, `CtrlCommandSets`, and `CtrlFabric` now
carry a `serial_number` field alongside `nvme_name`, matching
`CtrlIdentity`, `NvmeSmartLog`, `NvmeErrorLog`, and `OcpSmartData`.

Every metrics struct this crate serializes now self-identifies with both
the NVMe device name (e.g. `nvme0`) and the drive's serial number, so a
JSON object can be traced back to its physical drive even when handled
independently of the others. No new device reads are required — the
serial number is parsed from the same `nvme_id_ctrl` response each of
these structs was already built from.

#### Add `Device` accessor methods for all Identify Controller sub-structs

`Device` now exposes `capacity`, `capabilities`, `limits`, `thermals`,
`firmware`, `power_states`, `host_memory`, `arbitration`, `diagnostics`,
`advanced`, `command_sets`, and `fabric`, alongside the existing
`identity`. Each issues its own read-only Identify Controller Admin
command and returns the corresponding `Ctrl*` struct, so callers can
fetch just the category of controller data they need instead of
building `CtrlIdentity` and picking out fields by hand.

Note that calling several of these accessors on the same device fetches
the same underlying `nvme_id_ctrl` data multiple times, since each
issues its own Identify command rather than sharing a cached copy.

### Fixes

#### Remove unused src/nvme/helpers.rs

`src/nvme/helpers.rs` was never declared as a module (no `mod helpers;`
anywhere in the crate), so it was not compiled and unreachable. It
contained `parse_ascii_field` and `convert_cchar_to_u8_array_16`,
duplicates of the functions of the same name already defined and used
in `src/nvme/types.rs`. No behavior change.

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
