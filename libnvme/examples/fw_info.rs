//! Print firmware-slot information for each NVMe controller.
//!
//! Read-only. Demonstrates `Controller::fw_slot_log()`. Run with `sudo`
//! (or as a member of the `disk` group) since the Get Log Page admin
//! command needs access to `/dev/nvme*`.

use libnvme::Root;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = Root::scan()?;
    let mut any = false;
    for host in root.hosts() {
        for subsys in host.subsystems() {
            for ctrl in subsys.controllers() {
                any = true;
                print_fw(&ctrl)?;
            }
        }
    }
    if !any {
        println!("(no NVMe controllers found)");
    }
    Ok(())
}

fn print_fw(ctrl: &libnvme::Controller<'_>) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== {} ===", ctrl.name()?);
    let log = ctrl.fw_slot_log()?;
    println!("  AFI byte           : 0x{:02x}", log.afi());
    println!("  active slot        : {}", log.active_slot());
    match log.next_slot_to_activate() {
        Some(slot) => println!("  next activate slot : {}", slot),
        None => println!("  next activate slot : (none — current image stays active)"),
    }
    for slot in 1..=7 {
        match log.slot_firmware(slot) {
            Ok("") => println!("  slot {slot}             : (empty)"),
            Ok(fw) => println!("  slot {slot}             : {fw}"),
            Err(e) => println!("  slot {slot}             : error: {e}"),
        }
    }
    println!();
    Ok(())
}
