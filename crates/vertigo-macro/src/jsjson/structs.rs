use darling::FromAttributes;
use proc_macro::TokenStream;
use quote::quote;
use syn::{ext::IdentExt, DataStruct, Ident};

use crate::jsjson::attributes::{ContainerOpts, FieldOpts};

pub(super) fn impl_js_json_struct(
    name: &Ident,
    data: &DataStruct,
    container_opts: ContainerOpts,
) -> Result<TokenStream, String> {
    let mut field_list = Vec::new();

    for field in &data.fields {
        let Some(field_name) = &field.ident else {
            return super::newtypes::impl_js_json_newtype(name, data);
        };

        let attrs = &field.attrs;

        field_list.push((field_name, attrs));
    }

    let mut list_to_json = Vec::new();
    let mut list_from_json = Vec::new();

    for (field_name, attrs) in field_list {
        let field_unraw = field_name.unraw().to_string();
        let field_opts = FieldOpts::from_attributes(attrs).unwrap();

        let json_key = match field_opts.rename {
            Some(json_key) => json_key,
            None => match container_opts.rename_all {
                Some(rule) => rule.rename(&field_unraw),
                None => field_unraw.clone(),
            },
        };

        list_to_json.push(quote! {
            (#json_key.to_string(), self.#field_name.to_json()),
        });

        let unpack_expr = if let Some(default_expr) = field_opts.default {
            quote! {
                .unwrap_or_else(|_| #default_expr)
            }
        } else {
            quote! { ? }
        };

        list_from_json.push(quote! {
            #field_name: json.get_property(&context, #json_key)#unpack_expr,
        })
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
