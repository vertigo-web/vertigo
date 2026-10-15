use proc_macro::TokenStream;
use proc_macro2::{Group, Span, TokenStream as TokenStream2, TokenTree};
use quote::{ToTokens, quote};
use syn::{Attribute, Expr, FnArg, Ident, Meta, Pat, spanned::Spanned};

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

    // a: String = String::new() - props struct, the one `dom!` fills in
    let mut prop_fields = Vec::new();
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
                let default = take_default(&mut pat_type.attrs);
                let attrs = std::mem::take(&mut pat_type.attrs);
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
                let push_method_name = get_group_attrs_push_method_name(&name);
                let replace_method_name = get_group_attrs_replace_method_name(&name);

                if pat_type.ty.to_token_stream().to_string() == "AttrGroup" {
                    if let Some(default) = &default {
                        emit_error!(
                            default.span(),
                            "AttrGroup param is always optional, it can't have a default"
                        );
                    }
                    group_fields.push(quote! {
                        #(#attrs)* pub #pat_type
                    });
                    group_assigments.push(quote! {
                        #name: Default::default()
                    });
                    group_methods.push(quote! {
                        #[doc="Push single attribute to AttrGroup"]
                        pub fn #push_method_name(mut self, key: String, value: vertigo::AttrGroupValue) -> Self {
                            self.#name.insert(key, value);
                            self
                        }
                        #[doc="Replace AttrGroup with new AttrGroup (f. ex. upon spreading into component)"]
                        pub fn #replace_method_name(mut self, group: vertigo::AttrGroup) -> Self {
                            self.#name = group;
                            self
                        }
                    });
                } else {
                    let default = default.as_ref().map(field_default);
                    prop_fields.push(quote! {
                        #(#attrs)* pub #pat_type #default
                    });
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
        if let syn::FnArg::Typed(pat_type) = param
            && let syn::Pat::Ident(ident) = &*pat_type.pat
        {
            let param_name = ident.ident.clone();
            let mutability = ident.mutability;

            param_assignments.push(quote! {
                let #mutability #param_name = self.#param_name;
            });
        }
    }

    let visibility = &ast.vis;

    let component_name = get_component_name(&name);

    let result = quote! {
        #(#fn_attrs)*
        #visibility struct #name #impl_generics #where_clause {
            #(#prop_fields,)*
        }

        #visibility struct #component_name #impl_generics #where_clause {
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

/// Takes `#[default = <expr>]` out of a param's attributes.
fn take_default(attrs: &mut Vec<Attribute>) -> Option<Expr> {
    let mut default = None;
    attrs.retain(|attr| {
        if !attr.path().is_ident("default") {
            return true;
        }
        match &attr.meta {
            Meta::NameValue(name_value) if default.is_none() => {
                default = Some(name_value.value.clone());
            }
            Meta::NameValue(_) => emit_error!(attr.span(), "Duplicated default value"),
            _ => emit_error!(
                attr.span(),
                "Expected `#[default = <value>]`, the value being a constant expression, f. ex. `0.0` or `String::new()`"
            ),
        }
        false
    });
    default
}

/// ` = <expr>` for a prop's field in the props struct, or nothing when the compiler can't
/// do default field values (see build.rs) - then the prop is required like any other.
fn field_default(expr: &Expr) -> TokenStream2 {
    if cfg!(vertigo_default_props) {
        let expr = resolved_at_call_site(expr.to_token_stream());
        quote! { = #expr }
    } else {
        quote! {}
    }
}

/// Keeps the location of every token, so errors still point into the user's code, but makes
/// it resolve as this macro's output, which is what `allow_internal_unstable` looks at.
fn resolved_at_call_site(tokens: TokenStream2) -> TokenStream2 {
    tokens
        .into_iter()
        .map(|token| {
            let span = token.span().resolved_at(Span::call_site());
            let mut token = match token {
                TokenTree::Group(group) => TokenTree::Group(Group::new(
                    group.delimiter(),
                    resolved_at_call_site(group.stream()),
                )),
                other => other,
            };
            token.set_span(span);
            token
        })
        .collect()
}

pub fn get_component_name<T: ToTokens + Spanned>(constructor_name: &T) -> Ident {
    Ident::new(
        // Create a name which should not conflict with anything.
        // Underscore prefix makes it not appear in autocompletion.
        &format!("__{}Component", constructor_name.to_token_stream()),
        constructor_name.span(),
    )
}

/// Name for method to push single attribute to AttrGroup
pub fn get_group_attrs_push_method_name<T: ToTokens + Spanned>(group_name: &T) -> Ident {
    Ident::new(
        &format!("group_{}_push", group_name.to_token_stream()),
        group_name.span(),
    )
}

/// Name for method to replace AttrGroup with new AttrGroup
/// (f. ex. upon spreading into component)
pub fn get_group_attrs_replace_method_name<T: ToTokens + Spanned>(group_name: &T) -> Ident {
    Ident::new(
        &format!("group_{}_replace", group_name.to_token_stream()),
        group_name.span(),
    )
}
