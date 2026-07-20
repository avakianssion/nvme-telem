# Roadmap to 1.0

Working list of what's blocking a 1.0 release. Not a schedule — items get
checked off as they land, in roughly the order below.

## 1. Finish the `Device` API

`types.rs` already builds structs for every category the Identify
Controller data covers (`CtrlCapacity`, `CtrlCapabilities`, `CtrlLimits`,
`CtrlThermals`, `CtrlFirmware`, `CtrlPowerStates`, `CtrlHostMemory`,
`CtrlArbitration`, `CtrlAdvanced`, `CtrlCommandSets`, `CtrlFabric`, ...),
but `telemetry.rs` only exposes four of them through `Device`
(`smart_log`, `identity`, `error_log`, `ocp_smart_log`). Everything else
is currently unreachable from the public API.

No new ioctls needed here — these all parse the same `nvme_id_ctrl` that
`identity()` already fetches. Add the missing accessors (or one call that
returns them all) before 1.0, since this is the actual point of the crate.

## 2. Freeze the field/type shapes in `types.rs`

Once this ships as 1.0, struct fields and method signatures are a semver
commitment. Do one deliberate pass over `types.rs` first — field names,
integer widths, anything that's `pub` today — while we can still change it
for free on 0.x.

## 3. Decide the fate of the deprecated free functions

`get_smart_log`, `get_controller_identity`, `get_error_log`, and
`get_smart_add_log` have been deprecated since 0.3.2 in favor of `Device`.
Either drop them for 1.0 or commit to keeping them — shipping
deprecated-since-0.3.2 code into a 1.0 release reads as unfinished.

## 4. Test the parsing logic, not just the plumbing

15 `#[test]`s total right now. Bitfield decoding, ASCII field parsing, and
timestamp math in `types.rs` can and should be tested without real
hardware — none of that needs an ioctl. Add fixtures/unit tests for the
parsing paths that aren't already covered.

## 5. Update crate-level docs

`lib.rs` already advertises "organized data model, separate structs for
different metric categories" ahead of the actual API. Once (1) lands,
update the module docs and README example to reflect what's actually
exposed.

## 6. Versioning

Currently 0.3.2. Figure out what version bump gets us to 1.0 under the
existing `knope`/`.changeset` flow, and whether any of the above should
ship as their own pre-1.0 releases first rather than one big jump.
