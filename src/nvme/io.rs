//! Low-level NVMe I/O operations.
//!
//! This module provides direct access to NVMe device operations via ioctl system calls.
//! These functions perform raw hardware communication and return unparsed C structures
//! from the NVMe specification.
//!
//! # Safety
//!
//! All functions in this module require root/sudo privileges to access `/dev/nvme*` devices.
//! The functions use `unsafe` blocks internally to perform ioctl system calls, but the
//! unsafe code is isolated and the public API is safe to use.
//!
//! # Usage
//!
//! Most users should prefer the high-level API in the [`telemetry`](super::telemetry) module,
//! which automatically handles device identification and data parsing. Use this module only
//! when you need direct access to raw NVMe data structures or when implementing custom
//! telemetry collection logic.
//!
//! # Examples
//!
//! Reading raw SMART data:
//!
//! ```no_run
//! use nvme_telem::nvme::io::read_nvme_smart_log;
//!
//! # fn main() -> std::io::Result<()> {
//! let raw_smart = read_nvme_smart_log("/dev/nvme0")?;
//! println!("Temperature: {} K", u16::from_le_bytes([
//!     raw_smart.temperature[0],
//!     raw_smart.temperature[1]
//! ]));
//! # Ok(())
//! # }
//! ```
//!
//! Reading controller identification:
//!
//! ```no_run
//! use nvme_telem::nvme::io::read_nvme_id_ctrl;
//!
//! # fn main() -> std::io::Result<()> {
//! let id_ctrl = read_nvme_id_ctrl("/dev/nvme0")?;
//! println!("Vendor ID: 0x{:04x}", id_ctrl.vid);
//! # Ok(())
//! # }
//! ```

use nvme_cli_sys::{
    nvme_admin_cmd, nvme_admin_opcode::nvme_admin_get_log_page,
    nvme_admin_opcode::nvme_admin_identify, nvme_error_log_page, nvme_id_ctrl, nvme_smart_log,
};
use std::fs::OpenOptions;
use std::io;
use std::mem::{size_of, zeroed};
use std::os::unix::io::AsRawFd;

// =============================================================================
// IDENTIFY COMMANDS
// =============================================================================

/// Extract raw nvme_id_ctrl using the Identify admin command.
///
/// # Arguments
/// * `dev_path` - Path to the NVMe device (e.g., "/dev/nvme0")
///
/// # Errors
/// Returns an error if:
/// - The device cannot be opened
/// - The admin command fails
/// - The NVMe controller returns a non-zero status
pub fn read_nvme_id_ctrl(dev_path: &str) -> io::Result<nvme_id_ctrl> {
    let file = OpenOptions::new()
        .read(true)
        .write(true) // Admin permission required
        .open(dev_path)?;

    let fd = file.as_raw_fd();

    // Identify Controller payload is 4096 bytes based on the C bindings
    let mut id: nvme_id_ctrl = unsafe { zeroed() };

    let id_ptr = &mut id as *mut nvme_id_ctrl as u64;
    let id_len = size_of::<nvme_id_ctrl>() as u32;

    let cns: u8 = 0x01;
    let cntlid: u16 = 0x0000;
    let cdw10: u32 = (cns as u32) | ((cntlid as u32) << 16);

    let mut cmd: nvme_admin_cmd = unsafe { zeroed() };
    cmd.opcode = nvme_admin_identify as u8; // Identify (0x06)
    cmd.nsid = 0x0000_0000;
    cmd.addr = id_ptr;
    cmd.data_len = id_len;
    cmd.cdw10 = cdw10;
    cmd.cdw11 = 0;
    cmd.timeout_ms = 1000;

    let ret = unsafe { nvme_cli_sys::nvme_ioctl_admin_cmd(fd, &mut cmd) };

    match ret {
        Ok(0) => Ok(id),
        Ok(status) => Err(io::Error::other(format!(
            "NVMe admin command failed, status={:#x}",
            status
        ))),
        Err(e) => Err(io::Error::other(e.to_string())),
    }
}

// =============================================================================
// LOG PAGE COMMANDS
// =============================================================================

