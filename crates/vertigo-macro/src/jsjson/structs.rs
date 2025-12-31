use darling::FromAttributes;
use proc_macro::TokenStream;
use quote::quote;
use std::error::Error;
use syn::{DataStruct, Ident, ext::IdentExt, spanned::Spanned};

use crate::jsjson::attributes::{ContainerOpts, FieldOpts};

fn is_vec_u8(ty: &syn::Type) -> bool {
    let Ok(vec_u8_type) = syn::parse2::<syn::Type>(quote! { Vec<u8> }) else {
        emit_error!(ty.span(), "Unreachable: Unable to parse Vec<u8>");
        return false;
    };
    ty == &vec_u8_type
}

pub(super) fn impl_js_json_struct(
    name: &Ident,
    data: &DataStruct,
    container_opts: ContainerOpts,
) -> Result<TokenStream, Box<dyn Error>> {
    let mut field_list = Vec::new();

    for field in &data.fields {
        let Some(field_name) = &field.ident else {
            return super::newtypes::impl_js_json_newtype(name, data);
        };

        let attrs = &field.attrs;

        field_list.push((field_name, attrs, &field.ty));
    }

    let mut list_to_json = Vec::new();
    let mut list_from_json = Vec::new();

    for (field_name, attrs, field_ty) in field_list {
        let field_unraw = field_name.unraw().to_string();
        let field_opts = FieldOpts::from_attributes(attrs)?;

        let json_key = match field_opts.rename {
            Some(json_key) => json_key,
            None => match container_opts.rename_all {
                Some(rule) => rule.rename(&field_unraw),
                None => field_unraw.clone(),
            },
        };

        let unpack_expr = if let Some(default_expr) = field_opts.default {
            quote! {
                .unwrap_or_else(|_| #default_expr)
            }
        } else {
            quote! { ? }
        };

        if is_vec_u8(field_ty) {
            list_to_json.push(quote! {
                (#json_key.to_string(), vertigo::JsJson::Vec(self.#field_name)),
            });

            list_from_json.push(quote! {
                #field_name: json.get_property_jsjson(&context, #json_key).and_then(|item| {
                    match item {
                        vertigo::JsJson::Vec(v) => Ok(v),
                        other => {
                            let message = ["Vec<u8> expected, received ", other.typename()].concat();
                            Err(context.add(message))
                        }
                    }
                })#unpack_expr,
            })
        } else {
            list_to_json.push(quote! {
                (#json_key.to_string(), self.#field_name.to_json()),
            });

            list_from_json.push(quote! {
                #field_name: json.get_property(&context, #json_key)#unpack_expr,
            })
        }
    }

    let result = quote! {
        impl vertigo::JsJsonSerialize for #name {
            fn to_json(self) -> vertigo::JsJson {
                vertigo::JsJson::Object(::std::collections::BTreeMap::from([
                    #(#list_to_json)*
                ]))
            }
        }

        impl vertigo::JsJsonDeserialize for #name {
            fn from_json(context: vertigo::JsJsonContext, mut json: vertigo::JsJson) -> Result<Self, vertigo::JsJsonContext> {
                Ok(Self {
                    #(#list_from_json)*
                })
            }
        }
    };

    Ok(result.into())
}
