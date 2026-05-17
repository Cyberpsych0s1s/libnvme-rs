use std::io;
use std::marker::PhantomData;
use std::ptr::NonNull;

use libnvme_sys::{nvme_free_tree, nvme_root, nvme_root_t, nvme_scan};

use crate::host::Hosts;
use crate::Result;

/// The owning handle to the libnvme tree.
///
/// Created by [`Root::scan`], which calls `nvme_scan` to enumerate hosts,
/// subsystems, controllers, and namespaces on the system. All other handle
/// types ([`Host`], [`Subsystem`], [`Controller`], [`Namespace`]) borrow from
/// the `Root` via the `'r` lifetime, so dropping the `Root` cascades-frees
/// the entire tree via `nvme_free_tree`.
///
/// `Root` is `!Send` and `!Sync`. libnvme's scan state isn't documented as
/// thread-safe; this restriction may be relaxed in a future release after
/// an audit.
///
/// [`Host`]: crate::Host
/// [`Subsystem`]: crate::Subsystem
/// [`Controller`]: crate::Controller
/// [`Namespace`]: crate::Namespace
pub struct Root {
    inner: NonNull<nvme_root>,
    _not_send_sync: PhantomData<*const ()>,
}

impl Root {
    /// Scan the system and build a libnvme handle tree.
    ///
    /// Equivalent to `nvme_scan(NULL)` — uses the default config-file lookup.
    /// Returns the platform's last-set `errno` (via [`std::io::Error`]) if
    /// libnvme returns NULL.
    pub fn scan() -> Result<Self> {
        let raw = unsafe { nvme_scan(std::ptr::null()) };
        let inner = NonNull::new(raw).ok_or_else(io::Error::last_os_error)?;
        Ok(Root {
            inner,
            _not_send_sync: PhantomData,
        })
    }

    /// Iterate over the hosts in this tree.
    ///
    /// libnvme typically reports a single host (the local machine) unless
    /// the caller has configured Fabrics with multiple hostnqn entries.
    pub fn hosts(&self) -> Hosts<'_> {
        Hosts::new(self.inner.as_ptr())
    }

    #[allow(dead_code)]
    pub(crate) fn as_ptr(&self) -> nvme_root_t {
        self.inner.as_ptr()
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        unsafe { nvme_free_tree(self.inner.as_ptr()) };
    }
}

impl std::fmt::Debug for Root {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Root")
            .field("inner", &self.inner.as_ptr())
            .finish()
    }
}
