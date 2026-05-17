//! Fetch and print the SMART / Health log page for each NVMe controller.

use libnvme::Root;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = Root::scan()?;
    let mut any = false;
    for host in root.hosts() {
        for subsys in host.subsystems() {
            for ctrl in subsys.controllers() {
                any = true;
                print_smart(&ctrl)?;
            }
        }
    }
    if !any {
        println!("(no NVMe controllers found)");
    }
    Ok(())
}

fn print_smart(ctrl: &libnvme::Controller<'_>) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== {} ===", ctrl.name()?);
    let log = ctrl.smart_log()?;
    println!(
        "  temperature        : {} K ({} C)",
        log.temperature_kelvin(),
        log.temperature_celsius()
    );
    println!(
        "  available spare    : {}% (threshold {}%)",
        log.available_spare(),
        log.available_spare_threshold()
    );
    println!("  percentage used    : {}%", log.percentage_used());
    println!("  data units read    : {}", log.data_units_read());
    println!("  data units written : {}", log.data_units_written());
    println!("  host read cmds     : {}", log.host_read_commands());
    println!("  host write cmds    : {}", log.host_write_commands());
    println!("  power cycles       : {}", log.power_cycles());
    println!("  power on hours     : {}", log.power_on_hours());
    println!("  unsafe shutdowns   : {}", log.unsafe_shutdowns());
    println!("  media errors       : {}", log.media_errors());
    println!("  critical warning   : 0x{:02x}", log.critical_warning());
    println!();
    Ok(())
}
