use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use std::{
    collections::{HashSet, VecDeque},
    error::Error,
};
use syn::__private::quote::format_ident;

pub(crate) fn bind_inner(input: TokenStream) -> Option<TokenStream> {
    let input: TokenStream2 = input.into();
    let tokens = input.into_iter().collect::<Vec<_>>();

    let TokensParamsBody {
        bind_params,
        func_params: _,
        body,
    } = split_params_and_body(tokens.as_slice())?;

    let mut clone_stm = Vec::<TokenStream2>::new();
    let first_pipe = is_first_pipe_char(body.as_slice());
    let body = body.iter().cloned().collect::<TokenStream2>();

    let mut idents_seen = HashSet::new();

    for item in bind_params {
        let param_name = find_param_name(item)?;

        if !idents_seen.insert(param_name.clone()) {
            emit_error!(
                item.last()
                    .map(|i| i.span())
                    .unwrap_or_else(Span::call_site),
                "Conflicting variable name: {}",
                param_name
            );
        }

        let item_expr = item.iter().cloned().collect::<TokenStream2>();

        clone_stm.push(quote! {
            let #param_name = #item_expr.clone();
        });
    }

    let bind_result = if first_pipe {
        quote! {
            {
                #(#clone_stm)*

                move #body
            }
        }
    } else {
        quote! {
            {
                #(#clone_stm)*

                #body
            }
        }
    };

    Some(bind_result.into())
}

