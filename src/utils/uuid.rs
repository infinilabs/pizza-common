// MIT License
//
// Copyright (C) INFINI Labs & INFINI LIMITED. <hello@infini.ltd>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

// The Uuid Project is copyright 2013-2014, The Rust Project Developers and
// copyright 2018, The Uuid Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT License <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. All files in the project
// carrying such notice may not be copied, modified, or distributed except
// according to those terms.

//! A slightly modified [`uuid::Uuid`].
//!
//! This module is adapted from the original project <https://github.com/uuid-rs/uuid>.

use alloc::string::String;
use core::fmt;
use core::str::from_utf8_unchecked;
use core::str::FromStr;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid format: {}", self.message)
    }
}

const UUID_LEN: usize = 10;
const ASCII_LEN: usize = UUID_LEN * 2;

/// Adapted from: https://github.com/uuid-rs/uuid/blob/fe11291/src/fmt.rs#L152
///
/// Map numeric values of hex characters into their characters, and split 1 byte
/// into 2 bytes.
///
/// ```text
/// (first 4 bits| second 4 bits) => [(first 4 bits as a byte), (second 4 bits as a byte)]
/// ```
const fn encode(src: &[u8; UUID_LEN]) -> [u8; ASCII_LEN] {
    const CH_TABLE: [u8; 16] = [
        b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd', b'e',
        b'f',
    ];

    let mut buf = [0; ASCII_LEN];
    let mut i = 0usize;

    loop {
        let x = src[i];
        buf[i * 2] = CH_TABLE[(x >> 4) as usize];
        buf[i * 2 + 1] = CH_TABLE[(x & 0x0f) as usize];

        if i == 9 {
            break buf;
        }
        i += 1;
    }
}

/// Adapted from: https://github.com/uuid-rs/uuid/blob/fe11291/src/parser.rs#L159
///
/// Map lowercase hex characters to their numeric values, i.e., 'a' -> 10, then
/// store 2 characters in 1 byte.
///
/// ```text
/// [(first byte), (second byte)] => (first byte << 4 | second byte)
/// ```
///
/// # Errors
///
/// `s` should have length 20, and the bytes should be numeric values of lowercase
/// hex characters '0' - '9', 'a' - 'f', or an error would be returned.
fn decode(s: &[u8]) -> Result<[u8; UUID_LEN], ParseError> {
    const HEX_TABLE: [u8; 256] = {
        let mut buf = [0u8; 256];
        let mut i = 0u8;

        loop {
            let x = match i {
                b'0'..=b'9' => i - b'0',
                b'a'..=b'f' => i - b'a' + 10,
                _ => 0xff,
            };
            buf[i as usize] = x;

            if i == 255 {
                break buf;
            }
            i += 1
        }
    };

    // This length check here removes all subsequent bounds checks.
    if s.len() != Uuid::LENGTH {
        return Err(ParseError {
            message: alloc::format!(
                "Invalid UUID length, expected: {}, found: {}",
                Uuid::LENGTH,
                s.len()
            ),
        });
    }

    let mut buf = [0u8; UUID_LEN];
    let mut i = 0usize;

    loop {
        // Convert a two-char hex value (like `a8`) into a byte (like `10101000`)
        let h1 = HEX_TABLE[s[i * 2] as usize];
        let h2 = HEX_TABLE[s[i * 2 + 1] as usize];

        // We use `0xff` as a sentinel value to indicate an invalid hex
        // character sequence (like the letter `g`)
        if h1 | h2 == 0xff {
            return Err(ParseError {
                message: alloc::format!(
                    "invalid UUID character found: expect '0'-'9' or 'a'-'f', found: {} and {}",
                    char::from_u32(s[i * 2] as u32).expect("should be a valid char"),
                    char::from_u32(s[i * 2 + 1] as u32).expect("should be a valid char")
                ),
            });
        }

        // The upper nibble needs to be shifted into position to produce the
        // final byte value
        buf[i] = (h1 << 4) | h2;

        if i == 9 {
            break Ok(buf);
        }
        i += 1;
    }
}

