//! Implementations for the [WString] type.
//!
//! The type itself lives in the lib.rs file to avoid having to have a public alias, but
//! implementations live here.

use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use byteorder::{BigEndian, ByteOrder, LittleEndian};

use crate::utf16::{validate_raw_utf16, Utf16CharExt};
use crate::{Utf16Error, WStr, WString};

impl WString<LittleEndian> {
    /// Creates a new [WString] from raw bytes in little-endian byte order.
    pub fn from_utf16le(buf: Vec<u8>) -> Result<Self, Utf16Error> {
        Self::from_utf16(buf)
    }

    /// Converts a vector of bytes to a [WString], not checking validity.
    ///
    /// # Safety
    ///
    /// You must ensure the vector contains already valid UTF-16 with little-endian
    /// byte-order, otherwise you will get undefined behaviour.
    #[inline]
    pub unsafe fn from_utf16le_unchecked(buf: Vec<u8>) -> Self {
        Self::from_utf16_unchecked(buf)
    }
}

impl WString<BigEndian> {
    /// Creates a new [WString] from raw bytes in big-endian byte-order.
    pub fn from_utf16be(buf: Vec<u8>) -> Result<Self, Utf16Error> {
        Self::from_utf16(buf)
    }

    /// Converts a vector of bytes to a [WString], not checking validity.
    ///
    /// # Safety
    ///
    /// You must ensure the vector contains already valid UTF-16 with big-endian byte-order,
    /// otherwise you will get undefined behaviour.
    #[inline]
    pub unsafe fn from_utf16be_unchecked(buf: Vec<u8>) -> Self {
        Self::from_utf16_unchecked(buf)
    }
}

impl<E> WString<E>
where
    E: ByteOrder,
{
    /// Creates a new empty [WString].
    #[inline]
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            _endian: PhantomData,
        }
    }

    /// Creates a new empty [WString] with a capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(capacity),
            _endian: PhantomData,
        }
    }

    /// Converts a vector of bytes to a [WString].
    #[inline]
    pub fn from_utf16(buf: Vec<u8>) -> Result<Self, Utf16Error> {
        validate_raw_utf16::<E>(buf.as_slice())?;
        Ok(Self {
            buf,
            _endian: PhantomData,
        })
    }

    /// Converts a vector of bytes to a [WString], not checking validity.
    ///
    /// # Safety
    ///
    /// You must ensure the vector contains already valid UTF-16 in the correct byte-order,
    /// otherwise you will get undefined behaviour.
    #[inline]
    pub unsafe fn from_utf16_unchecked(buf: Vec<u8>) -> Self {
        Self {
            buf,
            _endian: PhantomData,
        }
    }

    /// Converts this string into a byte vector.
    #[inline]
    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }

    /// Returns a `&WStr` slice containing the entire string.
    #[inline]
    pub fn as_wstr(&self) -> &WStr<E> {
        self
    }

    /// Returns a `&mut WStr` slice containing the entire string.
    #[inline]
    pub fn as_mut_wstr(&mut self) -> &mut WStr<E> {
        self
    }

    /// Appends a string slicie onto the end of this string.
    #[inline]
    pub fn push_wstr(&mut self, string: &WStr<E>) {
        self.buf.extend_from_slice(string.as_bytes())
    }

    /// Returns the capacity in bytes.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    /// Ensure that this string has spare capacity of at least `additional` bytes.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.buf.reserve(additional)
    }

    /// Shrinks the capacity of this string to match its length.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.buf.shrink_to_fit()
    }

    /// Appends the given [char] to the end of this string.
    #[inline]
    pub fn push(&mut self, ch: char) {
        let mut buf = [0u8; 4];
        let byte_count = ch.encode_utf16_into::<E>(&mut buf);
        self.buf.extend_from_slice(&buf[..byte_count]);
    }

    /// Shortens this string to the specified length.
    ///
    /// The `new_len` is specified in bytes and not characters, just as [Self::len] returns
    /// the length in bytes.  If `new_len` is greater than the string's current length, this
    /// has no effect.
    ///
    /// Note that this method has no effect on the allocated capacity of the string.
    ///
    /// # Panics
    ///
    /// Panics if `new_len` does not lie on a [char] boundary.
    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        if new_len < self.len() {
            assert!(
                self.is_char_boundary(new_len),
                "new WString length not on char boundary"
            );
            self.buf.truncate(new_len)
        }
    }

    /// Removes the last character from the string bufer and returns it.
    ///
    /// Returns [None]if this string is empty.
    #[inline]
    pub fn pop(&mut self) -> Option<char> {
        let ch = self.chars().next_back()?;
        let newlen = self.len() - ch.len_utf16_bytes();
        unsafe {
            self.buf.set_len(newlen);
        }
        Some(ch)
    }

    /// Removes a [char] from this string at the given byte position and returns it.
    ///
    /// This is an `O(n)` operation as it requires copying every element in the buffer.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is larger than or equal to the string's length, or if it does not
    /// lie on a [char] boundary.
    #[inline]
    pub fn remove(&mut self, idx: usize) -> char {
        let ch = match self[idx..].chars().next() {
            Some(ch) => ch,
            None => panic!("cannot remove a char from the end of a string"),
        };
        let next = idx + ch.len_utf16_bytes();
        let len = self.len();
        unsafe {
            std::ptr::copy(
                self.buf.as_ptr().add(next),
                self.buf.as_mut_ptr().add(idx),
                len - next,
            );
            self.buf.set_len(len - (next - idx));
        }
        ch
    }

    /// Returns the length in bytes, not chars or graphemes.
    #[inline]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Returns `true` if the string has a [Self::len] of zero, `false` otherwise.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<E> Default for WString<E>
