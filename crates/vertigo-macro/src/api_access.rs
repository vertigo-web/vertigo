use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Expr};

pub(crate) fn api_access(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Expr);

    // Generate the output code
    let output = generate_calls(&input);

    TokenStream::from(quote! {
        vertigo::get_driver()
            .dom_access()
            #output
            .fetch()
    })
}

fn generate_calls(expr: &Expr) -> proc_macro2::TokenStream {
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
                    let inner_calls = generate_calls(receiver);
                    calls = quote! {
                        #inner_calls #calls
                    };
                }
                Expr::Field(field) => {
                    // Handle intermediate property (after another property: root.property1.property2.method())
                    let field_name = &field.member;
                    let inner_calls = generate_calls(&field.base);
                    calls = quote! {
                        #inner_calls.get(stringify!(#field_name)) #calls
                    };
                }
                Expr::Path(path) => {
                    // Handle root directly before a method ( root.method() )
                    calls = quote! {
                        .root(stringify!(#path)) #calls
                    };
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
            let calls = generate_calls(&field.base);
            quote! {
                 #calls .get(stringify!(#field_name))
            }
        }
        Expr::Path(path) => {
            // Handle root before a property ( root.property.method() )
            quote! {
                .root(stringify!(#path))
            }
        }
        _ => {
            emit_error!(
                expr.span(),
                "Expected an expression resulting in a method call, got {:?}",
                expr
            );
            quote! {}
        }
    }
}
