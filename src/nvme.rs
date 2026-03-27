// src/nvme.rs

use nvme_cli_sys::{
    nvme_admin_cmd, nvme_admin_opcode::nvme_admin_get_log_page,
    nvme_admin_opcode::nvme_admin_identify, nvme_error_log_page, nvme_id_ctrl, nvme_smart_log,
};
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io;
use std::mem::{size_of, zeroed};
use std::os::raw::c_char;
use std::os::unix::io::AsRawFd;

// =============================================================================
// CONTROLLER IDENTITY STRUCTS
// =============================================================================

/// Controller identity and basic information.
///
/// Contains fundamental identification data for the NVMe controller including
/// vendor information, serial numbers, firmware revision, and controller IDs.
#[derive(Debug, Serialize, PartialEq)]
pub struct CtrlIdentity {
    /// NVMe device name (e.g., "nvme0")
    pub nvme_name: String,

    /// PCI Vendor ID
    pub vid: u16,

    /// PCI Subsystem Vendor ID
    pub ssvid: u16,

    /// Serial Number (ASCII, space padded)
    pub serial_number: String,

    /// Model Number (ASCII, space padded)
    pub model_number: String,

    /// Firmware Revision (ASCII, space padded)
    pub firmware_rev: String,

    /// IEEE OUI Identifier (3 bytes)
    pub ieee_oui: [u8; 3],

    /// Controller ID
    pub cntlid: u16,

    /// NVMe specification version
    pub ver: u32,

    /// NVM Subsystem NQN (ASCII)
    pub subnqn: String,

    /// FRU GUID / Field Replaceable Unit GUID
    pub fguid: [u8; 16],

    /// Controller Type
    pub cntrltype: u8,
}

impl CtrlIdentity {
    pub fn new(nvme_name: String, raw: &nvme_id_ctrl) -> Self {
        Self {
            nvme_name,
            vid: raw.vid,
            ssvid: raw.ssvid,
            serial_number: parse_ascii_field(&raw.sn),
            model_number: parse_ascii_field(&raw.mn),
            firmware_rev: parse_ascii_field(&raw.fr),
            ieee_oui: raw.ieee,
            cntlid: raw.cntlid,
            ver: raw.ver,
            subnqn: parse_ascii_field(&raw.subnqn),
            fguid: convert_cchar_to_u8_array_16(&raw.fguid),
            cntrltype: raw.cntrltype,
        }
    }
}

// =============================================================================
// CONTROLLER CAPACITY & STORAGE STRUCTS
// =============================================================================

/// Controller capacity and storage information.
///
/// Provides details about total NVM capacity, unallocated space, and maximum
/// endurance group capacity for the controller.
#[derive(Debug, Serialize, PartialEq)]
pub struct CtrlCapacity {
    /// NVMe device name (e.g., "nvme0")
    pub nvme_name: String,

    /// Total NVM Capacity (bytes)
    pub total_nvm_bytes: u128,

    /// Unallocated NVM Capacity (bytes)
    pub unallocated_nvm_bytes: u128,

    /// Maximum Endurance Group Capacity (bytes)
    pub max_endurance_group_bytes: u128,

    /// Maximum Capacity of NVM Area
    pub max_nvm_area: u32,
}

impl CtrlCapacity {
    pub fn new(nvme_name: String, raw: &nvme_id_ctrl) -> Self {
        Self {
            nvme_name,
            total_nvm_bytes: u128::from_le_bytes(raw.tnvmcap),
            unallocated_nvm_bytes: u128::from_le_bytes(raw.unvmcap),
            max_endurance_group_bytes: u128::from_le_bytes(raw.megcap),
            max_nvm_area: raw.maxcna,
        }
    }
}

// =============================================================================
// CONTROLLER CAPABILITIES & FEATURES STRUCTS
// =============================================================================

/// Controller capabilities and feature support.
///
/// Bitfield indicators for optional admin commands, NVM commands, log page
/// attributes, sanitize capabilities, and various controller features.
#[derive(Debug, Serialize, PartialEq)]
pub struct CtrlCapabilities {
    /// NVMe device name (e.g., "nvme0")
    pub nvme_name: String,

    /// Optional Admin Command Support (bitfield)
    pub oacs: u16,

    /// Optional NVM Command Support (bitfield)
    pub oncs: u16,

    /// Log Page Attributes (bitfield)
    pub lpa: u8,

    /// Controller Attributes (bitfield)
    pub ctratt: u32,

