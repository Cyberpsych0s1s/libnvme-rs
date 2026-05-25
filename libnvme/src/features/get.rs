//! Get Features methods.

use libnvme_sys::{
    nvme_get_features_arbitration, nvme_get_features_async_event, nvme_get_features_auto_pst,
    nvme_get_features_endurance_event_cfg, nvme_get_features_err_recovery, nvme_get_features_hctm,
    nvme_get_features_host_behavior, nvme_get_features_host_id, nvme_get_features_host_mem_buf,
    nvme_get_features_iocs_profile, nvme_get_features_irq_coalesce, nvme_get_features_irq_config,
    nvme_get_features_kato, nvme_get_features_lba_range, nvme_get_features_lba_sts_interval,
    nvme_get_features_nopsc, nvme_get_features_num_queues, nvme_get_features_plm_config,
    nvme_get_features_plm_window, nvme_get_features_power_mgmt, nvme_get_features_resv_mask,
    nvme_get_features_resv_persist, nvme_get_features_rrl, nvme_get_features_sanitize,
    nvme_get_features_sw_progress, nvme_get_features_temp_thresh, nvme_get_features_timestamp,
    nvme_get_features_volatile_wc, nvme_get_features_write_atomic, nvme_get_features_write_protect,
};

#[cfg(has_err_recovery2)]
use libnvme_sys::nvme_get_features_err_recovery2;
#[cfg(has_host_mem_buf2)]
use libnvme_sys::nvme_get_features_host_mem_buf2;
#[cfg(has_lba_range2)]
use libnvme_sys::nvme_get_features_lba_range2;
#[cfg(has_resv_mask2)]
use libnvme_sys::nvme_get_features_resv_mask2;
#[cfg(has_resv_persist2)]
use libnvme_sys::nvme_get_features_resv_persist2;
#[cfg(has_temp_thresh2)]
use libnvme_sys::nvme_get_features_temp_thresh2;

use super::types::{AutoPst, HostBehavior, HostMemBufAttrs, LbaRangeType, PlmConfig, Timestamp};
use super::Features;
use crate::admin::FeatureSelect;
use crate::error::check_ret;
use crate::{Error, Result};

