use proc_macro::TokenStream;
use syn::{Data, DeriveInput};

mod enums;
mod newtypes;
mod structs;
mod tuple_fields;

pub(crate) fn impl_js_json_derive(ast: &DeriveInput) -> Result<TokenStream, String> {
    let name = &ast.ident;

    match ast.data {
        Data::Struct(ref data) => structs::impl_js_json_struct(name, data),
        Data::Enum(ref data) => enums::impl_js_json_enum(name, data),
        Data::Union(ref _data) => Err("Unions not supported yet".to_string()),
    }
}
