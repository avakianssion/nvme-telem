---
default: minor
---

# Add `serial_number` to all Identify Controller sub-structs

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
