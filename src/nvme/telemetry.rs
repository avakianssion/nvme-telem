//! High-level telemetry and monitoring API for NVMe devices.
//!
//! This module provides functions for collecting telemetry data
//! from NVMe controllers, automatically handling device identification and
//! data enrichment.

use crate::nvme::io::*;
use crate::nvme::ocp::{OcpSmartData, read_ocp_smart_log_fd};
use crate::nvme::types::*;
use std::fs::{self, OpenOptions};
use std::os::fd::{AsFd, OwnedFd};
use std::path::Path;

/// A handle to an open NVMe character device.
///
/// `Device` owns the underlying file descriptor for a device such as
/// `/dev/nvme0`, opened once via [`Device::open`]. Each accessor method
/// (`smart_log`, `identity`, `capacity`, `capabilities`, `limits`,
/// `thermals`, `firmware`, `power_states`, `host_memory`, `arbitration`,
/// `diagnostics`, `advanced`, `command_sets`, `fabric`, `error_log`,
/// `ocp_smart_log`) reuses that same descriptor to issue its NVMe Admin
/// commands, rather than reopening the device path per call.
///
/// The `identity`, `capacity`, `capabilities`, `limits`, `thermals`,
/// `firmware`, `power_states`, `host_memory`, `arbitration`,
/// `diagnostics`, `advanced`, `command_sets`, and `fabric` accessors all
/// parse different fields out of the same underlying Identify Controller
/// data (`nvme_id_ctrl`), just grouped by category. Each issues its own
/// Identify command rather than sharing one cached copy, so calling
/// several of them fetches the controller data multiple times.
///
/// The descriptor is closed automatically when the `Device` is dropped.
///
/// # Requirements
///
/// Opening a device and issuing any of the commands below requires
/// root/sudo privileges to access `/dev/nvme*`.
///
/// # Safety
///
/// All commands issued by `Device` are read-only NVMe Admin commands
/// (Identify and Get Log Page) — they do not modify device state.
#[derive(Debug)]
pub struct Device {
    fd: OwnedFd,
    nvme_name: String,
}

impl Device {
    /// Open an NVMe character device.
    ///
    /// The device is opened once; the resulting file descriptor is reused
    /// by every accessor method on the returned `Device`. Requires
    /// root/sudo privileges.
    ///
    /// # Parameters
    /// - `device` - Name of the storage controller assigned by the kernel (e.g. `nvme0`).
    ///
    /// # Errors
    ///
    /// Returns an error if the device path does not exist or cannot be
    /// opened (e.g. insufficient permissions).
    pub fn open(device: impl AsRef<Path>) -> Result<Device> {
        let file = OpenOptions::new()
            .read(true)
            .write(true) // Admin permission required
            .open(Path::new("/dev").join(&device))?;

        let nvme_name = device.as_ref().display().to_string();

        Ok(Device {
            fd: file.into(),
            nvme_name,
        })
    }

    /// Retrieve S.M.A.R.T./Health Information from this device.
    ///
    /// Issues an Identify Controller command (for the serial number) and a
    /// Get Log Page command for the S.M.A.R.T./Health Information log
    /// (Log ID 0x02). Both are read-only NVMe Admin commands.
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
    /// - The process lacks sufficient permissions (requires root/sudo)
    /// - The NVMe controller does not respond or returns an error status
    /// - The device is not a valid NVMe controller
    pub fn smart_log(&self) -> Result<NvmeSmartLog> {
        let id_ctrl = read_nvme_id_ctrl_fd(self.fd.as_fd())?;
        let serial_number = parse_ascii_field(&id_ctrl.sn);

        let raw_smart = read_nvme_smart_log_fd(self.fd.as_fd())?;

        Ok(NvmeSmartLog::new(
            self.nvme_name.clone(),
            serial_number,
            &raw_smart,
        ))
    }

    /// Retrieve Controller Identification data from this device.
    ///
    /// Issues a single, read-only Identify Controller NVMe Admin command.
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
    /// - The process lacks sufficient permissions (requires root/sudo)
    /// - The Identify Controller command fails
    /// - The device is not a valid NVMe controller
    pub fn identity(&self) -> Result<CtrlIdentity> {
        let raw = read_nvme_id_ctrl_fd(self.fd.as_fd())?;
        Ok(CtrlIdentity::new(self.nvme_name.clone(), &raw))
    }