    /// Optional Asynchronous Events Supported (bitfield)
    pub oaes: u32,

    /// Sanitize Capabilities (bitfield)
    pub sanicap: u32,

    /// SGL Support (bitfield)
    pub sgls: u32,

    /// Volatile Write Cache
    pub vwc: u8,

    /// Format NVM Attributes
    pub fna: u8,

    /// ANA Capabilities
    pub anacap: u8,

    /// ANA Transition Time
    pub anatt: u8,

    /// ANA Group Identifier Maximum
    pub anagrpmax: u32,

    /// Number of ANA Group Identifiers
    pub nanagrpid: u32,

    /// Fused Operation Support
    pub fuses: u16,

    /// Optional Copy Formats Supported
    pub ocfs: u16,

    /// Controller Multi-Path I/O and Namespace Sharing Capabilities
    pub cmic: u8,

    /// Replay Protected Memory Block Support
    pub rpmbs: u32,
}

impl CtrlCapabilities {
    pub fn new(nvme_name: String, raw: &nvme_id_ctrl) -> Self {
        Self {
            nvme_name,
            oacs: raw.oacs,
            oncs: raw.oncs,
            lpa: raw.lpa,
            ctratt: raw.ctratt,
            oaes: raw.oaes,
            sanicap: raw.sanicap,
            sgls: raw.sgls,
            vwc: raw.vwc,
            fna: raw.fna,
            anacap: raw.anacap,
            anatt: raw.anatt,
            anagrpmax: raw.anagrpmax,
            nanagrpid: raw.nanagrpid,
            fuses: raw.fuses,
            ocfs: raw.ocfs,
            cmic: raw.cmic,
            rpmbs: raw.rpmbs,
        }
    }
}

// =============================================================================
// CONTROLLER LIMITS & CONSTRAINTS STRUCTS
// =============================================================================

/// Controller operational limits and constraints.
///
/// Defines maximum transfer sizes, queue entry sizes, outstanding commands,
/// namespace counts, and atomic operation units.
#[derive(Debug, Serialize, PartialEq)]
pub struct CtrlLimits {
    /// NVMe device name (e.g., "nvme0")
    pub nvme_name: String,

    /// Maximum Data Transfer Size (power of 2)
    pub mdts: u8,

    /// Submission Queue Entry Size (encoded)
    pub sqes: u8,

    /// Completion Queue Entry Size (encoded)
    pub cqes: u8,

    /// Maximum Outstanding Commands
    pub maxcmd: u16,

    /// Number of Namespaces
    pub nn: u32,

    /// Maximum Number of Allowed Namespaces
    pub mnan: u32,

    /// Abort Command Limit
    pub acl: u8,

    /// Asynchronous Event Request Limit
    pub aerl: u8,

    /// Atomic Write Unit Normal (logical blocks)
    pub awun: u16,

    /// Atomic Write Unit Power Fail (logical blocks)
    pub awupf: u16,

    /// Atomic Compare & Write Unit (logical blocks)
    pub acwu: u16,

    /// NVM Set Identifier Maximum
    pub nsetidmax: u16,

    /// Endurance Group Identifier Maximum
    pub endgidmax: u16,
}

impl CtrlLimits {
    pub fn new(nvme_name: String, raw: &nvme_id_ctrl) -> Self {
        Self {
            nvme_name,
            mdts: raw.mdts,
            sqes: raw.sqes,
            cqes: raw.cqes,
            maxcmd: raw.maxcmd,
            nn: raw.nn,
            mnan: raw.mnan,
            acl: raw.acl,
            aerl: raw.aerl,
            awun: raw.awun,
            awupf: raw.awupf,
            acwu: raw.acwu,
            nsetidmax: raw.nsetidmax,
            endgidmax: raw.endgidmax,
        }
    }
}

// =============================================================================
// CONTROLLER THERMAL MANAGEMENT STRUCTS
// =============================================================================

/// Controller thermal management configuration.
///
/// Temperature thresholds and thermal management settings for the controller.
#[derive(Debug, Serialize, PartialEq)]
pub struct CtrlThermals {
    /// NVMe device name (e.g., "nvme0")
    pub nvme_name: String,

    /// Warning Composite Temperature Threshold (Kelvin)
    pub wctemp_k: u16,

    /// Critical Composite Temperature Threshold (Kelvin)
    pub cctemp_k: u16,

    /// Minimum Thermal Management Temperature (Kelvin)
    pub mntmt_k: u16,

