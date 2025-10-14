//! An explicitly sized string of lowercase hexadecimal characters.

cfg_if::cfg_if! {
    if #[cfg(feature = "nightly")] {
        use std::ascii::Char;

        /// `N` hex characters from '[0-9a-f]'.
        #[derive(Clone)]
        pub struct HexString<const N: usize>([Char; N]);
        impl<const N: usize> HexString<N> {
            /// View as a UTF-8 `str`.
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }
        impl<const N: usize> From<&[u8]> for HexString<N> {
            fn from(value: &[u8]) -> Self {
                let mut chars = value
                    .iter()
                    .filter_map(|b| Char::from_u8(*b))
                    .collect::<Vec<_>>();
                assert!(chars.iter().all(|c| char::from(*c).is_ascii_hexdigit()));
                assert_eq!(chars.len(), N, "string length should be {N}");
                chars = chars.into_iter().map(to_lower).collect();
                Self(chars.as_slice().try_into().unwrap())
            }
        }
        impl<const N: usize> Default for HexString<N> {
            fn default() -> Self {
                Self([Char::Digit0; N])
            }
        }

        fn to_lower(c: Char) -> Char {
            use Char::*;
            match c {
                CapitalA => SmallA,
                CapitalB => SmallB,
                CapitalC => SmallC,
                CapitalD => SmallD,
                CapitalE => SmallE,
                CapitalF => SmallF,
                other => other,
            }
        }
    } else {
        /// `N` hex characters from '[0-9a-f]'.
        #[derive(Clone)]
        pub struct HexString<const N: usize>(String);
        impl<const N: usize> HexString<N> {
            /// View as a UTF-8 `str`.
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }
        impl<const N: usize> From<&[u8]> for HexString<N> {
            fn from(value: &[u8]) -> Self {
                let mut string = String::from_utf8(value.to_vec()).expect("should be valid utf-8");
                assert!(string.chars().all(|c| c.is_ascii_hexdigit()));
                assert_eq!(string.len(), N, "string length should be {N}");
                string.make_ascii_lowercase();
                Self(string)
            }
        }
        impl<const N: usize> Default for HexString<N> {
            fn default() -> Self {
                Self((0..N).map(|_| "0").collect::<String>())
            }
        }
    }
}

impl<const N: usize> std::fmt::Debug for HexString<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "hex({N})\"{}\"", self.as_str())
    }
}

impl<const N: usize> std::fmt::Display for HexString<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<HexString<4>> for u16 {
    /// Produces an array index in ["0000" .. "ffff"].
    /// u16 indicates the range of possible values: [0 .. 65535]
    fn from(value: HexString<4>) -> Self {
        let hex_digits = "0123456789abcdef".chars().collect::<Vec<_>>();
        let mut lo = 0;
        let mut scale = 4096;

        for ch in value.as_str().chars() {
            let ch_pos = hex_digits.iter().position(|&d| d == ch).unwrap() as u16;
            lo += ch_pos * scale;
            scale /= 16;
        }
        lo
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_from_bytes_not_hex() {
        let _: HexString<1> = b"g".as_slice().into();
    }

    #[test]
    #[should_panic]
    fn test_from_bytes_invalid_length() {
        let _: HexString<3> = b"12".as_slice().into();
    }

    #[test]
    fn test_from_bytes_to_str() {
        let s: HexString<3> = b"AB1".as_slice().into();
        assert_eq!(s.as_str(), "ab1");
    }

    #[test]
    fn test_to_u16() {
        let cases = [
            (b"0000", u16::MIN),
            (b"ffff", u16::MAX),
            (b"7fff", u16::MAX / 2),
        ];
        for (input, expected) in cases {
            let s = HexString::<4>::from(input.as_slice());
            let result = u16::from(s);
            assert_eq!(result, expected);
        }
    }
}
