use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    DataStruct, Fields, GenericParam, Generics, Ident, ImplGenerics, Index, TypeGenerics,
    WhereClause, WherePredicate,
};

use crate::DeriveRedis;

impl DeriveRedis for DataStruct {
    fn derive_redis(&self, type_ident: Ident, type_generics: Generics) -> proc_macro::TokenStream {
        let (impl_g, ty_g, wc) = split_redis_generics_for_impl(&type_generics);
        let mut tt = match &self.fields {
            Fields::Named(field_named) => {
                let names = field_named
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect::<Vec<_>>();
                quote! {
                    impl #impl_g ::ya_binary_format::ToBytes for #type_ident #ty_g #wc {
                        fn to_bytes<W: ?Sized + ::ya_binary_format::ByteWriter>(&self, out: &mut W) {
                            use ya_binary_format::ToBytes;
                            #(self.#names.to_bytes(out);)*
                        }
                    }
                    impl #impl_g ::ya_binary_format::FromBytes for #type_ident #ty_g #wc {
                        fn from_bytes(b: &mut ::ya_binary_format::Bytes) -> Self {
                            use ::ya_binary_format::FromBytes;
                            #type_ident {
                                #(#names: FromBytes::from_bytes(b),)*
                            }
                        }
                    }
                }
            }
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
                    impl #impl_g ::ya_binary_format::ToBytes for #type_ident #ty_g #wc {
                        fn to_bytes<W: ?Sized + ::ya_binary_format::ByteWriter>(&self, out: &mut W) {
                            use ya_binary_format::ToBytes;
                            #(self.#indices.to_bytes(out);)*
                        }
                    }
                    impl #impl_g ::ya_binary_format::FromBytes for #type_ident #ty_g #wc {
                        fn from_bytes(b: &mut ::ya_binary_format::Bytes) -> Self {
                            use ya_binary_format::FromBytes;
                            #type_ident(#(#f::from_bytes(b),)*)
                        }
                    }
                }
            }
            Fields::Unit => quote! {
                impl #impl_g ::ya_binary_format::ToBytes for #type_ident #ty_g #wc {
                    fn to_bytes<W: ?Sized + ::ya_binary_format::ByteWriter>(&self, _out: &mut W) {}
                }
                impl #impl_g ::ya_binary_format::FromBytes for #type_ident #ty_g #wc {
                    fn from_bytes(b: &mut ::ya_binary_format::Bytes) -> Self {
                        #type_ident
                    }
                }
            },
        };
        tt.extend(quote! {
            impl #impl_g ::redis::ToRedisArgs for #type_ident #ty_g #wc {
                fn write_redis_args<W : ?Sized + redis::RedisWrite>(&self, out: &mut W) {
                    use ::ya_binary_format::{ByteWriter, ToBytes};
                    let mut buf = Vec::new();
                    self.to_bytes(&mut buf);
                    out.write_arg(&buf);
                }
            }
            impl #impl_g ::redis::FromRedisValue for #type_ident #ty_g #wc {
                fn from_redis_value(v: &::redis::Value) -> ::redis::RedisResult<Self> {
                    let mut b = match v {
                        ::redis::Value::Data(v) => ::ya_binary_format::Bytes::copy_from_slice(v),
                        _ => return Err(::redis::RedisError::from((
                            ::redis::ErrorKind::TypeError,
                            "the data got from redis was not single binary data",
                        ))),
                    };
                    Ok(::ya_binary_format::FromBytes::from_bytes(&mut b))
                }
            }
        });
        tt.into()
    }
}

// From https://github.com/TeXitoi/structopt/blob/master/structopt-derive/src/lib.rs
// Thank you!
fn split_redis_generics_for_impl(generics: &Generics) -> (ImplGenerics, TypeGenerics, TokenStream) {
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

        fn into_tokens(self) -> TokenStream {
            self.tokens
        }
    }

    let mut trait_bound_amendments = TraitBoundAmendments::new(generics.where_clause.as_ref());

    for param in &generics.params {
        if let GenericParam::Type(param) = param {
            let param_ident = &param.ident;
            trait_bound_amendments.add(quote! { #param_ident : ::ya_binary_format::ToBytes });
            trait_bound_amendments.add(quote! { #param_ident : ::ya_binary_format::FromBytes });
        }
    }

    if let Some(where_clause) = &generics.where_clause {
        for predicate in &where_clause.predicates {
            if let WherePredicate::Type(predicate) = predicate {
                let predicate_bounded_ty = &predicate.bounded_ty;
                trait_bound_amendments
                    .add(quote! { #predicate_bounded_ty : ::ya_binary_format::ToBytes });
                trait_bound_amendments
                    .add(quote! { #predicate_bounded_ty : ::ya_binary_format::FromBytes });
            }
        }
    }

    let trait_bound_amendments = trait_bound_amendments.into_tokens();

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let where_clause = quote! { #where_clause #trait_bound_amendments };

    (impl_generics, ty_generics, where_clause)
}
