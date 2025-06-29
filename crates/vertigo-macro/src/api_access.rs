use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{spanned::Spanned, Expr, Lit};

pub(crate) fn api_access(input: TokenStream) -> TokenStream {
    let input: TokenStream2 = input.into();

    use syn::parse::Parser;
    let data = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated
        .parse2(input)
        .unwrap();

    let mut param_iter = data.into_iter();
    let first_param = param_iter.next().unwrap();
    let first_param_span = first_param.span();

    let inner = match first_param {
        // something.method(arg)
        Expr::MethodCall(expr_method_call) => {
            emit_call_site_warning!("\n
            attrs: {:#?}
            receiver: {:#?}
            dot_token: {:#?}
            method: {:#?}
            turbofish: {:#?}
            paren_token: {:#?}
            args: {:#?}
            ",
            expr_method_call.attrs,
            expr_method_call.receiver,
            expr_method_call.dot_token,
            expr_method_call.method,
            expr_method_call.turbofish,
            expr_method_call.paren_token,
            expr_method_call.args
            );
            quote! {}
        }
        _ => {
            emit_error!(first_param_span, "Expected literal string as first parameter (property or function name) (2)");
            quote! {}
        }
    };

    quote! {
        vertigo::get_driver()
            .dom_access()
            // .root(#root)
            #inner
            .fetch()
    }.into()
}