    /// Retrieve controller capacity and storage information from this device.
    ///
    /// Issues a single, read-only Identify Controller NVMe Admin command.
    /// See [`CtrlCapacity`] for the fields returned.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The process lacks sufficient permissions (requires root/sudo)
    /// - The Identify Controller command fails
    /// - The device is not a valid NVMe controller
    pub fn capacity(&self) -> Result<CtrlCapacity> {
        let raw = read_nvme_id_ctrl_fd(self.fd.as_fd())?;
        Ok(CtrlCapacity::new(self.nvme_name.clone(), &raw))
    }

    /// Retrieve controller capabilities and feature support from this device.
    ///
    /// Issues a single, read-only Identify Controller NVMe Admin command.
    /// See [`CtrlCapabilities`] for the fields returned.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The process lacks sufficient permissions (requires root/sudo)
    /// - The Identify Controller command fails
    /// - The device is not a valid NVMe controller
    pub fn capabilities(&self) -> Result<CtrlCapabilities> {
        let raw = read_nvme_id_ctrl_fd(self.fd.as_fd())?;
        Ok(CtrlCapabilities::new(self.nvme_name.clone(), &raw))
    }

    /// Retrieve controller operational limits and constraints from this device.
    ///
    /// Issues a single, read-only Identify Controller NVMe Admin command.
    /// See [`CtrlLimits`] for the fields returned.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The process lacks sufficient permissions (requires root/sudo)
    /// - The Identify Controller command fails
    /// - The device is not a valid NVMe controller
    pub fn limits(&self) -> Result<CtrlLimits> {
        let raw = read_nvme_id_ctrl_fd(self.fd.as_fd())?;
        Ok(CtrlLimits::new(self.nvme_name.clone(), &raw))
    }

    /// Retrieve controller thermal management configuration from this device.
    ///
    /// Issues a single, read-only Identify Controller NVMe Admin command.
    /// See [`CtrlThermals`] for the fields returned.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The process lacks sufficient permissions (requires root/sudo)
    /// - The Identify Controller command fails
    /// - The device is not a valid NVMe controller
    pub fn thermals(&self) -> Result<CtrlThermals> {
        let raw = read_nvme_id_ctrl_fd(self.fd.as_fd())?;
        Ok(CtrlThermals::new(self.nvme_name.clone(), &raw))
    }

    /// Retrieve controller firmware update configuration from this device.
    ///
    /// Issues a single, read-only Identify Controller NVMe Admin command.
    /// See [`CtrlFirmware`] for the fields returned.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The process lacks sufficient permissions (requires root/sudo)
    /// - The Identify Controller command fails
    /// - The device is not a valid NVMe controller
    pub fn firmware(&self) -> Result<CtrlFirmware> {
        let raw = read_nvme_id_ctrl_fd(self.fd.as_fd())?;
        Ok(CtrlFirmware::new(self.nvme_name.clone(), &raw))
    }

    /// Retrieve controller power state descriptors from this device.
    ///
    /// Issues a single, read-only Identify Controller NVMe Admin command.
    /// See [`CtrlPowerStates`] for the fields returned.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The process lacks sufficient permissions (requires root/sudo)
    /// - The Identify Controller command fails
    /// - The device is not a valid NVMe controller
    pub fn power_states(&self) -> Result<CtrlPowerStates> {
        let raw = read_nvme_id_ctrl_fd(self.fd.as_fd())?;
        Ok(CtrlPowerStates::new(self.nvme_name.clone(), &raw))
    }

    /// Retrieve controller Host Memory Buffer (HMB) configuration from this device.
    ///
    /// Issues a single, read-only Identify Controller NVMe Admin command.
    /// See [`CtrlHostMemory`] for the fields returned.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The process lacks sufficient permissions (requires root/sudo)
    /// - The Identify Controller command fails
    /// - The device is not a valid NVMe controller
    pub fn host_memory(&self) -> Result<CtrlHostMemory> {
        let raw = read_nvme_id_ctrl_fd(self.fd.as_fd())?;
        Ok(CtrlHostMemory::new(self.nvme_name.clone(), &raw))
    }