impl Features<'_, '_> {
    /// Get Features — Arbitration (FID 01h).
    pub fn get_arbitration(&self, sel: FeatureSelect) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is a valid file descriptor for this controller; result is
        // a valid &mut u32 alive for the call; sel is a plain integer.
        let ret = unsafe { nvme_get_features_arbitration(fd, sel.as_raw(), &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Power Management (FID 02h).
    pub fn get_power_mgmt(&self, sel: FeatureSelect) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is a valid file descriptor for this controller; result is
        // a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_get_features_power_mgmt(fd, sel.as_raw(), &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — LBA Range Type (FID 03h).
    pub fn get_lba_range(&self, sel: FeatureSelect, data: &mut LbaRangeType) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; data points to a caller-owned LbaRangeType
        // alive for the call; result is a valid &mut u32.
        let ret =
            unsafe { nvme_get_features_lba_range(fd, sel.as_raw(), data as *mut _, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — LBA Range Type (FID 03h), NSID-scoped extended form.
    #[cfg(has_lba_range2)]
    pub fn get_lba_range2(
        &self,
        sel: FeatureSelect,
        nsid: u32,
        data: &mut LbaRangeType,
    ) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; data points to a caller-owned LbaRangeType
        // alive for the call; result is a valid &mut u32.
        let ret = unsafe {
            nvme_get_features_lba_range2(fd, sel.as_raw(), nsid, data as *mut _, &mut result)
        };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Temperature Threshold (FID 04h).
    pub fn get_temp_thresh(&self, sel: FeatureSelect) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_get_features_temp_thresh(fd, sel.as_raw(), &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Temperature Threshold (FID 04h), extended form.
    #[cfg(has_temp_thresh2)]
    pub fn get_temp_thresh2(&self, sel: FeatureSelect, tmpsel: u8, thsel: u32) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret =
            unsafe { nvme_get_features_temp_thresh2(fd, sel.as_raw(), tmpsel, thsel, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Error Recovery (FID 05h).
    pub fn get_err_recovery(&self, sel: FeatureSelect) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_get_features_err_recovery(fd, sel.as_raw(), &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Error Recovery (FID 05h), NSID-scoped extended form.
    #[cfg(has_err_recovery2)]
    pub fn get_err_recovery2(&self, sel: FeatureSelect, nsid: u32) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_get_features_err_recovery2(fd, sel.as_raw(), nsid, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Volatile Write Cache (FID 06h).
    pub fn get_volatile_wc(&self, sel: FeatureSelect) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_get_features_volatile_wc(fd, sel.as_raw(), &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Number of Queues (FID 07h).
    pub fn get_num_queues(&self, sel: FeatureSelect) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_get_features_num_queues(fd, sel.as_raw(), &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Interrupt Coalescing (FID 08h).
    pub fn get_irq_coalesce(&self, sel: FeatureSelect) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_get_features_irq_coalesce(fd, sel.as_raw(), &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Interrupt Vector Configuration (FID 09h).
    pub fn get_irq_config(&self, sel: FeatureSelect, iv: u16) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_get_features_irq_config(fd, sel.as_raw(), iv, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Write Atomicity Normal (FID 0Ah).
    pub fn get_write_atomic(&self, sel: FeatureSelect) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_get_features_write_atomic(fd, sel.as_raw(), &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Async Event Configuration (FID 0Bh).
    pub fn get_async_event(&self, sel: FeatureSelect) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_get_features_async_event(fd, sel.as_raw(), &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Auto Power State Transition (FID 0Ch).
    pub fn get_auto_pst(&self, sel: FeatureSelect, apst: &mut AutoPst) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; apst points to a caller-owned AutoPst alive
        // for the call; result is a valid &mut u32.
        let ret =
            unsafe { nvme_get_features_auto_pst(fd, sel.as_raw(), apst as *mut _, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Host Memory Buffer (FID 0Dh).
    pub fn get_host_mem_buf(&self, sel: FeatureSelect) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_get_features_host_mem_buf(fd, sel.as_raw(), &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Host Memory Buffer (FID 0Dh), extended form returning
    /// the full attributes struct.
    #[cfg(has_host_mem_buf2)]
    pub fn get_host_mem_buf2(
        &self,
        sel: FeatureSelect,
        attrs: &mut HostMemBufAttrs,
    ) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; attrs points to a caller-owned HostMemBufAttrs
        // alive for the call; result is a valid &mut u32.
        let ret = unsafe {
            nvme_get_features_host_mem_buf2(fd, sel.as_raw(), attrs as *mut _, &mut result)
        };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Timestamp (FID 0Eh).
    pub fn get_timestamp(&self, sel: FeatureSelect, ts: &mut Timestamp) -> Result<()> {
        let fd = self.fd()?;
        // SAFETY: fd is valid; ts points to a caller-owned Timestamp alive
        // for the call.
        let ret = unsafe { nvme_get_features_timestamp(fd, sel.as_raw(), ts as *mut _) };
        check_ret(ret)
    }

    /// Get Features — Keep Alive Timer (FID 0Fh).
    pub fn get_kato(&self, sel: FeatureSelect) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_get_features_kato(fd, sel.as_raw(), &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Host Controlled Thermal Management (FID 10h).
    pub fn get_hctm(&self, sel: FeatureSelect) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_get_features_hctm(fd, sel.as_raw(), &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Non-Operational Power State Config (FID 11h).
    pub fn get_nopsc(&self, sel: FeatureSelect) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_get_features_nopsc(fd, sel.as_raw(), &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Read Recovery Level (FID 12h).
    pub fn get_rrl(&self, sel: FeatureSelect) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_get_features_rrl(fd, sel.as_raw(), &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Predictable Latency Mode Config (FID 13h).
    pub fn get_plm_config(
        &self,
        sel: FeatureSelect,
        nvmsetid: u16,
        data: &mut PlmConfig,
    ) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; data points to a caller-owned PlmConfig alive
        // for the call; result is a valid &mut u32.
        let ret = unsafe {
            nvme_get_features_plm_config(fd, sel.as_raw(), nvmsetid, data as *mut _, &mut result)
        };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Predictable Latency Mode Window (FID 14h).
    pub fn get_plm_window(&self, sel: FeatureSelect, nvmsetid: u16) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_get_features_plm_window(fd, sel.as_raw(), nvmsetid, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — LBA Status Information Report Interval (FID 15h).
    pub fn get_lba_sts_interval(&self, sel: FeatureSelect) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_get_features_lba_sts_interval(fd, sel.as_raw(), &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Host Behavior Support (FID 16h).
    pub fn get_host_behavior(&self, sel: FeatureSelect, data: &mut HostBehavior) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; data points to a caller-owned HostBehavior alive
        // for the call; result is a valid &mut u32.
        let ret = unsafe {
            nvme_get_features_host_behavior(fd, sel.as_raw(), data as *mut _, &mut result)
        };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Sanitize Config (FID 17h).
    pub fn get_sanitize(&self, sel: FeatureSelect) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_get_features_sanitize(fd, sel.as_raw(), &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Endurance Group Event Configuration (FID 18h).
    pub fn get_endurance_event_cfg(&self, sel: FeatureSelect, endgid: u16) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret =
            unsafe { nvme_get_features_endurance_event_cfg(fd, sel.as_raw(), endgid, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Software Progress Marker (FID 80h).
    pub fn get_sw_progress(&self, sel: FeatureSelect) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_get_features_sw_progress(fd, sel.as_raw(), &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Host Identifier (FID 81h).
    ///
    /// `hostid` must be exactly 8 bytes when `exhid` is `false`, or 16 bytes
    /// when `exhid` is `true`.
    pub fn get_host_id(&self, sel: FeatureSelect, exhid: bool, hostid: &mut [u8]) -> Result<()> {
        let expected = if exhid { 16 } else { 8 };
        if hostid.len() != expected {
            return Err(Error::Os(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "hostid buffer must be 8 bytes (exhid=false) or 16 bytes (exhid=true)",
            )));
        }
        let fd = self.fd()?;
        // SAFETY: fd is valid; hostid is a caller-owned mutable slice of the
        // exact required length (checked above), alive for the call.
        let ret = unsafe {
            nvme_get_features_host_id(
                fd,
                sel.as_raw(),
                exhid,
                hostid.len() as u32,
                hostid.as_mut_ptr(),
            )
        };
        check_ret(ret)
    }

    /// Get Features — Reservation Notification Mask (FID 82h).
    pub fn get_resv_mask(&self, sel: FeatureSelect) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_get_features_resv_mask(fd, sel.as_raw(), &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Reservation Notification Mask (FID 82h), NSID-scoped
    /// extended form.
    #[cfg(has_resv_mask2)]
    pub fn get_resv_mask2(&self, sel: FeatureSelect, nsid: u32) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_get_features_resv_mask2(fd, sel.as_raw(), nsid, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Reservation Persistence (FID 83h).
    pub fn get_resv_persist(&self, sel: FeatureSelect) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_get_features_resv_persist(fd, sel.as_raw(), &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Reservation Persistence (FID 83h), NSID-scoped extended
    /// form.
    #[cfg(has_resv_persist2)]
    pub fn get_resv_persist2(&self, sel: FeatureSelect, nsid: u32) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_get_features_resv_persist2(fd, sel.as_raw(), nsid, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — Namespace Write Protection (FID 84h).
    pub fn get_write_protect(&self, sel: FeatureSelect, nsid: u32) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_get_features_write_protect(fd, nsid, sel.as_raw(), &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Get Features — IO Command Set Profile (FID 19h, NVMe 2.0).
    pub fn get_iocs_profile(&self, sel: FeatureSelect) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_get_features_iocs_profile(fd, sel.as_raw(), &mut result) };
        check_ret(ret)?;
        Ok(result)
    }
}
