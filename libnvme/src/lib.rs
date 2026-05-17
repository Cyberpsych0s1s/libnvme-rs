//! Safe, idiomatic Rust bindings for the Linux `libnvme` C library.
//!
//! `libnvme` is the userspace library that backs `nvme-cli`. This crate exposes
//! a memory-safe wrapper over its handle tree:
//!
//! ```text
//! Root → Host → Subsystem → Controller → Namespace
//! ```
//!
//! Every handle borrows from the [`Root`] via the `'r` lifetime, so dropping
//! the [`Root`] cascades-frees the entire tree.
//!
//! # Example
//!
//! ```no_run
//! use libnvme::Root;
//!
//! let root = Root::scan()?;
//! for host in root.hosts() {
//!     for subsys in host.subsystems() {
//!         for ctrl in subsys.controllers() {
//!             println!("{} {}", ctrl.name()?, ctrl.model()?);
//!             let id = ctrl.identify()?;
//!             println!("  NVMe spec: {}", id.nvme_version());
//!             for ns in ctrl.namespaces() {
//!                 println!("  {} ({} bytes)", ns.name()?, ns.size_bytes());
//!             }
//!         }
//!     }
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod controller;
mod error;
mod host;
mod identify;
mod log;
mod namespace;
mod path;
mod root;
mod subsystem;
mod util;

pub use controller::{Controller, Controllers};
pub use error::{Error, Result};
pub use host::{Host, Hosts};
pub use identify::{IdentifyController, IdentifyNamespace, LbaFormat, NvmeVersion};
pub use log::{ErrorLogEntry, FirmwareSlotLog, SmartLog};
pub use namespace::{Namespace, Namespaces};
pub use path::{Path, Paths};
pub use root::Root;
pub use subsystem::{Subsystem, Subsystems};