    /// Retrieve controller arbitration and quality of service settings from this device.
    ///
    /// Issues a single, read-only Identify Controller NVMe Admin command.
    /// See [`CtrlArbitration`] for the fields returned.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The process lacks sufficient permissions (requires root/sudo)
    /// - The Identify Controller command fails
    /// - The device is not a valid NVMe controller
    pub fn arbitration(&self) -> Result<CtrlArbitration> {
        let raw = read_nvme_id_ctrl_fd(self.fd.as_fd())?;
        Ok(CtrlArbitration::new(self.nvme_name.clone(), &raw))
    }

    /// Retrieve controller diagnostic and self-test capabilities from this device.
    ///
    /// Issues a single, read-only Identify Controller NVMe Admin command.
    /// See [`CtrlDiagnostics`] for the fields returned.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The process lacks sufficient permissions (requires root/sudo)
    /// - The Identify Controller command fails
    /// - The device is not a valid NVMe controller
    pub fn diagnostics(&self) -> Result<CtrlDiagnostics> {
        let raw = read_nvme_id_ctrl_fd(self.fd.as_fd())?;
        Ok(CtrlDiagnostics::new(self.nvme_name.clone(), &raw))
    }

    /// Retrieve controller advanced features and timing information from this device.
    ///
    /// Issues a single, read-only Identify Controller NVMe Admin command.
    /// See [`CtrlAdvanced`] for the fields returned.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The process lacks sufficient permissions (requires root/sudo)
    /// - The Identify Controller command fails
    /// - The device is not a valid NVMe controller
    pub fn advanced(&self) -> Result<CtrlAdvanced> {
        let raw = read_nvme_id_ctrl_fd(self.fd.as_fd())?;
        Ok(CtrlAdvanced::new(self.nvme_name.clone(), &raw))
    }

    /// Retrieve controller command set configuration from this device.
    ///
    /// Issues a single, read-only Identify Controller NVMe Admin command.
    /// See [`CtrlCommandSets`] for the fields returned.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The process lacks sufficient permissions (requires root/sudo)
    /// - The Identify Controller command fails
    /// - The device is not a valid NVMe controller
    pub fn command_sets(&self) -> Result<CtrlCommandSets> {
        let raw = read_nvme_id_ctrl_fd(self.fd.as_fd())?;
        Ok(CtrlCommandSets::new(self.nvme_name.clone(), &raw))
    }

    /// Retrieve controller fabric (NVMe-oF) configuration from this device.
    ///
    /// Issues a single, read-only Identify Controller NVMe Admin command.
    /// See [`CtrlFabric`] for the fields returned.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The process lacks sufficient permissions (requires root/sudo)
    /// - The Identify Controller command fails
    /// - The device is not a valid NVMe controller
    pub fn fabric(&self) -> Result<CtrlFabric> {
        let raw = read_nvme_id_ctrl_fd(self.fd.as_fd())?;
        Ok(CtrlFabric::new(self.nvme_name.clone(), &raw))
    }

    /// Retrieve Error Information Log from this device.
    ///
    /// Issues an Identify Controller command (to determine the ELPE —
    /// Error Log Page Entries — field) followed by a Get Log Page command
    /// for the Error Information log (Log ID 0x01). Both are read-only
    /// NVMe Admin commands.
    ///
    /// The number of error entries retrieved is automatically determined by
    /// querying the controller's ELPE field, ensuring all available error
    /// history is collected.
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
    /// - The process lacks sufficient permissions (requires root/sudo)
    /// - The Identify Controller command fails
    /// - The Get Log Page command fails
    /// - The device is not a valid NVMe controller
    pub fn error_log(&self) -> Result<NvmeErrorLog> {
        let id_ctrl = read_nvme_id_ctrl_fd(self.fd.as_fd())?;
        let diag = CtrlDiagnostics::new(self.nvme_name.clone(), &id_ctrl);
        let serial_number = parse_ascii_field(&id_ctrl.sn);

        // ELPE is 0-based, so 255 means 256 entries; widen before adding.
        let max_entries = u16::from(diag.elpe) + 1;

        let raw_entries = read_error_log_raw_fd(self.fd.as_fd(), max_entries)?;
        Ok(NvmeErrorLog::new(
            self.nvme_name.clone(),
            serial_number,
            raw_entries,
        ))
    }

