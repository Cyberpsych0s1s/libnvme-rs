#!/bin/bash
cd "$(dirname "$0")"
F=$(find target -name bindings.rs -print -quit)
echo "=== nvmf_disc_log_entry ==="
grep -A 30 "^pub struct nvmf_disc_log_entry " "$F"
echo
echo "=== nvmf_discovery_log ==="
grep -A 12 "^pub struct nvmf_discovery_log " "$F"
echo
echo "=== LID for discovery ==="
grep -E "NVME_LOG_LID_DISC" "$F"
echo
echo "=== nvmf_tsas_tcp / nvmf_tsas_rdma ==="
grep -A 8 "^pub struct nvmf_tsas_tcp " "$F"
echo
grep -A 8 "^pub struct nvmf_tsas_rdma " "$F"
