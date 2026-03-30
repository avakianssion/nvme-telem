//! OCP NVMe extensions.
//!
//! The OCP NVMe specification defines extended SMART/Health information log pages
//! that provide additional telemetry beyond the standard NVMe SMART log. This includes
//! detailed wear metrics, error statistics, and vendor-specific diagnostic data.
//!
//! # OCP Extended SMART Log (0xC0)
//!
//! This log page provides comprehensive drive health metrics including:
//! - Physical media units read/written
//! - Bad NAND block counts (user and system areas)
//! - Detailed error statistics (XOR recovery, ECC errors, E2E errors)
//! - Wear leveling metrics (erase counts, endurance estimates)
//! - Thermal management data
//! - PCIe error counts
//! - Power loss protection statistics
//!
//! # Vendor Support
//!
//! Not all NVMe drives support OCP extended logs. This implementation validates
//! the OCP GUID to ensure the returned data is genuine OCP telemetry and not
//! garbage data from an unsupported device.

use nvme_cli_sys::{nvme_admin_cmd, nvme_admin_opcode::nvme_admin_get_log_page};
use serde::Serialize;
use std::fs::OpenOptions;
use std::io;
use std::mem::{size_of, zeroed};
use std::os::unix::io::AsRawFd;

/// OCP S.M.A.R.T. / Health Information Extended Log (Log ID 0xC0)
///
/// Raw C-compatible structure matching the OCP specification exactly.
/// This struct is 512 bytes and uses packed representation to match
/// the hardware data format byte-for-byte without padding.
///
/// Source: linux-nvme/nvme-cli/blob/master/plugins/ocp/ocp-smart-extended-log.h
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct OcpSmartExtendedLog {
    /// Physical Media Units Written
    pub physical_media_units_written: [u8; 16],

    /// Physical Media Units Read
    pub physical_media_units_read: [u8; 16],

    /// Bad User NAND Blocks raw
    pub bad_user_nand_blocks_raw: [u8; 6],

    /// Bad User NAND Blocks normalized
    pub bad_user_nand_blocks_normalized: u16,

    /// Bad System NAND Blocks raw
    pub bad_system_nand_blocks_raw: [u8; 6],

    /// Bad System NAND Blocks normalized
    pub bad_system_nand_blocks_normalized: u16,

    /// XOR Recovery Count
    pub xor_recovery_count: u64,

    /// Uncorrectable Read Error Count
    pub uncorrectable_read_err_count: u64,

    /// Soft ECC Error Count
    pub soft_ecc_err_count: u64,

    /// End to End detected errors
    pub end_to_end_detected_err: u32,

    /// End to End corrected errors
    pub end_to_end_corrected_err: u32,

    /// System data percent used
    pub system_data_used_percent: u8,

    /// Refresh Counts
    pub refresh_counts: [u8; 7],

    /// Max User data erase counts
    pub user_data_erase_count_max: u32,

    /// Min User data erase counts
    pub user_data_erase_count_min: u32,

    /// Number of Thermal throttling events
    pub thermal_throttling_event_count: u8,

    /// Thermal throttling current status
    pub thermal_throttling_current_status: u8,

    /// DSSD Errata Version
    pub dssd_errata_version: u8,

    /// DSSD Point Version
    pub dssd_point_version: [u8; 2],

    /// DSSD Minor Version
    pub dssd_minor_version: [u8; 2],

    /// DSSD Major Version
    pub dssd_major_version: u8,

    /// PCIe Correctable Error Count
    pub pcie_correctable_err_count: u64,

    /// Incomplete Shutdowns
    pub incomplete_shutdowns: u32,

    /// Reserved
    rsvd116: [u8; 4],

    /// Percent free blocks
    pub percent_free_blocks: u8,

    /// Reserved
    rsvd121: [u8; 7],

    /// Capacitor health
    pub capacitor_health: u16,

    /// NVM Express Base Errata Version
    pub nvme_base_errata_version: u8,

    /// NVMe Command Set Errata Version
    pub nvme_cmdset_errata_version: u8,

    /// NVMe Over PCIe Errata Version
    pub nvme_over_pcie_errata_version: u8,

    /// NVMe MI Errata Version
    pub nvme_mi_errata_version: u8,

    /// Reserved
    rsvd134: [u8; 2],

    /// Unaligned I/O
    pub unaligned_io: u64,

    /// Security Version Number
    pub security_version: u64,

    /// Total NUSE - Namespace utilization
    pub total_nuse: u64,

    /// PLP start count
    pub plp_start_count: [u8; 16],

    /// Endurance Estimate
    pub endurance_estimate: [u8; 16],

    /// PCIe Link Retraining Count
    pub pcie_link_retraining_count: u64,

    /// Power State Change Count
    pub power_state_change_count: u64,

    /// Lowest Permitted Firmware Revision
    pub lowest_permitted_fw_rev: u64,

    /// Total media dies
    pub total_media_dies: u16,

    /// Total die failure tolerance
    pub total_die_failure_tolerance: u16,

    /// Media dies offline
    pub media_dies_offline: u16,

    /// Max temperature recorded
    pub max_temperature_recorded: u8,

    /// Reserved
    rsvd223: u8,

    /// NAND avg erase count
    pub nand_avg_erase_count: u64,

    /// Command timeouts
    pub command_timeouts: u32,

    /// Sys area program fail count raw
    pub sys_area_program_fail_count_raw: u32,

    /// Sys area program fail count normalized
    pub sys_area_program_fail_count_normalized: u8,

    /// Reserved
    rsvd241: [u8; 3],

    /// Sys area uncorrectable read count raw
    pub sys_area_uncorr_read_count_raw: u32,

    /// Sys area uncorrectable read count normalized
    pub sys_area_uncorr_read_count_normalized: u8,

    /// Reserved
    rsvd249: [u8; 3],

    /// Sys area erase fail count raw
    pub sys_area_erase_fail_count_raw: u32,

    /// Sys area erase fail count normalized
    pub sys_area_erase_fail_count_normalized: u8,

    /// Reserved
    rsvd257: [u8; 3],

    /// Max peak power capability
    pub max_peak_power_capability: u16,

    /// Current max avg power
    pub current_max_avg_power: u16,

    /// Lifetime power consumed
    pub lifetime_power_consumed: [u8; 6],

    /// DSSD firmware revision
    pub dssd_firmware_revision: [u8; 8],

    /// DSSD firmware build UUID
    pub dssd_firmware_build_uuid: [u8; 16],

    /// DSSD firmware build label
    pub dssd_firmware_build_label: [u8; 64],

    /// Reserved
    rsvd358: [u8; 136],

    /// Log page version
    pub log_page_version: u16,

    /// Log page GUID
    pub log_page_guid: [u8; 16],
}