    /// Retrieve and parse the OCP SMART Extended Log from this device.
    ///
    /// Issues an Identify Controller command (for the serial number)
    /// followed by a Get Log Page command for the OCP SMART Extended Log
    /// (Log ID 0xC0). Both are read-only NVMe Admin commands.
    ///
    /// # Returns
    ///
    /// Returns an [`OcpSmartData`] on success, or an [`std::io::Error`] if any
    /// underlying device read fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The process lacks sufficient permissions (requires root/sudo)
    /// * Reading the NVMe Identify Controller data fails.
    /// * Reading the OCP SMART Additional Log fails (including when the
    ///   device does not support the OCP extended SMART log).
    pub fn ocp_smart_log(&self) -> Result<OcpSmartData> {
        let id_ctrl = read_nvme_id_ctrl_fd(self.fd.as_fd())?;
        let serial_number = parse_ascii_field(&id_ctrl.sn);

        let raw_smart_add_log = read_ocp_smart_log_fd(self.fd.as_fd())?;

        Ok(OcpSmartData::new(
            self.nvme_name.clone(),
            serial_number,
            &raw_smart_add_log,
        ))
    }
}

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
#[deprecated(since = "0.3.2", note = "use Device::open(path)?.smart_log() instead")]
pub fn get_smart_log(dev_path: &str) -> Result<NvmeSmartLog> {
    Device::open(dev_path)?.smart_log()
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
#[deprecated(since = "0.3.2", note = "use Device::open(path)?.error_log() instead")]
pub fn get_error_log(dev_path: &str) -> Result<NvmeErrorLog> {
    Device::open(dev_path)?.error_log()
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
#[deprecated(since = "0.3.2", note = "use Device::open(path)?.identity() instead")]
pub fn get_controller_identity(dev_path: &str) -> Result<CtrlIdentity> {
    Device::open(dev_path)?.identity()
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
/// Returns an [`OcpSmartData`] on success, or an [`std::io::Error`] if any
/// underlying device read fails.
///
/// # Errors
///
/// This function will return an error if:
/// * Reading the NVMe Identify Controller data fails.
/// * Reading the OCP SMART Additional Log fails.
#[deprecated(
    since = "0.3.2",
    note = "use Device::open(path)?.ocp_smart_log() instead"
)]
pub fn get_smart_add_log(dev_path: &str) -> Result<OcpSmartData> {
    Device::open(dev_path)?.ocp_smart_log()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_nonexistent_path_returns_error() {
        let err = Device::open("/dev/nvme-telem-does-not-exist-xyz").unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    #[allow(deprecated)]
    fn deprecated_wrappers_fail_the_same_way_as_device_open() {
        let device = "nvme-telem-does-not-exist-xyz";

        let device_err = Device::open(device).unwrap_err();
        let smart_err = get_smart_log(device).unwrap_err();
        let identity_err = get_controller_identity(device).unwrap_err();
        let error_log_err = get_error_log(device).unwrap_err();
        let ocp_err = get_smart_add_log(device).unwrap_err();

        assert_eq!(device_err.kind(), smart_err.kind());
        assert_eq!(device_err.kind(), identity_err.kind());
        assert_eq!(device_err.kind(), error_log_err.kind());
        assert_eq!(device_err.kind(), ocp_err.kind());
    }

    #[test]
    fn device_name_strips_dev_prefix() {
        // /dev/null is always present, readable/writable, and lets us reach
        // past `open()` to confirm the name parsing without real NVMe hardware.
        let device = Device::open("null").expect("opening /dev/null should succeed");
        assert_eq!(device.nvme_name, "null");
    }

    #[test]
    fn drop_closes_fd_without_panicking() {
        let device = Device::open("null").expect("opening /dev/null should succeed");
        drop(device);
    }
}
