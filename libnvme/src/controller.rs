use std::io;
use std::marker::PhantomData;

use libnvme_sys::{
    nvme_ctrl_get_address, nvme_ctrl_get_fd, nvme_ctrl_get_firmware, nvme_ctrl_get_host_iface,
    nvme_ctrl_get_host_traddr, nvme_ctrl_get_model, nvme_ctrl_get_name, nvme_ctrl_get_numa_node,
    nvme_ctrl_get_phy_slot, nvme_ctrl_get_queue_count, nvme_ctrl_get_serial, nvme_ctrl_get_sqsize,
    nvme_ctrl_get_state, nvme_ctrl_get_subsysnqn, nvme_ctrl_get_traddr, nvme_ctrl_get_transport,
    nvme_ctrl_get_trsvcid, nvme_ctrl_identify, nvme_ctrl_t, nvme_get_log, nvme_get_log_args,
    nvme_id_ctrl, nvme_smart_log, nvme_subsystem_first_ctrl, nvme_subsystem_next_ctrl,
    nvme_subsystem_t, NVME_LOG_LID_SMART,
};

use crate::error::check_ret;
use crate::identify::IdentifyController;
use crate::log::SmartLog;
use crate::namespace::Namespaces;
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

    /// Fetch the SMART / Health Information log page (LID 02h) for this
    /// controller, aggregated across all namespaces.
    ///
    /// Requires the controller device to be openable by the calling process.
    /// If libnvme cannot open it (e.g. `EACCES` without root), the underlying
    /// `errno` is reported here as [`Error::Os`].
    pub fn smart_log(&self) -> Result<SmartLog> {
        // libnvme opens the controller device lazily inside nvme_ctrl_get_fd.
        // If that open fails it returns -1 with errno set; without this check
        // we'd pass -1 into nvme_get_log and the real EACCES would be masked
        // as EBADF.
        let fd = unsafe { nvme_ctrl_get_fd(self.inner) };
        if fd < 0 {
            return Err(Error::Os(io::Error::last_os_error()));
        }

        let mut log = Box::new(nvme_smart_log::default());
        let mut args = nvme_get_log_args {
            args_size: std::mem::size_of::<nvme_get_log_args>() as i32,
            fd,
            lid: NVME_LOG_LID_SMART,
            nsid: 0xFFFF_FFFF,
            log: log.as_mut() as *mut _ as *mut std::ffi::c_void,
            len: std::mem::size_of::<nvme_smart_log>() as u32,
            ..Default::default()
        };
        let ret = unsafe { nvme_get_log(&mut args) };
        check_ret(ret)?;
        Ok(SmartLog { inner: log })
    }

    /// Iterate over namespaces accessible through this controller.
    pub fn namespaces(&self) -> Namespaces<'r> {
        Namespaces::new(self.inner)
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
