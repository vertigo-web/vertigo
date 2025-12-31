use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Field, Ident, punctuated::Iter};

/// Takes tuple fields and returns (generated fields' names, generated encodes)
pub(super) fn get_encodes(fields_iter: Iter<'_, Field>) -> (Vec<Ident>, Vec<TokenStream>) {
    let field_idents = fields_iter
        .enumerate()
        .map(|(n, field)| Ident::new(&format!("f_{n}"), field.span()))
        .collect::<Vec<_>>();

    let field_encodes = field_idents
        .iter()
        .map(|field_ident| {
            quote! {
                #field_ident.to_json(),
            }
        })
        .collect::<Vec<_>>();

    (field_idents, field_encodes)
}

/// Takes tuple fields and returns generated encodes
pub(super) fn get_decodes(field_idents: Vec<Ident>) -> Vec<TokenStream> {
    field_idents
        .iter()
        .map(|_| {
            quote! {
                vertigo::JsJsonDeserialize::from_json(ctx.clone(), fields_rev.pop().unwrap())?,
            }
        })
        .collect::<Vec<_>>()
}