/// A short version of [`uuid::Uuid`], retaining only the first 10 bytes.
///
/// A version 4 UUID contains timestamp (by microseconds) and address family in
/// its first 72 bits, so it should be unique enough in a running system for
/// internal usage.
///
/// # NOTE
///
/// The encoded ascii version is still 20 bytes long.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Uuid([u8; UUID_LEN]);

impl Uuid {
    /// The length of the ASCII encoded string.
    pub const LENGTH: usize = ASCII_LEN;

    pub fn new() -> Self {
        Self::from_uuid(uuid::Uuid::new_v4())
    }

    pub const fn empty() -> Self {
        Self([0; UUID_LEN])
    }

    pub fn from_uuid(uuid: uuid::Uuid) -> Self {
        Self(uuid.as_bytes()[0..UUID_LEN].try_into().unwrap())
    }

    pub fn encode_with<T>(&self, f: impl FnOnce(&str) -> T) -> T {
        let raw = encode(&self.0);
        // SAFETY: The buffer is ASCII encoded
        let str = unsafe { from_utf8_unchecked(&raw) };
        f(str)
    }

    /// View this UUID as a slice of u8 bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Directly cast a sequence of raw bytes into an UUID.
    ///
    /// # Safety
    /// Callers need to ensure that these bytes are valid value for an UUID.
    pub unsafe fn from_bytes(bytes: [u8; UUID_LEN]) -> Self {
        Self(bytes)
    }
}

impl Default for Uuid {
    /// # NOTE
    ///
    /// Openraft requires that 2 default IDs should have the same value so we
    /// cannot implement it with `Uuid::new()`.
    fn default() -> Self {
        Self::empty()
    }
}

/// Required by [`openraft::testing::Suite`].
///
/// > Additional traits are required to be implemented by the store builder for testing:
/// >
/// > `C::NodeId` requires `From<u64>` to build a node id.
///
/// > link: https://github.com/datafuselabs/openraft/blob/6197f054a5e0e8df89a1666df96cb0f738bb807d/openraft/src/testing/log/suite.rs#L95
impl From<u64> for Uuid {
    fn from(value: u64) -> Self {
        let mut buf = [0u8; UUID_LEN];
        buf[0..8].clone_from_slice(value.to_be_bytes().as_slice());
        Self(buf)
    }
}

impl fmt::LowerHex for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.encode_with(|s| f.write_str(s))
    }
}

impl fmt::Debug for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(self, f)
    }
}

impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(self, f)
    }
}

impl<'de> Deserialize<'de> for Uuid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct UuidVisitor;
        impl<'de> serde::de::Visitor<'de> for UuidVisitor {
            type Value = Uuid;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("UUID")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Uuid::from_str(v).map_err(E::custom)
            }
        }
        deserializer.deserialize_str(UuidVisitor)
    }
}

impl Serialize for Uuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.encode_with(|s| serializer.serialize_str(s))
    }
}

impl FromStr for Uuid {
    type Err = ParseError;

    /// # NOTE
    ///
    /// The bytes in `s` should be numeric values of lowercase hex characters
    /// '0' - '9', 'a' - 'f', or `Err(invalid hex found)` would be returned.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        decode(s.as_bytes()).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::uuid::Uuid;
    use crate::utils::uuid::ASCII_LEN;
    use alloc::string::ToString;
    use core::str::FromStr;

    #[test]
    fn test_encode_decode() {
        // Random encoding/decoding tests.
        for _ in 0..1000 {
            let long = uuid::Uuid::new_v4();
            let short = Uuid::from_uuid(long);

            let long_str = long.simple().to_string();
            let short_str = short.to_string();
            assert_eq!(short_str, &long_str[0..ASCII_LEN]);

            let parsed = Uuid::from_str(&long_str[0..ASCII_LEN]).unwrap();
            assert_eq!(parsed, short);
        }
    }
}
