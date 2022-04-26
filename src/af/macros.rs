macro_rules! afi_definitions {
    (
        $(
            $( #[$class_attr:meta] )*
            $class_vis:vis class $class_name:ident {
                type Address = $address_ty:ty;
                type PrefixLength = $prefix_len_ty:ty;
                type Prefix = $prefix_ty:ty;
                type Netmask = $netmask_ty:ty;
                type Hostmask = $hostmask_ty:ty;
                $(
                    $( #[$afi_attr:meta] )*
                    $afi_vis:vis afi $afi_name:ident ($type_var:ident) {
                        type Octets = $octets_ty:ty;
                        type DefaultPrimitive = $default_primitive_ty:ty;
                        $(
                            primitive $primitive_ty:ty {
                                type Width = $width_ty:ty;
                                const MAX_LENGTH = $max_length:expr;
                                const ZERO = $zero:expr;
                                const ONES = $ones:expr;
                                const BROADCAST = $broadcast:expr;
                                const LOCALHOST = $localhost:expr;
                                const UNSPECIFIED = $unspecified:expr;
                                const LOCALHOST_NET = ($localhost_net:expr, $localhost_len:expr);
                                const BENCHMARK_NET = ($benchmark_net:expr, $benchmark_len:expr);
                                const MULTICAST_NET = ($multicast_net:expr, $multicast_len:expr);
                                fn parse_addr = $parse_addr_fn:path;
                                fn parse_prefix = $parse_prefix_fn:path;
                            }
                        )*
                    }
                )*
            }
        )*
    ) => {
        $(
            $(
                $(#[$afi_attr])*
                #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
                $afi_vis enum $afi_name {}

                impl Afi for $afi_name {
                    // TODO: this should be expressed as `[u8; N]` where `N` is
                    // an associated const
                    type Octets = $octets_ty;
                    fn as_enum() -> AfiEnum {
                        AfiEnum::$afi_name
                    }
                }

                impl DefaultPrimitives for $afi_name {
                    type Type = $default_primitive_ty;
                }

                $(
                    impl AddressPrimitive<$afi_name> for $primitive_ty {
                        type Width = $width_ty;

                        const MAX_LENGTH: Self::Width = $max_length;
                        const ZERO: Self = $zero;
                        const ONES: Self = $ones;

                        const BROADCAST: Option<Self> = $broadcast;
                        const LOCALHOST: Self = $localhost;
                        const UNSPECIFIED: Self = $unspecified;

                        const LOCALHOST_NET: (Self, Self::Width) = ($localhost_net, $localhost_len);
                        const BENCHMARK_NET: (Self, Self::Width) = ($benchmark_net, $benchmark_len);
                        const MULTICAST_NET: (Self, Self::Width) = ($multicast_net, $multicast_len);

                        fn leading_zeros(self) -> Self::Width {
                            self.leading_zeros() as Self::Width
                        }

                        fn to_be_bytes(self) -> <$afi_name as Afi>::Octets {
                            self.to_be_bytes()
                        }

                        fn from_be_bytes(bytes: <$afi_name as Afi>::Octets) -> Self {
                            Self::from_be_bytes(bytes)
                        }

                        fn parse_addr<S>(s: &S) -> Result<Self, Error<'static, $afi_name, $primitive_ty>>
                        where
                            S: AsRef<str> + ?Sized,
                        {
                            $parse_addr_fn(s.as_ref())
                        }

                        fn parse_prefix<S>(s: &S) -> Result<(Self, Self::Width), Error<'static, $afi_name, $primitive_ty>>
                        where
                            S: AsRef<str> + ?Sized,
                        {
                            $parse_prefix_fn(s.as_ref())
                        }
                    }
                )*
            )*

            $(#[$class_attr])*
            #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
            $class_vis enum $class_name {}

            impl Afis for $class_name {}

            impl<$( $type_var, )*> Primitives<$class_name> for ($( $type_var, )*)
            where
                $( $type_var: AddressPrimitive<$afi_name>, )*
            {}

            impl DefaultPrimitives for $class_name {
                type Type = (
                    $( <$afi_name as DefaultPrimitives>::Type, )*
                );
            }

            impl<$( $type_var, )*> AfiClass<$class_name, ($( $type_var, )*)> for $class_name
            where
                $( $type_var: AddressPrimitive<$afi_name>, )*
            {
                type Address = $address_ty;
                type PrefixLength = $prefix_len_ty;
                type Prefix = $prefix_ty;
                type Netmask = $netmask_ty;
                type Hostmask = $hostmask_ty;
            }
        )*

        /// Enumeration of IP address families.
        pub enum AfiEnum {
            $( $(
                /// $afi_name address family variant.
                $afi_name,
            )* )*
        }

        impl core::fmt::Display for AfiEnum {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                match self {
                    $( $(
                        Self::$afi_name => write!(f, stringify!($afi_name)),
                    )* )*
                }
            }
        }
    };
}
pub(super) use afi_definitions;
