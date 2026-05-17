#!/usr/bin/env bash
# Boot an Ubuntu 24.04 cloud image in QEMU with a virtual NVMe device
# attached, so libnvme-rs's destructive operations (Format / NS-mgmt /
# FW-commit) can be exercised against an NVMe controller we can break
# repeatedly without touching the host's real drives.
#
# Phase 1 (this script): boot, log in via console (tester / nvme),
# manually confirm `lsblk` and `nvme list` see the virtual device.
# Phase 2 (future): cloud-init-driven cargo test runs via 9p mount.

set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CACHE="$DIR/.cache"
mkdir -p "$CACHE"

UBUNTU_BASE="$CACHE/ubuntu-24.04-base.qcow2"
ROOT_OVERLAY="$CACHE/root.qcow2"
SEED_IMG="$CACHE/seed.img"
NVME_IMG="$CACHE/nvme.img"

UBUNTU_URL="https://cloud-images.ubuntu.com/releases/24.04/release/ubuntu-24.04-server-cloudimg-amd64.img"

if [[ ! -f "$UBUNTU_BASE" ]]; then
    echo "==> downloading Ubuntu 24.04 server cloud image (~600 MB)"
    wget --show-progress -O "$UBUNTU_BASE.partial" "$UBUNTU_URL"
    mv "$UBUNTU_BASE.partial" "$UBUNTU_BASE"
fi

# Copy-on-write overlay so the base image stays pristine.
if [[ ! -f "$ROOT_OVERLAY" ]]; then
    echo "==> creating root overlay (qcow2 backed by base, 10 GiB virtual)"
    qemu-img create -q -f qcow2 -F qcow2 -b "$UBUNTU_BASE" "$ROOT_OVERLAY" 10G
fi

echo "==> generating cloud-init seed"
cloud-localds "$SEED_IMG" "$DIR/cloud-init/user-data" "$DIR/cloud-init/meta-data"

if [[ ! -f "$NVME_IMG" ]]; then
    echo "==> creating 1 GiB virtual NVMe backing file"
    qemu-img create -q -f raw "$NVME_IMG" 1G
fi

KVM_ARG=()
if [[ -r /dev/kvm && -w /dev/kvm ]]; then
    KVM_ARG=(-enable-kvm -cpu host)
else
    echo "==> /dev/kvm not accessible, falling back to TCG (slow but works)"
    KVM_ARG=(-cpu max)
fi

echo "==> booting QEMU with virtual NVMe device"
echo "    login: tester / nvme   (Ctrl-A then x to quit)"
echo

exec qemu-system-x86_64 \
    "${KVM_ARG[@]}" \
    -m 2G -smp 2 \
    -nographic \
    -drive file="$ROOT_OVERLAY",if=virtio,format=qcow2 \
    -drive file="$SEED_IMG",if=virtio,format=raw \
    -drive file="$NVME_IMG",if=none,id=nvmedrive,format=raw \
    -device nvme,drive=nvmedrive,serial=DEADBEEF12345,model="QEMU NVMe Test" \
    -netdev user,id=net0,hostfwd=tcp::2222-:22 \
    -device virtio-net-pci,netdev=net0
