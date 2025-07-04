use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Attribute, Expr, Meta};

fn base_from_attrs(attrs: &[Attribute]) -> Option<TokenStream2> {
    if let Some(attr) = attrs.first() {
        match &attr.meta {
            Meta::Path(p) => {
                return Some(quote! {
                    #p
                })
            }
            _ => emit_warning!(attr.span(), "Invalid ref, should be #[DomElementRef]"),
        }
    }
    None
}

pub(crate) fn api_access(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Expr);

    let mut base = None;

    match &input {
        Expr::MethodCall(call) => {
            base = base_from_attrs(&call.attrs);
        }
        Expr::Field(field) => {
            base = base_from_attrs(&field.attrs);
        }
        Expr::Call(call) => {
            base = base_from_attrs(&call.attrs);
        }
        Expr::Path(path) => {
            base = base_from_attrs(&path.attrs);
        }
        _ => {
            emit_warning!(input.span(), "Unsupported base {:?}", input)
        }
    }

    // Generate the output code
    let output = generate_calls(&input, base.is_some());

    let base = base.unwrap_or_else(|| quote! { vertigo::get_driver() });

    TokenStream::from(quote! {
        #base
            .dom_access()
            #output
            .fetch()
    })
}

fn generate_calls(expr: &Expr, have_base: bool) -> proc_macro2::TokenStream {
    match expr {
        Expr::MethodCall(method_call) => {
            // Handle the last method call
            let receiver = &method_call.receiver;
            let method = &method_call.method;
            let args = &method_call.args.iter().collect::<Vec<_>>();

            // Generate the call for the current method
            let mut calls = quote! {
                .call(stringify!(#method), vec![#((#args).into()),*])
            };

            // If the receiver is another method call, property, or path, generate calls recursively
            match &**receiver {
                Expr::MethodCall(_) => {
                    // Handle intermediate method call
                    let inner_calls = generate_calls(receiver, have_base);
                    calls = quote! {
                        #inner_calls #calls
                    };
                }
                Expr::Field(field) => {
                    // Handle intermediate property (after another property: root.property1.property2.method())
                    let field_name = &field.member;
                    let inner_calls = generate_calls(&field.base, have_base);
                    calls = quote! {
                        #inner_calls.get(stringify!(#field_name)) #calls
                    };
                }
                Expr::Path(path) => {
                    // Handle root directly before a method ( root.method() )
                    if have_base {
                        calls = quote! {
                            .get(stringify!(#path)) #calls
                        };
                    } else {
                        calls = quote! {
                            .root(stringify!(#path)) #calls
                        };
                    }
                }
                Expr::Call(call) => {
                    // Handle root before a property ( root.property.method() )
                    let func = &call.func;
                    let args = &call.args.iter().collect::<Vec<_>>();

                    // Generate the call for the current method
                    if have_base {
                        calls = quote! {
                            .call(stringify!(#func), vec![#((#args).into()),*]) #calls
                        };
                    } else {
                        calls = quote! {
                            .root("window")
                            .call(stringify!(#func), vec![#((#args).into()),*]) #calls
                        };
                    }
                }
                _ => {
                    emit_error!(receiver.span(), "Unsupported receiver: {:?}", receiver);
                }
            }

            quote! {
                #calls
            }
        }
        Expr::Field(field) => {
            // Handle property after root ( root.property.method() )
            let field_name = &field.member;
            let calls = generate_calls(&field.base, have_base);
            quote! {
                 #calls .get(stringify!(#field_name))
            }
        }
        Expr::Path(path) => {
            // Handle root before a property ( root.property.method() )
            let ident = &path.path;
            if have_base {
                quote! {
                    .get(stringify!(#ident))
                }
            } else {
                quote! {
                    .root(stringify!(#ident))
                }
            }
        }
        Expr::Call(call) => {
            // Handle root before a property ( root.property.method() )
            let func = &call.func;
            let args = &call.args.iter().collect::<Vec<_>>();

            // Generate the call for the current method
            quote! {
                .call(stringify!(#func), vec![#((#args).into()),*])
            }
        }
        _ => {
            emit_error!(
                expr.span(),
                "Expected an expression resulting in a field, call or method call, got {:?}",
                expr
            );
            quote! {}
        }
    }
}
