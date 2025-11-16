use darling::FromAttributes;
use proc_macro::TokenStream;
use syn::{Data, DeriveInput};

use crate::jsjson::attributes::ContainerOpts;

mod attributes;
mod enums;
mod newtypes;
mod structs;
mod tuple_fields;

pub(crate) fn impl_js_json_derive(ast: &DeriveInput) -> Result<TokenStream, String> {
    let name = &ast.ident;

    let container_opts = match ContainerOpts::from_attributes(&ast.attrs) {
        Ok(opts) => opts,
        Err(e) => return Err(e.to_string()),
    };

    match ast.data {
        Data::Struct(ref data) => structs::impl_js_json_struct(name, data, container_opts),
        Data::Enum(ref data) => enums::impl_js_json_enum(name, data, container_opts),
        Data::Union(ref _data) => Err("Unions not supported yet".to_string()),
    }
}
