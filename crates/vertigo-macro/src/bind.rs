use std::collections::VecDeque;

use proc_macro::{TokenStream, TokenTree, Span};
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::Ident as Ident2;
use syn::__private::quote::format_ident;

fn split_by_token(mut tokens: VecDeque<TokenTree>, pattern_punct: char) -> Option<(VecDeque<TokenTree>, TokenTree, VecDeque<TokenTree>)> {
    let mut left = VecDeque::new();

    loop {
        if let Some(token) = tokens.pop_front() {
            let pattern_match = if let TokenTree::Punct(inner) = &token {
                inner.as_char() == pattern_punct
            } else {
                false
            };
    
            if pattern_match {
                return Some((left, token, tokens));
            }
    
            left.push_back(token);
        } else {
            return None;
        }
    }
}

type SplitTokenResult = (VecDeque<TokenTree>, TokenTree, VecDeque<TokenTree>, TokenTree, VecDeque<TokenTree>);

fn split_by_token2(tokens: VecDeque<TokenTree>, pattern1: char, pattern2: char) -> Option<SplitTokenResult> {
    let (left, token1, rest) = split_by_token(tokens, pattern1)?;
    let (center, token2, right) = split_by_token(rest, pattern2)?;
    Some((left, token1, center, token2, right))
}

fn split_by_function_brackets(tokens: VecDeque<TokenTree>) -> Option<(Vec<TokenTree>, Vec<TokenTree>)> {
    let (left, _, center, _, right) = split_by_token2(tokens, '|', '|')?;

    if !left.is_empty() {
        emit_error!(Span::call_site(), "Too many arguments before first pipe (vertical bar) sign");
        return None;
    }

    Some((
        center.into_iter().collect::<Vec<_>>(),
        right.into_iter().collect::<Vec<_>>(),
    ))
}

fn is_char(token: &TokenTree, char: char) -> bool {
    if let TokenTree::Punct(inner) = token {
        inner.as_char() == char
    } else {
        false
    }
}

enum ParamResult {
    Name(Ident2),
    TypeDeclaration(TokenStream2),
    Error
}

fn find_param_name(params: &[TokenTree]) -> ParamResult {
    if params.len() == 1 {
        if let Some(first) = params.first() {
            return if let TokenTree::Ident(value) = &first {
                ParamResult::Name(format_ident!("{}", value.to_string()))
            } else {
                emit_error!(Span::call_site(), "Can't find variable name, expected ident");
                ParamResult::Error
            };
        }
    }

    let params_setrem: TokenStream2 = params.iter().cloned().collect::<TokenStream>().into();
    ParamResult::TypeDeclaration(params_setrem)
}

pub fn bind_macro_fn(input: TokenStream) -> TokenStream {
    let mut clone_stm = Vec::<TokenStream2>::new();
    let mut params = Vec::<TokenStream2>::new();

    let body = if let Some((center, body)) = split_by_function_brackets(input.into_iter().collect::<VecDeque<_>>()) {
        let chunks = center
            .as_slice()
            .split(|token| {
                is_char(token, ',')
            });

        for chunk in chunks {
            let first_name = find_param_name(chunk);

            match first_name {
                ParamResult::TypeDeclaration(type_declaration) => {
                    params.push(type_declaration);
                },
                ParamResult::Name(name_param) => {
                    clone_stm.push(quote!{
                        let #name_param = #name_param.clone();
                    });
                },
                ParamResult::Error => {
                    emit_error!(Span::call_site(), "{}", "error przetwarzania pierwszego przetwarzania");
                }
            }
        }

        body
    } else {
        return quote!{""}.into();
    };
    
    let body: TokenStream2 = body.into_iter().collect::<TokenStream>().into();

    let result: TokenStream = quote! {
        {
            #(#clone_stm)*

            move |#(#params)*| #body
        }
    }.into();

    result
}
