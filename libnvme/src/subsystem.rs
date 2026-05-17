use std::marker::PhantomData;

#[cfg(has_subsystem_serial)]
use libnvme_sys::nvme_subsystem_get_serial;
use libnvme_sys::{
    nvme_first_subsystem, nvme_host_t, nvme_next_subsystem, nvme_subsystem_get_name,
    nvme_subsystem_get_nqn, nvme_subsystem_get_type, nvme_subsystem_t,
};

use crate::controller::Controllers;
use crate::util::cstr_to_str;
use crate::{Result, Root};

/// An NVMe subsystem.
///
/// A subsystem groups one or more controllers that share a common identity
/// (NQN, serial, model). For typical single-controller PCIe SSDs the subsystem
/// has exactly one controller; Fabrics and multipath setups can have several.
pub struct Subsystem<'r> {
    inner: nvme_subsystem_t,
    _marker: PhantomData<&'r Root>,
}

impl<'r> Subsystem<'r> {
    pub(crate) fn from_raw(inner: nvme_subsystem_t) -> Self {
        Subsystem {
            inner,
            _marker: PhantomData,
        }
    }

    /// The kernel-assigned subsystem name, e.g. `nvme-subsys0`.
    pub fn name(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_subsystem_get_name(self.inner)) }
    }

    /// The subsystem NVMe Qualified Name (NQN).
    pub fn nqn(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_subsystem_get_nqn(self.inner)) }
    }

    /// The subsystem type (e.g. `nvm`, `discovery`).
    pub fn subsystem_type(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_subsystem_get_type(self.inner)) }
    }

    /// The subsystem-level serial number, when reported.
    ///
    /// Only present when built against a libnvme version that exposes
    /// `nvme_subsystem_get_serial`. Older releases (notably the libnvme 1.8
    /// shipped in Ubuntu 24.04) do not have this symbol; on those builds the
    /// method is compiled out.
    #[cfg(has_subsystem_serial)]
    pub fn serial(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_subsystem_get_serial(self.inner)) }
    }

    /// Iterate over the controllers in this subsystem.
    pub fn controllers(&self) -> Controllers<'r> {
        Controllers::new(self.inner)
    }
}

impl std::fmt::Debug for Subsystem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subsystem")
            .field("name", &self.name().ok())
            .field("nqn", &self.nqn().ok())
            .finish()
    }
}

/// Iterator over [`Subsystem`] entries, returned by [`crate::Host::subsystems`].
pub struct Subsystems<'r> {
    host: nvme_host_t,
    cursor: nvme_subsystem_t,
    _marker: PhantomData<&'r Root>,
}

impl<'r> Subsystems<'r> {
    pub(crate) fn new(host: nvme_host_t) -> Self {
        let cursor = unsafe { nvme_first_subsystem(host) };
        Subsystems {
            host,
            cursor,
            _marker: PhantomData,
        }
    }
}

impl<'r> Iterator for Subsystems<'r> {
    type Item = Subsystem<'r>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor.is_null() {
            return None;
        }
        let current = self.cursor;
        self.cursor = unsafe { nvme_next_subsystem(self.host, current) };
        Some(Subsystem::from_raw(current))
    }
}