/// Extract raw nvme_smart_log from an NVMe device.
///
/// The S.M.A.R.T./Health Information log page (Log ID 0x02) provides information
/// over the life of the controller and is retained across power cycles unless
/// otherwise specified.
///
/// # Arguments
/// * `dev_path` - Path to the NVMe device (e.g., "/dev/nvme0")
///
/// # Errors
/// Returns an error if:
/// - The device cannot be opened
/// - The admin command fails
/// - The NVMe controller returns a non-zero status
pub fn read_nvme_smart_log(dev_path: &str) -> io::Result<nvme_smart_log> {
    let file = OpenOptions::new()
        .read(true)
        .write(true) // Admin permission required
        .open(dev_path)?;

    let fd = file.as_raw_fd();

    // Allocate memory for the response
    let mut log: nvme_smart_log = unsafe { zeroed() };

    let log_ptr = &mut log as *mut nvme_smart_log as u64;
    let log_len = size_of::<nvme_smart_log>() as u32;

    let log_id: u8 = 0x02; // S.M.A.R.T./Health Information - Log Page Identifier 0x02
    let numd: u32 = log_len / 4 - 1;
    let cdw10: u32 = (log_id as u32) | (numd << 16);

    let mut cmd: nvme_admin_cmd = unsafe { zeroed() };
    cmd.opcode = nvme_admin_get_log_page as u8;
    // Per spec, namespace identifier must be 0x00000000 or 0xFFFFFFFF
    cmd.nsid = 0xFFFF_FFFF;
    cmd.addr = log_ptr;
    cmd.data_len = log_len;
    cmd.cdw10 = cdw10;
    cmd.cdw11 = 0;
    cmd.timeout_ms = 1000;

    let ret = unsafe { nvme_cli_sys::nvme_ioctl_admin_cmd(fd, &mut cmd) };

    match ret {
        Ok(0) => Ok(log),
        Ok(status) => Err(io::Error::other(format!(
            "NVMe admin command failed, status={:#x}",
            status
        ))),
        Err(e) => Err(io::Error::other(e.to_string())),
    }
}

/// Extract raw nvme_error_log_page from an NVMe device.
///
/// Internal function to read raw error log entries.
/// Users should call `get_error_log()` instead.
///
/// # Arguments
/// * `dev_path` - Path to the NVMe device (e.g., "/dev/nvme0")
/// * `num_entries` - Number of entries of error logs implemented by the vendor.
///
/// # Errors
/// Returns an error if:
/// - The device cannot be opened
/// - The admin command fails
/// - The NVMe controller returns a non-zero status
pub(super) fn read_error_log_raw(
    dev_path: &str,
    num_entries: u16,
) -> io::Result<Vec<nvme_error_log_page>> {
    let file = OpenOptions::new().read(true).write(true).open(dev_path)?;

    let fd = file.as_raw_fd();

    // Allocate buffer for multiple entries
    let entries_count = num_entries as usize;
    let mut entries = vec![unsafe { zeroed::<nvme_error_log_page>() }; entries_count];

    let log_ptr = entries.as_mut_ptr() as u64;
    let log_len = (entries_count * size_of::<nvme_error_log_page>()) as u32;

    let log_id: u8 = 0x01; // Error Information Log
    let numd: u32 = log_len / 4 - 1;
    let cdw10: u32 = (log_id as u32) | (numd << 16);

    let mut cmd: nvme_admin_cmd = unsafe { zeroed() };
    cmd.opcode = nvme_admin_get_log_page as u8;
    cmd.nsid = 0xFFFF_FFFF;
    cmd.addr = log_ptr;
    cmd.data_len = log_len;
    cmd.cdw10 = cdw10;
    cmd.cdw11 = 0;
    cmd.timeout_ms = 5000;

    let ret = unsafe { nvme_cli_sys::nvme_ioctl_admin_cmd(fd, &mut cmd) };

    match ret {
        Ok(0) => Ok(entries),
        Ok(status) => Err(io::Error::other(format!(
            "Error log command failed, status={:#x}",
            status
        ))),
        Err(e) => Err(io::Error::other(e.to_string())),
    }
}
