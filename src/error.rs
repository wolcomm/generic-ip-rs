use core::fmt;

use crate::traits::{primitive, Afi, AfiClass};

#[derive(Debug)]
pub struct Error<A: Afi> {
    kind: ErrorKind<A>,
    msg: Option<&'static str>,
    source: Option<SourceError>,
}

impl<A: Afi> Error<A> {
    pub(crate) fn new<S: AsRef<str> + ?Sized + 'static>(
        kind: ErrorKind<A>,
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

impl<A: Afi> fmt::Display for Error<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(msg) = self.msg {
            write!(f, "{}: {}", self.kind, msg)
        } else {
            self.kind.fmt(f)
        }
    }
}

#[cfg(feature = "std")]
impl<A: Afi> std::error::Error for Error<A> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
    }
}

#[cfg(not(feature = "std"))]
impl<A: Afi> Error<A> {
    pub fn source(&self) -> Option<SourceError> {
        self.source
    }
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum ErrorKind<A: Afi> {
    PrefixLength(<A::Primitive as primitive::Address<A>>::Length),
    ParserError,
}

impl<A: Afi> fmt::Display for ErrorKind<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::PrefixLength(len) => {
                write!(f, "{} prefix-length {} out of bounds", A::as_afi(), len)
            }
            Self::ParserError => write!(f, "parser error"),
        }
    }
}

#[cfg(feature = "std")]
type SourceError = &(dyn std::error::Error + 'static);
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
