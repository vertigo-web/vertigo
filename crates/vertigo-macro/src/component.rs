use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, FnArg, Ident, Pat, Visibility};

pub(crate) fn component_inner(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::ItemFn);

    // Function name
    let name = &ast.sig.ident;

    let fn_attrs = &ast.attrs;

    // Generics and lifetimes
    let (impl_generics, ty_generics, where_clause) = &ast.sig.generics.split_for_impl();

    if ast.sig.output.to_token_stream().to_string() != "" {
        emit_error!(
            ast.sig.output.span(),
            "{} => \"{}\"",
            "Remove the information about the returned type. A component always returns DomNode",
            ast.sig.output.to_token_stream().to_string()
        );
        return quote! {}.into();
    }

    // a: String
    let mut struct_fields = Vec::new();
    // a: self.a
    let mut struct_assignments = Vec::new();
    // rest: AttrGroup
    let mut group_fields = Vec::new();
    // rest: Default::default()
    let mut group_assigments = Vec::new();
    // pub fn group_rest_push(&self, key: String, value: AttrGroupValue) -> Self { ..
    let mut group_methods = Vec::new();

    for field in ast.sig.inputs.clone().into_iter() {
        match field {
            FnArg::Receiver(_) => unreachable!(),
            FnArg::Typed(mut pat_type) => {
                let attrs = pat_type.attrs.drain(..);
                let entry = match pat_type.pat.as_mut() {
                    Pat::Ident(ident) => {
                        ident.mutability = None;
                        ident.clone()
                    }
                    _ => {
                        emit_warning!(pat_type.pat.span(), "Expected ident");
                        continue;
                    }
                };
                let name = entry.ident.clone();
                let method_name = get_group_attrs_method_name(&name);
                if pat_type.ty.to_token_stream().to_string() == "AttrGroup" {
                    group_fields.push(quote! {
                        #(#attrs)* pub #pat_type
                    });
                    group_assigments.push(quote! {
                        #name: Default::default()
                    });
                    group_methods.push(quote! {
                        pub fn #method_name(mut self, key: String, value: vertigo::AttrGroupValue) -> Self {
                            self.#name.insert(key, value);
                            self
                        }
                    });
                } else {
                    struct_fields.push(quote! {
                        #(#attrs)* pub #pat_type
                    });
                    struct_assignments.push(quote! {
                        #name: self.#name
                    });
                }
            }
        }
    }

    let body = ast.block;

    let mut param_assignments = Vec::new();
    for param in &ast.sig.inputs {
        if let syn::FnArg::Typed(pat_type) = param {
            if let syn::Pat::Ident(ident) = &*pat_type.pat {
                let param_name = ident.ident.clone();
                let mutability = ident.mutability;

                param_assignments.push(quote! {
                    let #mutability #param_name = self.#param_name;
                });
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

    let component_name = get_component_name(&name);

    let result = quote! {
        #(#fn_attrs)*
        #visibility2 struct #name #impl_generics #where_clause {
            #(#struct_fields,)*
        }

        #visibility2 struct #component_name #impl_generics #where_clause {
            #(#struct_fields,)*
            #(#group_fields,)*
        }

        impl #impl_generics #name #ty_generics #where_clause {
            pub fn into_component(self) -> #component_name #ty_generics {
                #component_name {
                    #(#struct_assignments,)*
                    #(#group_assigments,)*
                }
            }
            #[doc="Shorthand for `.into_component().mount()` - bypasses setting of dynamic attributes (AttrGroup)."]
            pub fn mount(self) -> vertigo::DomNode {
                self.into_component().mount()
            }
        }

        impl #impl_generics #component_name #ty_generics #where_clause {
            #(#group_methods)*

            pub fn mount(self) -> vertigo::DomNode {
                #(#param_assignments)*

                (#body).into()
            }
        }
    };

    result.into()
}

pub fn get_component_name<T: ToTokens + Spanned>(constructor_name: &T) -> Ident {
    Ident::new(
        // Create a name which should not conflict with anything.
        // Underscore prefix makes it not appear in autocompletion.
        &format!("__{}Component", constructor_name.to_token_stream()),
        constructor_name.span(),
    )
}

pub fn get_group_attrs_method_name<T: ToTokens + Spanned>(group_name: &T) -> Ident {
    Ident::new(
        &format!("group_{}_push", group_name.to_token_stream()),
        group_name.span(),
    )
}
