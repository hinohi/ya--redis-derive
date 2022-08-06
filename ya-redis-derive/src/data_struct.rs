use proc_macro::TokenStream;
use quote::quote;
use syn::{DataStruct, Fields, Generics, Ident, Index};

use crate::DeriveRedis;

impl DeriveRedis for DataStruct {
    fn derive_redis(&self, type_ident: Ident, type_generics: Generics) -> TokenStream {
        match &self.fields {
            Fields::Named(field_named) => {
                let names = field_named
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect::<Vec<_>>();
                quote! {
                    impl #type_generics ::redis::ToRedisArgs for #type_ident {
                        fn write_redis_args<W : ?Sized + redis::RedisWrite>(&self, out: &mut W) {
                            use ::ya_binary_format::{ByteWriter, ToBytes};
                            let mut buf = Vec::new();
                            #(self.#names.to_bytes(&mut buf);)*
                            out.write_arg(&buf);
                        }
                    }
                    impl #type_generics ::redis::FromRedisValue for #type_ident {
                        fn from_redis_value(&self, v: &::redis::Value) -> ::redis::RedisResult<Self> {
                            use ::ya_binary_format::{Bytes, FromBytes};
                            let mut b = match v {
                                ::redis::Value::Data(v) => Bytes::new(v),
                                _ => return Err(redis::RedisError::from(
                                    redis::ErrorKind::TypeError,
                                    "The data get from redis was not single binary data.",
                                )),
                            };
                            Ok(#type_ident {
                                #(#names: FromBytes::from_bytes(&mut b),)*
                            })
                        }
                    }
                }
            }
            .into(),
            Fields::Unnamed(field_unnamed) => {
                let indices = field_unnamed
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, _)| Index::from(i))
                    .collect::<Vec<_>>();
                quote! {
                    impl #type_generics ::redis::ToRedisArgs for #type_ident {
                        fn write_redis_args<W : ?Sized + redis::RedisWrite>(&self, out: &mut W) {
                            use ::ya_binary_format::{ByteWriter, ToBytes};
                            let mut buf = Vec::new();
                            #(self.#indices.to_bytes(&mut buf))*
                            out.write_arg(&buf);
                        }
                    }
                    impl #type_generics ::redis::FromRedisValue for #type_ident {
                        fn from_redis_value(&self, v: &::redis::Value) -> ::redis::RedisResult<Self> {
                            use ::ya_binary_format::{Bytes, FromBytes};
                            let mut b = match v {
                                ::redis::Value::Data(v) => Bytes::new(v),
                                _ => return Err(redis::RedisError::from(
                                    redis::ErrorKind::TypeError,
                                    "The data get from redis was not single binary data.",
                                )),
                            };
                            let t = FromBytes::from_bytes(&mut b);
                            Ok(#type_ident(#(t.#indices)*))
                        }
                    }
                }
            }
            .into(),
            Fields::Unit => quote! {
                impl ::redis::ToRedisArgs for #type_ident {
                    fn write_redis_args<W : ?Sized + redis::RedisWrite>(&self, out: &mut W) {
                        out.write_arg("");
                    }
                }
                impl ::redis::FromRedisValue for #type_ident {
                    fn from_redis_value(&self, _v: &::redis::Value) -> ::redis::RedisResult<Self> {
                        Ok(#type_ident)
                    }
                }
            }
            .into(),
        }
    }
}
