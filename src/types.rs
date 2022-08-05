use std::{
    collections::{BTreeSet, HashSet},
    hash::Hash,
    usize,
};

use bytes::Buf;

pub trait ByteWriter {
    fn write(&mut self, b: &[u8]);
}

impl ByteWriter for Vec<u8> {
    fn write(&mut self, b: &[u8]) {
        self.extend_from_slice(b);
    }
}

pub trait ToNoDelimiter {
    fn to_no_delimiter_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W);
}

pub trait FromNoDelimiter: Sized {
    fn from_no_delimiter_bytes(b: &[u8]) -> (Self, usize);
}

macro_rules! impl_no_delimiter_to_num {
    ($typ:ty, $get:ident) => {
        impl ToNoDelimiter for $typ {
            fn to_no_delimiter_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
                out.write(&self.to_le_bytes());
            }
        }

        impl FromNoDelimiter for $typ {
            fn from_no_delimiter_bytes(b: &[u8]) -> (Self, usize) {
                let mut b = b;
                (b.$get(), ::std::mem::size_of::<$typ>())
            }
        }
    };
}

impl_no_delimiter_to_num!(i8, get_i8);
impl_no_delimiter_to_num!(u8, get_u8);
impl_no_delimiter_to_num!(i16, get_i16_le);
impl_no_delimiter_to_num!(u16, get_u16_le);
impl_no_delimiter_to_num!(i32, get_i32_le);
impl_no_delimiter_to_num!(u32, get_u32_le);
impl_no_delimiter_to_num!(i64, get_i64_le);
impl_no_delimiter_to_num!(u64, get_u64_le);
impl_no_delimiter_to_num!(i128, get_i128_le);
impl_no_delimiter_to_num!(u128, get_u128_le);
impl_no_delimiter_to_num!(f32, get_f32_le);
impl_no_delimiter_to_num!(f64, get_f64_le);

impl ToNoDelimiter for usize {
    fn to_no_delimiter_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        if *self < 254 {
            out.write(&[*self as u8])
        } else if *self < (1 << 32) {
            out.write(&[254]);
            out.write(&(*self as u32).to_le_bytes());
        } else {
            out.write(&[255]);
            out.write(&(*self as u64).to_le_bytes());
        }
    }
}

impl FromNoDelimiter for usize {
    fn from_no_delimiter_bytes(b: &[u8]) -> (Self, usize) {
        match b[0] {
            254 => {
                let mut b = &b[1..];
                (b.get_u32_le() as usize, 5)
            }
            255 => {
                let mut b = &b[1..];
                (b.get_u64_le() as usize, 9)
            }
            v => (v as usize, 1),
        }
    }
}

impl ToNoDelimiter for () {
    fn to_no_delimiter_bytes<W: ?Sized + ByteWriter>(&self, _out: &mut W) {}
}

impl FromNoDelimiter for () {
    fn from_no_delimiter_bytes(_b: &[u8]) -> (Self, usize) {
        ((), 0)
    }
}

impl ToNoDelimiter for bool {
    fn to_no_delimiter_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        out.write(if *self { b"1" } else { b"0" });
    }
}

impl FromNoDelimiter for bool {
    fn from_no_delimiter_bytes(b: &[u8]) -> (Self, usize) {
        (b[0] == b'1', 1)
    }
}

impl ToNoDelimiter for String {
    fn to_no_delimiter_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        self.as_bytes().len().to_no_delimiter_bytes(out);
        out.write(self.as_bytes());
    }
}

impl FromNoDelimiter for String {
    fn from_no_delimiter_bytes(b: &[u8]) -> (Self, usize) {
        let (n, o) = usize::from_no_delimiter_bytes(b);
        let v = b[o..o + n].to_vec();
        (String::from_utf8(v).expect("Fail to parse"), o + n)
    }
}

impl<T: ToNoDelimiter> ToNoDelimiter for Option<T> {
    fn to_no_delimiter_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        match self {
            None => out.write(b"0"),
            Some(v) => {
                out.write(b"1");
                v.to_no_delimiter_bytes(out);
            }
        }
    }
}

impl<T: FromNoDelimiter> FromNoDelimiter for Option<T> {
    fn from_no_delimiter_bytes(b: &[u8]) -> (Self, usize) {
        if b[0] == b'0' {
            (None, 1)
        } else {
            let (v, o) = T::from_no_delimiter_bytes(&b[1..]);
            (Some(v), o + 1)
        }
    }
}

macro_rules! impl_no_delimiter_to_iter {
    ($typ:ident $(,$boundary:ident)*) => {
        impl<T: ToNoDelimiter $(+$boundary)*> ToNoDelimiter for $typ<T> {
            fn to_no_delimiter_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
                self.len().to_no_delimiter_bytes(out);
                for i in self.iter() {
                    i.to_no_delimiter_bytes(out);
                }
            }
        }

        impl<T: FromNoDelimiter $(+$boundary)*> FromNoDelimiter for $typ<T> {
            fn from_no_delimiter_bytes(b: &[u8]) -> (Self, usize) {
                let (n, mut o) = usize::from_no_delimiter_bytes(b);
                let ret = (0..n)
                    .map(|_| {
                        let (i, oo) = T::from_no_delimiter_bytes(&b[o..]);
                        o += oo;
                        i
                    })
                    .collect();
                (ret, o)
            }
        }
    };
}

impl_no_delimiter_to_iter!(Vec);
impl_no_delimiter_to_iter!(HashSet, Eq, Hash);
impl_no_delimiter_to_iter!(BTreeSet, Ord);
