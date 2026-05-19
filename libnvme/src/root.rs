use std::ffi::CString;
use std::io;
use std::marker::PhantomData;
use std::ptr::NonNull;

#[cfg(has_hostid_from_file)]
use libnvme_sys::nvmf_hostid_from_file;
#[cfg(has_hostid_generate)]
use libnvme_sys::nvmf_hostid_generate;
use libnvme_sys::{
    nvme_default_host, nvme_free_tree, nvme_lookup_host, nvme_root, nvme_scan,
    nvmf_hostnqn_from_file, nvmf_hostnqn_generate,
};

use crate::host::{Host, Hosts};
use crate::{Error, Result};

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

    /// Get or create the default host for this root.
    ///
    /// libnvme uses `/etc/nvme/hostnqn` and `/etc/nvme/hostid` if present,
    /// otherwise generates new identifiers and persists them.
    pub fn default_host(&self) -> Result<Host<'_>> {
        let raw = unsafe { nvme_default_host(self.inner.as_ptr()) };
        if raw.is_null() {
            return Err(Error::Os(io::Error::last_os_error()));
        }
        Ok(Host::from_raw(raw, self.inner.as_ptr()))
    }

    /// Look up or create a host with the given NQN and (optional) HostID.
    ///
    /// Use this when you need a non-default host identity — for instance, a
    /// fabrics client that wants to present a specific HostNQN to a target.
    pub fn lookup_host(&self, hostnqn: &str, hostid: Option<&str>) -> Result<Host<'_>> {
        let hostnqn_c = CString::new(hostnqn).map_err(|_| {
            Error::Os(io::Error::new(
                io::ErrorKind::InvalidInput,
                "interior NUL byte in hostnqn",
            ))
        })?;
        let hostid_c = match hostid {
            Some(h) => Some(CString::new(h).map_err(|_| {
                Error::Os(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "interior NUL byte in hostid",
                ))
            })?),
            None => None,
        };
        let hostid_ptr = match &hostid_c {
            Some(c) => c.as_ptr(),
            None => std::ptr::null(),
        };
        let raw = unsafe { nvme_lookup_host(self.inner.as_ptr(), hostnqn_c.as_ptr(), hostid_ptr) };
        if raw.is_null() {
            return Err(Error::Os(io::Error::last_os_error()));
        }
        Ok(Host::from_raw(raw, self.inner.as_ptr()))
    }
}

/// Generate a fresh HostNQN (NVMe spec format, embeds a freshly-generated UUID).
/// Equivalent to libnvme's `nvmf_hostnqn_generate`. Returned string is owned.
pub fn generate_hostnqn() -> Result<String> {
    let raw = unsafe { nvmf_hostnqn_generate() };
    take_owned_cstr(raw)
}

/// Generate a fresh HostID (random UUID, formatted as a hex string).
///
/// Only present when built against a libnvme that exposes
/// `nvmf_hostid_generate` (added after libnvme 1.8).
#[cfg(has_hostid_generate)]
pub fn generate_hostid() -> Result<String> {
    let raw = unsafe { nvmf_hostid_generate() };
    take_owned_cstr(raw)
}

/// Read the local HostNQN from `/etc/nvme/hostnqn` if it exists.
pub fn hostnqn_from_file() -> Result<String> {
    let raw = unsafe { nvmf_hostnqn_from_file() };
    take_owned_cstr(raw)
}

/// Read the local HostID from `/etc/nvme/hostid` if it exists.
///
/// Only present when built against a libnvme that exposes
/// `nvmf_hostid_from_file` (added after libnvme 1.8).
#[cfg(has_hostid_from_file)]
pub fn hostid_from_file() -> Result<String> {
    let raw = unsafe { nvmf_hostid_from_file() };
    take_owned_cstr(raw)
}

/// Take ownership of a libnvme-allocated `*mut c_char`, copy it into an owned
/// `String`, and free the original via libc's `free`.
fn take_owned_cstr(ptr: *mut std::os::raw::c_char) -> Result<String> {
    if ptr.is_null() {
        return Err(Error::NotAvailable);
    }
    let owned = unsafe { std::ffi::CStr::from_ptr(ptr) }
        .to_str()
        .map_err(Error::Utf8)?
        .to_owned();
    unsafe { libc_free(ptr as *mut _) };
    Ok(owned)
}

unsafe extern "C" {
    #[link_name = "free"]
    fn libc_free(ptr: *mut std::ffi::c_void);
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
