use std::{
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, VecDeque},
    hash::{BuildHasher, Hash},
};

use bytes::Buf;

use crate::Bytes;

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

pub trait FromNoDelimiter<'a>: Sized {
    fn from_no_delimiter_bytes(b: &mut Bytes<'a>) -> Self;
}

macro_rules! num_impls {
    ($typ:ty, $get:ident) => {
        impl ToNoDelimiter for $typ {
            fn to_no_delimiter_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
                out.write(&self.to_le_bytes());
            }
        }

        impl<'a> FromNoDelimiter<'a> for $typ {
            fn from_no_delimiter_bytes(b: &mut Bytes<'a>) -> Self {
                b.$get()
            }
        }
    };
}

num_impls!(i8, get_i8);
num_impls!(u8, get_u8);
num_impls!(i16, get_i16_le);
num_impls!(u16, get_u16_le);
num_impls!(i32, get_i32_le);
num_impls!(u32, get_u32_le);
num_impls!(i64, get_i64_le);
num_impls!(u64, get_u64_le);
num_impls!(i128, get_i128_le);
num_impls!(u128, get_u128_le);
num_impls!(f32, get_f32_le);
num_impls!(f64, get_f64_le);

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

impl<'a> FromNoDelimiter<'a> for usize {
    fn from_no_delimiter_bytes(b: &mut Bytes<'a>) -> Self {
        match b.get_u8() {
            254 => b.get_u32_le() as usize,
            255 => b.get_u64_le() as usize,
            v => v as usize,
        }
    }
}

impl ToNoDelimiter for () {
    fn to_no_delimiter_bytes<W: ?Sized + ByteWriter>(&self, _out: &mut W) {}
}

impl<'a> FromNoDelimiter<'a> for () {
    fn from_no_delimiter_bytes(_b: &mut Bytes<'a>) -> Self {
        ()
    }
}

impl ToNoDelimiter for bool {
    fn to_no_delimiter_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        out.write(if *self { b"1" } else { b"0" });
    }
}

impl<'a> FromNoDelimiter<'a> for bool {
    fn from_no_delimiter_bytes(b: &mut Bytes<'a>) -> Self {
        b.get_u8() == b'1'
    }
}

impl ToNoDelimiter for String {
    fn to_no_delimiter_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        self.as_bytes().len().to_no_delimiter_bytes(out);
        out.write(self.as_bytes());
    }
}

impl<'a> FromNoDelimiter<'a> for String {
    fn from_no_delimiter_bytes(b: &mut Bytes<'a>) -> Self {
        let n = usize::from_no_delimiter_bytes(b);
        let s = String::from_utf8(b.chunk()[..n].to_vec()).expect("Fail to parse");
        b.advance(n);
        s
    }
}

impl ToNoDelimiter for bytes::Bytes {
    fn to_no_delimiter_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        self.len().to_no_delimiter_bytes(out);
        out.write(self.chunk());
    }
}

impl<'a> FromNoDelimiter<'a> for bytes::Bytes {
    fn from_no_delimiter_bytes(b: &mut Bytes<'a>) -> Self {
        let n = usize::from_no_delimiter_bytes(b);
        let ret = bytes::Bytes::copy_from_slice(&b.chunk()[..n]);
        b.advance(n);
        ret
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

impl<'a, T: FromNoDelimiter<'a>> FromNoDelimiter<'a> for Option<T> {
    fn from_no_delimiter_bytes(b: &mut Bytes<'a>) -> Self {
        if b.get_u8() == b'0' {
            None
        } else {
            Some(T::from_no_delimiter_bytes(b))
        }
    }
}

impl<T: ToNoDelimiter> ToNoDelimiter for Box<T> {
    fn to_no_delimiter_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        self.as_ref().to_no_delimiter_bytes(out)
    }
}

impl<'a, T: FromNoDelimiter<'a>> FromNoDelimiter<'a> for Box<T> {
    fn from_no_delimiter_bytes(b: &mut Bytes<'a>) -> Self {
        Box::new(T::from_no_delimiter_bytes(b))
    }
}

macro_rules! iter_to_impl {
    ($self:ident, $out:ident) => {
        $self.len().to_no_delimiter_bytes($out);
        for i in $self.iter() {
            i.to_no_delimiter_bytes($out);
        }
    };
}

macro_rules! iter_from_impl {
    ($b:ident) => {
        (0..usize::from_no_delimiter_bytes($b))
            .map(|_| T::from_no_delimiter_bytes($b))
            .collect()
    };
}

impl<T: ToNoDelimiter> ToNoDelimiter for Vec<T> {
    fn to_no_delimiter_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        iter_to_impl!(self, out);
    }
}

impl<'a, T: FromNoDelimiter<'a>> FromNoDelimiter<'a> for Vec<T> {
    fn from_no_delimiter_bytes(b: &mut Bytes<'a>) -> Self {
        iter_from_impl!(b)
    }
}