    /// Maximum Thermal Management Temperature (Kelvin)
    pub mxtmt_k: u16,

    /// Host Controlled Thermal Management Attributes
    pub hctma: u16,
}

impl CtrlThermals {
    pub fn new(nvme_name: String, raw: &nvme_id_ctrl) -> Self {
        Self {
            nvme_name,
            wctemp_k: raw.wctemp,
            cctemp_k: raw.cctemp,
            mntmt_k: raw.mntmt,
            mxtmt_k: raw.mxtmt,
            hctma: raw.hctma,
        }
    }
}

// =============================================================================
// CONTROLLER FIRMWARE STRUCTS
// =============================================================================

/// Controller firmware update configuration.
///
/// Settings related to firmware updates including update capabilities,
/// granularity, and activation timing.
#[derive(Debug, Serialize, PartialEq)]
pub struct CtrlFirmware {
    /// NVMe device name (e.g., "nvme0")
    pub nvme_name: String,

    /// Firmware Updates (bitfield)
    pub frmw: u8,

    /// Firmware Update Granularity (4KB units, 0 = no limit)
    pub fwug: u8,

    /// Maximum Time for Firmware Activation (100ms units)
    pub mtfa: u16,
}

impl CtrlFirmware {
    pub fn new(nvme_name: String, raw: &nvme_id_ctrl) -> Self {
        Self {
            nvme_name,
            frmw: raw.frmw,
            fwug: raw.fwug,
            mtfa: raw.mtfa,
        }
    }
}

// =============================================================================
// CONTROLLER POWER MANAGEMENT STRUCTS
// =============================================================================

/// Controller power state descriptors.
///
/// Contains all 32 power state descriptors defined by the NVMe specification
/// along with the number of supported states and autonomous transition attributes.
#[derive(Debug, Serialize, PartialEq)]
pub struct CtrlPowerStates {
    /// NVMe device name (e.g., "nvme0")
    pub nvme_name: String,

    /// Number of Power States Supported (npss + 1)
    pub num_power_states: u8,

    /// Autonomous Power State Transition Attributes
    pub apsta: u8,
    // TODO - figure out the rust syntax here :)
    // Power State Descriptors (all 32 entries from spec)
    //pub power_state_descriptors: [nvme_id_power_state; 32],
}

impl CtrlPowerStates {
    pub fn new(nvme_name: String, raw: &nvme_id_ctrl) -> Self {
        Self {
            nvme_name,
            num_power_states: raw.npss,
            apsta: raw.apsta,
            //power_state_descriptors: raw.psd,
        }
    }
}

// =============================================================================
// CONTROLLER HOST MEMORY BUFFER STRUCTS
// =============================================================================

/// Controller host memory buffer configuration.
///
/// Settings for the Host Memory Buffer (HMB) feature, including preferred and
/// minimum sizes, and descriptor limits.
#[derive(Debug, Serialize, PartialEq)]
pub struct CtrlHostMemory {
    /// NVMe device name (e.g., "nvme0")
    pub nvme_name: String,

    /// Host Memory Buffer Preferred Size (4KB units)
    pub hmpre: u32,

    /// Host Memory Buffer Minimum Size (4KB units)
    pub hmmin: u32,

    /// Host Memory Buffer Minimum Descriptor Entry Size (4KB units)
    pub hmminds: u32,

    /// Host Memory Buffer Maximum Descriptor Entries
    pub hmmaxd: u16,
}

impl CtrlHostMemory {
    pub fn new(nvme_name: String, raw: &nvme_id_ctrl) -> Self {
        Self {
            nvme_name,
            hmpre: raw.hmpre,
            hmmin: raw.hmmin,
            hmminds: raw.hmminds,
            hmmaxd: raw.hmmaxd,
        }
    }
}

// =============================================================================
// CONTROLLER ARBITRATION & QOS STRUCTS
// =============================================================================

/// Controller arbitration and quality of service settings.
///
/// Configuration for weighted round-robin arbitration.
#[derive(Debug, Serialize, PartialEq)]
pub struct CtrlArbitration {
    /// NVMe device name (e.g., "nvme0")
    pub nvme_name: String,

    /// Recommended Arbitration Burst
    pub rab: u8,
}

impl CtrlArbitration {
    pub fn new(nvme_name: String, raw: &nvme_id_ctrl) -> Self {
        Self {
            nvme_name,
            rab: raw.rab,
        }
    }
}

