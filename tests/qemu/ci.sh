#!/usr/bin/env bash
# Headless / non-interactive runner for the QEMU NVMe fixture.
#
# Boots the fixture in the background, waits for SSH to come up,
# runs `cargo test` + `cargo run --example io_smoke` inside the
# guest, captures exit codes, and shuts down. Returns 0 only if
# everything inside the guest succeeded.
#
# Usage (CI):
#   bash tests/qemu/ci.sh
#
# Usage (local, for parity with CI):
#   bash tests/qemu/ci.sh
#
# Environment knobs:
#   GUEST_BOOT_TIMEOUT  — seconds to wait for SSH on port 2222 (default 600)
#   GUEST_TEST_TIMEOUT  — seconds to wait for `cargo test` (default 1800)
#   QEMU_EXTRA_ARGS     — additional `qemu-system-x86_64` args
#   KEEP_RUNNING        — set to non-empty to leave QEMU running after success

set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$DIR/../.." && pwd)"
CACHE="$DIR/.cache"
mkdir -p "$CACHE"

UBUNTU_BASE="$CACHE/ubuntu-24.04-base.qcow2"
ROOT_OVERLAY="$CACHE/root.qcow2"
SEED_IMG="$CACHE/seed.img"
NVME_IMG="$CACHE/nvme.img"
SSH_KEY="$CACHE/id_ed25519_ci"
QEMU_PID_FILE="$CACHE/qemu.pid"
QEMU_LOG="$CACHE/qemu.log"

UBUNTU_URL="https://cloud-images.ubuntu.com/releases/24.04/release/ubuntu-24.04-server-cloudimg-amd64.img"

GUEST_BOOT_TIMEOUT="${GUEST_BOOT_TIMEOUT:-600}"
GUEST_TEST_TIMEOUT="${GUEST_TEST_TIMEOUT:-1800}"

# -----------------------------------------------------------------------
# Tear-down (run on any exit).
# -----------------------------------------------------------------------
cleanup() {
    local exit_code=$?
    if [[ -n "${KEEP_RUNNING:-}" && $exit_code -eq 0 ]]; then
        echo "==> KEEP_RUNNING set, leaving QEMU running (pid $(cat "$QEMU_PID_FILE" 2>/dev/null || echo '?'))"
        return
    fi
    if [[ -f "$QEMU_PID_FILE" ]]; then
        local pid
        pid="$(cat "$QEMU_PID_FILE")"
        if kill -0 "$pid" 2>/dev/null; then
            echo "==> shutting down QEMU (pid $pid)"
            kill -TERM "$pid" 2>/dev/null || true
            for _ in $(seq 1 30); do
                if ! kill -0 "$pid" 2>/dev/null; then break; fi
                sleep 1
            done
            kill -KILL "$pid" 2>/dev/null || true
        fi
        rm -f "$QEMU_PID_FILE"
    fi
    if [[ $exit_code -ne 0 && -f "$QEMU_LOG" ]]; then
        echo "==> last 100 lines of QEMU console log:"
        tail -n 100 "$QEMU_LOG" || true
    fi
}
trap cleanup EXIT

# -----------------------------------------------------------------------
# Generate ephemeral SSH key for guest access.
# -----------------------------------------------------------------------
if [[ ! -f "$SSH_KEY" ]]; then
    echo "==> generating ephemeral SSH key for CI"
    ssh-keygen -t ed25519 -N '' -f "$SSH_KEY" -C 'libnvme-rs-ci' -q
fi
PUB_KEY="$(cat "$SSH_KEY.pub")"

# -----------------------------------------------------------------------
# Patch cloud-init's user-data to inject the CI public key. We don't
# modify the file on disk — instead we write a temp copy.
# -----------------------------------------------------------------------
USERDATA_CI="$CACHE/user-data.ci"
{
    cat "$DIR/cloud-init/user-data"
    cat <<EOF

# Injected by tests/qemu/ci.sh — drop the CI ed25519 pubkey into the
# tester account so the CI script can run \`ssh tester@127.0.0.1\`.
ssh_authorized_keys:
  - $PUB_KEY
EOF
} > "$USERDATA_CI"

# -----------------------------------------------------------------------
# Download / build images.
# -----------------------------------------------------------------------
if [[ ! -f "$UBUNTU_BASE" ]]; then
    echo "==> downloading Ubuntu 24.04 server cloud image (~600 MB)"
    wget -q --show-progress -O "$UBUNTU_BASE.partial" "$UBUNTU_URL"
    mv "$UBUNTU_BASE.partial" "$UBUNTU_BASE"
fi

if [[ ! -f "$ROOT_OVERLAY" ]]; then
    echo "==> creating root overlay"
    qemu-img create -q -f qcow2 -F qcow2 -b "$UBUNTU_BASE" "$ROOT_OVERLAY" 10G
fi

