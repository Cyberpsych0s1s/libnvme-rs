use std::io;
use std::marker::PhantomData;
use std::ptr::NonNull;

use libnvme_sys::{nvme_free_tree, nvme_root, nvme_root_t, nvme_scan};

use crate::Result;

pub struct Root {
    inner: NonNull<nvme_root>,
    _not_send_sync: PhantomData<*const ()>,
}

impl Root {
    pub fn scan() -> Result<Self> {
        let raw = unsafe { nvme_scan(std::ptr::null()) };
        let inner = NonNull::new(raw).ok_or_else(io::Error::last_os_error)?;
        Ok(Root {
            inner,
            _not_send_sync: PhantomData,
        })
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