// =============================================================================
// CONTROLLER DIAGNOSTICS & SELF-TEST STRUCTS
// =============================================================================

/// Controller diagnostic and self-test capabilities.
///
/// Information about device self-test features, timing, and error log capacity.
#[derive(Debug, Serialize, PartialEq)]
pub struct CtrlDiagnostics {
    /// NVMe device name (e.g., "nvme0")
    pub nvme_name: String,

    /// Extended Device Self-test Time (minutes)
    pub edstt: u16,

    /// Device Self-test Options (bitfield)
    pub dsto: u8,

    /// Error Log Page Entries (0-based, actual count is elpe + 1)
    pub elpe: u8,
}

impl CtrlDiagnostics {
    pub fn new(nvme_name: String, raw: &nvme_id_ctrl) -> Self {
        Self {
            nvme_name,
            edstt: raw.edstt,
            dsto: raw.dsto,
            elpe: raw.elpe,
        }
    }
}

// =============================================================================
// CONTROLLER ADVANCED FEATURES & TIMING STRUCTS
// =============================================================================

/// Controller advanced features and timing.
///
/// RTD3 (Runtime D3) latencies, command retry delays, subsystem reporting,
/// and other advanced controller features.
#[derive(Debug, Serialize, PartialEq)]
pub struct CtrlAdvanced {
    /// NVMe device name (e.g., "nvme0")
    pub nvme_name: String,

    /// RTD3 Resume Latency (microseconds)
    pub rtd3r_us: u32,

    /// RTD3 Entry Latency (microseconds)
    pub rtd3e_us: u32,

    /// Read Recovery Levels Supported
    pub rrls: u16,

    /// Command Retry Delay Time 1 (100ms units)
    pub crdt1: u16,

    /// Command Retry Delay Time 2 (100ms units)
    pub crdt2: u16,

    /// Command Retry Delay Time 3 (100ms units)
    pub crdt3: u16,

    /// NVM Subsystem Report
    pub nvmsr: u8,

    /// VPD Write Cycle Information
    pub vwci: u8,

    /// Management Endpoint Capabilities
    pub mec: u8,

    /// Keep Alive Support (100ms units)
    pub kas: u16,

    /// Persistent Event Log Size (4KB units)
    pub pels: u32,

    /// Domain Identifier
    pub domainid: u16,
}

impl CtrlAdvanced {
    pub fn new(nvme_name: String, raw: &nvme_id_ctrl) -> Self {
        Self {
            nvme_name,
            rtd3r_us: raw.rtd3r,
            rtd3e_us: raw.rtd3e,
            rrls: raw.rrls,
            crdt1: raw.crdt1,
            crdt2: raw.crdt2,
            crdt3: raw.crdt3,
            nvmsr: raw.nvmsr,
            vwci: raw.vwci,
            mec: raw.mec,
            kas: raw.kas,
            pels: raw.pels,
            domainid: raw.domainid,
        }
    }
}

// =============================================================================
// CONTROLLER COMMAND SET STRUCTS
// =============================================================================

/// Controller command set configuration.
///
/// Vendor-specific command support and namespace write protection capabilities.
#[derive(Debug, Serialize, PartialEq)]
pub struct CtrlCommandSets {
    /// NVMe device name (e.g., "nvme0")
    pub nvme_name: String,

    /// Admin Vendor Specific Command Configuration
    pub avscc: u8,

    /// NVM Vendor Specific Command Configuration
    pub icsvscc: u8,

    /// Namespace Write Protection Capabilities
    pub nwpc: u8,
}

impl CtrlCommandSets {
    pub fn new(nvme_name: String, raw: &nvme_id_ctrl) -> Self {
        Self {
            nvme_name,
            avscc: raw.avscc,
            icsvscc: raw.icsvscc,
            nwpc: raw.nwpc,
        }
    }
}

// =============================================================================
// CONTROLLER FABRIC (NVMe-oF) STRUCTS
// =============================================================================

/// Controller fabric (NVMe-oF) configuration.
///
/// Settings specific to NVMe over Fabrics including capsule sizes,
/// offsets, and fabric command support.
#[derive(Debug, Serialize, PartialEq)]
pub struct CtrlFabric {
    /// NVMe device name (e.g., "nvme0")
    pub nvme_name: String,

    /// I/O Command Capsule Supported Size (16-byte units)
    pub ioccsz: u32,

    /// I/O Response Capsule Supported Size (16-byte units)
    pub iorcsz: u32,

    /// In Capsule Data Offset (16-byte units)
    pub icdoff: u16,

