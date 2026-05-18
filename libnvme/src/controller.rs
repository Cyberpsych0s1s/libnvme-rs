use std::io;
use std::marker::PhantomData;

#[cfg(has_unique_discovery_ctrl)]
use libnvme_sys::nvme_ctrl_is_unique_discovery_ctrl;
#[cfg(has_dhchap_host_key)]
use libnvme_sys::nvme_ctrl_set_dhchap_host_key;
#[cfg(has_keyring)]
use libnvme_sys::nvme_ctrl_set_keyring;
#[cfg(has_tls_key)]
use libnvme_sys::nvme_ctrl_set_tls_key;
#[cfg(has_tls_key_identity)]
use libnvme_sys::nvme_ctrl_set_tls_key_identity;
use libnvme_sys::{
    nvme_cmd_get_log_lid, nvme_ctrl_get_address, nvme_ctrl_get_fd, nvme_ctrl_get_firmware,
    nvme_ctrl_get_host_iface, nvme_ctrl_get_host_traddr, nvme_ctrl_get_model, nvme_ctrl_get_name,
    nvme_ctrl_get_numa_node, nvme_ctrl_get_phy_slot, nvme_ctrl_get_queue_count,
    nvme_ctrl_get_serial, nvme_ctrl_get_sqsize, nvme_ctrl_get_state, nvme_ctrl_get_subsysnqn,
    nvme_ctrl_get_traddr, nvme_ctrl_get_transport, nvme_ctrl_get_trsvcid, nvme_ctrl_identify,
    nvme_ctrl_is_discovered, nvme_ctrl_is_discovery_ctrl, nvme_ctrl_is_persistent, nvme_ctrl_list,
    nvme_ctrl_reset, nvme_ctrl_set_dhchap_key, nvme_ctrl_set_persistent, nvme_ctrl_t,
    nvme_disconnect_ctrl, nvme_error_log_page, nvme_firmware_slot, nvme_fw_commit,
    nvme_fw_commit_args, nvme_fw_download_seq, nvme_get_log, nvme_get_log_args, nvme_id_ctrl,
    nvme_id_ns, nvme_ns_attach, nvme_ns_attach_args, nvme_ns_mgmt, nvme_ns_mgmt_args,
    nvme_smart_log, nvme_subsystem_first_ctrl, nvme_subsystem_next_ctrl, nvme_subsystem_t,
    NVME_LOG_LID_ERROR, NVME_LOG_LID_FW_SLOT, NVME_LOG_LID_SMART, NVME_NS_ATTACH_SEL_CTRL_ATTACH,
    NVME_NS_ATTACH_SEL_CTRL_DEATTACH, NVME_NS_MGMT_SEL_CREATE, NVME_NS_MGMT_SEL_DELETE,
};

use crate::admin::FirmwareAction;
use crate::error::check_ret;
use crate::fabrics::{fetch_discovery_log, DiscoveryLog};
use crate::identify::{IdentifyController, IdentifyNamespace};
use crate::log::{ErrorLogEntry, FirmwareSlotLog, SmartLog};
use crate::namespace::Namespaces;
use crate::path::Paths;
use crate::util::cstr_to_str;
use crate::{Error, Result, Root};

/// An NVMe controller.
///
/// Maps to a `/dev/nvmeN` character device. Controllers expose identity
/// (model, serial, firmware) and host one or more namespaces.
pub struct Controller<'r> {
    inner: nvme_ctrl_t,
    _marker: PhantomData<&'r Root>,
}

impl<'r> Controller<'r> {
    pub(crate) fn from_raw(inner: nvme_ctrl_t) -> Self {
        Controller {
            inner,
            _marker: PhantomData,
        }
    }

