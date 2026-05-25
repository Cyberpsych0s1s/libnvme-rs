# Security Policy

## Reporting a vulnerability

`libnvme-rs` is a safe Rust wrapper over a C library that issues
privileged NVMe admin and I/O commands to block devices. A soundness bug
in the wrapper could plausibly allow arbitrary memory corruption in a
process that already has access to `/dev/nvme*`, or a destructive
command issued against the wrong device.

**Please report security-sensitive issues privately**, not on the public
issue tracker:

- Preferred: open a [GitHub Security
  Advisory](https://github.com/Cyberpsych0s1s/libnvme-rs/security/advisories/new)
  on this repository (private until disclosed).
- Alternative: email the maintainer listed in `Cargo.toml`'s `authors`
  field. Use a subject line starting with `[libnvme-rs security]`.

Please include:

- A description of the issue and which version(s) are affected.
- A reproducer if you have one (a minimal Rust program, a QEMU
  invocation, or a description of the call sequence that triggers the
  bug).
- The impact you observed, and what you think the worst-case impact
  could be.

I'll acknowledge within 5 business days and aim to ship a fix within
30 days for high-severity issues. Coordinated-disclosure timelines
are negotiable.

## What counts as a security issue

- Memory-safety violations (UAF, double-free, OOB read/write) reachable
  from safe Rust code in `libnvme` (the wrapper crate).
- Destructive commands issued against the wrong target due to an
  off-by-one or aliasing bug in the wrapper (e.g. Format NVM hits the
  wrong NSID, namespace-delete acts on a stale handle).
- Soundness holes — APIs marked safe that can be misused to corrupt
  memory or trigger UB without any `unsafe` from the caller.

## What doesn't

- Bugs in `libnvme` itself (the C library) — report those at
  <https://github.com/linux-nvme/libnvme>.
- Bugs in the Linux kernel's NVMe driver — report to
  `linux-nvme@lists.infradead.org`.
- Userspace requiring `CAP_SYS_ADMIN` to talk to NVMe character devices
  is by design, not a wrapper bug.
- Destructive commands doing what the docs say they do (e.g. Sanitize
  erasing all user data on the namespace) — that's the documented
  behavior. The wrapper does carry `# Warning` doc-blocks on every
  destructive entry point to reduce the chance of accidental
  invocation.

## Supported versions

Only the latest minor-version line gets security updates. While the
crate is pre-1.0 (`0.x.y`), the most recent `0.x` series is supported.

| Version | Supported |
|---------|-----------|
| 0.7.x   | ✓         |
| < 0.7   | ✗         |
