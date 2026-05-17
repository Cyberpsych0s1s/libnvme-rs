use std::io;
use std::marker::PhantomData;

use libnvme_sys::{
    nvme_cmd_get_log_lid, nvme_ctrl_get_address, nvme_ctrl_get_fd, nvme_ctrl_get_firmware,
    nvme_ctrl_get_host_iface, nvme_ctrl_get_host_traddr, nvme_ctrl_get_model, nvme_ctrl_get_name,
    nvme_ctrl_get_numa_node, nvme_ctrl_get_phy_slot, nvme_ctrl_get_queue_count,
    nvme_ctrl_get_serial, nvme_ctrl_get_sqsize, nvme_ctrl_get_state, nvme_ctrl_get_subsysnqn,
    nvme_ctrl_get_traddr, nvme_ctrl_get_transport, nvme_ctrl_get_trsvcid, nvme_ctrl_identify,
    nvme_ctrl_t, nvme_error_log_page, nvme_firmware_slot, nvme_get_log, nvme_get_log_args,
    nvme_id_ctrl, nvme_smart_log, nvme_subsystem_first_ctrl, nvme_subsystem_next_ctrl,
    nvme_subsystem_t, NVME_LOG_LID_ERROR, NVME_LOG_LID_FW_SLOT, NVME_LOG_LID_SMART,
};

use crate::error::check_ret;
use crate::identify::IdentifyController;
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
