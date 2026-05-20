# Changelog

All notable changes to `libnvme-sys` and `libnvme` are recorded here. The format
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) with the
caveat that pre-1.0 minor-version bumps may include breaking changes.

## [0.6.1] – 2026-05-20

### Changed

- First crates.io publish. `Cargo.toml` metadata now includes `documentation`,
  `homepage`, and `rust-version` (1.85) fields. README leads with the example
  rather than the status section. Added this `CHANGELOG.md`. No source code
  changes versus 0.6.0.

## [0.6.0] – 2026-05-20

### Added

- `Controller::sanitize()` builder for Sanitize NVM (action, AUSE, pass count, overwrite invert, no-deallocate, pattern, NVMe 2.0 EMVS field — last is version-gated)
- `Controller::self_test(nsid, action)` for Device Self-Test
- `Controller::lockdown(args)` for NVMe 2.0+ Lockdown
- `Controller::security_send(...)` / `security_receive(...)` for security-protocol payloads
- `Controller::get_lba_status(args, buf)` for the LBA Status Descriptor admin command
- `Controller::set_property(offset, value)` / `get_property(offset)` for Fabrics controller registers
- `Controller::admin_passthru(args)` / `io_passthru(args)` — generic escape hatches
- Typed enums `SanitizeAction`, `SelfTestAction`
- New public types: `Sanitize` builder, `LockdownArgs`, `GetLbaStatusArgs`, `PassthruArgs`
- New struct-field probe pattern in `build.rs` (`has_sanitize_emvs`)

## [0.5.0] – 2026-05-19

### Added

- `Controller::features()` returning a `Features` accessor that wraps every
  per-feature Get/Set Features helper libnvme exposes — 69 typed methods total
  (32 set + 37 get)
- `FeatureSelect` enum (Current / Default / Saved / Supported)
- Re-exported feature-buffer types: `LbaRangeType`, `AutoPst`, `Timestamp`,
  `HostBehavior`, `PlmConfig`, `HostMemBufAttrs`
- Seven new `build.rs` probes for the NVMe 2.0 `*2` feature variants
  (`temp_thresh2`, `err_recovery2`, `lba_range2`, `host_mem_buf2`,
  `resv_mask2`, `resv_persist2`, `write_protect2`)

## [0.4.1] – 2026-05-19

### Fixed

- Build failure on libnvme 1.8 (Ubuntu 24.04): five Fabrics auth/TLS/discovery
  helpers (`set_dhchap_host_key`, `set_tls_key`, `set_tls_key_identity`,
  `set_keyring`, `set_unique_discovery_ctrl`/`is_unique_discovery_ctrl`) are
  newer than libnvme 1.8 and are now `#[cfg]`-gated
- Build failure on libnvme 1.8 for `nvmf_hostid_generate` /
  `nvmf_hostid_from_file` — also gated

## [0.4.0] – 2026-05-18

### Added

- `fabrics` module: `Transport` enum, `Connect` builder (14 chainable
  setters), `DiscoveryLog`, `DiscoveryLogEntry`
- `Root::default_host`, `Root::lookup_host`
- Free functions: `generate_hostnqn`, `generate_hostid`, `hostnqn_from_file`,
  `hostid_from_file`
- `Controller::disconnect` (consumes self), `reset`, `is_discovery_controller`,
  `was_discovered`, `is_unique_discovery_controller`, `is_persistent`,
  `set_persistent`, `discovery_log`, `set_dhchap_host_key`, `set_dhchap_key`,
  `set_tls_key`, `set_tls_key_identity`, `set_keyring`
- Bindgen allowlist widened in `libnvme-sys/build.rs` to include `nvmf_*`
  functions, types, and vars

## [0.3.0] – 2026-05-18

### Added

- `Controller::get_log_page<T>` — generic typed Get Log Page helper
- `Controller::fw_slot_log()` returning `FirmwareSlotLog`
- `Controller::error_log(max_entries)` returning `Vec<ErrorLogEntry>`
- `Path` and `Paths` multipath/ANA types; `Controller::paths()` and
  `Namespace::paths()` iterators
- `Namespace::format()` builder (LBA format, secure erase, protection info,
  metadata, timeout)
