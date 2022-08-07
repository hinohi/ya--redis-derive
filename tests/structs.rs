use redis::{FromRedisValue, ToRedisArgs, Value};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use ya_redis_derive::Redis;

fn do_test<T: FromRedisValue + ToRedisArgs + PartialEq + Debug>(v: T) {
    let mut args = v.to_redis_args();
    assert_eq!(args.len(), 1);
    let v2 = T::from_redis_value(&Value::Data(args.pop().unwrap())).unwrap();
    assert_eq!(v, v2);
}

#[derive(Debug, Eq, PartialEq, Redis, Deserialize, Serialize)]
struct A {
    a: i32,
    b: Option<String>,
    c: Vec<usize>,
    d: String,
    e: u128,
    f: (Option<u32>, bool),
}

#[test]
fn struct_named() {
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

#[derive(Debug, Eq, PartialEq, Redis, Deserialize, Serialize)]
struct B(bool, Vec<u8>, String, i32);

#[test]
fn struct_unnamed() {
    let b = B(true, vec![0; 1000], String::from("abc"), 123);
    do_test(b);
}

#[derive(Debug, Eq, PartialEq, Redis, Deserialize, Serialize)]
struct C;

#[test]
fn struct_unit() {
    do_test(C);
}

#[derive(Debug, Eq, PartialEq, Redis, Deserialize, Serialize)]
struct D<T: Copy> {
    a: T,
}

#[derive(Debug, Eq, PartialEq, Redis, Deserialize, Serialize)]
struct E<T: Eq>(T);

#[test]
fn struct_generics() {
    do_test(D { a: 42i32 });
    do_test(E(vec![1, 2, 3]));
}

#[derive(Debug, Eq, PartialEq, Redis, Deserialize, Serialize)]
struct Nest {
    a: A,
    b: B,
    c: C,
}

#[test]
fn struct_nest() {
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

#[derive(Debug, Eq, PartialEq, Redis, Deserialize, Serialize)]
enum EnumOnlyUnit {
    A,
    B,
    C,
}

#[test]
fn enum_only_unit() {
    do_test(EnumOnlyUnit::A);
    do_test(EnumOnlyUnit::B);
    do_test(EnumOnlyUnit::C);
    do_test(E(vec![EnumOnlyUnit::A, EnumOnlyUnit::C, EnumOnlyUnit::B]));
}

#[derive(Debug, Eq, PartialEq, Redis, Deserialize, Serialize)]
enum EnumMany {
    A,
    B(i128),
    C(bool, i16),
    D { a: i8 },
    E,
    F { b: (), c: i32, d: u32 },
}

#[test]
fn enum_many() {
    do_test(EnumMany::A);
    do_test(EnumMany::B(0));
    do_test(EnumMany::B(i128::MAX));
    do_test(EnumMany::B(i128::MIN));
    do_test(EnumMany::C(true, 0));
    do_test(EnumMany::C(true, 1));
    do_test(EnumMany::C(true, 100));
    do_test(EnumMany::C(false, 0));
    do_test(EnumMany::C(false, 1));
    do_test(EnumMany::C(false, 100));
    do_test(EnumMany::D { a: 0 });
    do_test(EnumMany::D { a: 1 });
    do_test(EnumMany::D { a: 127 });
    do_test(EnumMany::E);
    do_test(EnumMany::F { b: (), c: 0, d: 0 });
    do_test(EnumMany::F { b: (), c: 0, d: 10 });
    do_test(EnumMany::F { b: (), c: -1, d: 1 });
    do_test(EnumMany::F { b: (), c: -1, d: 1 });
}
