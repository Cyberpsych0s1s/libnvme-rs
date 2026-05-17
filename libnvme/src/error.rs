use std::io;
use std::str::Utf8Error;

/// Errors returned by this crate.
///
/// Marked `#[non_exhaustive]` so future variants (notably `Nvme(NvmeStatus)`
/// for device-reported status codes) can be added without a breaking release.
#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    /// A libnvme call failed at the OS level. The wrapped [`io::Error`]
    /// captures the platform's `errno`.
    Os(io::Error),

    /// libnvme returned NULL or a sentinel indicating the requested value
    /// is not set on this handle.
    NotAvailable,

    /// A C string returned by libnvme contained invalid UTF-8.
    Utf8(Utf8Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Os(e) => write!(f, "libnvme: {e}"),
            Error::NotAvailable => write!(f, "libnvme: value not available"),
            Error::Utf8(e) => write!(f, "libnvme: invalid UTF-8: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Os(e) => Some(e),
            Error::Utf8(e) => Some(e),
            Error::NotAvailable => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Os(e)
    }
}

impl From<Utf8Error> for Error {
    fn from(e: Utf8Error) -> Self {
        Error::Utf8(e)
    }
}

/// Result type alias for fallible operations in this crate.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    fn enoent() -> Error {
        io::Error::from_raw_os_error(2).into()
    }

    #[test]
    fn display_has_libnvme_prefix() {
        assert!(format!("{}", enoent()).starts_with("libnvme: "));
    }

    #[test]
    fn source_is_inner_io_error() {
        let err = enoent();
        let source = std::error::Error::source(&err).expect("source should exist");
        assert!(source.downcast_ref::<io::Error>().is_some());
    }

    #[test]
    fn from_io_error_yields_os_variant() {
        assert!(matches!(enoent(), Error::Os(_)));
    }

    #[test]
    fn not_available_has_no_source() {
        let err = Error::NotAvailable;
        assert!(std::error::Error::source(&err).is_none());
    }

    #[test]
    #[allow(invalid_from_utf8)]
    fn utf8_error_displays_with_prefix() {
        let bad = std::str::from_utf8(&[0xFFu8, 0xFE]).unwrap_err();
        let err: Error = bad.into();
        assert!(format!("{err}").starts_with("libnvme: invalid UTF-8"));
    }
}
