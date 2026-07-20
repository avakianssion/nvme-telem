---
default: minor
---

# Add `Device` accessor methods for all Identify Controller sub-structs

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
