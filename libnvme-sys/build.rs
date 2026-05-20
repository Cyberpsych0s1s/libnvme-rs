use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=DOCS_RS");

    // docs.rs builds in a sandbox without libnvme-dev installed. Detect via
    // the DOCS_RS env var and use the vendored headers in this case; skip
    // pkg-config (and therefore the library-link directive) so rustdoc
    // doesn't need the runtime library either.
    let docs_rs = env::var_os("DOCS_RS").is_some();

    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .derive_debug(true)
        .derive_default(true)
        .layout_tests(false)
        .generate_comments(true)
        .prepend_enum_name(false)
        .allowlist_function("nvme_.*")
        .allowlist_function("nvmf_.*")
        .allowlist_type("nvme_.*")
        .allowlist_type("nvmf_.*")
        .allowlist_type("__[blu]e?\\d+")
        .allowlist_var("NVME_.*")
        .allowlist_var("nvme_.*")
        .allowlist_var("NVMF_.*")
        .allowlist_var("nvmf_.*");

    // Always expose the vendored-headers path as cargo metadata so the
    // sibling `libnvme` crate's build script can find them. Because this
    // crate declares `links = "nvme"` in Cargo.toml, downstream crates
    // see this as `DEP_NVME_VENDORED_HEADERS` in their build environment.
    let vendor = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("vendored-headers");
    println!("cargo::metadata=vendored_headers={}", vendor.display());

    if docs_rs {
        // Point bindgen at the vendored headers. Don't link.
        builder = builder.clang_arg(format!("-I{}", vendor.display()));
    } else {
        let libnvme = pkg_config::Config::new()
            .atleast_version("1.6")
            .probe("libnvme")
            .expect("libnvme not found via pkg-config (install libnvme-dev >= 1.6)");
        for path in &libnvme.include_paths {
            builder = builder.clang_arg(format!("-I{}", path.display()));
        }
    }

    let bindings = builder
        .generate()
        .expect("bindgen failed to generate libnvme bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("failed to write bindings.rs");
}
