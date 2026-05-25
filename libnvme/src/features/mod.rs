//! Get/Set Features admin commands.
//!
//! Wraps libnvme's per-feature typed helpers — one method per feature ID,
//! matching the libnvme function name with the `nvme_set_features_` /
//! `nvme_get_features_` prefix stripped.
//!
//! For each NVMe feature, libnvme exposes:
//! - `set_<feature>` — write the feature value, optionally `save` it across resets
//! - `get_<feature>` — read the feature value with a [`FeatureSelect`] view
//!
//! All methods are exposed through [`Controller::features`].
//!
//! For features whose value doesn't fit in a single dword (LBA range type,
//! auto-PST, host memory buffer attributes, etc.) the relevant libnvme
//! struct is re-exported as a typed alias on this module.

mod get;
mod set;
mod types;

pub use types::{AutoPst, HostBehavior, HostMemBufAttrs, LbaRangeType, PlmConfig, Timestamp};

use crate::{Controller, Result};

/// Get/Set Features accessor for a controller.
///
/// Created by [`Controller::features`]. Borrows the controller; all methods
/// require the controller's `/dev/nvmeN` to be openable (root or `disk`
/// group on most distros).
pub struct Features<'a, 'r> {
    pub(crate) ctrl: &'a Controller<'r>,
}

impl Features<'_, '_> {
    pub(crate) fn fd(&self) -> Result<std::os::raw::c_int> {
        self.ctrl.open_fd()
    }
}