// Compile-time assertion: struct must be exactly 512 bytes per OCP spec
const _: () = assert!(size_of::<OcpSmartExtendedLog>() == 512);

/// OCP S.M.A.R.T. / Health Information Extended Log.
///
/// Parsed and organized OCP telemetry data with proper Rust types.
/// This struct provides a type-safe interface to the raw OCP log data.
#[derive(Debug, Serialize)]
pub struct OcpSmartData {
    // NVMe device name (e.g., "nvme0")
    pub nvme_name: String,

    // NVMe Serial Number
    pub serial_number: String,

    // Media units
    pub physical_media_units_written: u128,
    pub physical_media_units_read: u128,

    // Bad blocks
    pub bad_user_nand_blocks_raw: u64,
    pub bad_user_nand_blocks_normalized: u16,
    pub bad_system_nand_blocks_raw: u64,
    pub bad_system_nand_blocks_normalized: u16,

    // Error counts
    pub xor_recovery_count: u64,
    pub uncorrectable_read_errors: u64,
    pub soft_ecc_errors: u64,
    pub e2e_errors_detected: u32,
    pub e2e_errors_corrected: u32,

    // Usage and wear
    pub system_data_percent_used: u8,
    pub user_data_erase_count_max: u32,
    pub user_data_erase_count_min: u32,
    pub nand_avg_erase_count: u64,

    // Thermal
    pub thermal_throttling_events: u8,
    pub thermal_throttling_status: u8,
    pub max_temperature_recorded: u8,

    // PCIe
    pub pcie_correctable_errors: u64,
    pub pcie_link_retraining_count: u64,

    // Power and shutdowns
    pub incomplete_shutdowns: u32,
    pub power_state_changes: u64,

    // Capacity
    pub percent_free_blocks: u8,
    pub capacitor_health: u16,

    // I/O
    pub unaligned_io: u64,
    pub command_timeouts: u32,

    // Security and firmware
    pub security_version: u64,
    pub lowest_permitted_fw_rev: u64,

    // Namespace utilization
    pub total_nuse: u64,

    // PLP (Power Loss Protection)
    pub plp_start_count: u128,

