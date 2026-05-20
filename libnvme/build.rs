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
    // Fabrics auth + TLS surfaces added after libnvme 1.8.
    ("has_dhchap_host_key", "nvme_ctrl_set_dhchap_host_key"),
    ("has_tls_key", "nvme_ctrl_set_tls_key"),
    ("has_tls_key_identity", "nvme_ctrl_set_tls_key_identity"),
    ("has_keyring", "nvme_ctrl_set_keyring"),
    (
        "has_unique_discovery_ctrl",
        "nvme_ctrl_set_unique_discovery_ctrl",
    ),
    ("has_hostid_generate", "nvmf_hostid_generate"),
    ("has_hostid_from_file", "nvmf_hostid_from_file"),
    // Feature "*2" variants — NVMe 2.0 extended forms, post-libnvme-1.8.
    ("has_temp_thresh2", "nvme_set_features_temp_thresh2"),
    ("has_err_recovery2", "nvme_get_features_err_recovery2"),
    ("has_lba_range2", "nvme_get_features_lba_range2"),
    ("has_host_mem_buf2", "nvme_get_features_host_mem_buf2"),
    ("has_resv_mask2", "nvme_set_features_resv_mask2"),
    ("has_resv_persist2", "nvme_set_features_resv_persist2"),
    ("has_write_protect2", "nvme_set_features_write_protect2"),
    // Struct-field probes (look for distinctive field names in the headers).
    // `nvme_sanitize_nvm_args::emvs` was added with NVMe 2.0's Emulated
    // Media Verify support; libnvme 1.8 lacks the field entirely.
    ("has_sanitize_emvs", "bool emvs"),
];

fn main() {
    // Same DOCS_RS dance as libnvme-sys/build.rs: when building on docs.rs
    // there's no system libnvme, so probe against the headers vendored in
    // the sibling -sys crate. Path is communicated via cargo metadata
    // (because libnvme-sys declares `links = "nvme"` it can export
    // `vendored_headers=...` and we read it from `DEP_NVME_VENDORED_HEADERS`).
    let docs_rs = std::env::var_os("DOCS_RS").is_some();
    let include_paths: Vec<std::path::PathBuf> = if docs_rs {
        let path = std::env::var("DEP_NVME_VENDORED_HEADERS").expect(
            "libnvme-sys did not export `vendored_headers` metadata; \
             ensure DEP_NVME_VENDORED_HEADERS is set when DOCS_RS is set",
        );
        vec![std::path::PathBuf::from(path)]
    } else {
        let libnvme = pkg_config::Config::new()
            .atleast_version("1.6")
            .probe("libnvme")
            .expect("libnvme not found via pkg-config (install libnvme-dev >= 1.6)");
        libnvme.include_paths
    };

    let headers = collect_header_text(&include_paths);

    for (cfg_name, symbol) in PROBES {
        println!("cargo::rustc-check-cfg=cfg({cfg_name})");
        if headers.contains(symbol) {
            println!("cargo::rustc-cfg={cfg_name}");
        }
    }

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=DOCS_RS");
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