impl<T: ToNoDelimiter, S> ToNoDelimiter for HashSet<T, S> {
    fn to_no_delimiter_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        iter_to_impl!(self, out);
    }
}

impl<'a, T, S> FromNoDelimiter<'a> for HashSet<T, S>
where
    T: FromNoDelimiter<'a> + Eq + Hash,
    S: BuildHasher + Default,
{
    fn from_no_delimiter_bytes(b: &mut Bytes<'a>) -> Self {
        iter_from_impl!(b)
    }
}

impl<T: ToNoDelimiter> ToNoDelimiter for BTreeSet<T> {
    fn to_no_delimiter_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        iter_to_impl!(self, out);
    }
}

impl<'a, T: FromNoDelimiter<'a> + Ord> FromNoDelimiter<'a> for BTreeSet<T> {
    fn from_no_delimiter_bytes(b: &mut Bytes<'a>) -> Self {
        iter_from_impl!(b)
    }
}

impl<T: ToNoDelimiter> ToNoDelimiter for VecDeque<T> {
    fn to_no_delimiter_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        iter_to_impl!(self, out);
    }
}

impl<'a, T: FromNoDelimiter<'a>> FromNoDelimiter<'a> for VecDeque<T> {
    fn from_no_delimiter_bytes(b: &mut Bytes<'a>) -> Self {
        iter_from_impl!(b)
    }
}

impl<T: ToNoDelimiter> ToNoDelimiter for BinaryHeap<T> {
    fn to_no_delimiter_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        iter_to_impl!(self, out);
    }
}

impl<'a, T: FromNoDelimiter<'a> + Ord> FromNoDelimiter<'a> for BinaryHeap<T> {
    fn from_no_delimiter_bytes(b: &mut Bytes<'a>) -> Self {
        iter_from_impl!(b)
    }
}

macro_rules! kv_to_impl {
    ($self:ident, $out:ident) => {
        $self.len().to_no_delimiter_bytes($out);
        for (k, v) in $self.iter() {
            k.to_no_delimiter_bytes($out);
            v.to_no_delimiter_bytes($out);
        }
    };
}

macro_rules! kv_from_impl {
    ($b:ident) => {
        (0..usize::from_no_delimiter_bytes($b))
            .map(|_| {
                (
                    K::from_no_delimiter_bytes($b),
                    V::from_no_delimiter_bytes($b),
                )
            })
            .collect()
    };
}

impl<K: ToNoDelimiter, V: ToNoDelimiter, S> ToNoDelimiter for HashMap<K, V, S> {
    fn to_no_delimiter_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        kv_to_impl!(self, out);
    }
}

impl<'a, K, V, S> FromNoDelimiter<'a> for HashMap<K, V, S>
where
    K: FromNoDelimiter<'a> + Eq + Hash,
    V: FromNoDelimiter<'a>,
    S: BuildHasher + Default,
{
    fn from_no_delimiter_bytes(b: &mut Bytes<'a>) -> Self {
        kv_from_impl!(b)
    }
}

impl<K: ToNoDelimiter, V: ToNoDelimiter> ToNoDelimiter for BTreeMap<K, V> {
    fn to_no_delimiter_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        kv_to_impl!(self, out);
    }
}

impl<'a, K, V> FromNoDelimiter<'a> for BTreeMap<K, V>
where
    K: FromNoDelimiter<'a> + Ord,
    V: FromNoDelimiter<'a>,
{
    fn from_no_delimiter_bytes(b: &mut Bytes<'a>) -> Self {
        kv_from_impl!(b)
    }
}

macro_rules! tuple_impls {
    (($T:ident, $i:tt),) => {
        tuple_impls!(@impl ($T, $i),);
    };
    (($T:ident, $i:tt), $( ($U:ident, $j:tt) ,)+) => {
        tuple_impls!($( ($U, $j), )+);
        tuple_impls!(@impl ($T, $i), $( ($U, $j), )+);
    };
    (@impl $( ($T:ident, $i:tt), )+) => {
        impl<$($T: ToNoDelimiter,)+> ToNoDelimiter for ($($T,)+) {
            fn to_no_delimiter_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
                $(self.$i.to_no_delimiter_bytes(out);)+
            }
        }

        impl<'a, $($T: FromNoDelimiter<'a>,)+> FromNoDelimiter<'a> for ($($T,)+) {
            fn from_no_delimiter_bytes(b: &mut Bytes<'a>) -> Self {
                ($($T::from_no_delimiter_bytes(b),)+)
            }
        }
    };
}

tuple_impls!(
    (H, 7),
    (G, 6),
    (F, 5),
    (E, 4),
    (D, 3),
    (C, 2),
    (B, 1),
    (A, 0),
);
