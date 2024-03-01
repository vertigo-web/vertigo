use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{Visibility, __private::ToTokens};

pub(crate) fn component_inner(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::ItemFn);

    // Function name
    let name = &ast.sig.ident;

    // Generics and lifetimes
    let (impl_generics, ty_generics, where_clause) = &ast.sig.generics.split_for_impl();

    if ast.sig.output.to_token_stream().to_string() != "" {
        emit_error!(
            Span::call_site(),
            "{} => \"{}\"",
            "remove the information about the returned type. A component always returns DomNode",
            ast.sig.output.to_token_stream().to_string()
        );
        return quote! {}.into();
    }

    let mut struct_fields = Vec::new();

    for field in ast.sig.inputs.iter() {
        struct_fields.push(quote! {
            pub #field
        })
    }

    let body = ast.block;

    let mut param_names = Vec::new();
    for param in &ast.sig.inputs {
        if let syn::FnArg::Typed(pat_type) = param {
            if let syn::Pat::Ident(ident) = &*pat_type.pat {
                let param_name = ident.ident.clone();

                param_names.push(quote! {
                    let #param_name = self.#param_name;
                })
            }
        }
    }

    let visibility = &ast.vis;

    let visibility2 = match visibility {
        Visibility::Public(_) => {
            quote! {
                pub
            }
        }
        _ => {
            quote! {}
        }
    };

    let result = quote! {
        #visibility2 struct #name #impl_generics #where_clause {
            #(#struct_fields,)*
        }

        impl #impl_generics #name #ty_generics #where_clause {
            pub fn mount(self) -> vertigo::DomNode {
                #(#param_names)*

                (#body).into()
            }
        }
    };

    result.into()
}
