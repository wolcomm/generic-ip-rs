pub(crate) mod ipv4;
pub(crate) mod ipv6;

trait Number: Eq + Sized {
    const ZERO: Self;
    fn checked_add(self, rhs: u8) -> Option<Self>;
    fn checked_mul(self, rhs: u8) -> Option<Self>;
    fn to_be_bytes<const N: usize>(self) -> [u8; N];
}

macro_rules! impl_number {
    ( $( $ty:ty: $n:literal ),* $(,)? ) => {
        $(
            impl Number for $ty {
                const ZERO: Self = 0;
                fn checked_add(self, rhs: u8) -> Option<Self> {
                    self.checked_add(rhs.into())
                }
                fn checked_mul(self, rhs: u8) -> Option<Self> {
                    self.checked_mul(rhs.into())
                }
                fn to_be_bytes<const N: usize>(self) -> [u8; N] {
                    let mut buf = [0; N];
                    self.to_be_bytes().into_iter().enumerate().for_each(|(i, octet)| buf[i] = octet);
                    buf
                }
            }
        )*
    };
}

impl_number! { u8: 1, u16: 2 }

#[derive(Debug)]
struct Parser<'a> {
    state: &'a [u8],
}

impl<'a> Parser<'a> {
    const fn new(input: &'a str) -> Self {
        Self {
            state: input.as_bytes(),
        }
    }

    const fn is_eof(&self) -> bool {
        self.state.is_empty()
    }

    fn atomically<F, T>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(&mut Self) -> Option<T>,
    {
        let saved = self.state;
        let result = f(self);
        if result.is_none() {
            self.state = saved;
        }
        result
    }

    fn take(&mut self) -> Option<u8> {
        self.state.split_first().map(|(next, tail)| {
            self.state = tail;
            *next
        })
    }

    fn skip(&mut self, bytes: &[u8]) -> Option<&mut Self> {
        bytes
            .iter()
            .try_for_each(|c| self.take().and_then(|next| next.eq(c).then_some(())))
            .map(|_| self)
    }

    fn take_digit(&mut self, radix: u8) -> Option<u8> {
        match self.take() {
            Some(val @ b'0'..=b'9') => Some(val & 0x0f),
            Some(val @ (b'a'..=b'f' | b'A'..=b'F')) if radix > 10 => Some((val & 0x0f) + 0x09),
            _ => None,
        }
        .filter(|val| val < &radix)
    }

    fn take_number<T>(&mut self, radix: u8, max_digits: usize, leading_zeros: bool) -> Option<T>
    where
        T: Number,
    {
        let mut result = T::ZERO;
        let mut digits: usize = 0;
        while let Some(digit) = self.atomically(|p| p.take_digit(radix)) {
            if !leading_zeros && digits > 0 && result == T::ZERO {
                return None;
            }
            digits += 1;
            result = result
                .checked_mul(radix)
                .and_then(|val| val.checked_add(digit))?;
            if digits == max_digits {
                break;
            }
        }
        (digits > 0).then_some(result)
    }

    fn take_separated<F>(&mut self, sep: &[u8], lim: usize, mut f: F) -> usize
    where
        F: FnMut(&mut Self, usize) -> Option<(usize, bool)>,
    {
        let mut count = 0;
        _ = (0..lim).try_for_each(|i| {
            self.atomically(|p| {
                if i > 0 {
                    _ = p.skip(sep)?;
                }
                f(p, i)
            })
            .and_then(|(taken, cont)| {
                count += taken;
                cont.then_some(())
            })
        });
        count
    }

    fn take_ipv4_octets(&mut self) -> Option<[u8; 4]> {
        let mut buf = [0; 4];
        (self.take_separated(b".", buf.len(), |p, i| {
            buf[i] = p.take_number(10, 3, false)?;
            Some((1, true))
        }) == 4)
            .then_some(buf)
    }

    fn take_ipv6_parts(&mut self, buf: &mut [u16]) -> (usize, bool) {
        let mut took_ipv4 = false;
        let limit = buf.len();
        let taken = self.take_separated(b":", limit, |p, i| {
            if let Some([a, b, c, d]) = (i < limit - 1)
                .then(|| p.atomically(Self::take_ipv4_octets))
                .flatten()
            {
                buf[i] = u16::from_be_bytes([a, b]);
                buf[i + 1] = u16::from_be_bytes([c, d]);
                took_ipv4 = true;
                Some((2, false))
            } else {
                buf[i] = p.take_number(16, 4, true)?;
                Some((1, true))
            }
        });
        (taken, took_ipv4)
    }

    fn take_ipv6_segments(&mut self) -> Option<[u16; 8]> {
        let mut buf = [0; 8];
        let (head, took_ipv4) = self.take_ipv6_parts(&mut buf);
        if head == 8 {
            Some(buf)
        } else if took_ipv4 {
            None
        } else {
            _ = self.skip(b"::")?;
            let mut addtional = [0; 7];
            let limit = 7 - head;
            let (tail, _) = self.take_ipv6_parts(&mut addtional[..limit]);
            buf[8 - tail..8].copy_from_slice(&addtional[..tail]);
            Some(buf)
        }
    }

    fn take_length(&mut self) -> Option<u8> {
        self.skip(b"/").and_then(|p| p.take_number(10, 3, false))
    }

    fn take_only<F, T>(&mut self, mut f: F) -> Option<T>
    where
        F: FnMut(&mut Self) -> Option<T>,
    {
        let result = f(self)?;
        self.is_eof().then_some(result)
    }

    fn take_with_length<F, T>(&mut self, mut f: F) -> Option<(T, u8)>
    where
        F: FnMut(&mut Self) -> Option<T>,
    {
        let result = f(self)?;
        let len = self.take_length()?;
        self.is_eof().then_some((result, len))
    }

    fn take_with_length_range<F, T>(&mut self, mut f: F) -> Option<(T, u8, u8, u8)>
    where
        F: FnMut(&mut Self) -> Option<T>,
    {
        let result = f(self)?;
        let mut buf = [0; 3];
        self.skip(b"/").and_then(|p| {
            (p.take_separated(b",", buf.len(), |p, i| {
                buf[i] = p.take_number(10, 3, false)?;
                Some((1, true))
            }) == 3)
                .then_some(())
        })?;
        let [len, lower, upper] = buf;
        self.is_eof().then_some((result, len, lower, upper))
    }
}
