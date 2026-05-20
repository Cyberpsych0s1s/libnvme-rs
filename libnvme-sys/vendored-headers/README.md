# Vendored libnvme headers

These are unmodified header files from
[linux-nvme/libnvme](https://github.com/linux-nvme/libnvme), included in
this crate so that documentation builds (notably docs.rs) can run
`bindgen` without needing the `libnvme-dev` system package installed.

**On normal user builds** (your machine, CI, anywhere `libnvme-dev` is
available), `build.rs` ignores this directory and uses the system
headers found via `pkg-config`. The vendored copy is only consulted
when the `DOCS_RS` environment variable is set (which docs.rs's
sandboxed builder always sets).

## License

The headers carry their original `SPDX-License-Identifier:
LGPL-2.1-or-later` notice. They remain under that license; vendoring
does not relicense them. The rest of this crate is dual-licensed
Apache-2.0 OR MIT (see [LICENSE-APACHE](../../LICENSE-APACHE) and
[LICENSE-MIT](../../LICENSE-MIT)).

Header files are interface declarations and including them in a
downstream crate is the well-established pattern for `-sys` crates
that wrap GPL/LGPL libraries (see `openssl-sys`, `gtk-sys`, etc.).
The runtime `libnvme` library itself is still LGPL and must be present
on the user's system at runtime — vendoring the headers does not
package the library.

## Refreshing

These headers are from libnvme 1.16.1. To refresh against a newer
upstream:

```sh
cp /usr/include/libnvme.h libnvme-sys/vendored-headers/
cp /usr/include/nvme/*.h libnvme-sys/vendored-headers/nvme/
```

then verify the crate still builds locally and on a system with the
matching `libnvme-dev` installed.
