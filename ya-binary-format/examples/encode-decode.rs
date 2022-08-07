use std::collections::HashMap;

use ya_binary_format::{Buf, Bytes, FromBytes, ToBytes, Write};

#[derive(Debug, Eq, PartialEq)]
struct MyStruct {
    a: i32,
    v: Vec<usize>,
    o1: Option<i8>,
    o2: Option<u16>,
    s: String,
    t: (u32, u64),
    m: HashMap<String, Option<String>>,
}

impl ToBytes for MyStruct {
    fn to_bytes<W: ?Sized + Write>(&self, out: &mut W) {
        self.a.to_bytes(out);
        self.v.to_bytes(out);
        self.o1.to_bytes(out);
        self.o2.to_bytes(out);
        self.s.to_bytes(out);
        self.t.to_bytes(out);
        self.m.to_bytes(out);
    }
}

impl FromBytes for MyStruct {
    fn from_bytes(b: &mut Bytes) -> Self {
        let a = FromBytes::from_bytes(b);
        let v = FromBytes::from_bytes(b);
        let o1 = FromBytes::from_bytes(b);
        let o2 = FromBytes::from_bytes(b);
        let s = FromBytes::from_bytes(b);
        let t = FromBytes::from_bytes(b);
        let m = FromBytes::from_bytes(b);
        MyStruct {
            a,
            v,
            o1,
            o2,
            s,
            t,
            m,
        }
    }
}

fn main() {
    let a = MyStruct {
        a: 123,
        v: vec![0, 1, 254, 255, 1 << 40],
        o1: None,
        o2: Some(256),
        s: String::from("あいうえおabcdefg"),
        t: (10000, 1000000),
        m: {
            let mut m = HashMap::new();
            m.insert("a".to_string(), Some("b".to_string()));
            m.insert("123".to_string(), None);
            m.insert("".to_string(), Some("".to_string()));
            m
        },
    };
    println!("{:?}", a);

    let mut buf = Vec::new();
    a.to_bytes(&mut buf);
    println!("{:?}", buf);
    println!("{}", buf.len());

    let mut b = Bytes::new(&buf);
    let v = MyStruct::from_bytes(&mut b);
    assert_eq!(a, v);
    assert_eq!(b.remaining(), 0);
}
