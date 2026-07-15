---
default: patch
---

# Remove unused src/nvme/helpers.rs

`src/nvme/helpers.rs` was never declared as a module (no `mod helpers;`
anywhere in the crate), so it was not compiled and unreachable. It
contained `parse_ascii_field` and `convert_cchar_to_u8_array_16`,
duplicates of the functions of the same name already defined and used
in `src/nvme/types.rs`. No behavior change.
