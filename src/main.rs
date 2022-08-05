use redis::{
    Client, Commands, ErrorKind, FromRedisValue, RedisError, RedisResult, RedisWrite, ToRedisArgs,
    Value,
};

use ya_redis_derive::{FromNoDelimiter, ToNoDelimiter};

#[derive(Debug, Eq, PartialEq)]
struct MyStruct {
    a: i32,
    v: Vec<usize>,
    o1: Option<i8>,
    o2: Option<u16>,
}

impl ToRedisArgs for MyStruct {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite,
    {
        let mut buf = Vec::new();
        self.a.to_no_delimiter_bytes(&mut buf);
        self.v.to_no_delimiter_bytes(&mut buf);
        self.o1.to_no_delimiter_bytes(&mut buf);
        self.o2.to_no_delimiter_bytes(&mut buf);
        out.write_arg(&buf);
    }
}

impl FromRedisValue for MyStruct {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        match v {
            Value::Data(b) => {
                let mut o = 0;
                let (a, offset) = FromNoDelimiter::from_no_delimiter_bytes(b);
                o += offset;
                let (v, offset) = FromNoDelimiter::from_no_delimiter_bytes(&b[o..]);
                o += offset;
                let (o1, offset) = FromNoDelimiter::from_no_delimiter_bytes(&b[o..]);
                o += offset;
                let (o2, _) = FromNoDelimiter::from_no_delimiter_bytes(&b[o..]);
                Ok(MyStruct { a, v, o1, o2 })
            }
            _ => Err(RedisError::from((
                ErrorKind::TypeError,
                "Expect bytes response",
            ))),
        }
    }
}

fn main() {
    let cli = Client::open("redis://localhost").expect("No redis server at localhost");
    let mut conn = cli.get_connection().expect("Fail to get connection");

    let a = MyStruct {
        a: 123,
        v: vec![0, 1, 254, 255, 1 << 40],
        o1: None,
        o2: Some(256),
    };
    println!("{:?}", a);

    let _: bool = conn.set("a", &a).expect("Fail to set");
    let b: Vec<u8> = conn.get("a").expect("Fail to get");
    println!("{:?}", b);
    println!("{}", b.len());

    let v: MyStruct = conn.get("a").expect("Fail to get");
    assert_eq!(a, v);
}
