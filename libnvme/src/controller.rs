use std::marker::PhantomData;

use libnvme_sys::{
    nvme_ctrl_get_address, nvme_ctrl_get_fd, nvme_ctrl_get_firmware, nvme_ctrl_get_model,
    nvme_ctrl_get_name, nvme_ctrl_get_serial, nvme_ctrl_get_state, nvme_ctrl_get_transport,
    nvme_ctrl_identify, nvme_ctrl_t, nvme_get_log, nvme_get_log_args, nvme_id_ctrl, nvme_smart_log,
    nvme_subsystem_first_ctrl, nvme_subsystem_next_ctrl, nvme_subsystem_t, NVME_LOG_LID_SMART,
};

use crate::error::check_ret;
use crate::identify::IdentifyController;
use crate::log::SmartLog;
use crate::namespace::Namespaces;
use crate::util::cstr_to_str;
use crate::{Result, Root};

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

    /// Transport-specific address (e.g. PCIe BDF or Fabrics address).
    pub fn address(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_ctrl_get_address(self.inner)) }
    }

    /// Controller state as reported by the kernel: `live`, `resetting`, etc.
    pub fn state(&self) -> Result<&'r str> {
        unsafe { cstr_to_str(nvme_ctrl_get_state(self.inner)) }
    }

    /// Issue the Identify Controller admin command and return the decoded
    /// data structure.
    pub fn identify(&self) -> Result<IdentifyController> {
        let mut id = Box::new(nvme_id_ctrl::default());
        let ret = unsafe { nvme_ctrl_identify(self.inner, id.as_mut() as *mut _) };
        check_ret(ret)?;
        Ok(IdentifyController { inner: id })
    }

    /// Fetch the SMART / Health Information log page (LID 02h) for this
    /// controller, aggregated across all namespaces.
    pub fn smart_log(&self) -> Result<SmartLog> {
        let mut log = Box::new(nvme_smart_log::default());
        let fd = unsafe { nvme_ctrl_get_fd(self.inner) };
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