    /// Fabric Controller Attributes
    pub fcatt: u8,

    /// Management Service Data Block Descriptor
    pub msdbd: u8,

    /// Optional Fabric Commands Support
    pub ofcs: u16,
}

impl CtrlFabric {
    pub fn new(nvme_name: String, raw: &nvme_id_ctrl) -> Self {
        Self {
            nvme_name,
            ioccsz: raw.ioccsz,
            iorcsz: raw.iorcsz,
            icdoff: raw.icdoff,
            fcatt: raw.fcatt,
            msdbd: raw.msdbd,
            ofcs: raw.ofcs,
        }
    }
}

// =============================================================================
// LOG PAGE STRUCTS
// =============================================================================

/// NVMe S.M.A.R.T./Health Information Log (Log Page 0x02).
///
/// Provides S.M.A.R.T. and general health information over the life of the controller.
/// Data is retained across power cycles unless otherwise specified.
#[derive(Debug, Serialize, PartialEq)]
pub struct NvmeSmartLog {
    /// NVMe device name (e.g., "nvme0")
    pub nvme_name: String,

    /// Critical Warning bitmask (Byte 00):
    /// - Bit 0: Available spare below threshold
    /// - Bit 1: Temperature threshold condition
    /// - Bit 2: NVM subsystem degraded reliability
    /// - Bit 3: All media read-only
    /// - Bit 4: Volatile memory backup failed
    /// - Bit 5: Persistent memory region read-only
    /// - Bit 6: Indeterminate personality state
    /// - Bit 7: Reserved
    ///
    /// A value of 0 means no critical warnings
    pub critical_warning: u8,

    /// Composite Temperature (Bytes 02:01).
    ///
    /// Current temperature in Kelvins representing the composite temperature
    /// of the controller and associated namespaces.
    pub temperature: u16,

    /// Available Spare (Byte 03).
    ///
    /// Normalized percentage (0-100%) of remaining spare capacity available.
    pub avail_spare: u8,

    /// Available Spare Threshold (Byte 04).
    ///
    /// When Available Spare falls below this threshold, an asynchronous event may occur.
    /// Normalized percentage (0-100%).
    pub spare_thresh: u8,

    /// Percentage Used (Byte 05).
    ///
    /// Vendor specific estimate of the percentage of NVM subsystem life used.
    /// Value of 100 indicates estimated endurance has been consumed.
    /// May exceed 100. Updated once per power-on hour.
    pub percent_used: u8,

    /// Endurance Group Critical Warning Summary (Byte 06).
    ///
    /// - Bit 0: Endurance Group available spare capacity below threshold
    /// - Bit 1: Reserved
    /// - Bit 2: Endurance Group degraded reliability
    /// - Bit 3: Endurance Group read-only
    /// - Bits 4-7: Reserved
    pub endurance_grp_critical_warning_summary: u8,

    /// Data Units Read (Bytes 47:32).
    ///
    /// Number of 512-byte data units read from controller.
    /// Reported in thousands (value of 1 = 1,000 units).
    /// Does not include metadata.
    pub data_units_read: u128,

    /// Data Units Written (Bytes 63:48).
    ///
    /// Number of 512-byte data units written to controller.
    /// Reported in thousands (value of 1 = 1,000 units).
    /// Does not include metadata.
    pub data_units_written: u128,

    /// Host Read Commands (Bytes 79:64).
    ///
    /// Number of S.M.A.R.T. Host Read Commands completed by the controller.
    pub host_read_commands: u128,

    /// Host Write Commands (Bytes 95:80).
    ///
    /// Number of User Data Out Commands completed by the controller.
    pub host_write_commands: u128,

    /// Controller Busy Time (Bytes 111:96).
    ///
    /// Amount of time controller is busy with I/O commands.
    /// Reported in minutes.
    pub controller_busy_time: u128,

    /// Power Cycles (Bytes 127:112).
    ///
    /// Number of power cycles.
    pub power_cycles: u128,

    /// Power On Hours (Bytes 143:128).
    ///
    /// Number of power-on hours.
    /// May not include time controller was powered in non-operational state.
    pub power_on_hours: u128,

    /// Unsafe Shutdowns / Unexpected Power Losses (Bytes 159:144).
    ///
    /// Count of unexpected power losses where controller was not ready
    /// to be powered off or media was not in shutdown state.
    pub unsafe_shutdowns: u128,

