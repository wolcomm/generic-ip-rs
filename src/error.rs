use core::fmt;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    msg: Option<&'static str>,
    source: Option<SourceError>,
}

impl Error {
    pub(crate) fn new<S: AsRef<str> + ?Sized + 'static>(
        kind: ErrorKind,
        msg: Option<&'static S>,
        source: Option<SourceError>,
    ) -> Self {
        Self {
            kind,
            msg: msg.map(S::as_ref),
            source,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(msg) = self.msg {
            write!(f, "{}: {}", self.kind, msg)
        } else {
            self.kind.fmt(f)
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
    }
}

#[cfg(not(feature = "std"))]
impl Error {
    pub fn source(&self) -> Option<SourceError> {
        self.source
    }
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum ErrorKind {
    PrefixLength,
    ParserError,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::PrefixLength => {
                write!(f, "prefix-length out of bounds")
            }
            Self::ParserError => write!(f, "parser error"),
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
