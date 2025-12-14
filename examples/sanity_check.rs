// examples/sanity_check.rs

use nvme_telem::nvme::*;

fn main() {
    println!("--- NVMe Sanity Check ---\n");

    // Test 1: List controllers
    println!("Test 1: Discovering NVMe controllers...");
    let controllers = list_nvme_controllers();

    if controllers.is_empty() {
        println!("❌ No NVMe controllers found!");
        println!("   This might be okay if you don't have NVMe hardware.");
        return;
    }

    println!(
        "✅ Found {} controller(s): {:?}\n",
        controllers.len(),
        controllers
    );

    // Test 2-3: Try to read data from each controller
    for ctrl in controllers {
        let dev_path = format!("/dev/{}", ctrl);
        println!("\n{}", "=".repeat(60));
        println!("Testing: {}", ctrl);
        println!("{}", "=".repeat(60));

        // Test 2: SMART Log
        print!("\nTest 2: Reading SMART log... ");
        match get_nvme_smart_log_raw(&dev_path) {
            Ok(raw) => {
                let smart = NvmeSmartLog::new(ctrl.clone(), &raw);
                println!("✅ Success!");
                println!("  Temperature: {:?} K", smart.temperature);
                println!("  Power Cycles: {:?}", smart.power_cycles);
                println!("  Power On Hours: {:?}", smart.power_on_hours);
                println!(
                    "  Data Written: {:?} (thousands of 512-byte units)",
                    smart.data_units_written
                );
                println!("  Critical Warning: {:?}", smart.critical_warning);
            }
            Err(e) => {
                println!("❌ Failed: {}", e);
                println!("   (This might require sudo/root access)");
            }
        }

        // Test 3: Controller Identity
        print!("\nTest 3: Reading controller identity... ");
        match get_nvme_id_ctrl_raw(&dev_path) {
            Ok(raw) => {
                println!("✅ Success!");

                let identity = CtrlIdentity::new(ctrl.clone(), &raw);
                println!("\n  Identity:");
                println!("    Vendor ID: 0x{:04x}", identity.vid);
                println!("    Subsystem Vendor ID: 0x{:04x}", identity.ssvid);
                println!("    Serial: {}", identity.serial_number);
                println!("    Model: {}", identity.model_number);
                println!("    Firmware: {}", identity.firmware_rev);

                let capacity = CtrlCapacity::new(ctrl.clone(), &raw);
                println!("\n  Capacity:");
                println!(
                    "    Total NVM: {} bytes ({} GB)",
                    capacity.total_nvm_bytes,
                    capacity.total_nvm_bytes / 1_000_000_000
                );

                let thermals = CtrlThermals::new(ctrl.clone(), &raw);
                println!("\n  Thermal:");
                println!("    Warning Temp: {} K", thermals.wctemp_k);
                println!("    Critical Temp: {} K", thermals.cctemp_k);

                let limits = CtrlLimits::new(ctrl.clone(), &raw);
                println!("\n  Limits:");
                println!("    Max Data Transfer Size: 2^{} pages", limits.mdts);
                println!("    Number of Namespaces: {}", limits.nn);
                println!("    Max Outstanding Commands: {}", limits.maxcmd);
            }
            Err(e) => {
                println!("❌ Failed: {}", e);
                println!("   (This might require sudo/root access)");
            }
        }
    }

    println!("\n\n{}", "=".repeat(60));
    println!("Sanity check complete!");
    println!("{}", "=".repeat(60));
}
