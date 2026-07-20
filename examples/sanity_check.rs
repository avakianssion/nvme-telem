// examples/sanity_check.rs
use nvme_telem::nvme::{self, Device};

fn main() {
    println!("--- NVMe Sanity Check ---\n");

    // Test 1: List controllers
    println!("Test 1: Discovering NVMe controllers...");
    let controllers = nvme::list_nvme_controllers();
    if controllers.is_empty() {
        println!("[FAIL] No NVMe controllers found!");
        println!("       This might be okay if you don't have NVMe hardware.");
        return;
    }
    println!(
        "[OK] Found {} controller(s): {:?}\n",
        controllers.len(),
        controllers
    );

    // Test 2-5: Try to read data from each controller
    for ctrl in controllers {
        let dev_path = format!("/dev/{}", ctrl);
        println!("\n{}", "=".repeat(60));
        println!("Testing: {}", ctrl);
        println!("{}", "=".repeat(60));

        let device = match Device::open(&dev_path) {
            Ok(device) => device,
            Err(e) => {
                println!("[FAIL] Could not open {}: {}", dev_path, e);
                println!("       (This might require sudo/root access)");
                continue;
            }
        };

        // Every metrics object this crate returns carries `nvme_name` and
        // `serial_number` so it can be identified on its own. Collect the
        // serial number reported by each log below and confirm they agree.
        let mut serials: Vec<(&str, String)> = Vec::new();

        // Test 2: S.M.A.R.T. Log
        print!("\nTest 2: Reading SMART log... ");
        match device.smart_log() {
            Ok(smart) => {
                println!("[OK] Success!");
                println!(
                    "  Device: {} (S/N: {})",
                    smart.nvme_name, smart.serial_number
                );
                serials.push(("smart_log", smart.serial_number.clone()));
                println!("  Temperature: {} K", smart.temperature);
                println!("  Power Cycles: {}", smart.power_cycles);
                println!("  Power On Hours: {}", smart.power_on_hours);
                println!(
                    "  Data Written: {} (thousands of 512-byte units)",
                    smart.data_units_written
                );
                println!("  Critical Warning: {:#x}", smart.critical_warning);
            }
            Err(e) => {
                println!("[FAIL] Failed: {}", e);
                println!("       (This might require sudo/root access)");
            }
        }

        // Test 3: Controller Identity
        print!("\nTest 3: Reading controller identity... ");
        match device.identity() {
            Ok(identity) => {
                println!("[OK] Success!");
                println!("\n  Identity:");
                println!("    Vendor ID: 0x{:04x}", identity.vid);
                println!("    Subsystem Vendor ID: 0x{:04x}", identity.ssvid);
                println!("    Serial: {}", identity.serial_number);
                println!("    Model: {}", identity.model_number);
                println!("    Firmware: {}", identity.firmware_rev);
                serials.push(("identity", identity.serial_number.clone()));
            }
            Err(e) => {
                println!("[FAIL] Failed: {}", e);
                println!("       (This might require sudo/root access)");
            }
        }

        // Test 4: Error Log
        print!("\nTest 4: Reading error log (0x01)... ");
        match device.error_log() {
            Ok(error_log) => {
                println!("[OK] Success!");
                println!(
                    "  Device: {} (S/N: {})",
                    error_log.nvme_name, error_log.serial_number
                );
                serials.push(("error_log", error_log.serial_number.clone()));
                if error_log.entries.is_empty() {
                    println!("  No errors recorded - healthy drive!");
                } else {
                    println!("  Found {} error(s):", error_log.entries.len());
                    // Show first 5 errors
                    for (i, entry) in error_log.entries.iter().take(5).enumerate() {
                        println!("\n  Error {}:", i + 1);
                        println!("    Error Count: {}", entry.error_count);
                        println!("    Submission Queue ID: {}", entry.submission_queue_id);
                        println!("    Command ID: {}", entry.command_id);
                        println!("    Status Field: {:#x}", entry.status_field);
                        println!("    LBA: {:#x}", entry.lba);
                        println!("    Namespace ID: {}", entry.namespace_id);
                    }
                    if error_log.entries.len() > 5 {
                        println!("\n  ... and {} more error(s)", error_log.entries.len() - 5);
                    }
                }
            }
            Err(e) => {
                println!("[FAIL] Failed: {}", e);
                println!("       (This might require sudo/root access)");
            }
        }

        // Test 5: OCP Extended SMART Log
        print!("\nTest 5: Reading OCP extended SMART log (0xC0)... ");
        match device.ocp_smart_log() {
            Ok(smart_add_log) => {
                println!("[OK] Success!");
                println!(
                    "  Device: {} (S/N: {})",
                    smart_add_log.nvme_name, smart_add_log.serial_number
                );
                serials.push(("ocp_smart_log", smart_add_log.serial_number.clone()));
                println!(
                    "  Physical Media Written: {}",
                    smart_add_log.physical_media_units_written
                );
                println!(
                    "  Physical Media Read: {}",
                    smart_add_log.physical_media_units_read
                );
                println!(
                    "  Bad User NAND Blocks: {}",
                    smart_add_log.bad_user_nand_blocks_raw
                );
                println!(
                    "  Bad System NAND Blocks: {}",
                    smart_add_log.bad_system_nand_blocks_raw
                );
                println!(
                    "  Percent Free Blocks: {}%",
                    smart_add_log.percent_free_blocks
                );
                println!(
                    "  User Data Erase Count (Max): {}",
                    smart_add_log.user_data_erase_count_max
                );
                println!(
                    "  User Data Erase Count (Min): {}",
                    smart_add_log.user_data_erase_count_min
                );
                println!(
                    "  NAND Avg Erase Count: {}",
                    smart_add_log.nand_avg_erase_count
                );
                println!(
                    "  Thermal Throttling Events: {}",
                    smart_add_log.thermal_throttling_events
                );
                println!(
                    "  PCIe Correctable Errors: {}",
                    smart_add_log.pcie_correctable_errors
                );
                println!(
                    "  Incomplete Shutdowns: {}",
                    smart_add_log.incomplete_shutdowns
                );
                println!("  Unaligned I/O: {}", smart_add_log.unaligned_io);
                println!("  Command Timeouts: {}", smart_add_log.command_timeouts);
                println!("  Total Media Dies: {}", smart_add_log.total_media_dies);
                println!("  Media Dies Offline: {}", smart_add_log.media_dies_offline);
                println!("  Log Page Version: {}", smart_add_log.log_page_version);
            }
            Err(e) => {
                println!("[FAIL] Not available: {}", e);
                println!(
                    "       (OCP extended SMART is vendor-specific - not all drives support it)"
                );
            }
        }

        // Test 6: Identifier consistency across all collected logs
        print!("\nTest 6: Checking serial number consistency across logs... ");
        match serials.split_first() {
            Some(((_, first), rest)) => {
                if let Some((label, mismatched)) = rest.iter().find(|(_, sn)| sn != first) {
                    println!("[FAIL] Mismatch!");
                    println!("  Expected S/N: {} (from {})", first, serials[0].0);
                    println!("  Got S/N: {} (from {})", mismatched, label);
                } else {
                    println!("[OK] All {} log(s) agree: S/N {}", serials.len(), first);
                }
            }
            None => println!("[SKIP] No logs were successfully read for this device"),
        }
    }

    println!("\n\n{}", "=".repeat(60));
    println!("Sanity check complete!");
    println!("{}", "=".repeat(60));
}
