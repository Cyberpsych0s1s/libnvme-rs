//! NVMe log pages.
//!
//! Currently exposes the SMART / Health Information log page (LID 02h).
//! Generic log-page support is planned for a later release.

use libnvme_sys::nvme_smart_log;

/// Decoded SMART / Health Information log page (LID 02h).
///
/// Counters that the spec defines as 128-bit are exposed as `u128`. Times
/// are reported in minutes (per the NVMe spec).
pub struct SmartLog {
    pub(crate) inner: Box<nvme_smart_log>,
}

impl SmartLog {
    /// Critical warning bitfield: spare, temperature, degraded reliability,
    /// media read-only, volatile memory backup failure, persistent memory
    /// read-only. See `NVME_SMART_CRIT_*` constants in `libnvme-sys`.
    pub fn critical_warning(&self) -> u8 {
        self.inner.critical_warning
    }

    /// Composite temperature in Kelvin.
    pub fn temperature_kelvin(&self) -> u16 {
        // Stored as a 16-bit little-endian field in two bytes.
        u16::from_le_bytes(self.inner.temperature)
    }

    /// Composite temperature in Celsius.
    pub fn temperature_celsius(&self) -> i16 {
        (self.temperature_kelvin() as i32 - 273) as i16
    }

    /// Available spare capacity as a percentage of normalized spare.
    pub fn available_spare(&self) -> u8 {
        self.inner.avail_spare
    }

    /// Threshold below which `critical_warning` flags the spare condition.
    pub fn available_spare_threshold(&self) -> u8 {
        self.inner.spare_thresh
    }

    /// Vendor-specific estimate of life used, as a percentage. May exceed 100.
    pub fn percentage_used(&self) -> u8 {
        self.inner.percent_used
    }

    /// Endurance group critical warning summary.
    pub fn endurance_group_critical_warning(&self) -> u8 {
        self.inner.endu_grp_crit_warn_sumry
    }

    /// Number of 512-byte data units the host has read from the controller,
    /// in thousands. Multiply by 1000 and by 512 for total bytes read.
    pub fn data_units_read(&self) -> u128 {
        u128::from_le_bytes(self.inner.data_units_read)
    }

    /// Number of 512-byte data units the host has written to the controller,
    /// in thousands. See [`data_units_read`](Self::data_units_read).
    pub fn data_units_written(&self) -> u128 {
        u128::from_le_bytes(self.inner.data_units_written)
    }

    /// Number of read commands completed by the controller.
    pub fn host_read_commands(&self) -> u128 {
        u128::from_le_bytes(self.inner.host_reads)
    }

    /// Number of write commands completed by the controller.
    pub fn host_write_commands(&self) -> u128 {
        u128::from_le_bytes(self.inner.host_writes)
    }

    /// Time the controller has been busy with I/O commands, in minutes.
    pub fn controller_busy_time_minutes(&self) -> u128 {
        u128::from_le_bytes(self.inner.ctrl_busy_time)
    }

    /// Number of power-on/off cycles experienced by the controller.
    pub fn power_cycles(&self) -> u128 {
        u128::from_le_bytes(self.inner.power_cycles)
    }

    /// Cumulative power-on time in hours.
    pub fn power_on_hours(&self) -> u128 {
        u128::from_le_bytes(self.inner.power_on_hours)
    }

    /// Number of unsafe shutdowns (loss of power without a normal shutdown).
    pub fn unsafe_shutdowns(&self) -> u128 {
        u128::from_le_bytes(self.inner.unsafe_shutdowns)
    }

    /// Number of media or data integrity errors detected.
    pub fn media_errors(&self) -> u128 {
        u128::from_le_bytes(self.inner.media_errors)
    }

    /// Number of entries in the Error Information Log.
    pub fn num_error_log_entries(&self) -> u128 {
        u128::from_le_bytes(self.inner.num_err_log_entries)
    }

    /// Time the composite temperature has exceeded the warning threshold,
    /// in minutes.
    pub fn warning_temp_time_minutes(&self) -> u32 {
        self.inner.warning_temp_time
    }

    /// Time the composite temperature has exceeded the critical threshold,
    /// in minutes.
    pub fn critical_temp_time_minutes(&self) -> u32 {
        self.inner.critical_comp_time
    }

    /// Temperature sensor reading in Kelvin (`index` is `1..=8`).
    /// Returns `None` for out-of-range indices or unreported sensors (`0`).
    pub fn temperature_sensor_kelvin(&self, index: u8) -> Option<u16> {
        if !(1..=8).contains(&index) {
            return None;
        }
        let value = self.inner.temp_sensor[usize::from(index - 1)];
        if value == 0 {
            None
        } else {
            Some(value)
        }
    }
}

impl std::fmt::Debug for SmartLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SmartLog")
            .field("critical_warning", &self.critical_warning())
            .field("temperature_celsius", &self.temperature_celsius())
            .field("percentage_used", &self.percentage_used())
            .field("power_cycles", &self.power_cycles())
            .field("power_on_hours", &self.power_on_hours())
            .field("unsafe_shutdowns", &self.unsafe_shutdowns())
            .finish()
    }
}
