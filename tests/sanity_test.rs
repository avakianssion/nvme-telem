// tests/sanity_test.rs

use nvme_telem::collector::nvme::*;

#[test]
#[ignore] // Run with: cargo test -- --ignored
fn sanity_check_nvme_discovery() {
    let controllers = list_nvme_controllers();

    // This should at least not panic
    println!("Found {} controllers", controllers.len());

    if !controllers.is_empty() {
        println!("Controllers: {:?}", controllers);
    }
}

#[test]
#[ignore]
fn sanity_check_smart_log() {
    let controllers = list_nvme_controllers();

    if controllers.is_empty() {
        println!("Skipping: No NVMe controllers found");
        return;
    }

    for ctrl in controllers {
        let dev_path = format!("/dev/{}", ctrl);

        match get_nvme_smart_log_raw(&dev_path) {
            Ok(raw) => {
                let smart = NvmesSmartLog::new(ctrl.clone(), &raw);
                println!("{}: Temperature = {:?}K", ctrl, smart.temperature);

                // Basic sanity checks
                if let Some(temp) = smart.temperature {
                    assert!(
                        temp > 200 && temp < 500,
                        "Temperature out of range: {}",
                        temp
                    );
                }
            }
            Err(e) => {
                eprintln!(
                    "Failed to read SMART for {}: {} (may need sudo)",
                    dev_path, e
                );
            }
        }
    }
}

#[test]
#[ignore]
fn sanity_check_controller_id() {
    let controllers = list_nvme_controllers();

    if controllers.is_empty() {
        println!("Skipping: No NVMe controllers found");
        return;
    }

    for ctrl in controllers {
        let dev_path = format!("/dev/{}", ctrl);

        match get_nvme_id_ctrl_raw(&dev_path) {
            Ok(raw) => {
                let identity = CtrlIdentity::new(ctrl.clone(), &raw);
                println!(
                    "{}: VID=0x{:04x}, Model={}",
                    ctrl, identity.vid, identity.model_number
                );

                // VID should be non-zero for real devices
                assert_ne!(identity.vid, 0, "Vendor ID is zero");

                let capacity = CtrlCapacity::new(ctrl.clone(), &raw);
                let thermals = CtrlThermals::new(ctrl.clone(), &raw);
                let limits = CtrlLimits::new(ctrl, &raw);

                // Verify they construct without panic
                println!("  Capacity: {} bytes", capacity.total_nvm_bytes);
                println!("  Warning temp: {}K", thermals.wctemp_k);
                println!("  Max namespaces: {}", limits.nn);
            }
            Err(e) => {
                eprintln!("Failed to read ID for {}: {} (may need sudo)", dev_path, e);
            }
        }
    }
}