    /// Kernel-assigned controller name, e.g. `nvme0`.
    pub fn name(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_ctrl_get_name(self.inner)) }
    }

    /// Controller model string from Identify Controller (whitespace-trimmed).
    pub fn model(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_ctrl_get_model(self.inner)) }
    }

    /// Controller serial number from Identify Controller (whitespace-trimmed).
    pub fn serial(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_ctrl_get_serial(self.inner)) }
    }

    /// Firmware revision string from Identify Controller (whitespace-trimmed).
    pub fn firmware(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_ctrl_get_firmware(self.inner)) }
    }

    /// Transport type: `pcie`, `tcp`, `rdma`, `fc`, or `loop`.
    pub fn transport(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_ctrl_get_transport(self.inner)) }
    }

    /// Composite transport address (e.g. PCIe BDF or Fabrics traddr/trsvcid pair).
    pub fn address(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_ctrl_get_address(self.inner)) }
    }

    /// Controller state as reported by the kernel: `live`, `resetting`, etc.
    pub fn state(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_ctrl_get_state(self.inner)) }
    }

    /// NUMA node the controller is attached to, as a string read from sysfs.
    /// Typically `"-1"` on single-socket or non-NUMA systems.
    pub fn numa_node(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_ctrl_get_numa_node(self.inner)) }
    }

    /// Number of I/O queues, as a string read from sysfs.
    pub fn queue_count(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_ctrl_get_queue_count(self.inner)) }
    }

    /// Submission Queue size, as a string read from sysfs.
    pub fn sq_size(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_ctrl_get_sqsize(self.inner)) }
    }

    /// Physical PCIe slot identifier, as a string read from sysfs.
    pub fn phy_slot(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_ctrl_get_phy_slot(self.inner)) }
    }

    /// The parent subsystem's NQN, read from sysfs without an admin command.
    pub fn subsystem_nqn(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_ctrl_get_subsysnqn(self.inner)) }
    }

    /// Transport target address (Fabrics `traddr`). For PCIe controllers this
    /// is typically empty or the BDF, depending on libnvme version.
    pub fn transport_address(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_ctrl_get_traddr(self.inner)) }
    }

    /// Transport service identifier (Fabrics `trsvcid`, e.g. port number).
    pub fn transport_service_id(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_ctrl_get_trsvcid(self.inner)) }
    }

    /// Host-side transport address (Fabrics `host_traddr`).
    pub fn host_transport_address(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_ctrl_get_host_traddr(self.inner)) }
    }

    /// Host network interface used by this controller (Fabrics `host_iface`).
    pub fn host_interface(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_ctrl_get_host_iface(self.inner)) }
    }

    /// Issue the Identify Controller admin command and return the decoded
    /// data structure.
    ///
    /// Requires an open file descriptor on the controller device, which may
    /// require elevated privileges (root, or membership in the `disk` group
    /// on most distributions).
    pub fn identify(&self) -> Result<IdentifyController> {
        let mut id = Box::new(nvme_id_ctrl::default());
        let ret = unsafe { nvme_ctrl_identify(self.inner, id.as_mut() as *mut _) };
        check_ret(ret)?;
        Ok(IdentifyController { inner: id })
    }

    /// Open the controller device and return its file descriptor.
    ///
    /// libnvme opens the device lazily on first use; if that open fails
    /// (e.g. `EACCES` without root) it returns `-1` with `errno` set. This
    /// helper translates that into [`Error::Os`] so callers don't have to
    /// repeat the check.
    fn open_fd(&self) -> Result<std::os::raw::c_int> {
        let fd = unsafe { nvme_ctrl_get_fd(self.inner) };
        if fd < 0 {
            Err(Error::Os(io::Error::last_os_error()))
        } else {
            Ok(fd)
        }
    }

    /// Fetch a fixed-size log page typed as `T`.
    ///
    /// Generic over the libnvme struct layout for the page (e.g.
    /// [`libnvme_sys::nvme_smart_log`]). The struct must implement [`Default`]
    /// (all libnvme log-page structs do).
    ///
    /// For variable-length log pages (Error Information, Persistent Event)
    /// use the page-specific helpers like [`Self::error_log`].
    ///
    /// `nsid` is `0xFFFFFFFF` for controller-wide pages, otherwise the target
    /// namespace identifier.
    pub fn get_log_page<T: Default>(&self, lid: u8, nsid: u32) -> Result<Box<T>> {
        let fd = self.open_fd()?;
        let mut buf: Box<T> = Box::default();
        let mut args = nvme_get_log_args {
            args_size: std::mem::size_of::<nvme_get_log_args>() as i32,
            fd,
            lid: lid as nvme_cmd_get_log_lid,
            nsid,
            log: buf.as_mut() as *mut _ as *mut std::ffi::c_void,
            len: std::mem::size_of::<T>() as u32,
            ..Default::default()
        };
        let ret = unsafe { nvme_get_log(&mut args) };
        check_ret(ret)?;
        Ok(buf)
    }

    /// Fetch the SMART / Health Information log page (LID 02h), aggregated
    /// across all namespaces.
    pub fn smart_log(&self) -> Result<SmartLog> {
        let inner = self.get_log_page::<nvme_smart_log>(NVME_LOG_LID_SMART as u8, 0xFFFF_FFFF)?;
        Ok(SmartLog { inner })
    }

    /// Fetch the Firmware Slot Information log page (LID 03h).
    pub fn fw_slot_log(&self) -> Result<FirmwareSlotLog> {
        let inner =
            self.get_log_page::<nvme_firmware_slot>(NVME_LOG_LID_FW_SLOT as u8, 0xFFFF_FFFF)?;
        Ok(FirmwareSlotLog { inner })
    }

    /// Fetch up to `max_entries` entries from the Error Information log page
    /// (LID 01h). Entries are returned newest-first; unused slots have
    /// `error_count == 0`.
    ///
    /// Use [`IdentifyController::error_log_page_entries`](crate::IdentifyController::error_log_page_entries)
    /// to discover how many entries the controller supports.
    pub fn error_log(&self, max_entries: u32) -> Result<Vec<ErrorLogEntry>> {
        if max_entries == 0 {
            return Ok(Vec::new());
        }
        let fd = self.open_fd()?;
        let entry_size = std::mem::size_of::<nvme_error_log_page>();
        let total_len = entry_size
            .checked_mul(max_entries as usize)
            .ok_or(Error::NotAvailable)?;

        let mut entries: Vec<nvme_error_log_page> =
            vec![nvme_error_log_page::default(); max_entries as usize];
        let mut args = nvme_get_log_args {
            args_size: std::mem::size_of::<nvme_get_log_args>() as i32,
            fd,
            lid: NVME_LOG_LID_ERROR,
            nsid: 0xFFFF_FFFF,
            log: entries.as_mut_ptr() as *mut std::ffi::c_void,
            len: total_len as u32,
            ..Default::default()
        };
        let ret = unsafe { nvme_get_log(&mut args) };
        check_ret(ret)?;

        Ok(entries
            .into_iter()
            .map(|inner| ErrorLogEntry { inner })
            .collect())
    }

    /// Iterate over namespaces accessible through this controller.
    pub fn namespaces(&self) -> Namespaces<'r> {
        Namespaces::new(self.inner)
    }

    /// Iterate over the multipath paths reachable through this controller.
    /// Empty on non-multipath setups (most consumer PCIe SSDs).
    pub fn paths(&self) -> Paths<'r> {
        Paths::from_controller(self.inner)
    }

    /// Download a firmware image to the controller without activating it.
    ///
    /// **Destructive.** The image is written into the controller's transfer
    /// buffer; a subsequent [`Self::fw_commit`] call selects which slot
    /// receives the image and when it becomes active. Sending a malformed
    /// or wrong-vendor firmware here followed by a Commit can brick the
    /// controller.
    ///
    /// libnvme handles chunking internally (`nvme_fw_download_seq`); the
    /// caller passes the full image as a single byte slice.
    pub fn fw_download(&self, image: &[u8]) -> Result<()> {
        let fd = self.open_fd()?;
        // 0 = transfer size from controller's reported `fwug` field
        let xfer = 0;
        let ret = unsafe {
            nvme_fw_download_seq(
                fd,
                image.len() as u32,
                xfer,
                0,
                image.as_ptr() as *mut std::ffi::c_void,
            )
        };
        check_ret(ret)
    }

    /// Commit a previously-downloaded firmware image to a slot and/or
    /// activate it.
    ///
    /// **Destructive.** See [`FirmwareAction`] for the semantic of each
    /// commit action. Slot indices are `1..=7`. `bpid` selects boot
    /// partition `1` (`false`) or `2` (`true`); ignored for non-boot-partition
    /// actions.
    pub fn fw_commit(&self, slot: u8, action: FirmwareAction, bpid: bool) -> Result<()> {
        let fd = self.open_fd()?;
        let mut args = nvme_fw_commit_args {
            result: std::ptr::null_mut(),
            args_size: std::mem::size_of::<nvme_fw_commit_args>() as i32,
            fd,
            timeout: 0,
            action: action.as_raw(),
            slot,
            bpid,
        };
        let ret = unsafe { nvme_fw_commit(&mut args) };
        check_ret(ret)
    }

    /// Create a new namespace using the supplied [`IdentifyNamespace`] as a
    /// template. Returns the kernel-assigned NSID on success.
    ///
    /// **Destructive.** After create, the namespace must be attached to one
    /// or more controllers via [`Self::attach_namespace`] before any I/O
    /// is possible.
    ///
    /// Only supported on controllers whose OACS bit 3 (Namespace Management)
    /// is set — most consumer SSDs do not implement this.
    pub fn create_namespace(&self, template: &IdentifyNamespace) -> Result<u32> {
        let fd = self.open_fd()?;
        let mut new_nsid: u32 = 0;
        // libnvme requires a mutable pointer to the template; we copy first
        // to avoid mutating the caller's IdentifyNamespace.
        let mut id_ns: nvme_id_ns = *template.inner;
        let mut args = nvme_ns_mgmt_args {
            result: &mut new_nsid as *mut _,
            ns: &mut id_ns as *mut _,
            args_size: std::mem::size_of::<nvme_ns_mgmt_args>() as i32,
            fd,
            timeout: 0,
            nsid: 0,
            sel: NVME_NS_MGMT_SEL_CREATE,
            csi: 0,
            rsvd1: [0; 3],
            rsvd2: std::ptr::null_mut(),
            data: std::ptr::null_mut(),
        };
        let ret = unsafe { nvme_ns_mgmt(&mut args) };
        check_ret(ret)?;
        Ok(new_nsid)
    }

    /// Delete the namespace with the given NSID.
    ///
    /// **Destructive — irreversible.** Any host or controller still using
    /// the namespace will see subsequent I/O fail.
    pub fn delete_namespace(&self, nsid: u32) -> Result<()> {
        let fd = self.open_fd()?;
        let mut args = nvme_ns_mgmt_args {
            result: std::ptr::null_mut(),
            ns: std::ptr::null_mut(),
            args_size: std::mem::size_of::<nvme_ns_mgmt_args>() as i32,
            fd,
            timeout: 0,
            nsid,
            sel: NVME_NS_MGMT_SEL_DELETE,
            csi: 0,
            rsvd1: [0; 3],
            rsvd2: std::ptr::null_mut(),
            data: std::ptr::null_mut(),
        };
        let ret = unsafe { nvme_ns_mgmt(&mut args) };
        check_ret(ret)
    }

    /// Attach a namespace to the listed controllers.
    ///
    /// `controller_ids` is a slice of NVMe controller IDs (CNTLID, 16-bit).
    /// Empty slice is a no-op (well-defined per spec, but typically a
    /// programming mistake — consider asserting in caller code).
    pub fn attach_namespace(&self, nsid: u32, controller_ids: &[u16]) -> Result<()> {
        self.ns_attach_op(nsid, controller_ids, NVME_NS_ATTACH_SEL_CTRL_ATTACH)
    }

    /// Detach a namespace from the listed controllers. The namespace itself
    /// remains in existence — see [`Self::delete_namespace`] to remove it.
    pub fn detach_namespace(&self, nsid: u32, controller_ids: &[u16]) -> Result<()> {
        self.ns_attach_op(nsid, controller_ids, NVME_NS_ATTACH_SEL_CTRL_DEATTACH)
    }

    fn ns_attach_op(
        &self,
        nsid: u32,
        controller_ids: &[u16],
        sel: libnvme_sys::nvme_ns_attach_sel,
    ) -> Result<()> {
        if controller_ids.len() > 2047 {
            // nvme_ctrl_list.identifier is fixed at 2047 entries.
            return Err(Error::NotAvailable);
        }
        let fd = self.open_fd()?;
        let mut list = nvme_ctrl_list {
            num: controller_ids.len() as u16,
            identifier: [0; 2047],
        };
        for (i, &id) in controller_ids.iter().enumerate() {
            list.identifier[i] = id;
        }
        let mut args = nvme_ns_attach_args {
            result: std::ptr::null_mut(),
            ctrlist: &mut list as *mut _,
            args_size: std::mem::size_of::<nvme_ns_attach_args>() as i32,
            fd,
            timeout: 0,
            nsid,
            sel,
        };
        let ret = unsafe { nvme_ns_attach(&mut args) };
        check_ret(ret)
    }

    /// Disconnect a fabrics controller from its target.
    ///
    /// Consumes `self` since the underlying handle is no longer usable
    /// after a successful disconnect.
    pub fn disconnect(self) -> Result<()> {
        let ret = unsafe { nvme_disconnect_ctrl(self.inner) };
        check_ret(ret)
    }

    /// Reset the controller. Equivalent to writing `1` to
    /// `/sys/class/nvme/nvmeN/reset_controller`.
    pub fn reset(&self) -> Result<()> {
        let fd = self.open_fd()?;
        let ret = unsafe { nvme_ctrl_reset(fd) };
        check_ret(ret)
    }

    /// True if this controller has been marked as a discovery controller.
    pub fn is_discovery_controller(&self) -> bool {
        unsafe { nvme_ctrl_is_discovery_ctrl(self.inner) }
    }

    /// True if this controller's record was sourced from a discovery service.
    pub fn was_discovered(&self) -> bool {
        unsafe { nvme_ctrl_is_discovered(self.inner) }
    }

    /// True if this is a unique discovery controller (NVMe spec ≥ 2.0).
    ///
    /// Only present when built against a libnvme that exposes
    /// `nvme_ctrl_is_unique_discovery_ctrl` (added after libnvme 1.8).
    #[cfg(has_unique_discovery_ctrl)]
    pub fn is_unique_discovery_controller(&self) -> bool {
        unsafe { nvme_ctrl_is_unique_discovery_ctrl(self.inner) }
    }

    /// True if libnvme is keeping this controller alive across reconnects.
    pub fn is_persistent(&self) -> bool {
        unsafe { nvme_ctrl_is_persistent(self.inner) }
    }

    /// Toggle the persistent flag for this controller.
    pub fn set_persistent(&self, persistent: bool) {
        unsafe { nvme_ctrl_set_persistent(self.inner, persistent) };
    }

    /// Fetch the Discovery Log Page (LID 0x70) from a discovery controller.
    ///
    /// Only meaningful when `self.is_discovery_controller()` is `true`.
    /// `max_retries` is forwarded to libnvme — `0` lets libnvme pick its
    /// default. The returned [`DiscoveryLog`] owns its allocation and frees
    /// it on drop.
    pub fn discovery_log(&self, max_retries: i32) -> Result<DiscoveryLog> {
        fetch_discovery_log(self.inner, max_retries)
    }

    /// Set the DH-HMAC-CHAP host key (used to authenticate ourselves to
    /// the target).
    ///
    /// Only present when built against a libnvme that exposes
    /// `nvme_ctrl_set_dhchap_host_key` (added after libnvme 1.8).
    #[cfg(has_dhchap_host_key)]
    pub fn set_dhchap_host_key(&self, key: &str) -> Result<()> {
        let c = fabrics_cstring(key, "interior NUL byte in DH-HMAC-CHAP host key")?;
        unsafe { nvme_ctrl_set_dhchap_host_key(self.inner, c.as_ptr()) };
        Ok(())
    }

    /// Set the DH-HMAC-CHAP target key (used to authenticate the target).
    pub fn set_dhchap_key(&self, key: &str) -> Result<()> {
        let c = fabrics_cstring(key, "interior NUL byte in DH-HMAC-CHAP key")?;
        unsafe { nvme_ctrl_set_dhchap_key(self.inner, c.as_ptr()) };
        Ok(())
    }

    /// Set the TLS pre-shared key for this controller.
    ///
    /// Only present when built against a libnvme that exposes
    /// `nvme_ctrl_set_tls_key` (TLS support added after libnvme 1.8).
    #[cfg(has_tls_key)]
    pub fn set_tls_key(&self, key: &str) -> Result<()> {
        let c = fabrics_cstring(key, "interior NUL byte in TLS key")?;
        unsafe { nvme_ctrl_set_tls_key(self.inner, c.as_ptr()) };
        Ok(())
    }

    /// Set the TLS key identity (NQN-formatted identity used in PSK lookup).
    ///
    /// Only present when built against a libnvme that exposes
    /// `nvme_ctrl_set_tls_key_identity`.
    #[cfg(has_tls_key_identity)]
    pub fn set_tls_key_identity(&self, identity: &str) -> Result<()> {
        let c = fabrics_cstring(identity, "interior NUL byte in TLS key identity")?;
        unsafe { nvme_ctrl_set_tls_key_identity(self.inner, c.as_ptr()) };
        Ok(())
    }

    /// Set the keyring used to lookup TLS/DH-HMAC-CHAP keys.
    ///
    /// Only present when built against a libnvme that exposes
    /// `nvme_ctrl_set_keyring`.
    #[cfg(has_keyring)]
    pub fn set_keyring(&self, keyring: &str) -> Result<()> {
        let c = fabrics_cstring(keyring, "interior NUL byte in keyring name")?;
        unsafe { nvme_ctrl_set_keyring(self.inner, c.as_ptr()) };
        Ok(())
    }
}

