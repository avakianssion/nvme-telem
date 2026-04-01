//! High-level telemetry and monitoring API for NVMe devices.
//!
//! This module provides functions for collecting telemetry data
//! from NVMe controllers, automatically handling device identification and
//! data enrichment.

use crate::nvme::io::*;
use crate::nvme::ocp::{OcpSmartData, read_ocp_smart_log};
use crate::nvme::types::{self, *};
use std::fs;

/// Retrieve S.M.A.R.T./Health Information from an NVMe device.
///
/// This function collects comprehensive health and telemetry data from the specified
/// NVMe controller, including device identification (serial number) for proper tracking.
/// The data includes temperature, wear indicators, power statistics, error counts, and
/// thermal management metrics.
///
/// # Arguments
///
/// * `dev_path` - Path to the NVMe character device (e.g., `"/dev/nvme0"`)
///
/// # Returns
///
/// Returns [`NvmeSmartLog`] containing:
/// - Device identification (name and serial number)
/// - Critical warnings and health status
/// - Temperature readings from all available sensors
/// - Storage capacity usage (percentage used, available spare)
/// - Lifetime statistics (power cycles, power-on hours, data read/written)
/// - Error and reliability metrics (media errors, unsafe shutdowns)
/// - Thermal management history
///
/// # Errors
///
/// This function will return an error if:
/// - The device path does not exist or cannot be opened
/// - The process lacks sufficient permissions (requires root/sudo)
/// - The NVMe controller does not respond or returns an error status
/// - The device is not a valid NVMe controller
pub fn get_smart_log(dev_path: &str) -> Result<NvmeSmartLog> {
    let nvme_name = dev_path.trim_start_matches("/dev/").to_string();

    // Get serial number from Identify Controller
    let id_ctrl = read_nvme_id_ctrl(dev_path)?;
    let serial_number = types::parse_ascii_field(&id_ctrl.sn);

    // Get SMART data
    let raw_smart = read_nvme_smart_log(dev_path)?;

    // Combine into complete struct
    Ok(NvmeSmartLog::new(nvme_name, serial_number, &raw_smart))
}

/// Retrieve Error Information Log from an NVMe device.
///
/// This function collects the complete error history from the specified NVMe controller,
/// including device identification. The error log contains detailed information about
/// all errors encountered by the controller, stored in a circular buffer.
///
/// The number of error entries retrieved is automatically determined by querying the
/// controller's ELPE (Error Log Page Entries) field, ensuring all available error
/// history is collected.
///
/// # Arguments
///
/// * `dev_path` - Path to the NVMe character device (e.g., `"/dev/nvme0"`)
///
/// # Returns
///
/// Returns [`NvmeErrorLog`] containing:
/// - Device identification (name and serial number)
/// - Vector of error entries, each including:
///   - Error count and timestamp information
///   - Command details (queue ID, command ID)
///   - Error status and location
///   - Affected LBA and namespace
///   - Vendor-specific diagnostic data
///
/// Note: Only populated error entries are returned (entries with `error_count != 0`).
/// A healthy drive may return an empty error list.
///
/// # Errors
///
/// This function will return an error if:
/// - The device path does not exist or cannot be opened
/// - The process lacks sufficient permissions (requires root/sudo)
/// - The Identify Controller command fails
/// - The Get Log Page command fails
/// - The device is not a valid NVMe controller
pub fn get_error_log(dev_path: &str) -> Result<NvmeErrorLog> {
    let nvme_name = dev_path.trim_start_matches("/dev/").to_string();

    // Query controller to get ELPE
    let id_ctrl = read_nvme_id_ctrl(dev_path)?;
    let diag = CtrlDiagnostics::new(nvme_name.clone(), &id_ctrl);
    let serial_number = types::parse_ascii_field(&id_ctrl.sn);

    // Calculate number of entries (ELPE is 0-based)
    let max_entries = (diag.elpe + 1) as u16;

    let raw_entries = read_error_log_raw(dev_path, max_entries)?;
    Ok(NvmeErrorLog::new(nvme_name, serial_number, raw_entries))
}

/// Retrieve Controller Identification data from an NVMe device.
///
/// This function collects comprehensive identification and configuration information
/// from the specified NVMe controller. This data is fundamental for device inventory,
/// compatibility checking, and feature detection.
///
/// # Arguments
///
/// * `dev_path` - Path to the NVMe character device (e.g., `"/dev/nvme0"`)
///
/// # Returns
///
/// Returns [`CtrlIdentity`] containing:
/// - Vendor information (PCI VID, Subsystem VID, IEEE OUI)
/// - Device identification (serial number, model number, firmware revision)
/// - Controller identifiers (Controller ID, NVM Subsystem NQN)
/// - Hardware identification (FRU GUID)
/// - NVMe specification version supported by the controller
/// - Controller type
///
/// # Errors
///
/// This function will return an error if:
/// - The device path does not exist or cannot be opened
/// - The process lacks sufficient permissions (requires root/sudo)
/// - The Identify Controller command fails
/// - The device is not a valid NVMe controller
pub fn get_controller_identity(dev_path: &str) -> Result<CtrlIdentity> {
    let nvme_name = dev_path.trim_start_matches("/dev/").to_string();
    let raw = read_nvme_id_ctrl(dev_path)?;
    Ok(CtrlIdentity::new(nvme_name, &raw))
}

/// Discover NVMe controllers available on the system.
///
/// Scans `/sys/class/nvme` to enumerate all NVMe controllers exposed by the kernel.
/// This is typically the first step before collecting telemetry data from specific devices.
///
/// # Returns
///
/// Returns a vector of NVMe controller names (e.g., `["nvme0", "nvme1"]`).
/// Returns an empty vector if no controllers are found or if `/sys/class/nvme` cannot be read.
///
/// # Note
///
/// The returned names can be used to construct device paths by prepending `/dev/`
/// (e.g., `nvme0` becomes `/dev/nvme0`).
pub fn list_nvme_controllers() -> Vec<String> {
    let mut names = Vec::new();

    if let Ok(entries) = fs::read_dir("/sys/class/nvme") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            names.push(name);
        }
    }

    names
}

/// Retrieve and parses the OCP SMART Extended Log for a given NVMe device.
///
/// This function reads the NVMe Identify Controller data to extract the device's
/// serial number, then fetches the OCP SMART Additional Log and combines both
/// into a complete [`OcpSmartData`] structure.
///
/// # Arguments
///
/// * `dev_path` - The filesystem path to the NVMe device (e.g., `"/dev/nvme0"`).
///
/// # Returns
///
/// Returns an [`OcpSmartData`] on success, or an [`io::Error`] if any
/// underlying device read fails.
///
/// # Errors
///
/// This function will return an error if:
/// * Reading the NVMe Identify Controller data fails.
/// * Reading the OCP SMART Additional Log fails.
pub fn get_smart_add_log(dev_path: &str) -> Result<OcpSmartData> {
    let nvme_name = dev_path.trim_start_matches("/dev/").to_string();

    // Get serial number from Identify Controller
    let id_ctrl = read_nvme_id_ctrl(dev_path)?;
    let serial_number = types::parse_ascii_field(&id_ctrl.sn);

    // Get SMART ADD LOG data
    let raw_smart_add_log = read_ocp_smart_log(dev_path)?;

    // Combine into complete struct
    Ok(OcpSmartData::new(
        nvme_name,
        serial_number,
        &raw_smart_add_log,
    ))
}
