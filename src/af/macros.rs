macro_rules! afi_definitions {
    (
        $(
            $( #[$attr:meta] )*
            $vis:vis afi $name:ident {
                type Addr = $addr_ty:ty;
                fn parse_addr = $parse_addr_fn:path;
                fn parse_prefix = $parse_prefix_fn:path;
            }
        )*
    ) => {
        $(
            $(#[$attr])*
            #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
            $vis enum $name {}

            impl Afi for $name {
                type Addr = $addr_ty;

                fn as_enum() -> AfiEnum {
                    AfiEnum::$name
                }

                fn parse_addr<S>(s: &S) -> Result<Self::Addr, Error<'static, Self>>
                where
                    S: AsRef<str> + ?Sized,
                {
                    $parse_addr_fn(s.as_ref())
                }

                fn parse_prefix<S>(s: &S) -> Result<(Self::Addr, WidthOf<Self::Addr>), Error<'static, Self>>
                where
                    S: AsRef<str> + ?Sized,
                {
                    $parse_prefix_fn(s.as_ref())
                }
            }

        )*

        /// Enumeration of IP address families.
        pub enum AfiEnum {
            $(
                /// $name address family variant.
                $name,
            )*
        }

        impl core::fmt::Display for AfiEnum {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                match self {
                    $(
                        Self::$name => write!(f, stringify!($name)),
                    )*
                }
            }
        }
    };
}
pub(super) use afi_definitions;
