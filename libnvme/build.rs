//! Symbol probe over the installed libnvme headers.
//!
//! Rather than tying a feature to a libnvme version number (which requires
//! knowing when each function was added), we read the headers and check for
//! the symbol's textual presence. This is more robust against distro
//! backports and avoids a maintenance burden of version mappings.

use std::fs;
use std::path::Path;

/// Each entry is `(cfg_flag, symbol_to_probe_for)`.
///
/// When `<symbol_to_probe_for>` appears in the libnvme headers, the
/// `<cfg_flag>` is emitted, which gates the corresponding method in source.
/// Add a new row when v0.x work needs a function not present in older releases.
const PROBES: &[(&str, &str)] = &[
    ("has_subsystem_serial", "nvme_subsystem_get_serial"),
    ("has_subsystem_model", "nvme_subsystem_get_model"),
    ("has_subsystem_fw_rev", "nvme_subsystem_get_fw_rev"),
    ("has_subsystem_iopolicy", "nvme_subsystem_get_iopolicy"),
    (
        "has_subsystem_application",
        "nvme_subsystem_get_application",
    ),
    ("has_path_numa_nodes", "nvme_path_get_numa_nodes"),
    ("has_path_queue_depth", "nvme_path_get_queue_depth"),
];

fn main() {
    let libnvme = pkg_config::Config::new()
        .atleast_version("1.6")
        .probe("libnvme")
        .expect("libnvme not found via pkg-config (install libnvme-dev >= 1.6)");

    let headers = collect_header_text(&libnvme.include_paths);

    for (cfg_name, symbol) in PROBES {
        println!("cargo::rustc-check-cfg=cfg({cfg_name})");
        if headers.contains(symbol) {
            println!("cargo::rustc-cfg={cfg_name}");
        }
    }

    println!("cargo::rerun-if-changed=build.rs");
}

fn collect_header_text(include_paths: &[std::path::PathBuf]) -> String {
    let mut text = String::new();
    for include in include_paths {
        append_if_exists(&mut text, &include.join("libnvme.h"));
        let nvme_dir = include.join("nvme");
        if nvme_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&nvme_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "h") {
                        append_if_exists(&mut text, &path);
                    }
                }
            }
        }
    }
    text
}

fn append_if_exists(buf: &mut String, path: &Path) {
    if let Ok(s) = fs::read_to_string(path) {
        buf.push_str(&s);
    }
}
