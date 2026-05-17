# QEMU NVMe Test Fixture

A virtual NVMe controller in a Linux guest, so libnvme-rs's destructive
operations (Format NVM, namespace create / delete, firmware commit) can
be exercised against a real NVMe protocol stack without touching the
host's actual drives.

## Phase 1 (current): boot and manual verification

```sh
bash tests/qemu/run.sh
```

First invocation downloads the Ubuntu 24.04 cloud image (~600 MB) into
`tests/qemu/.cache/`. Subsequent invocations skip the download.

The script boots an Ubuntu 24.04 guest with:

- 2 GiB RAM, 2 vCPUs, KVM acceleration (or TCG fallback)
- Virtio root + cloud-init seed
- **A 1 GiB virtual NVMe drive** as `/dev/nvme0`, model `"QEMU NVMe
  Test"`, serial `DEADBEEF12345`
- SSH forwarded to host port 2222

Login on the serial console as **`tester` / `nvme`**, then verify the
virtual NVMe is present:

```sh
lsblk
sudo nvme list
sudo nvme id-ctrl /dev/nvme0
```

Quit QEMU with **Ctrl-A then `x`**.

## Phase 2 (next): automated `cargo test` inside the guest

To be added. Plan:

1. Cloud-init installs `build-essential`, `libnvme-dev`, `clang`,
   `libclang-dev`, `pkg-config`, `nvme-cli`, and `rustup`.
2. The host repository is mounted into the guest via 9p
   (`-virtfs local,path=...,mount_tag=hostshare`).
3. A `runtests.sh` script inside the guest runs `cargo test
   --test qemu_*` against the virtual NVMe and writes results back to
   the host through the 9p share.
4. Host-side `tests/qemu/run.sh test` orchestrates the whole thing,
   returning the guest's exit code.

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
