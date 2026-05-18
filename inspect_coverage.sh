#!/bin/bash
set -e
F=$(find target -name bindings.rs -print -quit)

echo "=== Raw libnvme public functions exposed by bindgen ==="
total_fns=$(grep -c "^    pub fn nvme_" "$F")
echo "$total_fns"
echo

echo "=== Functions actually referenced by libnvme/src/ ==="
# Find every libnvme_sys::<symbol> use across the safe wrapper
used=$(grep -rEoh "nvme_[a-z][a-z_0-9]*" libnvme/src/ libnvme/build.rs | grep -E "^nvme_" | sort -u)
used_count=$(echo "$used" | wc -l)
echo "$used_count distinct symbols"
echo

echo "=== Breakdown by category ==="
echo "$used" | awk '
/^nvme_ctrl_/   { ctrl++; next }
/^nvme_ns_/    { ns++; next }
/^nvme_subsystem_/ { subsys++; next }
/^nvme_host_/  { host++; next }
/^nvme_path_/  { path++; next }
/^nvme_first_subsystem|^nvme_next_subsystem|^nvme_first_host|^nvme_next_host/ { iter++; next }
/^nvme_scan|^nvme_free_tree|^nvme_root/  { root++; next }
/^nvme_(identify|get_log|format_nvm|ns_mgmt|ns_attach|fw_download|fw_commit)/ { admin++; next }
/^nvme_id_ctrl|^nvme_id_ns|^nvme_smart_log|^nvme_error_log_page|^nvme_firmware_slot|^nvme_ctrl_list|^nvme_get_log_args|^nvme_format_nvm_args|^nvme_fw_commit_args|^nvme_ns_mgmt_args|^nvme_ns_attach_args|^nvme_lbaf|^nvme_cmd_/ { type_struct++; next }
{ other++ }
END {
  printf "  controller getters       : %d\n", ctrl
  printf "  namespace getters        : %d\n", ns
  printf "  subsystem getters        : %d\n", subsys
  printf "  host getters             : %d\n", host
  printf "  path getters             : %d\n", path
  printf "  tree iter / root         : %d\n", iter + root
  printf "  admin command fns        : %d\n", admin
  printf "  structs / type aliases   : %d\n", type_struct
  printf "  other (consts, enums)    : %d\n", other
}'

echo
echo "=== Total libnvme symbols matching common command-class prefixes ==="
for prefix in nvme_connect nvme_disconnect nvme_security nvme_dev_self_test nvme_set_features nvme_get_features nvme_sanitize nvme_zns nvme_dim nvme_mi_ nvme_admin_passthru; do
    count=$(grep -c "^    pub fn ${prefix}" "$F" 2>/dev/null || echo 0)
    printf "  %-30s %d\n" "$prefix*" "$count"
done
