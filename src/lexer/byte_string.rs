use serde::{Deserialize, Serialize};
use std::cmp::{Eq, PartialEq};
use std::ops::{Deref, DerefMut};
use std::str::from_utf8;

/// A wrapper for Vec<u8> that provides a human-readable Debug impl and
/// a few other conveniences.
///
/// The Trunk lexer and parser work mainly with byte strings because
/// valid PHP code is not required to be valid UTF-8.
#[derive(PartialEq, Eq, PartialOrd, Clone, Serialize, Deserialize)]
pub struct ByteString(pub(crate) Vec<u8>);

impl ByteString {
    pub fn new(bytes: Vec<u8>) -> Self {
        ByteString(bytes)
    }
}

impl std::fmt::Display for ByteString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for &b in &self.0 {
            match b {
                0 => write!(f, "\\0")?,
                b'\n' | b'\r' | b'\t' => write!(f, "{}", b.escape_ascii())?,
                0x01..=0x19 | 0x7f..=0xff => write!(f, "\\x{:02x}", b)?,
                _ => write!(f, "{}", b as char)?,
            }
        }

        Ok(())
    }
}

impl std::fmt::Debug for ByteString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"")?;
        for &b in &self.0 {
            match b {
                0 => write!(f, "\\0")?,
                b'\n' | b'\r' | b'\t' => write!(f, "{}", b.escape_ascii())?,
                0x01..=0x19 | 0x7f..=0xff => write!(f, "\\x{:02x}", b)?,
                _ => write!(f, "{}", b as char)?,
            }
        }
        write!(f, "\"")?;
        Ok(())
    }
}

impl<const N: usize> PartialEq<&[u8; N]> for ByteString {
    fn eq(&self, other: &&[u8; N]) -> bool {
        &self.0 == other
    }
}

impl From<Vec<u8>> for ByteString {
    fn from(bytes: Vec<u8>) -> Self {
        ByteString::new(bytes)
    }
}

impl From<&[u8]> for ByteString {
    fn from(bytes: &[u8]) -> Self {
        ByteString::new(bytes.to_vec())
    }
}

impl<const N: usize> From<&[u8; N]> for ByteString {
    fn from(bytes: &[u8; N]) -> Self {
        ByteString::new(bytes.to_vec())
    }
}

impl From<&str> for ByteString {
    fn from(bytes: &str) -> Self {
        ByteString::new(bytes.as_bytes().to_vec())
    }
}

impl From<String> for ByteString {
    fn from(bytes: String) -> Self {
        ByteString::new(bytes.into_bytes())
    }
}

impl From<ByteString> for String {
    fn from(bytes: ByteString) -> Self {
        String::from(from_utf8(&bytes.0).unwrap())
    }
}

impl Deref for ByteString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Vec<u8> {
        &self.0
    }
}

impl DerefMut for ByteString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_string_debug() {
        assert_eq!(format!("{:?}", ByteString::from("abc")), r#""abc""#);
        assert_eq!(
            format!("{:?}", ByteString::from("\0\n\r\t")),
            r#""\0\n\r\t""#
        );
        assert_eq!(
            format!("{:?}", ByteString::from(b"\x01\x10\x7f\xff")),
            r#""\x01\x10\x7f\xff""#
        );
    }
}
