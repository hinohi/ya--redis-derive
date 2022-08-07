use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod impls;

#[proc_macro_derive(Redis)]
pub fn derive_redis(tokenstream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokenstream as DeriveInput);
    let type_ident = input.ident;
    let type_generics = input.generics;
    impls::derive_redis(type_ident, type_generics)
}
