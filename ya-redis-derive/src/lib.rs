/*!
Yet another Redis derive

## Example

```rust
use redis::{Client, Commands, Connection};
use ya_redis_derive::Redis;

#[derive(Debug, Eq, PartialEq, Redis)]
struct MyStruct {
    id: i64,
    name: String,
    description: Option<String>,
    is_genius: bool,
    friend_ids: Vec<i64>,
}

fn main() {
    // Example: docker run --rm -p 6379:6379 redis
    let redis_client = Client::open("redis://localhost").unwrap();
    let mut redis_con = redis_client.get_connection().expect("Fail to connect redis server");

    let a = MyStruct {
        id: 123,
        name: String::from("名無しの権兵衛"),
        description: Some(String::from("とてもクールなライブラリ")),
        is_genius: true,
        friend_ids: vec![0, 1, 1000000],
    };
    let _: bool = redis_con.set("key-a", &a).expect("Fail to set");
    let a2: Option<MyStruct> = redis_con.get("key-a").expect("Fail to get");
    assert!(a2.is_some());
    assert_eq!(a, a2.unwrap());
}
```
*/
use proc_macro::TokenStream;
use syn::{parse_macro_input, Data, DeriveInput, Generics, Ident};

mod data_struct;

#[proc_macro_derive(Redis)]
pub fn derive_redis(tokenstream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokenstream as DeriveInput);
    let type_ident = input.ident;
    let type_generics = input.generics;
    match input.data {
        Data::Struct(data_struct) => data_struct.derive_redis(type_ident, type_generics),
        Data::Enum(_) => todo!(),
        Data::Union(_) => todo!(),
    }
}

trait DeriveRedis {
    fn derive_redis(&self, type_ident: Ident, type_generics: Generics) -> TokenStream;
}