where
    E: ByteOrder,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<E> Deref for WString<E>
where
    E: ByteOrder,
{
    type Target = WStr<E>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { WStr::from_utf16_unchecked(self.buf.as_slice()) }
    }
}

impl<E> DerefMut for WString<E>
where
    E: ByteOrder,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { WStr::from_utf16_unchecked_mut(self.buf.as_mut_slice()) }
    }
}

#[cfg(test)]
mod tests {
    use byteorder::LE;

    use super::*;

    #[test]
    fn test_new() {
        let s: WString<LE> = WString::new();
        assert_eq!(s.len(), 0);
        assert_eq!(s.capacity(), 0);
        assert_eq!(s.to_utf8(), "");
    }

    #[test]
    fn test_with_capacity() {
        let s: WString<LE> = WString::with_capacity(5);
        assert_eq!(s.capacity(), 5);
        assert_eq!(s.len(), 0);
        assert_eq!(s.to_utf8(), "");
    }

    #[test]
    fn test_from_utf16() {
        let b = b"h\x00e\x00l\x00l\x00o\x00";
        let s: WString<LE> = WString::from_utf16(b.to_vec()).unwrap();
        assert_eq!(s.buf, b);
        assert_eq!(s.to_utf8(), "hello");
    }

    #[test]
    fn test_from_utf16_le_be() {
        let b_le = b"h\x00e\x00l\x00l\x00o\x00";
        let s_le = WString::from_utf16le(b_le.to_vec()).unwrap();
        assert_eq!(s_le.to_utf8(), "hello");

        let b_be = b"\x00h\x00e\x00l\x00l\x00o";
        let s_be = WString::from_utf16be(b_be.to_vec()).unwrap();
        assert_eq!(s_be.to_utf8(), "hello");
    }

    #[test]
    fn test_from_utf16_unchecked() {
        let b_le = b"h\x00e\x00l\x00l\x00o\x00";
        let s_le: WString<LE> = unsafe { WString::from_utf16_unchecked(b_le.to_vec()) };
        assert_eq!(s_le.to_utf8(), "hello");

        let s_le = unsafe { WString::from_utf16le_unchecked(b_le.to_vec()) };
        assert_eq!(s_le.to_utf8(), "hello");

        let b_be = b"\x00h\x00e\x00l\x00l\x00o";
        let s_be = unsafe { WString::from_utf16be_unchecked(b_be.to_vec()) };
        assert_eq!(s_be.to_utf8(), "hello");
    }

    #[test]
    fn test_into_bytes() {
        let b = b"h\x00e\x00l\x00l\x00o\x00";
        let s = WString::from_utf16le(b.to_vec()).unwrap();
        assert_eq!(s.into_bytes(), b);
    }

    #[test]
    fn test_as_wstr() {
        let b = b"h\x00e\x00l\x00l\x00o\x00";
        let wstr = WStr::from_utf16le(b).unwrap();
        let wstring = WString::from_utf16le(b.to_vec()).unwrap();
        assert_eq!(wstr, wstring.as_wstr());
    }

