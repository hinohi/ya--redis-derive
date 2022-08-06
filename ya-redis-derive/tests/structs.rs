use redis::{FromRedisValue, ToRedisArgs, Value};
use ya_redis_derive::Redis;

#[derive(Debug, Eq, PartialEq, Redis)]
struct A {
    a: i32,
    b: Option<String>,
    c: Vec<usize>,
    d: String,
    e: u128,
}

#[test]
fn test_a() {
    let a = A {
        a: i32::MAX,
        b: Some(String::from("アイウ")),
        c: vec![10000, 256, i32::MAX as usize, 255, 254, 253, 0],
        d: String::new(),
        e: u128::MAX,
    };
    let mut args = a.to_redis_args();
    assert_eq!(args.len(), 1);
    let a2 = A::from_redis_value(&Value::Data(args.pop().unwrap())).unwrap();
    assert_eq!(a, a2);
}
