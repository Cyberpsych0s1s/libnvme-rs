use std::marker::PhantomData;

use libnvme_sys::{
    nvme_first_host, nvme_host_get_hostid, nvme_host_get_hostnqn, nvme_host_t, nvme_next_host,
    nvme_root_t,
};

use crate::subsystem::Subsystems;
use crate::util::cstr_to_str;
use crate::{Result, Root};

/// A host entry in the libnvme tree.
///
/// In the libnvme model, a "host" represents a local or fabrics identity
/// (NVMe Qualified Name and ID) under which subsystems are attached. Most
/// systems have exactly one. Borrows from the parent [`Root`] via `'r`.
pub struct Host<'r> {
    inner: nvme_host_t,
    _marker: PhantomData<&'r Root>,
}

impl<'r> Host<'r> {
    pub(crate) fn from_raw(inner: nvme_host_t) -> Self {
        Host {
            inner,
            _marker: PhantomData,
        }
    }

    /// The Host NVMe Qualified Name (HostNQN), e.g. `nqn.2014-08.org.nvmexpress:uuid:...`.
    pub fn hostnqn(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_host_get_hostnqn(self.inner)) }
    }

    /// The host identifier (HostID) as a UUID-formatted string.
    pub fn hostid(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_host_get_hostid(self.inner)) }
    }

    /// Iterate over the subsystems attached to this host.
    pub fn subsystems(&self) -> Subsystems<'r> {
        Subsystems::new(self.inner)
    }
}

impl std::fmt::Debug for Host<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Host")
            .field("hostnqn", &self.hostnqn().ok())
            .finish()
    }
}

/// Iterator over [`Host`] entries, returned by [`Root::hosts`].
pub struct Hosts<'r> {
    root: nvme_root_t,
    cursor: nvme_host_t,
    _marker: PhantomData<&'r Root>,
}

impl<'r> Hosts<'r> {
    pub(crate) fn new(root: nvme_root_t) -> Self {
        let cursor = unsafe { nvme_first_host(root) };
        Hosts {
            root,
            cursor,
            _marker: PhantomData,
        }
    }
}

impl<'r> Iterator for Hosts<'r> {
    type Item = Host<'r>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor.is_null() {
            return None;
        }
        let current = self.cursor;
        self.cursor = unsafe { nvme_next_host(self.root, current) };
        Some(Host::from_raw(current))
    }
}
