# QEMU NVMe Test Fixture

A virtual NVMe controller in a Linux guest, so libnvme-rs's destructive
operations (Format NVM, namespace create / delete, firmware commit) can
be exercised against a real NVMe protocol stack without touching the
host's actual drives.

## Usage

```sh
bash tests/qemu/run.sh
```

First invocation downloads the Ubuntu 24.04 cloud image (~600 MB) into
`tests/qemu/.cache/` and provisions the guest via cloud-init (installs
`build-essential`, `pkg-config`, `libnvme-dev`, `clang`, `libclang-dev`,
`nvme-cli`, `rustup`, initialises Rust stable for the `tester` user, and
9p-mounts the host repo at `/mnt/host`). Provisioning takes ~3–5 minutes
including the rustup download. Subsequent invocations re-use the
overlay and boot in ~10 seconds.

The guest has:

- 2 GiB RAM, 2 vCPUs, KVM acceleration (or TCG fallback)
- Virtio root + cloud-init seed
- **A 1 GiB virtual NVMe drive** as `/dev/nvme0`, serial `DEADBEEF12345`
- 9p host-repo share mounted at `/mnt/host`
- SSH forwarded to host port 2222

Login on the serial console as **`tester` / `nvmenvme`** and verify the
virtual NVMe is present:

```sh
lsblk
sudo nvme list
sudo nvme id-ctrl /dev/nvme0
```

To build and test libnvme-rs inside the guest:

```sh
cd /mnt/host
cargo build --workspace --all-targets
sudo -E env "PATH=$PATH" cargo test --workspace
```

`CARGO_TARGET_DIR=/tmp/target` is pre-set in the `tester` user's
`.bashrc` so build artifacts go to the guest-local fs (fast) instead of
through the 9p mount (slow).

The `sudo -E env "PATH=$PATH"` preserves the cargo bin path and
environment when running tests as root — needed because the Identify
and SMART log admin commands require privileged access to `/dev/nvme0`.

Quit QEMU with **Ctrl-A then `x`**.

## Re-running cloud-init after changing `cloud-init/user-data`

Cloud-init only runs on first boot. Once it has provisioned a disk, it
records that and never runs again. To re-provision with edited user-data,
remove the overlay so it's rebuilt from the pristine base on next boot:

```sh
rm tests/qemu/.cache/root.qcow2
bash tests/qemu/run.sh
```

The base image stays cached, so this is fast.

## Cache contents

`tests/qemu/.cache/` (git-ignored) holds:

| File | Why |
|---|---|
| `ubuntu-24.04-base.qcow2` | The pristine Ubuntu cloud image. Never written to — root overlay is built on top |
| `root.qcow2` | Copy-on-write overlay. Throw it away to start clean |
| `seed.img` | Cloud-init seed ISO; regenerated each run from `cloud-init/` |
| `nvme.img` | Backing file for the virtual NVMe drive. Throw away to format-reset |

Delete the whole directory to fully reset the fixture.

## Why Ubuntu 24.04?

Same libnvme version as CI (`1.8`). Anything that builds and tests
cleanly here will pass CI; anything that fails here is a CI failure
caught before pushing.

When the destructive-op work needs to be tested against newer libnvme
(e.g. to confirm a probe gates correctly), swap the base image URL for
a rolling-release distro (Arch or Fedora cloud image).