    #[test]
    fn test_as_mut_wstr() {
        let b = b"h\x00e\x00l\x00l\x00o\x00";
        let wstr = WStr::from_utf16le(b).unwrap();
        let mut wstring = WString::from_utf16le(b.to_vec()).unwrap();
        let m: &mut WStr<_> = wstring.as_mut_wstr();
        assert_eq!(m, wstr);
    }

    #[test]
    fn test_push_wstr() {
        let b = b"h\x00e\x00l\x00l\x00o\x00";
        let mut wstring = WString::from_utf16le(b.to_vec()).unwrap();
        let b = b" \x00w\x00o\x00r\x00l\x00d\x00";
        let wstr = WStr::from_utf16le(b).unwrap();
        wstring.push_wstr(wstr);
        assert_eq!(wstring.to_utf8(), "hello world");
    }

    #[test]
    fn test_reserve() {
        let mut s: WString<LE> = WString::with_capacity(0);
        assert_eq!(s.capacity(), 0);
        s.reserve(42);
        assert!(s.capacity() >= 42);
    }

    #[test]
    fn test_shrink_to_fit() {
        let mut s: WString<LE> = WString::with_capacity(42);
        assert!(s.capacity() >= 42);
        s.shrink_to_fit();
        assert_eq!(s.capacity(), 0);
    }

    #[test]
    fn test_push() {
        let mut s: WString<LE> = WString::new();
        s.push('h');
        s.push('i');
        assert_eq!(s.as_bytes(), b"h\x00i\x00");
        assert_eq!(s.to_utf8(), "hi");

        s.push('\u{10000}');
        assert_eq!(s.as_bytes(), b"h\x00i\x00\x00\xd8\x00\xdc");
        assert_eq!(s.to_utf8(), "hi\u{10000}");
    }

    #[test]
    fn test_truncate() {
        let b = b"h\x00e\x00l\x00l\x00o\x00";
        let mut s = WString::from_utf16le(b.to_vec()).unwrap();

        s.truncate(20);
        assert_eq!(s.to_utf8(), "hello");

        s.truncate(4);
        assert_eq!(s.to_utf8(), "he");
    }

    #[test]
    #[should_panic]
    fn test_truncate_no_char_boundary() {
        let b = b"h\x00e\x00l\x00l\x00o\x00";
        let mut s = WString::from_utf16le(b.to_vec()).unwrap();

        s.truncate(1);
    }

    #[test]
    fn test_pop() {
        let b = b"a\x00\x00\xd8\x00\xdch\x00i\x00";
        let mut s = WString::from_utf16le(b.to_vec()).unwrap();
        assert_eq!(s.to_utf8(), "a\u{10000}hi");

        assert_eq!(s.pop(), Some('i'));
        assert_eq!(s.to_utf8(), "a\u{10000}h");

        assert_eq!(s.pop(), Some('h'));
        assert_eq!(s.to_utf8(), "a\u{10000}");

        assert_eq!(s.pop(), Some('\u{10000}'));
        assert_eq!(s.to_utf8(), "a");

        assert_eq!(s.pop(), Some('a'));
        assert!(s.is_empty());
    }

    #[test]
    fn test_remove() {
        let b = b"a\x00\x00\xd8\x00\xdch\x00i\x00";
        let mut s = WString::from_utf16le(b.to_vec()).unwrap();

        assert_eq!(s.remove(2), '\u{10000}');
        assert_eq!(s.remove(2), 'h');
        assert_eq!(s.to_utf8(), "ai");
    }

    #[test]
    fn test_deref() {
        let b = b"h\x00e\x00l\x00l\x00o\x00";
        let wstring = WString::from_utf16le(b.to_vec()).unwrap();
        let wstr = WStr::from_utf16le(b).unwrap();
        assert_eq!(wstring.deref(), wstr);
    }

    #[test]
    fn test_deref_mut() {
        let b = b"h\x00e\x00l\x00l\x00o\x00";
        let v = Vec::from(&b[..]);
        let mut s = WString::from_utf16le(v).unwrap();
        let wstr = s.deref_mut();
        unsafe {
            let buf = wstr.as_bytes_mut();
            buf.copy_from_slice(b"w\x00o\x00r\x00l\x00d\x00");
        }
        assert_eq!(s.to_utf8(), "world");
    }
}
