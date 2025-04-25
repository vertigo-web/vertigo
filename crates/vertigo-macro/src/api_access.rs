use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{spanned::Spanned, Expr, Lit};

pub(crate) fn api_access_inner(root: &str, input: TokenStream) -> TokenStream {
    let input: TokenStream2 = input.into();

    use syn::parse::Parser;
    let data = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated
        .parse2(input)
        .unwrap();

    let mut param_iter = data.into_iter();
    let first_param = param_iter.next().unwrap();
    let first_param_span = first_param.span();

    let inner = match first_param {
        Expr::Lit(expr_lit) => {
            match expr_lit.lit {
                Lit::Str(param_lit) => {
                    let param_repr = param_lit.token().to_string();
                    let param_str = param_repr.trim_matches('"');
                    if let Some(func_name) = param_str.strip_suffix("()") {
                        // Function call
                        let mut args = vec![];

                        for arg in param_iter {
                            args.push(quote! { vertigo::JsValue::from(#arg) })
                        }

                        quote! { .call(#func_name, vec![ #(#args,)* ]) }
                    } else {
                        // Property get
                        if let Some(arg) = param_iter.next() {
                            diagnostic!(
                                arg.span(),
                                proc_macro_error::Level::Error,
                                "Properties don't accept arguments, missing () in func name?"
                            )
                                .span_suggestion(first_param_span, "hint", format!("Try `{param_str}()`"))
                                .emit()
                        }

                        quote! { .get(#param_str) }
                    }
                }
                _ => {
                    emit_error!(expr_lit.span(), "Expected literal string as first parameter (property or function name) (1)");
                    quote! {}
                }
            }
        }
        _ => {
            emit_error!(first_param_span, "Expected literal string as first parameter (property or function name) (2)");
            quote! {}
        }
    };

    quote! {
        vertigo::get_driver()
            .dom_access()
            .root(#root)
            #inner
            .fetch()
    }.into()
}
