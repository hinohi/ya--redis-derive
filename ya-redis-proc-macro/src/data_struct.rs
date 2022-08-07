use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataStruct, GenericParam, Generics, Ident, ImplGenerics, TypeGenerics, WhereClause};

use crate::DeriveRedis;

impl DeriveRedis for DataStruct {
    fn derive_redis(&self, type_ident: Ident, type_generics: Generics) -> proc_macro::TokenStream {
        let (ser_impl_g, ser_ty_g, ser_wc) = split_for_ser(&type_generics);
        let (de_impl_g, de_ty_g, de_wc) = split_for_de(&type_generics);
        quote! (
            impl #ser_impl_g ::redis::ToRedisArgs for #type_ident #ser_ty_g #ser_wc {
                fn write_redis_args<W : ?Sized + redis::RedisWrite>(&self, out: &mut W) {
                    out.write_arg(&::ya_redis_derive::to_bytes(self));
                }
            }
            impl #de_impl_g ::redis::FromRedisValue for #type_ident #de_ty_g #de_wc {
                fn from_redis_value(v: &::redis::Value) -> ::redis::RedisResult<Self> {
                    match v {
                        ::redis::Value::Data(v) => Ok(::ya_redis_derive::from_bytes(v)),
                        _ => Err(::redis::RedisError::from((
                            ::redis::ErrorKind::TypeError,
                            "the data got from redis was not single binary data",
                        ))),
                    }
                }
            }
        )
        .into()
    }
}

/// From https://github.com/TeXitoi/structopt/blob/master/structopt-derive/src/lib.rs
/// Thank you!
struct TraitBoundAmendments {
    tokens: TokenStream,
    need_where: bool,
    need_comma: bool,
}

impl TraitBoundAmendments {
    fn new(where_clause: Option<&WhereClause>) -> Self {
        let tokens = TokenStream::new();
        let (need_where, need_comma) = if let Some(where_clause) = where_clause {
            if where_clause.predicates.trailing_punct() {
                (false, false)
            } else {
                (false, true)
            }
        } else {
            (true, false)
        };
        Self {
            tokens,
            need_where,
            need_comma,
        }
    }

    fn add(&mut self, amendment: TokenStream) {
        if self.need_where {
            self.tokens.extend(quote! { where });
            self.need_where = false;
        }
        if self.need_comma {
            self.tokens.extend(quote! { , });
        }
        self.tokens.extend(amendment);
        self.need_comma = true;
    }
}

fn split_for_ser(generics: &Generics) -> (ImplGenerics, TypeGenerics, TokenStream) {
    let mut t = TraitBoundAmendments::new(generics.where_clause.as_ref());
    for param in &generics.params {
        if let GenericParam::Type(param) = param {
            let param_ident = &param.ident;
            t.add(quote! { #param_ident : ::serde::ser::Serialize });
        }
    }
    let trait_bound_amendments = t.tokens;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let where_clause = quote! { #where_clause #trait_bound_amendments };
    (impl_generics, ty_generics, where_clause)
}

fn split_for_de(generics: &Generics) -> (ImplGenerics, TypeGenerics, TokenStream) {
    let mut t = TraitBoundAmendments::new(generics.where_clause.as_ref());
    for param in &generics.params {
        if let GenericParam::Type(param) = param {
            let param_ident = &param.ident;
            t.add(quote! { #param_ident : ::serde::de::DeserializeOwned });
        }
    }
    let trait_bound_amendments = t.tokens;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let where_clause = quote! { #where_clause #trait_bound_amendments };
    (impl_generics, ty_generics, where_clause)
}
