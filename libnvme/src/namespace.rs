use std::marker::PhantomData;

use libnvme_sys::{
    nvme_ctrl_first_ns, nvme_ctrl_next_ns, nvme_ctrl_t, nvme_format_nvm, nvme_format_nvm_args,
    nvme_id_ns, nvme_ns_get_csi, nvme_ns_get_eui64, nvme_ns_get_firmware, nvme_ns_get_generic_name,
    nvme_ns_get_lba_count, nvme_ns_get_lba_size, nvme_ns_get_lba_util, nvme_ns_get_meta_size,
    nvme_ns_get_model, nvme_ns_get_name, nvme_ns_get_nguid, nvme_ns_get_nsid, nvme_ns_get_serial,
    nvme_ns_get_uuid, nvme_ns_identify, nvme_ns_t,
};

use crate::admin::{MetadataSettings, ProtectionInfo, ProtectionLocation, SecureErase};
use crate::error::check_ret;
use crate::identify::IdentifyNamespace;
use crate::path::Paths;
use crate::util::cstr_to_str;
use crate::{Result, Root};

/// An NVMe namespace.
///
/// Maps to a `/dev/nvmeXnY` block device. A namespace is an addressable
/// region of logical blocks, with its own LBA format, identifier(s), and size.
pub struct Namespace<'r> {
    inner: nvme_ns_t,
    _marker: PhantomData<&'r Root>,
}

impl<'r> Namespace<'r> {
    pub(crate) fn from_raw(inner: nvme_ns_t) -> Self {
        Namespace {
            inner,
            _marker: PhantomData,
        }
    }

    /// Kernel-assigned namespace name, e.g. `nvme0n1`.
    pub fn name(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_ns_get_name(self.inner)) }
    }

    /// Generic-namespace name, e.g. `ng0n1`. The generic device exposes the
    /// namespace via `/dev/ng*` for passthrough I/O.
    pub fn generic_name(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_ns_get_generic_name(self.inner)) }
    }

    /// Namespace identifier (1-based, unique within the controller).
    pub fn nsid(&self) -> u32 {
        (unsafe { nvme_ns_get_nsid(self.inner) }) as u32
    }

    /// Logical block size in bytes (typically 512 or 4096).
    pub fn lba_size(&self) -> u32 {
        (unsafe { nvme_ns_get_lba_size(self.inner) }) as u32
    }

    /// Metadata bytes per LBA, or `0` if metadata is not used in the active
    /// LBA format.
    pub fn meta_size(&self) -> u32 {
        (unsafe { nvme_ns_get_meta_size(self.inner) }) as u32
    }

    /// Total number of logical blocks in the namespace.
    pub fn lba_count(&self) -> u64 {
        unsafe { nvme_ns_get_lba_count(self.inner) }
    }

    /// Number of logical blocks actually allocated within the namespace
    /// (`nuse` in Identify Namespace).
    pub fn lba_utilization(&self) -> u64 {
        unsafe { nvme_ns_get_lba_util(self.inner) }
    }

    /// Total namespace size in bytes (`lba_count * lba_size`).
    pub fn size_bytes(&self) -> u64 {
        self.lba_count().saturating_mul(u64::from(self.lba_size()))
    }

    /// Command Set Identifier. `0` = NVM, `1` = Key-Value, `2` = Zoned.
    pub fn csi(&self) -> u8 {
        (unsafe { nvme_ns_get_csi(self.inner) }) as u8
    }

    /// Model string of the controller that owns this namespace
    /// (whitespace-trimmed). Convenience wrapper that avoids walking back up
    /// through `Subsystem` / `Controller`.
    pub fn model(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_ns_get_model(self.inner)) }
    }

    /// Serial number of the controller that owns this namespace
    /// (whitespace-trimmed).
    pub fn serial(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_ns_get_serial(self.inner)) }
    }

    /// Firmware revision of the controller that owns this namespace
    /// (whitespace-trimmed).
    pub fn firmware(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_ns_get_firmware(self.inner)) }
    }

    /// 128-bit namespace UUID, or all-zero if not reported.
    pub fn uuid(&self) -> [u8; 16] {
        let mut out = [0u8; 16];
        unsafe { nvme_ns_get_uuid(self.inner, out.as_mut_ptr()) };
        out
    }

    /// 128-bit Namespace Globally Unique Identifier (NGUID), or all-zero.
    pub fn nguid(&self) -> [u8; 16] {
        let ptr = unsafe { nvme_ns_get_nguid(self.inner) };
        if ptr.is_null() {
            return [0; 16];
        }
        let mut out = [0u8; 16];
        unsafe { std::ptr::copy_nonoverlapping(ptr, out.as_mut_ptr(), 16) };
        out
    }

    /// 64-bit IEEE Extended Unique Identifier (EUI-64), or all-zero.
    pub fn eui64(&self) -> [u8; 8] {
        let ptr = unsafe { nvme_ns_get_eui64(self.inner) };
        if ptr.is_null() {
            return [0; 8];
        }
        let mut out = [0u8; 8];
        unsafe { std::ptr::copy_nonoverlapping(ptr, out.as_mut_ptr(), 8) };
        out
    }

    /// Issue the Identify Namespace admin command and return the decoded
    /// data structure.
    pub fn identify(&self) -> Result<IdentifyNamespace> {
        let mut id = Box::new(nvme_id_ns::default());
        let ret = unsafe { nvme_ns_identify(self.inner, id.as_mut() as *mut _) };
        check_ret(ret)?;
        Ok(IdentifyNamespace { inner: id })
    }

    /// Iterate over the multipath paths through which this namespace is
    /// reachable. Empty on non-multipath setups.
    pub fn paths(&self) -> Paths<'r> {
        Paths::from_namespace(self.inner)
    }

    /// Begin building a Format NVM admin command for this namespace.
    ///
    /// **Destructive.** Format NVM erases all user data in the namespace and
    /// applies the configured LBA format, protection settings, and secure
    /// erase mode. The returned [`Format`] builder is inert until
    /// [`Format::execute`] is called.
    ///
    /// ```no_run
    /// # use libnvme::{Root, SecureErase};
    /// # let root = Root::scan()?;
    /// # let ns: libnvme::Namespace<'_> = todo!();
    /// ns.format()
    ///     .lba_format(0)
    ///     .secure_erase(SecureErase::Cryptographic)
    ///     .execute()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn format(&self) -> Format<'_, 'r> {
        Format::new(self)
    }
}

