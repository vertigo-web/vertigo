use proc_macro::TokenStream;

pub(crate) fn impl_single_request_trait_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl vertigo::SingleRequestTrait for #name {
            fn into_string(self) -> Result<String, String> {
                vertigo::serde_json::to_string(&self)
                    .map_err(|err| format!("error serializing {}", err))
            }

            fn from_string(data: &str) -> Result<Self, String> {
                vertigo::serde_json::from_str::<Self>(data)
                    .map_err(|err| format!("error deserializing {}", err))
            }
        }
    };
    gen.into()
}

pub(crate) fn impl_list_request_trait_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl vertigo::ListRequestTrait for #name {
            fn list_into_string(vec: Vec<Self>) -> Result<String, String> {
                vertigo::serde_json::to_string::<Vec<Self>>(&vec)
                    .map_err(|err| format!("error serializing list {}", err))
            }

            fn list_from_string(data: &str) -> Result<Vec<Self>, String> {
                let result = vertigo::serde_json::from_str::<Vec<#name>>(data)
                    .map_err(|err| format!("error deserializing list {}", err))?;

                Ok(result)
            }
        }
    };
    gen.into()
}
