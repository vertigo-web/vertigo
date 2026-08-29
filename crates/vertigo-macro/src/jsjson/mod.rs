use darling::FromAttributes;
use proc_macro::TokenStream;
use quote::quote;
use std::error::Error;
use syn::{Data, DeriveInput, spanned::Spanned};

use crate::jsjson::attributes::ContainerOpts;

mod attributes;
mod enums;
mod newtypes;
mod structs;
mod tuple_fields;

/// Assembles a `JsJson::Object` out of `vertigo::object_insert` calls against [`object_ident`].
///
/// See `object_insert` for why the fields go in one at a time rather than through
/// `BTreeMap::from([..])`.
pub(super) fn js_json_object(inserts: &[proc_macro2::TokenStream]) -> proc_macro2::TokenStream {
    let object = object_ident();

    quote! {
        {
            let mut #object = ::std::collections::BTreeMap::new();
            #(#inserts)*
            vertigo::JsJson::Object(#object)
        }
    }
}

/// The local [`js_json_object`] builds into, named so it cannot collide with a field of the
/// item being derived - the inserts sit in the same scope as destructured fields.
pub(super) fn object_ident() -> syn::Ident {
    syn::Ident::new("__vertigo_json_object", proc_macro2::Span::call_site())
}

/// `Vec<u8>` travels as [`JsJson::Vec`] - one length-prefixed byte run - rather than as a
/// list of numbers, which would cost a `JsJson::Number` per byte.
///
/// Shared between the struct and enum encoders because they must agree: a field of the same
/// type has to serialize the same way whichever kind of item declares it.
pub(super) fn is_vec_u8(ty: &syn::Type) -> bool {
    let Ok(vec_u8_type) = syn::parse2::<syn::Type>(quote! { Vec<u8> }) else {
        emit_error!(ty.span(), "Unreachable: Unable to parse Vec<u8>");
        return false;
    };
    ty == &vec_u8_type
}

pub(crate) fn impl_js_json_derive(ast: &DeriveInput) -> Result<TokenStream, Box<dyn Error>> {
    let name = &ast.ident;

    let container_opts = ContainerOpts::from_attributes(&ast.attrs)?;

    match ast.data {
        Data::Struct(ref data) => structs::impl_js_json_struct(name, data, container_opts),
        Data::Enum(ref data) => enums::impl_js_json_enum(name, data, container_opts),
        Data::Union(ref _data) => Err("Unions not supported yet".into()),
    }
}