echo "==> generating cloud-init seed (with CI pubkey)"
cloud-localds "$SEED_IMG" "$USERDATA_CI" "$DIR/cloud-init/meta-data"

if [[ ! -f "$NVME_IMG" ]]; then
    echo "==> creating 1 GiB virtual NVMe backing file"
    qemu-img create -q -f raw "$NVME_IMG" 1G
fi

# -----------------------------------------------------------------------
# Pick KVM vs TCG. GH-hosted runners don't have /dev/kvm; TCG fallback
# is slow but functional.
# -----------------------------------------------------------------------
KVM_ARG=()
if [[ -r /dev/kvm && -w /dev/kvm ]]; then
    KVM_ARG=(-enable-kvm -cpu host)
    echo "==> KVM available, using hardware acceleration"
else
    KVM_ARG=(-cpu max)
    echo "==> /dev/kvm not accessible, falling back to TCG (slow but works)"
fi

# -----------------------------------------------------------------------
# Boot QEMU in the background, redirecting console to a log file.
# -----------------------------------------------------------------------
echo "==> booting QEMU (background)"
nohup qemu-system-x86_64 \
    "${KVM_ARG[@]}" \
    -m 2G -smp 2 \
    -display none \
    -serial file:"$QEMU_LOG" \
    -drive file="$ROOT_OVERLAY",if=virtio,format=qcow2 \
    -drive file="$SEED_IMG",if=virtio,format=raw \
    -drive file="$NVME_IMG",if=none,id=nvmedrive,format=raw \
    -device nvme,drive=nvmedrive,serial=DEADBEEF12345 \
    -netdev user,id=net0,hostfwd=tcp::2222-:22 \
    -device virtio-net-pci,netdev=net0 \
    -virtfs local,path="$REPO",mount_tag=hostshare,security_model=mapped-xattr,id=hostshare \
    ${QEMU_EXTRA_ARGS:-} \
    >/dev/null 2>&1 &
echo $! > "$QEMU_PID_FILE"
QEMU_PID="$(cat "$QEMU_PID_FILE")"
echo "==> qemu pid=$QEMU_PID"

# -----------------------------------------------------------------------
# Wait for SSH on port 2222. cloud-init provisioning (apt update +
# rustup + 9p mount) usually finishes in ~3–5 minutes; TCG-mode boot
# can push that to ~10 minutes on first run.
# -----------------------------------------------------------------------
echo "==> waiting for SSH on 127.0.0.1:2222 (max ${GUEST_BOOT_TIMEOUT}s)"
SSH_OPTS=(
    -i "$SSH_KEY"
    -p 2222
    -o StrictHostKeyChecking=no
    -o UserKnownHostsFile=/dev/null
    -o ConnectTimeout=5
    -o LogLevel=ERROR
)
deadline=$(( $(date +%s) + GUEST_BOOT_TIMEOUT ))
until ssh "${SSH_OPTS[@]}" tester@127.0.0.1 "true" 2>/dev/null; do
    if (( $(date +%s) > deadline )); then
        echo "==> ERROR: SSH never came up within ${GUEST_BOOT_TIMEOUT}s"
        exit 1
    fi
    if ! kill -0 "$QEMU_PID" 2>/dev/null; then
        echo "==> ERROR: QEMU process exited before SSH came up"
        exit 1
    fi
    sleep 5
done

echo "==> SSH up; cloud-init may still be finishing — waiting for status=done"
ssh "${SSH_OPTS[@]}" tester@127.0.0.1 \
    "until cloud-init status --wait 2>/dev/null | grep -q 'status: done'; do sleep 5; done"
echo "==> cloud-init finished"

# -----------------------------------------------------------------------
# Confirm the virtual NVMe device is present.
# -----------------------------------------------------------------------
echo "==> guest: verifying NVMe device"
ssh "${SSH_OPTS[@]}" tester@127.0.0.1 "sudo nvme list"

# -----------------------------------------------------------------------
# Run the test suite + smoke examples inside the guest.
# Everything runs as root (`sudo -E`) because Format/Sanitize/I/O
# commands need /dev/nvme0 privileged access.
#
# Build artifacts go to /tmp/target (guest-local fs) — keep them off
# the 9p mount, where writes are an order of magnitude slower.
# -----------------------------------------------------------------------
echo "==> guest: building + testing libnvme-rs"
ssh "${SSH_OPTS[@]}" tester@127.0.0.1 bash <<'GUEST'
set -euxo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_TARGET_DIR=/tmp/target

cd /mnt/host

# Build first (cargo test will rebuild but the cache helps).
cargo build --workspace --all-targets

# Privileged tests + smoke examples run as root.
sudo -E env "PATH=$PATH" cargo test --workspace
sudo -E env "PATH=$PATH" cargo run --example io_smoke -p libnvme
GUEST

echo "==> guest tests passed"