fn fabrics_cstring(s: &str, err_msg: &'static str) -> Result<std::ffi::CString> {
    std::ffi::CString::new(s).map_err(|_| {
        Error::Os(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            err_msg,
        ))
    })
}

impl std::fmt::Debug for Controller<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Controller")
            .field("name", &self.name().ok())
            .field("model", &self.model().ok())
            .field("serial", &self.serial().ok())
            .field("transport", &self.transport().ok())
            .finish()
    }
}

/// Iterator over [`Controller`] entries, returned by [`crate::Subsystem::controllers`].
pub struct Controllers<'r> {
    subsystem: nvme_subsystem_t,
    cursor: nvme_ctrl_t,
    _marker: PhantomData<&'r Root>,
}

impl<'r> Controllers<'r> {
    pub(crate) fn new(subsystem: nvme_subsystem_t) -> Self {
        let cursor = unsafe { nvme_subsystem_first_ctrl(subsystem) };
        Controllers {
            subsystem,
            cursor,
            _marker: PhantomData,
        }
    }
}

impl<'r> Iterator for Controllers<'r> {
    type Item = Controller<'r>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor.is_null() {
            return None;
        }
        let current = self.cursor;
        self.cursor = unsafe { nvme_subsystem_next_ctrl(self.subsystem, current) };
        Some(Controller::from_raw(current))
    }
}
