//! Print the Identify Controller data for each NVMe controller.

use libnvme::Root;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = Root::scan()?;
    let mut any = false;
    for host in root.hosts() {
        for subsys in host.subsystems() {
            for ctrl in subsys.controllers() {
                any = true;
                print_controller(&ctrl)?;
            }
        }
    }
    if !any {
        println!("(no NVMe controllers found)");
    }
    Ok(())
}

fn print_controller(ctrl: &libnvme::Controller<'_>) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== {} ===", ctrl.name()?);
    let id = ctrl.identify()?;
    println!("  vendor id          : 0x{:04x}", id.vendor_id());
    println!("  subsystem vendor   : 0x{:04x}", id.subsystem_vendor_id());
    println!("  model              : {}", id.model_number()?);
    println!("  serial             : {}", id.serial_number()?);
    println!("  firmware           : {}", id.firmware_revision()?);
    println!("  nvme spec          : {}", id.nvme_version());
    println!("  controller id      : {}", id.controller_id());
    println!("  controller type    : {}", id.controller_type());
    println!("  num namespaces     : {}", id.num_namespaces());
    println!(
        "  total nvm capacity : {} bytes",
        id.total_nvm_capacity_bytes()
    );
    println!(
        "  warn temp threshold: {} K",
        id.warning_temp_threshold_kelvin()
    );
    println!(
        "  crit temp threshold: {} K",
        id.critical_temp_threshold_kelvin()
    );
    println!();
    Ok(())
}
