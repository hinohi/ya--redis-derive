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
