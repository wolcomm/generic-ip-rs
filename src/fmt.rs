use core::fmt;

use crate::{
    concrete::{Ipv4, Ipv6},
    traits::{
        primitive::{self, IntoIpv6Segments as _},
        Afi,
    },
};

#[derive(Copy, Clone, Default)]
struct Span {
    start: usize,
    length: usize,
}

fn fmt_segments(segments: &[u16], sep: char, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if let Some((first, tail)) = segments.split_first() {
        write!(f, "{:x}", first)?;
        tail.iter()
            .try_for_each(|segment| write!(f, "{}{:x}", sep, segment))?;
    }
    Ok(())
}

/// TODO: implement directly for `ConcreteAddress`, using `::octets()`
/// IP address style formatting.
pub trait AddressDisplay<A: Afi> {
    /// Format `Self` using the canical respresentation for IP addresses of
    /// address-family `A`.
    fn fmt_addr(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl<P: primitive::Address<Ipv4>> AddressDisplay<Ipv4> for P {
    fn fmt_addr(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_be_bytes().fmt_addr(f)
    }
}

#[allow(clippy::many_single_char_names)]
impl AddressDisplay<Ipv4> for <Ipv4 as Afi>::Octets {
    fn fmt_addr(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [a, b, c, d] = self;
        write!(f, "{}.{}.{}.{}", a, b, c, d)
    }
}

#[allow(clippy::many_single_char_names)]
impl<P: primitive::Address<Ipv6>> AddressDisplay<Ipv6> for P {
    fn fmt_addr(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.into_segments() {
            // TODO:
            // Use `P::UNSPECIFIED` and `P::LOCALHOST` to derive const
            // patterns here.
            // Needs `const_trait_impl`.
            [0, 0, 0, 0, 0, 0, 0, 0] => f.write_str("::"),
            [0, 0, 0, 0, 0, 0, 0, 1] => f.write_str("::1"),
            [0, 0, 0, 0, 0, mapped @ (0 | 0xffff), high, low] => {
                f.write_str("::")?;
                if mapped == 0xffff {
                    f.write_str("ffff:")?;
                }
                let [a, b] = high.to_be_bytes();
                let [c, d] = low.to_be_bytes();
                ([a, b, c, d]).fmt_addr(f)
            }
            segments => {
                let (head, tail) = {
                    let mut longest = Span::default();
                    let mut current = Span::default();
                    segments.iter().enumerate().for_each(|(i, segment)| {
                        if segment == &0 {
                            if current.length == 0 {
                                current.start = i;
                            }
                            current.length += 1;
                            if current.length > longest.length {
                                longest = current;
                            }
                        } else {
                            current = Span::default();
                        }
                    });
                    if longest.length > 1 {
                        (
                            &segments[0..longest.start],
                            Some(&segments[longest.start + longest.length..]),
                        )
                    } else {
                        (&segments[..], None)
                    }
                };
                fmt_segments(head, ':', f)?;
                if let Some(tail) = tail {
                    f.write_str("::")?;
                    fmt_segments(tail, ':', f)?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use core::fmt::Write;
    use core::marker::PhantomData;

    use super::*;

    struct FmtWrapper<A: Afi, T: AddressDisplay<A>> {
        inner: T,
        _marker: PhantomData<A>,
    }

    impl<A: Afi, T: AddressDisplay<A>> From<T> for FmtWrapper<A, T> {
        fn from(inner: T) -> Self {
            Self {
                inner,
                _marker: PhantomData,
            }
        }
    }

    impl<A: Afi, T: AddressDisplay<A>> fmt::Display for FmtWrapper<A, T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.inner.fmt_addr(f)
        }
    }

    struct Writer<'a> {
        buf: &'a mut [u8],
        cursor: usize,
    }

    impl<'a> Writer<'a> {
        fn new(buf: &'a mut [u8]) -> Self {
            Self { buf, cursor: 0 }
        }

        const fn len(&self) -> usize {
            self.cursor
        }
    }

    impl Write for Writer<'_> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            let bytes = s.as_bytes();
            let tail = &mut self.buf[self.cursor..];
            if tail.len() < bytes.len() {
                return Err(core::fmt::Error);
            }
            let target = &mut tail[..bytes.len()];
            target.copy_from_slice(bytes);
            self.cursor += bytes.len();
            Ok(())
        }
    }

    macro_rules! assert_fmt {
        ( $( $name:ident: $num:literal => $repr:literal ),* $(,)? ) => {
            $(
                #[test]
                fn $name() {
                    let mut buf = [0u8; 40];
                    let mut writer = Writer::new(&mut buf);
                    write!(writer, "{}", FmtWrapper::from($num)).unwrap();
                    let len = writer.len();
                    let repr = core::str::from_utf8(&buf[..len]).unwrap();
                    assert_eq!(repr, $repr)
                }
            )*
        }
    }

    assert_fmt! {
        ipv4_unspecified: 0u32 => "0.0.0.0",
        ipv4_loopback: 0x7f00_0001u32 => "127.0.0.1",
        ipv6_unspecified: 0u128 => "::",
        ipv6_loopback: 1u128 => "::1",
        ipv6_ipv4_compat: 0x7f00_0001u128 => "::127.0.0.1",
        ipv6_ipv4_mapped: 0xffff_7f00_0001u128 => "::ffff:127.0.0.1",
        ipv6_simple: 0x2001_0db8_0001_0001_0001_0001_0001_0001u128 => "2001:db8:1:1:1:1:1:1",
        ipv6_elided_tail: 0x2001_0db8_0001_0001_0000_0000_0000_0000u128 => "2001:db8:1:1::",
        ipv6_elided_head: 0x000f_dead_beefu128 => "::f:dead:beef",
        ipv6_elided_mid: 0x0001_0000_0000_0000_0000_0000_0000_0001u128 => "1::1",
    }
}