/// Builder for the Format NVM admin command.
///
/// Created via [`Namespace::format`]. All fields default to "no-op /
/// conservative" values (LBA format 0, no secure erase, PI disabled),
/// so calling [`Format::execute`] without further chaining performs a
/// metadata-only format.
pub struct Format<'a, 'r> {
    ns: &'a Namespace<'r>,
    lba_format: u8,
    secure_erase: SecureErase,
    protection_info: ProtectionInfo,
    protection_location: ProtectionLocation,
    metadata: MetadataSettings,
    lba_format_upper: u8,
    timeout_ms: u32,
}

impl<'a, 'r> Format<'a, 'r> {
    fn new(ns: &'a Namespace<'r>) -> Self {
        Format {
            ns,
            lba_format: 0,
            secure_erase: SecureErase::None,
            protection_info: ProtectionInfo::Disabled,
            protection_location: ProtectionLocation::Last,
            metadata: MetadataSettings::Separate,
            lba_format_upper: 0,
            timeout_ms: 0,
        }
    }

    /// Select the LBA format (an index into the array reported by
    /// Identify Namespace). The active format becomes the new namespace
    /// format after the command completes.
    pub fn lba_format(mut self, index: u8) -> Self {
        self.lba_format = index;
        self
    }

    /// For NVMe 2.0+ controllers that expose more than 16 LBA formats, the
    /// upper bits of the format index live here.
    pub fn lba_format_upper(mut self, upper: u8) -> Self {
        self.lba_format_upper = upper;
        self
    }

    /// Configure secure-erase behaviour.
    pub fn secure_erase(mut self, mode: SecureErase) -> Self {
        self.secure_erase = mode;
        self
    }

    /// Configure end-to-end data protection.
    pub fn protection_info(mut self, pi: ProtectionInfo) -> Self {
        self.protection_info = pi;
        self
    }

    /// Where PI guard bytes sit within metadata (only meaningful when
    /// `protection_info` is not [`ProtectionInfo::Disabled`]).
    pub fn protection_location(mut self, location: ProtectionLocation) -> Self {
        self.protection_location = location;
        self
    }

    /// Whether metadata is separate or interleaved with LBA data.
    pub fn metadata(mut self, settings: MetadataSettings) -> Self {
        self.metadata = settings;
        self
    }

    /// Per-command timeout in milliseconds. `0` (the default) means the
    /// libnvme/kernel default — usually plenty for any reasonable format,
    /// but on large drives with `SecureErase::UserData` this can be raised.
    pub fn timeout_ms(mut self, ms: u32) -> Self {
        self.timeout_ms = ms;
        self
    }

    /// Execute the Format NVM command. Blocks until the controller reports
    /// completion or returns an error.
    pub fn execute(self) -> Result<()> {
        let fd = unsafe { libnvme_sys::nvme_ns_get_fd(self.ns.inner) };
        if fd < 0 {
            return Err(crate::Error::Os(std::io::Error::last_os_error()));
        }
        let mut args = nvme_format_nvm_args {
            result: std::ptr::null_mut(),
            args_size: std::mem::size_of::<nvme_format_nvm_args>() as i32,
            fd,
            timeout: self.timeout_ms,
            nsid: self.ns.nsid(),
            mset: self.metadata.as_raw(),
            pi: self.protection_info.as_raw(),
            pil: self.protection_location.as_raw(),
            ses: self.secure_erase.as_raw(),
            lbaf: self.lba_format,
            rsvd1: [0; 7],
            lbafu: self.lba_format_upper,
        };
        let ret = unsafe { nvme_format_nvm(&mut args) };
        check_ret(ret)
    }
}

impl std::fmt::Debug for Namespace<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Namespace")
            .field("name", &self.name().ok())
            .field("nsid", &self.nsid())
            .field("size_bytes", &self.size_bytes())
            .field("lba_size", &self.lba_size())
            .finish()
    }
}

/// Iterator over [`Namespace`] entries reachable through a [`crate::Controller`].
pub struct Namespaces<'r> {
    ctrl: nvme_ctrl_t,
    cursor: nvme_ns_t,
    _marker: PhantomData<&'r Root>,
}

impl<'r> Namespaces<'r> {
    pub(crate) fn new(ctrl: nvme_ctrl_t) -> Self {
        let cursor = unsafe { nvme_ctrl_first_ns(ctrl) };
        Namespaces {
            ctrl,
            cursor,
            _marker: PhantomData,
        }
    }
}

impl<'r> Iterator for Namespaces<'r> {
    type Item = Namespace<'r>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor.is_null() {
            return None;
        }
        let current = self.cursor;
        self.cursor = unsafe { nvme_ctrl_next_ns(self.ctrl, current) };
        Some(Namespace::from_raw(current))
    }
}
