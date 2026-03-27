// examples/sanity_check.rs
use nvme_telem::nvme;
use nvme_telem::vendors;

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

        // Test 2: S.M.A.R.T. Log
        print!("\nTest 2: Reading SMART log... ");
        match nvme::get_smart_log(&dev_path) {
            Ok(smart) => {
                println!("[OK] Success!");
                println!(
                    "  Device: {} (S/N: {})",
                    smart.nvme_name, smart.serial_number
                );
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
        match nvme::get_controller_identity(&dev_path) {
            Ok(identity) => {
                println!("[OK] Success!");
                println!("\n  Identity:");
                println!("    Vendor ID: 0x{:04x}", identity.vid);
                println!("    Subsystem Vendor ID: 0x{:04x}", identity.ssvid);
                println!("    Serial: {}", identity.serial_number);
                println!("    Model: {}", identity.model_number);
                println!("    Firmware: {}", identity.firmware_rev);
            }
            Err(e) => {
                println!("[FAIL] Failed: {}", e);
                println!("       (This might require sudo/root access)");
            }
        }

        // Test 4: Error Log
        print!("\nTest 4: Reading error log (0x01)... ");
        match nvme::get_error_log(&dev_path) {
            Ok(error_log) => {
                println!("[OK] Success!");
                println!(
                    "  Device: {} (S/N: {})",
                    error_log.nvme_name, error_log.serial_number
                );
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
        match vendors::read_ocp_smart_log(&dev_path) {
            Ok(raw) => {
                let ocp = vendors::OcpSmartData::new(ctrl.clone(), &raw);
                println!("[OK] Success!");
                println!("  Device: {}", ocp.nvme_name);
                println!(
                    "  Physical Media Written: {}",
                    ocp.physical_media_units_written
                );
                println!("  Physical Media Read: {}", ocp.physical_media_units_read);
                println!("  Bad User NAND Blocks: {}", ocp.bad_user_nand_blocks_raw);
                println!(
                    "  Bad System NAND Blocks: {}",
                    ocp.bad_system_nand_blocks_raw
                );
                println!("  Percent Free Blocks: {}%", ocp.percent_free_blocks);
                println!(
                    "  User Data Erase Count (Max): {}",
                    ocp.user_data_erase_count_max
                );
                println!(
                    "  User Data Erase Count (Min): {}",
                    ocp.user_data_erase_count_min
                );
                println!("  NAND Avg Erase Count: {}", ocp.nand_avg_erase_count);
                println!(
                    "  Thermal Throttling Events: {}",
                    ocp.thermal_throttling_events
                );
                println!("  PCIe Correctable Errors: {}", ocp.pcie_correctable_errors);
                println!("  Incomplete Shutdowns: {}", ocp.incomplete_shutdowns);
                println!("  Unaligned I/O: {}", ocp.unaligned_io);
                println!("  Command Timeouts: {}", ocp.command_timeouts);
                println!("  Total Media Dies: {}", ocp.total_media_dies);
                println!("  Media Dies Offline: {}", ocp.media_dies_offline);
                println!("  Log Page Version: {}", ocp.log_page_version);
            }
            Err(e) => {
                println!("[FAIL] Not available: {}", e);
                println!(
                    "       (OCP extended SMART is vendor-specific - not all drives support it)"
                );
            }
        }
    }

    println!("\n\n{}", "=".repeat(60));
    println!("Sanity check complete!");
    println!("{}", "=".repeat(60));
}
