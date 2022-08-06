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

pub trait ToBytes {
    fn to_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W);
}

pub trait FromBytes<'a>: Sized {
    fn from_bytes(b: &mut Bytes<'a>) -> Self;
}

macro_rules! num_impls {
    ($typ:ty, $get:ident) => {
        impl ToBytes for $typ {
            fn to_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
                out.write(&self.to_le_bytes());
            }
        }

        impl<'a> FromBytes<'a> for $typ {
            fn from_bytes(b: &mut Bytes<'a>) -> Self {
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

impl ToBytes for usize {
    fn to_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
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

impl<'a> FromBytes<'a> for usize {
    fn from_bytes(b: &mut Bytes<'a>) -> Self {
        match b.get_u8() {
            254 => b.get_u32_le() as usize,
            255 => b.get_u64_le() as usize,
            v => v as usize,
        }
    }
}

impl ToBytes for () {
    fn to_bytes<W: ?Sized + ByteWriter>(&self, _out: &mut W) {}
}

impl<'a> FromBytes<'a> for () {
    fn from_bytes(_b: &mut Bytes<'a>) -> Self {
        ()
    }
}

impl ToBytes for bool {
    fn to_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        out.write(if *self { b"1" } else { b"0" });
    }
}

impl<'a> FromBytes<'a> for bool {
    fn from_bytes(b: &mut Bytes<'a>) -> Self {
        b.get_u8() == b'1'
    }
}

impl ToBytes for String {
    fn to_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        self.as_bytes().len().to_bytes(out);
        out.write(self.as_bytes());
    }
}

impl<'a> FromBytes<'a> for String {
    fn from_bytes(b: &mut Bytes<'a>) -> Self {
        let n = usize::from_bytes(b);
        let s = String::from_utf8(b.chunk()[..n].to_vec()).expect("Fail to parse");
        b.advance(n);
        s
    }
}

impl ToBytes for bytes::Bytes {
    fn to_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        self.len().to_bytes(out);
        out.write(self.chunk());
    }
}

impl<'a> FromBytes<'a> for bytes::Bytes {
    fn from_bytes(b: &mut Bytes<'a>) -> Self {
        let n = usize::from_bytes(b);
        let ret = bytes::Bytes::copy_from_slice(&b.chunk()[..n]);
        b.advance(n);
        ret
    }
}

impl<T: ToBytes> ToBytes for Option<T> {
    fn to_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        match self {
            None => out.write(b"0"),
            Some(v) => {
                out.write(b"1");
                v.to_bytes(out);
            }
        }
    }
}

impl<'a, T: FromBytes<'a>> FromBytes<'a> for Option<T> {
    fn from_bytes(b: &mut Bytes<'a>) -> Self {
        if b.get_u8() == b'0' {
            None
        } else {
            Some(T::from_bytes(b))
        }
    }
}

impl<T: ToBytes> ToBytes for Box<T> {
    fn to_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        self.as_ref().to_bytes(out)
    }
}

impl<'a, T: FromBytes<'a>> FromBytes<'a> for Box<T> {
    fn from_bytes(b: &mut Bytes<'a>) -> Self {
        Box::new(T::from_bytes(b))
    }
}

macro_rules! iter_to_impl {
    ($self:ident, $out:ident) => {
        $self.len().to_bytes($out);
        for i in $self.iter() {
            i.to_bytes($out);
        }
    };
}

macro_rules! iter_from_impl {
    ($b:ident) => {
        (0..usize::from_bytes($b))
            .map(|_| T::from_bytes($b))
            .collect()
    };
}

impl<T: ToBytes> ToBytes for Vec<T> {
    fn to_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        iter_to_impl!(self, out);
    }
}

impl<'a, T: FromBytes<'a>> FromBytes<'a> for Vec<T> {
    fn from_bytes(b: &mut Bytes<'a>) -> Self {
        iter_from_impl!(b)
    }
}