    /// Media and Data Integrity Errors (Bytes 175:160).
    ///
    /// Number of occurrences where controller detected un-recovered data integrity error.
    /// Includes uncorrectable ECC, CRC checksum failure, LBA tag mismatch.
    pub media_errors: u128,

    /// Number of Error Information Log Entries (Bytes 191:176).
    ///
    /// Number of Error Information Log Entries over the life of the controller.
    pub num_err_log_entries: u128,

    /// Warning Composite Temperature Time (Bytes 195:192).
    ///
    /// Time in minutes that Composite Temperature is >= Warning Threshold
    /// and < Critical Threshold.
    pub warning_temp_time: u32,

    /// Critical Composite Temperature Time (Bytes 199:196).
    ///
    /// Time in minutes that Composite Temperature is >= Critical Threshold.
    pub critical_comp_time: u32,

    /// Temperature Sensor 1 (Bytes 201:200).
    ///
    /// Current temperature reported by temperature sensor 1 in Kelvins.
    pub temperature_sensor_1: Option<u16>,

    /// Temperature Sensor 2 (Bytes 203:202).
    ///
    /// Current temperature reported by temperature sensor 2 in Kelvins.
    pub temperature_sensor_2: Option<u16>,

    /// Temperature Sensor 3 (Bytes 205:204).
    ///
    /// Current temperature reported by temperature sensor 3 in Kelvins.
    pub temperature_sensor_3: Option<u16>,

    /// Temperature Sensor 4 (Bytes 207:206).
    ///
    /// Current temperature reported by temperature sensor 4 in Kelvins.
    pub temperature_sensor_4: Option<u16>,

    /// Temperature Sensor 5 (Bytes 209:208).
    ///
    /// Current temperature reported by temperature sensor 5 in Kelvins.
    pub temperature_sensor_5: Option<u16>,

    /// Temperature Sensor 6 (Bytes 211:210).
    ///
    /// Current temperature reported by temperature sensor 6 in Kelvins.
    pub temperature_sensor_6: Option<u16>,

    /// Temperature Sensor 7 (Bytes 213:212).
    ///
    /// Current temperature reported by temperature sensor 7 in Kelvins.
    pub temperature_sensor_7: Option<u16>,

    /// Temperature Sensor 8 (Bytes 215:214).
    ///
    /// Current temperature reported by temperature sensor 8 in Kelvins.
    pub temperature_sensor_8: Option<u16>,

    /// Thermal Management Temperature 1 Transition Count (Bytes 219:216).
    ///
    /// Number of times controller transitioned to lower power states to reduce
    /// temperature after rising above Thermal Management Temperature 1.
    /// Does not wrap after reaching 0xFFFFFFFF.
    pub thm_temp1_trans_count: u32,

    /// Thermal Management Temperature 2 Transition Count (Bytes 223:220).
    ///
    /// Number of times controller performed heavy thermal throttling to reduce
    /// temperature after rising above Thermal Management Temperature 2.
    /// Does not wrap after reaching 0xFFFFFFFF.
    pub thm_temp2_trans_count: u32,

    /// Total Time For Thermal Management Temperature 1 (Bytes 227:224).
    ///
    /// Number of seconds controller spent in lower power states due to
    /// Thermal Management Temperature 1. Reported in seconds.
    /// Does not wrap after reaching 0xFFFFFFFF.
    pub thm_temp1_total_time: u32,

    /// Total Time For Thermal Management Temperature 2 (Bytes 231:228).
    ///
    /// Number of seconds controller spent performing heavy throttling due to
    /// Thermal Management Temperature 2. Reported in seconds.
    /// Does not wrap after reaching 0xFFFFFFFF.
    pub thm_temp2_total_time: u32,
}

