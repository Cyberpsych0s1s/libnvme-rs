//! Smoke test for `Namespace::format`.
//!
//! **Refuses to run unless the controller's model string equals `QEMU NVMe Ctrl`.**
//! This is a deliberate safety latch — on real hardware the Format NVM admin
//! command erases all user data on the namespace, and a slipped fingertip
//! would otherwise wipe whoever's boot drive.
//!
//! To exercise on the QEMU fixture:
//!
//! ```sh
//! bash tests/qemu/run.sh
//! # then inside the guest:
//! cd /mnt/host && CARGO_TARGET_DIR=/tmp/target sudo -E cargo run \
//!     --example format_smoke -p libnvme
//! ```

use libnvme::{Root, SecureErase};

const QEMU_MODEL: &str = "QEMU NVMe Ctrl";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = Root::scan()?;
    let mut formatted = 0;
    for host in root.hosts() {
        for subsys in host.subsystems() {
            for ctrl in subsys.controllers() {
                let model = ctrl.model().unwrap_or("?").trim();
                if model != QEMU_MODEL {
                    println!(
                        "skipping {}: model {model:?} != {QEMU_MODEL:?} (refusing to format real hardware)",
                        ctrl.name()?
                    );
                    continue;
                }
                for ns in ctrl.namespaces() {
                    let nsid = ns.nsid();
                    let lba_size_before = ns.lba_size();
                    let lba_count_before = ns.lba_count();
                    println!(
                        "about to format {}: NSID={nsid}, {lba_count_before} blocks × {lba_size_before} B",
                        ns.name()?
                    );
                    ns.format()
                        .lba_format(0)
                        .secure_erase(SecureErase::None)
                        .execute()?;
                    println!("  -> format succeeded");
                    formatted += 1;
                }
            }
        }
    }
    if formatted == 0 {
        println!("(no QEMU virtual NVMe controllers found)");
    } else {
        println!("formatted {formatted} namespace(s)");
    }
    Ok(())
}
