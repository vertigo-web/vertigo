use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Expr, Meta};

/// Converts pseudo-javascript expression to DomAccess chain.
pub(crate) fn js_expression(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Expr);

    // Extract possible #[node_ref] as a base
    let base = extract_base(&input);

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

/// Recursively generate consecutive calls to DomAccess methods based on pseudo-javascript expression.
fn generate_calls(expr: &Expr, have_base: bool) -> proc_macro2::TokenStream {
    match expr {
        Expr::MethodCall(method_call) => {
            let receiver = &method_call.receiver;
            let method = &method_call.method;
            let args = &method_call.args.iter().collect::<Vec<_>>();

            // If the receiver is another method call, property, or path, generate calls recursively
            let receiver_calls = generate_calls(receiver, have_base);

            quote! {
                #receiver_calls.call(stringify!(#method), vec![#((#args).into()),*])
            }
        }
        Expr::Field(field) => {
            let field_name = &field.member;
            let parent_calls = generate_calls(&field.base, have_base);
            quote! {
                #parent_calls.get(stringify!(#field_name))
            }
        }
        Expr::Path(path) => {
            // Handle root before a property ( document.property )
            // or a property after a node reference ( #node_ref.firstChild )
            let path = &path.path;
            let method = if have_base {
                quote! { get }
            } else {
                quote! { root }
            };
            quote! {
                .#method(stringify!(#path))
            }
        }
        Expr::Call(call) => {
            // Handle bare function (add `window` as a root if no base (no node_ref))
            let func = &call.func;
            let args = &call.args.iter().collect::<Vec<_>>();

            let root = (!have_base).then_some(quote! { .root("window") });
            quote! {
                #root
                .call(stringify!(#func), vec![#((#args).into()),*])
            }
        }
        Expr::Group(group) => {
            // Just unwrap group - this happens for node references ( #node_ref.[group] )
            generate_calls(&group.expr, have_base)
        }
        _ => {
            emit_error!(
                expr.span(),
                "Expected an expression resulting in a field, call or method call, got {:#?}",
                expr
            );
            quote! {}
        }
    }
}

// Extracting base is the same for every type.
fn extract_base(input: &Expr) -> Option<TokenStream2> {
    let attrs = match &input {
        Expr::MethodCall(call) => &call.attrs,
        Expr::Field(field) => &field.attrs,
        Expr::Call(call) => &call.attrs,
        Expr::Path(path) => &path.attrs,
        Expr::Group(group) => &group.attrs,
        _ => {
            emit_warning!(input.span(), "Unsupported base {:?}", input);
            return None;
        }
    };

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