    // Endurance
    pub endurance_estimate: u128,

    // Die information
    pub total_media_dies: u16,
    pub total_die_failure_tolerance: u16,
    pub media_dies_offline: u16,

    // System area failures
    pub sys_area_program_fail_count_raw: u32,
    pub sys_area_program_fail_count_normalized: u8,
    pub sys_area_uncorr_read_count_raw: u32,
    pub sys_area_uncorr_read_count_normalized: u8,
    pub sys_area_erase_fail_count_raw: u32,
    pub sys_area_erase_fail_count_normalized: u8,

    // Power capability
    pub max_peak_power_capability: u16,
    pub current_max_avg_power: u16,
    pub lifetime_power_consumed: u64,

    // Firmware info
    pub dssd_firmware_revision: String,

    // Log metadata
    pub log_page_version: u16,
}

impl OcpSmartData {
    /// Create a new OcpSmartData from raw OCP log data.
    pub fn new(nvme_name: String, serial_number: String, raw: &OcpSmartExtendedLog) -> Self {
        // Convert 16-byte arrays to u128
        let media_written = u128::from_le_bytes(raw.physical_media_units_written);
        let media_read = u128::from_le_bytes(raw.physical_media_units_read);
        let plp_count = u128::from_le_bytes(raw.plp_start_count);
        let endurance = u128::from_le_bytes(raw.endurance_estimate);

        // Convert 6-byte arrays to u64
        let bad_user_blocks = u64::from_le_bytes([
            raw.bad_user_nand_blocks_raw[0],
            raw.bad_user_nand_blocks_raw[1],
            raw.bad_user_nand_blocks_raw[2],
            raw.bad_user_nand_blocks_raw[3],
            raw.bad_user_nand_blocks_raw[4],
            raw.bad_user_nand_blocks_raw[5],
            0,
            0,
        ]);

        let bad_system_blocks = u64::from_le_bytes([
            raw.bad_system_nand_blocks_raw[0],
            raw.bad_system_nand_blocks_raw[1],
            raw.bad_system_nand_blocks_raw[2],
            raw.bad_system_nand_blocks_raw[3],
            raw.bad_system_nand_blocks_raw[4],
            raw.bad_system_nand_blocks_raw[5],
            0,
            0,
        ]);

        let lifetime_power = u64::from_le_bytes([
            raw.lifetime_power_consumed[0],
            raw.lifetime_power_consumed[1],
            raw.lifetime_power_consumed[2],
            raw.lifetime_power_consumed[3],
            raw.lifetime_power_consumed[4],
            raw.lifetime_power_consumed[5],
            0,
            0,
        ]);

        // Parse firmware revision as ASCII
        let fw_rev = String::from_utf8_lossy(&raw.dssd_firmware_revision)
            .trim_end_matches('\0')
            .trim()
            .to_string();

        Self {
            nvme_name,
            serial_number,
            physical_media_units_written: media_written,
            physical_media_units_read: media_read,
            bad_user_nand_blocks_raw: bad_user_blocks,
            bad_user_nand_blocks_normalized: raw.bad_user_nand_blocks_normalized,
            bad_system_nand_blocks_raw: bad_system_blocks,
            bad_system_nand_blocks_normalized: raw.bad_system_nand_blocks_normalized,
            xor_recovery_count: raw.xor_recovery_count,
            uncorrectable_read_errors: raw.uncorrectable_read_err_count,
            soft_ecc_errors: raw.soft_ecc_err_count,
            e2e_errors_detected: raw.end_to_end_detected_err,
            e2e_errors_corrected: raw.end_to_end_corrected_err,
            system_data_percent_used: raw.system_data_used_percent,
            user_data_erase_count_max: raw.user_data_erase_count_max,
            user_data_erase_count_min: raw.user_data_erase_count_min,
            nand_avg_erase_count: raw.nand_avg_erase_count,
            thermal_throttling_events: raw.thermal_throttling_event_count,
            thermal_throttling_status: raw.thermal_throttling_current_status,
            max_temperature_recorded: raw.max_temperature_recorded,
            pcie_correctable_errors: raw.pcie_correctable_err_count,
            pcie_link_retraining_count: raw.pcie_link_retraining_count,
            incomplete_shutdowns: raw.incomplete_shutdowns,
            power_state_changes: raw.power_state_change_count,
            percent_free_blocks: raw.percent_free_blocks,
            capacitor_health: raw.capacitor_health,
            unaligned_io: raw.unaligned_io,
            command_timeouts: raw.command_timeouts,
            security_version: raw.security_version,
            lowest_permitted_fw_rev: raw.lowest_permitted_fw_rev,
            total_nuse: raw.total_nuse,
            plp_start_count: plp_count,
            endurance_estimate: endurance,
            total_media_dies: raw.total_media_dies,
            total_die_failure_tolerance: raw.total_die_failure_tolerance,
            media_dies_offline: raw.media_dies_offline,
            sys_area_program_fail_count_raw: raw.sys_area_program_fail_count_raw,
            sys_area_program_fail_count_normalized: raw.sys_area_program_fail_count_normalized,
            sys_area_uncorr_read_count_raw: raw.sys_area_uncorr_read_count_raw,
            sys_area_uncorr_read_count_normalized: raw.sys_area_uncorr_read_count_normalized,
            sys_area_erase_fail_count_raw: raw.sys_area_erase_fail_count_raw,
            sys_area_erase_fail_count_normalized: raw.sys_area_erase_fail_count_normalized,
            max_peak_power_capability: raw.max_peak_power_capability,
            current_max_avg_power: raw.current_max_avg_power,
            lifetime_power_consumed: lifetime_power,
            dssd_firmware_revision: fw_rev,
            log_page_version: raw.log_page_version,
        }
    }
}

