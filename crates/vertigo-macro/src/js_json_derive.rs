use proc_macro::TokenStream;
use syn::Data;

pub(crate) fn impl_js_json_derive(ast: &syn::DeriveInput) -> Result<TokenStream, String> {
    let structure_name = &ast.ident;

    let Data::Struct(ref data) = ast.data else {
        return Err(String::from("This macro can only be used for the structure"));
    };

    let mut field_list = Vec::new();

    for field in data.fields.iter() {
        let Some(field_name) = &field.ident else {
            return Err(String::from("Problem with specifying the field name"));
        };

        field_list.push(field_name);
    }

    let mut list_to_json = Vec::new();
    let mut list_from_json = Vec::new();

    for field_name in field_list {
        let field_name_string = field_name.to_string();
        list_to_json.push(quote!{
            (#field_name_string.to_string(), self.#field_name.to_json()),
        });

        list_from_json.push(quote!{
            #field_name: json.get_property(&context, #field_name_string)?,
        })
    }

    let result = quote! {
        impl vertigo::JsJsonSerialize for #structure_name {
            fn to_json(self) -> vertigo::JsJson {
                vertigo::JsJson::Object(::std::collections::HashMap::from([
                    #(#list_to_json)*
                ]))
            }
        }
        
        impl vertigo::JsJsonDeserialize for #structure_name {
            fn from_json(context: vertigo::JsJsonContext, mut json: vertigo::JsJson) -> Result<Self, vertigo::JsJsonContext> {
                Ok(Self {
                    #(#list_from_json)*
                })
            }
        }
    };

    Ok(result.into())
}
