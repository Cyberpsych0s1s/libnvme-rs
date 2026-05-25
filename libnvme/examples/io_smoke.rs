//! Smoke test for the v0.7 I/O command surface: write -> read -> compare
//! -> verify -> write_zeroes -> dsm(deallocate) -> flush.
//!
//! **Refuses to run unless the controller's model string equals
//! `QEMU NVMe Ctrl`.** Same safety latch as `format_smoke` — these commands
//! mutate user data, so they must never touch a real disk.
//!
//! ```sh
//! bash tests/qemu/run.sh
//! # inside the guest:
//! cd /mnt/host && CARGO_TARGET_DIR=/tmp/target sudo -E cargo run \
//!     --example io_smoke -p libnvme
//! ```

use libnvme::{DsmAttr, DsmRange, Root};

const QEMU_MODEL: &str = "QEMU NVMe Ctrl";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = Root::scan()?;
    let mut exercised = 0;

    for host in root.hosts() {
        for subsys in host.subsystems() {
            for ctrl in subsys.controllers() {
                let model = ctrl.model().unwrap_or("?").trim();
                if model != QEMU_MODEL {
                    println!(
                        "skipping {}: model {model:?} != {QEMU_MODEL:?} (refusing on real hardware)",
                        ctrl.name()?
                    );
                    continue;
                }

                for ns in ctrl.namespaces() {
                    let lba_size = ns.lba_size();
                    println!(
                        "exercising {} (NSID {}, {} B LBAs)",
                        ns.name()?,
                        ns.nsid(),
                        lba_size
                    );

                    // ---- Write one LBA with a recognizable pattern -----
                    let mut pattern = vec![0u8; lba_size as usize];
                    for (i, b) in pattern.iter_mut().enumerate() {
                        *b = (i & 0xFF) as u8;
                    }
                    ns.write(0, 1, &pattern).fua().execute()?;
                    println!("  write ok");

                    // ---- Read it back ---------------------------------
                    let got = ns.read_to_vec(0, 1)?;
                    assert_eq!(got, pattern, "read did not return the bytes we wrote");
                    println!("  read ok (round-trip matches)");

                    // ---- Compare (should succeed) ---------------------
                    ns.compare(0, 1, &pattern).execute()?;
                    println!("  compare ok");

                    // ---- Verify (controller-side integrity check) -----
                    ns.verify(0, 1).execute()?;
                    println!("  verify ok");

                    // ---- Write zeroes over the LBA --------------------
                    ns.write_zeroes(0, 1).execute()?;
                    let zeroed = ns.read_to_vec(0, 1)?;
                    assert!(zeroed.iter().all(|&b| b == 0), "expected all-zero LBA");
                    println!("  write_zeroes ok");

                    // ---- DSM deallocate (TRIM) ------------------------
                    ns.dsm(DsmAttr::DEALLOCATE)
                        .ranges(&[DsmRange::new(0, 1)])
                        .execute()?;
                    println!("  dsm(deallocate) ok");

                    // ---- Flush ----------------------------------------
                    ns.flush()?;
                    println!("  flush ok");

                    exercised += 1;
                }
            }
        }
    }

    if exercised == 0 {
        println!("(no QEMU virtual NVMe controllers found)");
    } else {
        println!("\nall I/O commands exercised on {exercised} namespace(s)");
    }
    Ok(())
}
