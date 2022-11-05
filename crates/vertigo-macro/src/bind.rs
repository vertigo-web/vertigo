use proc_macro::{TokenStream, TokenTree, Span};
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::Ident as Ident2;
use syn::__private::quote::format_ident;

fn is_char(token: &TokenTree, char: char) -> bool {
    if let TokenTree::Punct(inner) = token {
        inner.as_char() == char
    } else {
        false
    }
}

fn find_param_name2(params: &[TokenTree]) -> Option<Ident2> {
    if params.len() == 1 {
        if let Some(first) = params.first() {
            return if let TokenTree::Ident(value) = &first {
                Some(format_ident!("{}", value.to_string()))
            } else {
                emit_error!(Span::call_site(), "Can't find variable name, expected ident");
                None
            };
        }
    }

    emit_error!(Span::call_site(), "Can't find variable name, expected ident");
    None
}

fn is_first_pipe_char(list: &[TokenTree]) -> bool {
    let Some(first) = list.first() else {
        return false;
    };

    let TokenTree::Punct(char) = first else {
        return false;
    };

    char.as_char() == '|'
}

pub fn bind_macro_fn(input: TokenStream) -> TokenStream {
    let tokens = input.into_iter().collect::<Vec<_>>();

    let mut chunks = tokens
        .as_slice()
        .split(|token| {
            is_char(token, ',')
        })
        .collect::<Vec<_>>();

    let mut clone_stm = Vec::<TokenStream2>::new();
    let body = chunks.pop().unwrap();
    let first_pipe = is_first_pipe_char(body);
    let body: TokenStream2 = body.iter().cloned().collect::<TokenStream>().into();

    for item in chunks {
        let Some(name_param) = find_param_name2(item) else {
            return quote! {""}.into();
        };

        clone_stm.push(quote!{
            let #name_param = #name_param.clone();
        });
    }

    if first_pipe {
        quote! {
            {
                #(#clone_stm)*

                move #body
            }
        }.into()
    } else {
        quote! {
            {
                #(#clone_stm)*

                #body
            }
        }.into()
    }
}
