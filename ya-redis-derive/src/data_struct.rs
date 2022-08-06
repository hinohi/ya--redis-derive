use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DataStruct, Fields, Generics, Ident, Index};

use crate::DeriveRedis;

impl DeriveRedis for DataStruct {
    fn derive_redis(&self, type_ident: Ident, type_generics: Generics) -> TokenStream {
        let (impl_g, ty_g, wc) = type_generics.split_for_impl();
        match &self.fields {
            Fields::Named(field_named) => {
                let names = field_named
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect::<Vec<_>>();
                quote! {
                    impl #impl_g ::redis::ToRedisArgs for #type_ident #ty_g #wc {
                        fn write_redis_args<W : ?Sized + redis::RedisWrite>(&self, out: &mut W) {
                            use ::ya_binary_format::{ByteWriter, ToBytes};
                            let mut buf = Vec::new();
                            #(self.#names.to_bytes(&mut buf);)*
                            out.write_arg(&buf);
                        }
                    }
                    impl #impl_g ::redis::FromRedisValue for #type_ident #ty_g #wc {
                        fn from_redis_value(v: &::redis::Value) -> ::redis::RedisResult<Self> {
                            use ::ya_binary_format::{Bytes, FromBytes};
                            let mut b = match v {
                                ::redis::Value::Data(v) => Bytes::copy_from_slice(v),
                                _ => return Err(::redis::RedisError::from((
                                    ::redis::ErrorKind::TypeError,
                                    "the data got from redis was not single binary data",
                                ))),
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
                let f = field_unnamed
                    .unnamed
                    .iter()
                    .map(|_| format_ident!("FromBytes"));
                quote! {
                    impl #type_generics ::redis::ToRedisArgs for #type_ident {
                        fn write_redis_args<W : ?Sized + redis::RedisWrite>(&self, out: &mut W) {
                            use ::ya_binary_format::{ByteWriter, ToBytes};
                            let mut buf = Vec::new();
                            #(self.#indices.to_bytes(&mut buf);)*
                            out.write_arg(&buf);
                        }
                    }
                    impl #type_generics ::redis::FromRedisValue for #type_ident {
                        fn from_redis_value(v: &::redis::Value) -> ::redis::RedisResult<Self> {
                            use ::ya_binary_format::{Bytes, FromBytes};
                            let mut b = match v {
                                ::redis::Value::Data(v) => Bytes::copy_from_slice(v),
                                _ => return Err(::redis::RedisError::from((
                                    ::redis::ErrorKind::TypeError,
                                    "the data got from redis was not single binary data",
                                ))),
                            };
                            Ok(#type_ident(#(#f::from_bytes(&mut b),)*))
                        }
                    }
                }
            }
            .into(),
            Fields::Unit => quote! {
                impl ::redis::ToRedisArgs for #type_ident {
                    fn write_redis_args<W : ?Sized + redis::RedisWrite>(&self, out: &mut W) {
                        out.write_arg(b"");
                    }
                }
                impl ::redis::FromRedisValue for #type_ident {
                    fn from_redis_value(_v: &::redis::Value) -> ::redis::RedisResult<Self> {
                        Ok(#type_ident)
                    }
                }
            }
            .into(),
        }
    }
}