impl NvmeSmartLog {
    /// Create a new NvmeSmartLog from raw nvme_smart_log data.
    pub fn new(nvme_name: String, raw: &nvme_smart_log) -> Self {
        Self {
            nvme_name,

            critical_warning: raw.critical_warning,
            temperature: u16::from_le_bytes([raw.temperature[0], raw.temperature[1]]),
            avail_spare: raw.avail_spare,
            spare_thresh: raw.spare_thresh,
            percent_used: raw.percent_used,
            endurance_grp_critical_warning_summary: raw.endu_grp_crit_warn_sumry,

            data_units_read: u128::from_le_bytes(raw.data_units_read),
            data_units_written: u128::from_le_bytes(raw.data_units_written),
            host_read_commands: u128::from_le_bytes(raw.host_reads),
            host_write_commands: u128::from_le_bytes(raw.host_writes),
            controller_busy_time: u128::from_le_bytes(raw.ctrl_busy_time),
            power_cycles: u128::from_le_bytes(raw.power_cycles),
            power_on_hours: u128::from_le_bytes(raw.power_on_hours),
            unsafe_shutdowns: u128::from_le_bytes(raw.unsafe_shutdowns),
            media_errors: u128::from_le_bytes(raw.media_errors),
            num_err_log_entries: u128::from_le_bytes(raw.num_err_log_entries),

            warning_temp_time: raw.warning_temp_time,
            critical_comp_time: raw.critical_comp_time,

            temperature_sensor_1: match raw.temp_sensor[0] {
                0 => None,
                v => Some(v),
            },
            temperature_sensor_2: match raw.temp_sensor[1] {
                0 => None,
                v => Some(v),
            },
            temperature_sensor_3: match raw.temp_sensor[2] {
                0 => None,
                v => Some(v),
            },
            temperature_sensor_4: match raw.temp_sensor[3] {
                0 => None,
                v => Some(v),
            },
            temperature_sensor_5: match raw.temp_sensor[4] {
                0 => None,
                v => Some(v),
            },
            temperature_sensor_6: match raw.temp_sensor[5] {
                0 => None,
                v => Some(v),
            },
            temperature_sensor_7: match raw.temp_sensor[6] {
                0 => None,
                v => Some(v),
            },
            temperature_sensor_8: match raw.temp_sensor[7] {
                0 => None,
                v => Some(v),
            },

            thm_temp1_trans_count: raw.thm_temp1_trans_count,
            thm_temp2_trans_count: raw.thm_temp2_trans_count,
            thm_temp1_total_time: raw.thm_temp1_total_time,
            thm_temp2_total_time: raw.thm_temp2_total_time,
        }
    }
}

/// Error Information Log Entry (Log Identifier 0x01).
///
/// Contains detailed information about a specific error event that occurred
/// on the NVMe controller. Error entries are stored in a circular buffer,
/// with the oldest entries being overwritten when the buffer is full.
#[derive(Debug, Serialize, PartialEq)]
pub struct ErrorEntry {
    /// Error Count: a 64-bit incrementing error count, indicating a unique
    /// identifier for this error.
    ///
    /// The error count starts at 1h, is incremented for each unique error
    /// log entry, and is retained across power off conditions. A value of
    /// 0h indicates an invalid entry (used when there are lost entries or
    /// fewer errors than the maximum number of entries the controller supports).
    /// If the value reaches FFFFFFFFFFFFFFFFh, it rolls over to 1h when incremented.
    pub error_count: u64,

    /// Submission Queue ID: indicates the Submission Queue Identifier of the
    /// command that the error information is associated with.
    ///
    /// If the error is not specific to a particular command, this field is
    /// set to FFFFh.
    pub submission_queue_id: u16,

    /// Command ID: indicates the Command Identifier of the command that the
    /// error is associated with.
    ///
    /// If the error is not specific to a particular command, this field is
    /// set to FFFFh.
    pub command_id: u16,

    /// Status Field: indicates the Status Field for the command that completed.
    ///
    /// Bits 15-1: Status code and type. If the error is not specific to a
    /// particular command, this field reports the most applicable status value.
    /// Bit 0: Phase Tag that may have been posted for the command.
    pub status_field: u16,

    /// Parameter Error Location: indicates the byte and bit of the command
    /// parameter that the error is associated with, if applicable.
    ///
    /// If the parameter spans multiple bytes or bits, this location indicates
    /// the first byte and bit of the parameter.
    /// - Bits 10-8: Bit in command that contained the error (valid values: 0-7)
    /// - Bits 7-0: Byte in command that contained the error (valid values: 0-63)
    pub parameter_error_location: u16,

    /// LBA: indicates the first Logical Block Address that experienced the
    /// error condition, if applicable.
    pub lba: u64,

    /// Namespace ID: indicates the NSID of the namespace that the error is
    /// associated with, if applicable.
    pub namespace_id: u32,

    /// Vendor Specific Information Available: If there is additional vendor
    /// specific error information available, this field provides the log page
    /// identifier associated with that page.
    ///
    /// A value of 0h indicates that no additional information is available.
    /// Valid values are in the range of 80h to FFh.
    pub vendor_specific: u8,

