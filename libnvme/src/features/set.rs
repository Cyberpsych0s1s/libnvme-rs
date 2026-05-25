//! Set Features methods.

use libnvme_sys::{
    nvme_set_features_arbitration, nvme_set_features_async_event, nvme_set_features_auto_pst,
    nvme_set_features_endurance_evt_cfg, nvme_set_features_err_recovery, nvme_set_features_hctm,
    nvme_set_features_host_behavior, nvme_set_features_host_id, nvme_set_features_iocs_profile,
    nvme_set_features_irq_coalesce, nvme_set_features_irq_config, nvme_set_features_lba_range,
    nvme_set_features_lba_sts_interval, nvme_set_features_nopsc, nvme_set_features_plm_config,
    nvme_set_features_plm_window, nvme_set_features_power_mgmt, nvme_set_features_resv_mask,
    nvme_set_features_resv_persist, nvme_set_features_rrl, nvme_set_features_sanitize,
    nvme_set_features_sw_progress, nvme_set_features_temp_thresh, nvme_set_features_timestamp,
    nvme_set_features_volatile_wc, nvme_set_features_write_atomic, nvme_set_features_write_protect,
};

#[cfg(has_resv_mask2)]
use libnvme_sys::nvme_set_features_resv_mask2;
#[cfg(has_resv_persist2)]
use libnvme_sys::nvme_set_features_resv_persist2;
#[cfg(has_temp_thresh2)]
use libnvme_sys::nvme_set_features_temp_thresh2;
#[cfg(has_write_protect2)]
use libnvme_sys::nvme_set_features_write_protect2;

use super::types::{AutoPst, HostBehavior, LbaRangeType, PlmConfig};
use super::Features;
use crate::error::check_ret;
use crate::{Error, Result};

