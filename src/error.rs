use core::fmt;

/// An error describing a failed operation on an IP object.
#[derive(Clone, Copy, Debug)]
pub struct Error {
    kind: Kind,
    msg: Option<&'static str>,
    source: Option<SourceError>,
}

impl Error {
    pub(crate) fn new<S: AsRef<str> + ?Sized + 'static>(
        kind: Kind,
        msg: Option<&'static S>,
        source: Option<SourceError>,
    ) -> Self {
        Self {
            kind,
            msg: msg.map(S::as_ref),
            source,
        }
    }

    /// Returns the [`Kind`] of error.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{error::Kind, Address, Ipv4};
    ///
    /// let err = "10.0.0.256".parse::<Address<Ipv4>>().unwrap_err();
    /// assert_eq!(err.kind(), Kind::ParserError);
    /// ```
    #[must_use]
    pub const fn kind(&self) -> Kind {
        self.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(msg) = self.msg {
            write!(f, "{}: {}", self.kind, msg)
        } else {
            self.kind.fmt(f)
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<SourceError> {
        self.source
    }
}

#[cfg(not(feature = "std"))]
impl Error {
    /// Returns the underyling source error, if it exists.
    ///
    /// This method is provided for interface compatibility with
    /// `std::error::Error` in a `no_std` environment.
    #[must_use]
    pub fn source(&self) -> Option<SourceError> {
        self.source
    }
}

/// The "kind" of an [`Error`].
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum Kind {
    /// An [`Error`] resulting from an operation on a prefix-length.
    PrefixLength,
    /// An [`Error`] resulting from a parser failure.
    ParserError,
    /// An [`Error`] resulting from an attempt to convert between incompatible
    /// address families.
    AfiMismatch,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PrefixLength => {
                write!(f, "prefix-length out of bounds")
            }
            Self::ParserError => write!(f, "parser error"),
            Self::AfiMismatch => write!(f, "address family mis-match"),
        }
    }
}

#[cfg(feature = "std")]
type SourceError = &'static (dyn std::error::Error + 'static);
#[cfg(not(feature = "std"))]
type SourceError = &'static (dyn core::any::Any);

macro_rules! err {
    ( $kind:expr ) => {
        $crate::error::Error::new::<&'static str>($kind, None, None)
    };
    ( $kind:expr, $msg:expr ) => {
        $crate::error::Error::new($kind, Some($msg), None)
    };
}
pub(crate) use err;