    /// Transport Type (TRTYPE): indicates the Transport Type of the transport
    /// associated with the error.
    ///
    /// The values in this field are the same as the TRTYPE values in the
    /// Discovery Log Page Entry. If the error is not transport related, this
    /// field is cleared to 0h. If the error is transport related, this field
    /// is set to the type of the transport.
    pub transport_type: u8,

    /// Command Specific Information: contains command specific information.
    ///
    /// If used, the command definition specifies the information returned.
    pub command_specific: u64,

    /// Transport Type Specific Information: contains information specific
    /// to the transport type indicated in the transport_type field.
    pub transport_type_specific_info: u16,
}

impl ErrorEntry {
    pub fn new(raw: &nvme_error_log_page) -> Self {
        Self {
            error_count: u64::from_le(raw.error_count),
            submission_queue_id: u16::from_le(raw.sqid),
            command_id: u16::from_le(raw.cmdid),
            status_field: u16::from_le(raw.status_field),
            parameter_error_location: u16::from_le(raw.parm_error_location),
            lba: u64::from_le(raw.lba),
            namespace_id: u32::from_le(raw.nsid),
            vendor_specific: raw.vs,
            transport_type: raw.trtype,
            command_specific: u64::from_le(raw.cs),
            transport_type_specific_info: u16::from_le(raw.trtype_spec_info),
        }
    }
}

/// Error Information Log (Log Page 0x01).
/// This struct is to be used by user to extract ErrorEntry
#[derive(Debug, Serialize)]
pub struct NvmeErrorLog {
    pub nvme_name: String,
    pub serial_number: String,
    pub entries: Vec<ErrorEntry>,
}

impl NvmeErrorLog {
    pub fn new(
        nvme_name: String,
        serial_number: String,
        raw_entries: Vec<nvme_error_log_page>,
    ) -> Self {
        // Filter out unpopulated entries (error_count == 0)
        let entries = raw_entries
            .iter()
            .filter(|e| e.error_count != 0)
            .map(ErrorEntry::new)
            .collect();

        Self {
            nvme_name,
            serial_number,
            entries,
        }
    }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Parse ASCII field from raw c_char byte array, trimming spaces and nulls.
///
/// NVMe spec uses C char arrays for ASCII fields. This function converts
/// to unsigned bytes, then parses as UTF-8 and trims whitespace/nulls.
/// This helper function is required because serial number, model number,
/// and firmware revision are space padded.
fn parse_ascii_field(bytes: &[c_char]) -> String {
    let unsigned: Vec<u8> = bytes.iter().map(|&b| b as u8).collect();

    String::from_utf8_lossy(&unsigned)
        .trim_end_matches('\0')
        .trim()
        .to_string()
}

/// Convert [c_char; N] to [u8; N] for GUID fields.
fn convert_cchar_to_u8_array_16(bytes: &[c_char; 16]) -> [u8; 16] {
    let mut result = [0u8; 16];
    for (i, &b) in bytes.iter().enumerate() {
        result[i] = b as u8;
    }
    result
}

// =============================================================================
// DISCOVERY
// =============================================================================

/// Discover NVMe controllers exposed on the server.
///
/// Returns a list of NVMe controller names found in /sys/class/nvme.
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
/// Users should call `read_error_log()` instead.
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
fn read_error_log_raw(dev_path: &str, num_entries: u16) -> io::Result<Vec<nvme_error_log_page>> {
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

/// Read Error Information Log (Log Page 0x01).
///
/// Automatically determines the correct number of entries to read by querying
/// the controller's ELPE (Error Log Page Entries) field.
///
/// # Arguments
/// * `dev_path` - Path to the NVMe device (e.g., "/dev/nvme0")
///
/// # Errors
/// Returns an error if:
/// - The device cannot be opened
/// - The identify command fails
/// - The error log command fails
pub fn read_error_log(dev_path: &str) -> io::Result<NvmeErrorLog> {
    let nvme_name = dev_path.trim_start_matches("/dev/").to_string();

    // Query controller to get ELPE
    let id_ctrl = read_nvme_id_ctrl(dev_path)?;
    let diag = CtrlDiagnostics::new(nvme_name.clone(), &id_ctrl);
    let serial_number = parse_ascii_field(&id_ctrl.sn);

    // Calculate number of entries (ELPE is 0-based)
    let max_entries = (diag.elpe + 1) as u16;

    let raw_entries = read_error_log_raw(dev_path, max_entries)?;
    Ok(NvmeErrorLog::new(nvme_name, serial_number, raw_entries))
}
