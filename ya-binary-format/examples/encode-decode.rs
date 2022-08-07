use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ya_binary_format::{from_bytes, to_bytes};

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
struct MyStruct {
    a: i32,
    v: Vec<usize>,
    o1: Option<i8>,
    o2: Option<u16>,
    s: String,
    t: (u32, u64),
    m: HashMap<String, Option<String>>,
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

    let buf = to_bytes(&a);
    println!("{:?}", buf);
    println!("{}", buf.len());

    let v: MyStruct = from_bytes(&buf);
    assert_eq!(a, v);
}
