use redis::{FromRedisValue, ToRedisArgs, Value};
use std::fmt::Debug;
use ya_redis_derive::Redis;

fn do_test<T: FromRedisValue + ToRedisArgs + PartialEq + Debug>(v: T) {
    let mut args = v.to_redis_args();
    assert_eq!(args.len(), 1);
    let v2 = T::from_redis_value(&Value::Data(args.pop().unwrap())).unwrap();
    assert_eq!(v, v2);
}

#[derive(Debug, Eq, PartialEq, Redis)]
struct A {
    a: i32,
    b: Option<String>,
    c: Vec<usize>,
    d: String,
    e: u128,
    f: (Option<u32>, bool),
}

#[test]
fn test_struct_named() {
    let a = A {
        a: i32::MAX,
        b: Some(String::from("アイウ")),
        c: vec![10000, 256, i32::MAX as usize, 255, 254, 253, 0],
        d: String::new(),
        e: u128::MAX,
        f: (None, false),
    };
    do_test(a);
}

#[derive(Debug, Eq, PartialEq, Redis)]
struct B(bool, Vec<u8>, String, i32);

#[test]
fn test_struct_unnamed() {
    let b = B(true, vec![0; 1000], String::from("abc"), 123);
    do_test(b);
}

#[derive(Debug, Eq, PartialEq, Redis)]
struct C;

#[test]
fn test_struct_unit() {
    do_test(C);
}

#[derive(Debug, Eq, PartialEq, Redis)]
struct D<T: Copy> {
    a: T,
}

#[derive(Debug, Eq, PartialEq, Redis)]
struct E<T: Eq>(T);

#[test]
fn test_struct_generics() {
    do_test(D { a: 42i32 });
    do_test(E(vec![1, 2, 3]));
}

#[derive(Debug, Eq, PartialEq, Redis)]
struct Nest {
    a: A,
    b: B,
    c: C,
}

#[test]
fn test_struct_nest() {
    let n = Nest {
        a: A {
            a: 0,
            b: None,
            c: vec![10],
            d: String::from("000"),
            e: 0,
            f: (None, true),
        },
        b: B(false, vec![1, 1, 2, 3, 5], String::from("999"), 42),
        c: C,
    };
    do_test(n);
}