/// Read OCP Extended SMART log from an NVMe device.
///
/// # Arguments
///
/// * `dev_path` - Path to the NVMe character device (e.g., `"/dev/nvme0"`)
///
/// # Returns
///
/// Returns the raw OCP SMART Extended Log structure.
///
/// # Errors
///
/// This function will return an error if:
/// - The device path does not exist or cannot be opened
/// - The process lacks sufficient permissions (requires root/sudo)
/// - The NVMe controller does not respond or returns an error status
/// - The device does not support OCP extended SMART log (invalid or mismatched GUID)
///
/// # Note
///
/// Not all NVMe drives support the OCP extended SMART log. This function validates
/// the OCP GUID in the returned data to ensure the device genuinely supports this
/// feature and is not returning garbage data.
pub fn read_ocp_smart_log(dev_path: &str) -> io::Result<OcpSmartExtendedLog> {
    let file = OpenOptions::new().read(true).write(true).open(dev_path)?;
    let fd = file.as_raw_fd();
    let mut log: OcpSmartExtendedLog = unsafe { zeroed() };
    let log_ptr = &mut log as *mut OcpSmartExtendedLog as u64;
    let log_len = size_of::<OcpSmartExtendedLog>() as u32;

    let log_id: u8 = 0xC0; // OCP SMART extended log
    let numd: u32 = log_len / 4 - 1;
    let cdw10: u32 = (log_id as u32) | (numd << 16);

    let mut cmd: nvme_admin_cmd = unsafe { zeroed() };
    cmd.opcode = nvme_admin_get_log_page as u8;
    cmd.nsid = 0xFFFF_FFFF;
    cmd.addr = log_ptr;
    cmd.data_len = log_len;
    cmd.cdw10 = cdw10;
    cmd.cdw11 = 0;
    cmd.timeout_ms = 1000;

    let ret = unsafe { nvme_cli_sys::nvme_ioctl_admin_cmd(fd, &mut cmd) };
    match ret {
        Ok(0) => {
            // Validate OCP GUID to ensure device actually supports this log page
            // Source: linux-nvme/nvme-cli/blob/master/plugins/ocp/ocp-smart-extended-log.c
            const OCP_GUID: [u8; 16] = [
                0xC5, 0xAF, 0x10, 0x28, 0xEA, 0xBF, 0xF2, 0xA4, 0x9C, 0x4F, 0x6F, 0x7C, 0xC9, 0x14,
                0xD5, 0xAF,
            ];

            // Check if GUID is all zeros (unsupported)
            if log.log_page_guid == [0u8; 16] {
                return Err(io::Error::other(
                    "Device does not support OCP extended SMART log (invalid GUID)",
                ));
            }

            // Check if GUID matches the expected OCP GUID
            if log.log_page_guid != OCP_GUID {
                return Err(io::Error::other(format!(
                    "Device does not support OCP extended SMART log (unexpected GUID: {:02X?})",
                    log.log_page_guid
                )));
            }

            Ok(log)
        }
        Ok(status) => Err(io::Error::other(format!(
            "OCP SMART log command failed, status={:#x}",
            status
        ))),
        Err(e) => Err(io::Error::other(e.to_string())),
    }
}
