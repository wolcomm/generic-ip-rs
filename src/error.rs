use core::fmt;

use crate::{af::Afi, primitive::AddressPrimitive};

#[derive(Debug)]
pub struct Error<'a, A: Afi, P: AddressPrimitive<A>> {
    kind: ErrorKind<A, P>,
    msg: Option<&'a str>,
    source: Option<SourceError<'a>>,
}

impl<'a, A: Afi, P: AddressPrimitive<A>> Error<'a, A, P> {
    pub(crate) fn new<S: AsRef<str> + ?Sized + 'a>(
        kind: ErrorKind<A, P>,
        msg: Option<&'a S>,
        source: Option<SourceError<'a>>,
    ) -> Self {
        Self {
            kind,
            msg: msg.map(S::as_ref),
            source,
        }
    }
}

impl<A: Afi, P: AddressPrimitive<A>> fmt::Display for Error<'_, A, P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(msg) = self.msg {
            write!(f, "{}: {}", self.kind, msg)
        } else {
            self.kind.fmt(f)
        }
    }
}

#[cfg(feature = "std")]
impl<A: Afi, P: AddressPrimitive<A>> std::error::Error for Error<'_, A, P> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
    }
}

#[cfg(not(feature = "std"))]
impl<'a, A: Afi, P: AddressPrimitive<A>> Error<'a, A, P> {
    pub fn source(&self) -> Option<SourceError<'a>> {
        self.source
    }
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum ErrorKind<A: Afi, P: AddressPrimitive<A>> {
    PrefixLength(P::Width),
    ParserError,
}

impl<A: Afi, P: AddressPrimitive<A>> fmt::Display for ErrorKind<A, P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::PrefixLength(len) => {
                write!(f, "{} prefix-length {} out of bounds", A::as_enum(), len)
            }
            Self::ParserError => write!(f, "parser error"),
        }
    }
}

#[cfg(feature = "std")]
type SourceError<'a> = &'a (dyn std::error::Error + 'static);
#[cfg(not(feature = "std"))]
type SourceError<'a> = &'a (dyn core::any::Any);

macro_rules! err {
    ( $kind:expr ) => {
        $crate::error::Error::new::<&'static str>($kind, None, None)
    };
    ( $kind:expr, $msg:expr ) => {
        $crate::error::Error::new($kind, Some($msg), None)
    };
}
pub(crate) use err;
