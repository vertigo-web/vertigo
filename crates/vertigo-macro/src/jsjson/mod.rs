use darling::FromAttributes;
use proc_macro::TokenStream;
use std::error::Error;
use syn::{Data, DeriveInput};

use crate::jsjson::attributes::ContainerOpts;

mod attributes;
mod enums;
mod newtypes;
mod structs;
mod tuple_fields;

pub(crate) fn impl_js_json_derive(ast: &DeriveInput) -> Result<TokenStream, Box<dyn Error>> {
    let name = &ast.ident;

    let container_opts = ContainerOpts::from_attributes(&ast.attrs)?;

    match ast.data {
        Data::Struct(ref data) => structs::impl_js_json_struct(name, data, container_opts),
        Data::Enum(ref data) => enums::impl_js_json_enum(name, data, container_opts),
        Data::Union(ref _data) => Err("Unions not supported yet".into()),
    }
}
