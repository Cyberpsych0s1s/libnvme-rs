//! Typed parameters for NVMe admin commands.
//!
//! Each enum corresponds to one of libnvme's `nvme_cmd_*` typedefs (which
//! bindgen exposes as `c_uint` aliases). Using these instead of raw integers
//! at the API surface catches typos at compile time and makes call sites
//! self-documenting.

use libnvme_sys::{
    nvme_cmd_format_mset, nvme_cmd_format_pi, nvme_cmd_format_pil, nvme_cmd_format_ses,
    nvme_fw_commit_ca, nvme_get_features_sel,
};

/// How user data should be erased during a Format NVM operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum SecureErase {
    /// No secure erase — only metadata is overwritten.
    #[default]
    None = 0,
    /// User-data block-by-block erase. Time scales with namespace size.
    UserData = 1,
    /// Cryptographic erase — destroys the media-encryption key.
    /// Effectively instantaneous regardless of capacity.
    Cryptographic = 2,
}

impl SecureErase {
    pub(crate) fn as_raw(self) -> nvme_cmd_format_ses {
        self as u8 as nvme_cmd_format_ses
    }
}

/// End-to-end Data Protection type to apply when formatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ProtectionInfo {
    /// PI disabled (the common case for consumer SSDs).
    #[default]
    Disabled = 0,
    /// PI Type 1 (Guard tag, Application tag, Reference tag).
    Type1 = 1,
    /// PI Type 2.
    Type2 = 2,
    /// PI Type 3.
    Type3 = 3,
}

impl ProtectionInfo {
    pub(crate) fn as_raw(self) -> nvme_cmd_format_pi {
        self as u8 as nvme_cmd_format_pi
    }
}

/// Where the PI guard bytes sit within each LBA.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ProtectionLocation {
    /// PI in the last 8 bytes of metadata (the common choice).
    #[default]
    Last = 0,
    /// PI in the first 8 bytes of metadata.
    First = 1,
}

impl ProtectionLocation {
    pub(crate) fn as_raw(self) -> nvme_cmd_format_pil {
        self as u8 as nvme_cmd_format_pil
    }
}

/// How metadata is transferred relative to LBA data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum MetadataSettings {
    /// Metadata passed through a separate buffer.
    #[default]
    Separate = 0,
    /// Metadata interleaved with LBA data (extended LBA format).
    Extended = 1,
}

impl MetadataSettings {
    pub(crate) fn as_raw(self) -> nvme_cmd_format_mset {
        self as u8 as nvme_cmd_format_mset
    }
}

/// What the firmware-commit admin command should do with the slot it targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FirmwareAction {
    /// Replace the image in the slot. The image takes effect on the next
    /// controller reset.
    Replace = 0,
    /// Replace the image and mark this slot to be activated on next reset.
    ReplaceAndActivate = 1,
    /// Mark an already-loaded slot as the one to activate on next reset.
    SetActive = 2,
    /// Replace and activate the image *immediately*, with no reset.
    /// Only supported on controllers that advertise that capability in OACS.
    ReplaceAndActivateImmediate = 3,
    /// Replace a boot-partition image.
    ReplaceBootPartition = 6,
    /// Activate a boot-partition image already in a slot.
    ActivateBootPartition = 7,
}

impl FirmwareAction {
    pub(crate) fn as_raw(self) -> nvme_fw_commit_ca {
        self as u8 as nvme_fw_commit_ca
    }
}

/// Which "view" of a feature value the Get Features command should return.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum FeatureSelect {
    /// The currently-active value for the feature.
    #[default]
    Current = 0,
    /// The default value the controller ships with.
    Default = 1,
    /// The most recently saved value (if the feature supports save).
    Saved = 2,
    /// The set of supported capabilities for the feature, encoded in the
    /// result dword. See the NVMe spec for the per-feature encoding.
    Supported = 3,
}

impl FeatureSelect {
    pub(crate) fn as_raw(self) -> nvme_get_features_sel {
        self as u8 as nvme_get_features_sel
    }
}