pub(crate) fn bind_spawn_inner(input: TokenStream) -> Option<TokenStream> {
    let input: TokenStream2 = input.into();
    let tokens = input.into_iter().collect::<Vec<_>>();

    let TokensParamsBody {
        bind_params,
        func_params,
        mut body,
    } = split_params_and_body(tokens.as_slice())?;

    let bind_params: Vec<TokenStream2> = bind_params
        .into_iter()
        .map(convert_tokens_to_stream)
        .collect::<Vec<_>>();

    let func_params: Vec<TokenStream2> = func_params
        .unwrap_or_default()
        .into_iter()
        .map(convert_tokens_to_stream)
        .collect::<Vec<_>>();

    // Remove possible parameters from body
    let mut start_from = 0;
    let mut pipes = 0;
    for (i, token) in body.clone().into_iter().enumerate() {
        if let TokenTree::Punct(punct) = token {
            if punct.as_char() == '{' {
                break;
            }
            if punct.as_char() == '|' {
                pipes += 1;
            }
            if pipes >= 2 {
                start_from = i + 1;
                break;
            }
        }
    }

    if start_from > 0 {
        body = body.splice(start_from.., []).collect();
    }

    let inner_body: TokenStream2 = TokenStream2::from_iter(body);

    Some(
        quote! {
            {
                vertigo::bind!(#(#bind_params,)* |#(#func_params,)*| {
                    vertigo::get_driver().spawn(vertigo::bind!(#(#bind_params,)* #inner_body));
                })
            }
        }
        .into(),
    )
}

pub(crate) fn bind_rc_inner(input: TokenStream) -> Option<TokenStream> {
    let input: TokenStream2 = input.into();
    let tokens = input.into_iter().collect::<Vec<_>>();

    let TokensParamsBody {
        bind_params,
        func_params,
        body,
    } = split_params_and_body(tokens.as_slice())?;

    let bind_params: Vec<TokenStream2> = bind_params
        .into_iter()
        .map(convert_tokens_to_stream)
        .collect::<Vec<_>>();

    let Some(func_params) = func_params else {
        emit_call_site_error!("The macro can only take functions");
        return None;
    };

    let types = {
        let mut types_macro: Vec<TokenStream2> = Vec::new();

        for type_items in func_params.into_iter() {
            let Ok(type_item) = get_type(type_items) else {
                emit_error!(
                    type_items
                        .first()
                        .map(|i| i.span())
                        .unwrap_or_else(Span::call_site),
                    "The macro can only take functions"
                );
                return None;
            };

            let type_item = convert_tokens_to_stream(type_item);
            types_macro.push(type_item);
        }

        types_macro
    };

    let body: TokenStream2 = convert_tokens_to_stream(body.as_slice());

    Some(quote!{
        {
            let func: std::rc::Rc::<dyn Fn(#(#types,)*) -> _> = std::rc::Rc::new(vertigo::bind!(#(#bind_params,)* #body));
            func
        }
    }.into())
}

fn is_char(token: &TokenTree, char: char) -> bool {
    if let TokenTree::Punct(inner) = token {
        inner.as_char() == char
    } else {
        false
    }
}

fn find_param_name(params: &[TokenTree]) -> Option<Ident> {
    if let Some(last) = params.last() {
        if let TokenTree::Ident(value) = &last {
            Some(format_ident!("{}", value.to_string()))
        } else {
            emit_error!(
                Span::call_site(),
                "Can't find variable name, expected ident (1)"
            );
            None
        }
    } else {
        emit_error!(
            Span::call_site(),
            "Can't find variable name, expected ident (2)"
        );
        None
    }
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

struct TokensParamsBody<'a> {
    bind_params: Vec<&'a [TokenTree]>,
    func_params: Option<Vec<&'a [TokenTree]>>,
    body: Vec<TokenTree>,
}

fn contains_bracket(tokens: &[TokenTree]) -> bool {
    for token in tokens {
        if let TokenTree::Punct(inner) = token
            && inner.as_char() == '|'
        {
            return true;
        }
    }

    false
}

fn split_params_and_body_function(tokens: &'_ [TokenTree]) -> Option<TokensParamsBody<'_>> {
    let mut chunks = tokens
        .split(|token| is_char(token, '|'))
        .collect::<VecDeque<_>>();

    if chunks.len() > 3 {
        emit_error!(tokens[3].span(), "Too many brackets '|' (2 were expected)");
        return None;
    }

    let Some(params_chunk) = chunks.pop_front() else {
        emit_call_site_error!("Two brackets '|' were expected");
        return None;
    };

    let bind_params = params_chunk
        .split(|token| is_char(token, ','))
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>();

    let Some(func_params) = chunks.pop_front() else {
        emit_error!(tokens[0].span(), "Two brackets '|' were expected");
        return None;
    };

    let func_params = func_params
        .split(|token| is_char(token, ','))
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>();

    let body = {
        let mut occurred_bracket = false;
        let mut body = Vec::new();
        for token in tokens {
            if is_char(token, '|') {
                occurred_bracket = true;
            }

            if occurred_bracket {
                body.push(token.clone());
            }
        }

        body
    };

    Some(TokensParamsBody {
        bind_params,
        func_params: Some(func_params),
        body,
    })
}

fn split_params_and_body_block(tokens: &[TokenTree]) -> Option<TokensParamsBody<'_>> {
    let mut chunks = tokens
        .split(|token| is_char(token, ','))
        .collect::<Vec<_>>();

    let Some(body) = chunks.pop() else {
        emit_call_site_error!("Two brackets '|' were expected");
        return None;
    };

    Some(TokensParamsBody {
        bind_params: chunks,
        func_params: None,
        body: body.to_vec(),
    })
}

fn split_params_and_body(tokens: &[TokenTree]) -> Option<TokensParamsBody<'_>> {
    let bracket_contain = contains_bracket(tokens);

    if bracket_contain {
        split_params_and_body_function(tokens)
    } else {
        split_params_and_body_block(tokens)
    }
}

fn convert_tokens_to_stream(tokens: &[TokenTree]) -> TokenStream2 {
    tokens.iter().cloned().collect::<TokenStream2>()
}

fn get_type(tokens: &[TokenTree]) -> Result<&[TokenTree], Box<dyn Error>> {
    let mut tokens = tokens
        .split(|token| is_char(token, ':'))
        .collect::<VecDeque<_>>();

    if tokens.len() != 2 {
        return Err("type must be specified for all function parameters".into());
    }

    let _ = tokens.pop_front().ok_or("unreachable (1)")?;
    let type_tokens = tokens.pop_front().ok_or("unreachable (2)")?;

    Ok(type_tokens)
}
