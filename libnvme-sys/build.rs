use std::env;
use std::path::PathBuf;

fn main() {
    let libnvme = pkg_config::Config::new()
        .atleast_version("1.6")
        .probe("libnvme")
        .expect("libnvme not found via pkg-config; install libnvme-dev (>= 1.6)");

    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=build.rs");

    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .derive_debug(true)
        .derive_default(true)
        .layout_tests(false)
        .generate_comments(true)
        .prepend_enum_name(false)
        .allowlist_function("nvme_.*")
        .allowlist_type("nvme_.*")
        .allowlist_type("__[blu]e?\\d+")
        .allowlist_var("NVME_.*")
        .allowlist_var("nvme_.*");

    for path in &libnvme.include_paths {
        builder = builder.clang_arg(format!("-I{}", path.display()));
    }

    let bindings = builder
        .generate()
        .expect("bindgen failed to generate libnvme bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("failed to write bindings.rs");
}
