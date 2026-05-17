use std::marker::PhantomData;

use libnvme_sys::{
    nvme_ctrl_get_address, nvme_ctrl_get_firmware, nvme_ctrl_get_model, nvme_ctrl_get_name,
    nvme_ctrl_get_serial, nvme_ctrl_get_state, nvme_ctrl_get_transport, nvme_ctrl_t,
    nvme_subsystem_first_ctrl, nvme_subsystem_next_ctrl, nvme_subsystem_t,
};

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
