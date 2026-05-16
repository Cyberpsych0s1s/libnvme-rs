use std::io;

#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    Os(io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Os(e) => write!(f, "libnvme: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Os(e) => Some(e),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Os(e)
    }
}

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
}