impl<T: ToBytes, S> ToBytes for HashSet<T, S> {
    fn to_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        iter_to_impl!(self, out);
    }
}

impl<'a, T, S> FromBytes<'a> for HashSet<T, S>
where
    T: FromBytes<'a> + Eq + Hash,
    S: BuildHasher + Default,
{
    fn from_bytes(b: &mut Bytes<'a>) -> Self {
        iter_from_impl!(b)
    }
}

impl<T: ToBytes> ToBytes for BTreeSet<T> {
    fn to_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        iter_to_impl!(self, out);
    }
}

impl<'a, T: FromBytes<'a> + Ord> FromBytes<'a> for BTreeSet<T> {
    fn from_bytes(b: &mut Bytes<'a>) -> Self {
        iter_from_impl!(b)
    }
}

impl<T: ToBytes> ToBytes for VecDeque<T> {
    fn to_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        iter_to_impl!(self, out);
    }
}

impl<'a, T: FromBytes<'a>> FromBytes<'a> for VecDeque<T> {
    fn from_bytes(b: &mut Bytes<'a>) -> Self {
        iter_from_impl!(b)
    }
}

impl<T: ToBytes> ToBytes for BinaryHeap<T> {
    fn to_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        iter_to_impl!(self, out);
    }
}

impl<'a, T: FromBytes<'a> + Ord> FromBytes<'a> for BinaryHeap<T> {
    fn from_bytes(b: &mut Bytes<'a>) -> Self {
        iter_from_impl!(b)
    }
}

macro_rules! kv_to_impl {
    ($self:ident, $out:ident) => {
        $self.len().to_bytes($out);
        for (k, v) in $self.iter() {
            k.to_bytes($out);
            v.to_bytes($out);
        }
    };
}

macro_rules! kv_from_impl {
    ($b:ident) => {
        (0..usize::from_bytes($b))
            .map(|_| (K::from_bytes($b), V::from_bytes($b)))
            .collect()
    };
}

impl<K: ToBytes, V: ToBytes, S> ToBytes for HashMap<K, V, S> {
    fn to_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        kv_to_impl!(self, out);
    }
}

impl<'a, K, V, S> FromBytes<'a> for HashMap<K, V, S>
where
    K: FromBytes<'a> + Eq + Hash,
    V: FromBytes<'a>,
    S: BuildHasher + Default,
{
    fn from_bytes(b: &mut Bytes<'a>) -> Self {
        kv_from_impl!(b)
    }
}

impl<K: ToBytes, V: ToBytes> ToBytes for BTreeMap<K, V> {
    fn to_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
        kv_to_impl!(self, out);
    }
}

impl<'a, K, V> FromBytes<'a> for BTreeMap<K, V>
where
    K: FromBytes<'a> + Ord,
    V: FromBytes<'a>,
{
    fn from_bytes(b: &mut Bytes<'a>) -> Self {
        kv_from_impl!(b)
    }
}

macro_rules! tuple_impls {
    ($($T:ident $i:tt)+) => {
        impl<$($T: ToBytes,)+> ToBytes for ($($T,)+) {
            fn to_bytes<W: ?Sized + ByteWriter>(&self, out: &mut W) {
                $(self.$i.to_bytes(out);)+
            }
        }

        impl<'a, $($T: FromBytes<'a>,)+> FromBytes<'a> for ($($T,)+) {
            fn from_bytes(b: &mut Bytes<'a>) -> Self {
                ($($T::from_bytes(b),)+)
            }
        }
    };
}

tuple_impls!(A 0);
tuple_impls!(A 0 B 1);
tuple_impls!(A 0 B 1 C 2);
tuple_impls!(A 0 B 1 C 2 D 3);
tuple_impls!(A 0 B 1 C 2 D 3 E 4);
tuple_impls!(A 0 B 1 C 2 D 3 E 4 F 5);
tuple_impls!(A 0 B 1 C 2 D 3 E 4 F 5 G 6);