- `Controller::fw_download(image)` and `fw_commit(slot, action, bpid)`
- `Controller::create_namespace(template)`, `delete_namespace(nsid)`,
  `attach_namespace(nsid, ctrlids)`, `detach_namespace(...)`
- `admin` module with typed enums: `SecureErase`, `ProtectionInfo`,
  `ProtectionLocation`, `MetadataSettings`, `FirmwareAction`
- `examples/fw_info.rs` and `examples/format_smoke.rs` (the latter has a
  model-name safety latch — only formats when controller model is
  `"QEMU NVMe Ctrl"`)
- QEMU NVMe test fixture in `tests/qemu/`

## [0.2.1] – 2026-05-17

### Fixed

- `Controller::smart_log()` was masking `EACCES` as `EBADF` because the
  file descriptor returned by `nvme_ctrl_get_fd` (which opens the device
  lazily) wasn't checked before use. Fix introduces a `open_fd` helper
  that propagates the real `errno`.

### Added

- 9 new Controller sysfs accessors: `numa_node`, `queue_count`, `sq_size`,
  `phy_slot`, `subsystem_nqn`, `transport_address`, `transport_service_id`,
  `host_transport_address`, `host_interface`
- 7 new Namespace accessors: `generic_name`, `meta_size`, `lba_utilization`,
  `csi`, `model`, `serial`, `firmware`
- 4 version-gated Subsystem accessors: `model`, `firmware`, `iopolicy`,
  `application` (`build.rs` probes for the symbol presence)

## [0.2.0] – 2026-05-17

### Added

- `Controller::identify()` returning `IdentifyController` with ~25 decoded
  accessors over `nvme_id_ctrl`
- `Namespace::identify()` returning `IdentifyNamespace` with `LbaFormat` helper
- `Controller::smart_log()` returning `SmartLog` with 18 decoded SMART
  accessors
- `Error::Nvme(u32)` variant for device-reported NVMe status codes,
  `Error::Utf8` for invalid UTF-8 in libnvme-returned strings,
  `Error::NotAvailable` for NULL pointers
- `check_ret` helper mapping libnvme's `c_int` return convention (0 ok,
  negative = `-errno`, positive = NVMe status) to `Result`
- `util::fixed_ascii_to_str` for decoding NVMe spec ASCII fields
- Examples: `id_ctrl`, `smart_log`
- Symbol-probing `build.rs` for `has_subsystem_serial` (the version-gating
  scheme); see `tests/qemu/` for the test fixture

## [0.1.0] – 2026-05-17

### Added

- Initial release
- Tree iteration: `Host` → `Subsystem` → `Controller` → `Namespace`
- Controller properties: `name`, `model`, `serial`, `firmware`, `transport`,
  `address`, `state`
- Namespace properties: `name`, `nsid`, `lba_size`, `lba_count`, `size_bytes`,
  `uuid`, `nguid`, `eui64`
- Subsystem and Host basics (`nqn`, `hostid`, `type`)
- `Root::scan()` entry point, `Drop` cascading `nvme_free_tree`
- `Error` / `Result` types
- `examples/scan.rs` and `examples/list_nvme.rs`
- Dual-license (MIT or Apache-2.0)
- CI (Ubuntu 24.04, libnvme 1.8) running `cargo build`, `cargo test`,
  `cargo clippy`, `cargo fmt --check`

[0.6.1]: https://github.com/Cyberpsych0s1s/libnvme-rs/releases/tag/v0.6.1
[0.6.0]: https://github.com/Cyberpsych0s1s/libnvme-rs/releases/tag/v0.6.0
[0.5.0]: https://github.com/Cyberpsych0s1s/libnvme-rs/releases/tag/v0.5.0
[0.4.1]: https://github.com/Cyberpsych0s1s/libnvme-rs/releases/tag/v0.4.1
[0.4.0]: https://github.com/Cyberpsych0s1s/libnvme-rs/releases/tag/v0.4.0
[0.3.0]: https://github.com/Cyberpsych0s1s/libnvme-rs/releases/tag/v0.3.0
[0.2.1]: https://github.com/Cyberpsych0s1s/libnvme-rs/releases/tag/v0.2.1
[0.2.0]: https://github.com/Cyberpsych0s1s/libnvme-rs/releases/tag/v0.2.0
[0.1.0]: https://github.com/Cyberpsych0s1s/libnvme-rs/releases/tag/v0.1.0