impl Features<'_, '_> {
    /// Set Features — Arbitration (FID 01h).
    pub fn set_arbitration(&self, ab: u8, lpw: u8, mpw: u8, hpw: u8, save: bool) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is a valid file descriptor for this controller; result is
        // a valid &mut u32 alive for the call.
        let ret =
            unsafe { nvme_set_features_arbitration(fd, ab, lpw, mpw, hpw, save, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — Power Management (FID 02h).
    pub fn set_power_mgmt(&self, ps: u8, wh: u8, save: bool) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_set_features_power_mgmt(fd, ps, wh, save, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — LBA Range Type (FID 03h).
    pub fn set_lba_range(
        &self,
        nsid: u32,
        nr_ranges: u8,
        save: bool,
        data: &mut LbaRangeType,
    ) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; data points to a caller-owned LbaRangeType
        // alive for the call; result is a valid &mut u32.
        let ret = unsafe {
            nvme_set_features_lba_range(fd, nsid, nr_ranges, save, data as *mut _, &mut result)
        };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — Temperature Threshold (FID 04h).
    pub fn set_temp_thresh(&self, tmpth: u16, tmpsel: u8, thsel: u32, save: bool) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret =
            unsafe { nvme_set_features_temp_thresh(fd, tmpth, tmpsel, thsel, save, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — Temperature Threshold (FID 04h), extended form with
    /// an upper temperature byte.
    ///
    /// Available on libnvme builds that expose `nvme_set_features_temp_thresh2`.
    #[cfg(has_temp_thresh2)]
    pub fn set_temp_thresh2(
        &self,
        tmpth: u16,
        tmpsel: u8,
        thsel: u32,
        tmpthh: u8,
        save: bool,
    ) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe {
            nvme_set_features_temp_thresh2(fd, tmpth, tmpsel, thsel, tmpthh, save, &mut result)
        };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — Error Recovery (FID 05h).
    pub fn set_err_recovery(&self, nsid: u32, tler: u16, dulbe: bool, save: bool) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret =
            unsafe { nvme_set_features_err_recovery(fd, nsid, tler, dulbe, save, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — Volatile Write Cache (FID 06h).
    pub fn set_volatile_wc(&self, wce: bool, save: bool) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_set_features_volatile_wc(fd, wce, save, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — Interrupt Coalescing (FID 08h).
    pub fn set_irq_coalesce(&self, thr: u8, time: u8, save: bool) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_set_features_irq_coalesce(fd, thr, time, save, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — Interrupt Vector Configuration (FID 09h).
    pub fn set_irq_config(&self, iv: u16, cd: bool, save: bool) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_set_features_irq_config(fd, iv, cd, save, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — Write Atomicity Normal (FID 0Ah).
    pub fn set_write_atomic(&self, dn: bool, save: bool) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_set_features_write_atomic(fd, dn, save, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — Async Event Configuration (FID 0Bh).
    pub fn set_async_event(&self, events: u32, save: bool) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_set_features_async_event(fd, events, save, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — Auto Power State Transition (FID 0Ch).
    pub fn set_auto_pst(&self, apste: bool, save: bool, apst: &mut AutoPst) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; apst points to a caller-owned AutoPst alive
        // for the call; result is a valid &mut u32.
        let ret =
            unsafe { nvme_set_features_auto_pst(fd, apste, save, apst as *mut _, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — Host Memory Buffer (FID 0Dh) — sub-command via Timestamp.
    ///
    /// (libnvme exposes Timestamp as its own setter; see [`Self::set_timestamp`].)
    pub fn set_timestamp(&self, save: bool, timestamp: u64) -> Result<()> {
        let fd = self.fd()?;
        // SAFETY: fd is a valid file descriptor for this controller; the
        // timestamp argument is a plain integer.
        let ret = unsafe { nvme_set_features_timestamp(fd, save, timestamp) };
        check_ret(ret)
    }

    /// Set Features — Host Controlled Thermal Management (FID 10h).
    pub fn set_hctm(&self, tmt2: u16, tmt1: u16, save: bool) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_set_features_hctm(fd, tmt2, tmt1, save, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — Non-Operational Power State Config (FID 11h).
    pub fn set_nopsc(&self, noppme: bool, save: bool) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_set_features_nopsc(fd, noppme, save, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — Read Recovery Level (FID 12h).
    pub fn set_rrl(&self, rrl: u8, nvmsetid: u16, save: bool) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_set_features_rrl(fd, rrl, nvmsetid, save, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — Predictable Latency Mode Config (FID 13h).
    pub fn set_plm_config(
        &self,
        enable: bool,
        nvmsetid: u16,
        save: bool,
        data: &mut PlmConfig,
    ) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; data points to a caller-owned PlmConfig alive
        // for the call; result is a valid &mut u32.
        let ret = unsafe {
            nvme_set_features_plm_config(fd, enable, nvmsetid, save, data as *mut _, &mut result)
        };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — Predictable Latency Mode Window (FID 14h).
    pub fn set_plm_window(&self, sel: u32, nvmsetid: u16, save: bool) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_set_features_plm_window(fd, sel, nvmsetid, save, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — LBA Status Information Report Interval (FID 15h).
    pub fn set_lba_sts_interval(&self, lsiri: u16, lsipi: u16, save: bool) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret =
            unsafe { nvme_set_features_lba_sts_interval(fd, lsiri, lsipi, save, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — Host Behavior Support (FID 16h).
    pub fn set_host_behavior(&self, save: bool, data: &mut HostBehavior) -> Result<()> {
        let fd = self.fd()?;
        // SAFETY: fd is valid; data points to a caller-owned HostBehavior
        // alive for the call.
        let ret = unsafe { nvme_set_features_host_behavior(fd, save, data as *mut _) };
        check_ret(ret)
    }

    /// Set Features — Sanitize Config (FID 17h).
    pub fn set_sanitize(&self, nodrm: bool, save: bool) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_set_features_sanitize(fd, nodrm, save, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — Endurance Group Event Configuration (FID 18h).
    pub fn set_endurance_evt_cfg(&self, endgid: u16, egwarn: u8, save: bool) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret =
            unsafe { nvme_set_features_endurance_evt_cfg(fd, endgid, egwarn, save, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — Software Progress Marker (FID 80h).
    pub fn set_sw_progress(&self, pbslc: u8, save: bool) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_set_features_sw_progress(fd, pbslc, save, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — Host Identifier (FID 81h).
    ///
    /// `hostid` must be exactly 8 bytes when `exhid` is `false`, or 16 bytes
    /// when `exhid` is `true`.
    pub fn set_host_id(&self, exhid: bool, save: bool, hostid: &mut [u8]) -> Result<()> {
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
        let ret = unsafe { nvme_set_features_host_id(fd, exhid, save, hostid.as_mut_ptr()) };
        check_ret(ret)
    }

    /// Set Features — Reservation Notification Mask (FID 82h).
    pub fn set_resv_mask(&self, mask: u32, save: bool) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_set_features_resv_mask(fd, mask, save, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — Reservation Notification Mask (FID 82h), NSID-scoped
    /// extended form. Available on libnvme builds that expose
    /// `nvme_set_features_resv_mask2`.
    #[cfg(has_resv_mask2)]
    pub fn set_resv_mask2(&self, nsid: u32, mask: u32, save: bool) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_set_features_resv_mask2(fd, nsid, mask, save, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — Reservation Persistence (FID 83h).
    pub fn set_resv_persist(&self, ptpl: bool, save: bool) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_set_features_resv_persist(fd, ptpl, save, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — Reservation Persistence (FID 83h), NSID-scoped extended
    /// form. Available on libnvme builds that expose
    /// `nvme_set_features_resv_persist2`.
    #[cfg(has_resv_persist2)]
    pub fn set_resv_persist2(&self, nsid: u32, ptpl: bool, save: bool) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_set_features_resv_persist2(fd, nsid, ptpl, save, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — Namespace Write Protection (FID 84h).
    pub fn set_write_protect(&self, state: u32, save: bool) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_set_features_write_protect(fd, state, save, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — Namespace Write Protection (FID 84h), NSID-scoped
    /// extended form. Available on libnvme builds that expose
    /// `nvme_set_features_write_protect2`.
    #[cfg(has_write_protect2)]
    pub fn set_write_protect2(&self, nsid: u32, state: u32, save: bool) -> Result<u32> {
        let fd = self.fd()?;
        let mut result = 0u32;
        // SAFETY: fd is valid; result is a valid &mut u32 alive for the call.
        let ret = unsafe { nvme_set_features_write_protect2(fd, nsid, state, save, &mut result) };
        check_ret(ret)?;
        Ok(result)
    }

    /// Set Features — IO Command Set Profile (FID 19h, NVMe 2.0).
    pub fn set_iocs_profile(&self, iocsi: u16, save: bool) -> Result<()> {
        let fd = self.fd()?;
        // SAFETY: fd is a valid file descriptor for this controller; remaining
        // arguments are plain integers/bools.
        let ret = unsafe { nvme_set_features_iocs_profile(fd, iocsi, save) };
        check_ret(ret)
    }
}
