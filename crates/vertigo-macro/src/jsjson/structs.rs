use proc_macro::TokenStream;
use quote::quote;
use syn::{ext::IdentExt, DataStruct, Ident};

pub(super) fn impl_js_json_struct(name: &Ident, data: &DataStruct) -> Result<TokenStream, String> {
    let mut field_list = Vec::new();

    for field in &data.fields {
        let Some(field_name) = &field.ident else {
            return super::newtypes::impl_js_json_newtype(name, data);
        };

        field_list.push(field_name);
    }

    let mut list_to_json = Vec::new();
    let mut list_from_json = Vec::new();

    for field_name in field_list {
        let field_unraw = field_name.unraw().to_string();
        list_to_json.push(quote! {
            (#field_unraw.to_string(), self.#field_name.to_json()),
        });

        list_from_json.push(quote! {
            #field_name: json.get_property(&context, #field_unraw)?,
        })
    }

    let result = quote! {
        impl vertigo::JsJsonSerialize for #name {
            fn to_json(self) -> vertigo::JsJson {
                vertigo::JsJson::Object(::std::collections::HashMap::from([
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
