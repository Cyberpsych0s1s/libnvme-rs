//! Re-exports of libnvme's feature-data structs so callers don't need
//! `libnvme-sys` for the typed `set_*` / `get_*` helpers that take buffers.

use libnvme_sys::{
    nvme_feat_auto_pst, nvme_feat_host_behavior, nvme_host_mem_buf_attrs, nvme_lba_range_type,
    nvme_plm_config, nvme_timestamp,
};

pub type LbaRangeType = nvme_lba_range_type;
pub type AutoPst = nvme_feat_auto_pst;
pub type Timestamp = nvme_timestamp;
pub type HostBehavior = nvme_feat_host_behavior;
pub type PlmConfig = nvme_plm_config;
pub type HostMemBufAttrs = nvme_host_mem_buf_attrs;
