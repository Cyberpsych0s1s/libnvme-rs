//! NVMe log pages.
//!
//! Each typed log page (e.g. [`SmartLog`], [`FirmwareSlotLog`]) wraps the
//! raw `nvme_*_log` struct returned by libnvme's Get Log Page admin command.
//!
//! The fixed-size pages are fetched via
//! [`Controller::get_log_page`](crate::Controller::get_log_page), a generic
//! helper parameterized by the libnvme struct type. The Error Information
//! log is variable-sized (an array of entries) and uses a dedicated method.

use libnvme_sys::{nvme_error_log_page, nvme_firmware_slot, nvme_smart_log};

use crate::util::fixed_ascii_to_str;
use crate::{Error, Result};

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

/// Firmware Slot Information log page (LID 03h).
///
/// Reports which firmware slot is currently active, which (if any) is
/// scheduled for activation on next reset, and the firmware revision string
/// stored in each of the up-to-7 slots.
pub struct FirmwareSlotLog {
    pub(crate) inner: Box<nvme_firmware_slot>,
}

impl FirmwareSlotLog {
    /// Raw Active Firmware Info byte (AFI).
    ///
    /// Bits 0–2: currently active slot. Bits 4–6: slot scheduled for
    /// activation on next reset (`0` if none).
    pub fn afi(&self) -> u8 {
        self.inner.afi
    }

    /// Currently active firmware slot (`1..=7`).
    pub fn active_slot(&self) -> u8 {
        self.inner.afi & 0x07
    }

    /// Slot that will activate on next controller-level reset, if any.
    pub fn next_slot_to_activate(&self) -> Option<u8> {
        let next = (self.inner.afi >> 4) & 0x07;
        if next == 0 {
            None
        } else {
            Some(next)
        }
    }

    /// Firmware revision string stored in the given slot (`1..=7`).
    ///
    /// Returns [`Error::NotAvailable`] for indices outside that range; an
    /// empty string if the slot is unused.
    pub fn slot_firmware(&self, slot: u8) -> Result<&str> {
        if !(1..=7).contains(&slot) {
            return Err(Error::NotAvailable);
        }
        fixed_ascii_to_str(&self.inner.frs[usize::from(slot - 1)])
    }
}

impl std::fmt::Debug for FirmwareSlotLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FirmwareSlotLog")
            .field("active_slot", &self.active_slot())
            .field("next_slot_to_activate", &self.next_slot_to_activate())
            .field("slot_1", &self.slot_firmware(1).ok())
            .field("slot_2", &self.slot_firmware(2).ok())
            .field("slot_3", &self.slot_firmware(3).ok())
            .finish()
    }
}

/// One entry from the Error Information log page (LID 01h).
///
/// The error log is a ring buffer; entry `0` is the most recent and
/// `error_count == 0` indicates an unused slot.
#[derive(Clone, Copy)]
pub struct ErrorLogEntry {
    pub(crate) inner: nvme_error_log_page,
}

impl ErrorLogEntry {
    /// Monotonic counter; `0` indicates this slot has never recorded an error.
    pub fn error_count(&self) -> u64 {
        self.inner.error_count
    }

    /// Submission Queue ID associated with the error.
    pub fn submission_queue_id(&self) -> u16 {
        self.inner.sqid
    }

    /// Command ID of the failing command.
    pub fn command_id(&self) -> u16 {
        self.inner.cmdid
    }

    /// Status field of the completion entry (NVMe status code in low bits,
    /// status code type in upper bits).
    pub fn status_field(&self) -> u16 {
        self.inner.status_field
    }

    /// Byte/bit offset within the command parameters that caused the error,
    /// if applicable.
    pub fn parameter_error_location(&self) -> u16 {
        self.inner.parm_error_location
    }

    /// First LBA of the failure, when relevant for the operation.
    pub fn lba(&self) -> u64 {
        self.inner.lba
    }

    /// Namespace ID for the failing command (`0` if not namespace-scoped).
    pub fn nsid(&self) -> u32 {
        self.inner.nsid
    }

    /// Vendor-specific information byte.
    pub fn vendor_specific(&self) -> u8 {
        self.inner.vs
    }

    /// Transport type for Fabrics errors.
    pub fn transport_type(&self) -> u8 {
        self.inner.trtype
    }

    /// Command Set Identifier for the failing command.
    pub fn csi(&self) -> u8 {
        self.inner.csi
    }

    /// Opcode of the failing command.
    pub fn opcode(&self) -> u8 {
        self.inner.opcode
    }
}

impl std::fmt::Debug for ErrorLogEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErrorLogEntry")
            .field("error_count", &self.error_count())
            .field(
                "status_field",
                &format_args!("0x{:04x}", self.status_field()),
            )
            .field("opcode", &format_args!("0x{:02x}", self.opcode()))
            .field("nsid", &self.nsid())
            .field("lba", &self.lba())
            .finish()
    }
}
