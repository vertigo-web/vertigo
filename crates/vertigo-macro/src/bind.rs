use std::collections::VecDeque;

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

fn find_param_name(params: &[TokenTree]) -> Option<Ident2> {
    if params.len() == 1 {
        if let Some(first) = params.first() {
            return if let TokenTree::Ident(value) = &first {
                Some(format_ident!("{}", value.to_string()))
            } else {
                emit_error!(Span::call_site(), "Can't find variable name, expected ident (1)");
                None
            };
        }
    }

    emit_error!(Span::call_site(), "Can't find variable name, expected ident (2)");
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

struct TokensParamsBody<'a> {
    bind_params: Vec<&'a [TokenTree]>,
    func_params: Option<Vec<&'a [TokenTree]>>,
    body: Vec<TokenTree>,
}

fn is_bracket_contain(tokens: &[TokenTree]) -> bool {
    for token in tokens {
        if let TokenTree::Punct(inner) = token {
            if inner.as_char() == '|' {
                return true;
            }
        }
    }

    false
}

fn split_params_and_body_function(tokens: &[TokenTree]) -> Result<TokensParamsBody, String> {
    let mut chunks = tokens
        .split(|token| {
            is_char(token, '|')
        })
        .collect::<VecDeque<_>>();

    if chunks.len() != 3 {
        return Err("Two brackets '|' were expected".to_string());
    }

    let bind_params = chunks
        .pop_front()
        .unwrap()
        .split(|token| {
            is_char(token, ',')
        })
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>();

    let func_params = chunks
        .pop_front()
        .unwrap()
        .split(|token| {
            is_char(token, ',')
        })
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

    Ok(TokensParamsBody {
        bind_params,
        func_params: Some(func_params),
        body,
    })
}

fn split_params_and_body_block(tokens: &[TokenTree]) -> Result<TokensParamsBody, String> {
    let mut chunks = tokens
        .split(|token| {
            is_char(token, ',')
        })
        .collect::<Vec<_>>();

    let body = chunks.pop().unwrap().to_vec();

    Ok(TokensParamsBody {
        bind_params: chunks,
        func_params: None,
        body,
    })
}

fn split_params_and_body(tokens: &[TokenTree]) -> Result<TokensParamsBody, String> {
    let bracket_contain = is_bracket_contain(tokens);

    if bracket_contain {
        split_params_and_body_function(tokens)
    } else {
        split_params_and_body_block(tokens)
    }
}

pub fn bind_macro_fn(input: TokenStream) -> Result<TokenStream, String> {
    let tokens = input.into_iter().collect::<Vec<_>>();

    let TokensParamsBody { bind_params, func_params: _, body } = split_params_and_body(tokens.as_slice())?;

    let mut clone_stm = Vec::<TokenStream2>::new();
    let first_pipe = is_first_pipe_char(body.as_slice());
    let body: TokenStream2 = body.iter().cloned().collect::<TokenStream>().into();

    for item in bind_params {
        let Some(name_param) = find_param_name(item) else {
            return Ok(quote! {""}.into());
        };

        clone_stm.push(quote!{
            let #name_param = #name_param.clone();
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

    Ok(bind_result.into())

}

fn convert_tokens_to_stream(tokens: &[TokenTree]) -> TokenStream2 {
    tokens
        .iter()
        .cloned()
        .collect::<TokenStream>()
        .into()
}

pub fn bind_spawn_fn(input: TokenStream) -> Result<TokenStream, String> {
    let tokens = input.into_iter().collect::<Vec<_>>();

    let TokensParamsBody { bind_params, func_params: _, body } = split_params_and_body(tokens.as_slice())?;

    let bind_params: Vec<TokenStream2> = bind_params
        .into_iter()
        .map(convert_tokens_to_stream)
        .collect::<Vec<_>>();

    let body: TokenStream2 = convert_tokens_to_stream(body.as_slice());

    Ok(quote! {
        {
            vertigo::bind!(#(#bind_params,)* || {
                vertigo::get_driver().spawn(vertigo::bind!(#(#bind_params,)* #body));
            })
        }
    }.into())
}

fn get_type(tokens: &[TokenTree]) -> Result<&[TokenTree], String> {
    let mut tokens = tokens
        .split(|token| {
            is_char(token, ':')
        })
        .collect::<VecDeque<_>>();

    if tokens.len() != 2 {
        return Err("type must be specified for all function parameters".to_string());
    }

    let _ = tokens.pop_front().unwrap();
    let type_tokens = tokens.pop_front().unwrap();

    Ok(type_tokens)
}

pub fn bind_rc_fn(input: TokenStream) -> Result<TokenStream, String> {
    let tokens = input.into_iter().collect::<Vec<_>>();

    let TokensParamsBody { bind_params, func_params, body } = split_params_and_body(tokens.as_slice())?;
    let bind_params: Vec<TokenStream2> = bind_params
        .into_iter()
        .map(convert_tokens_to_stream)
        .collect::<Vec<_>>();

    let Some(func_params) = func_params else {
        return Err("The macro can only take functions".to_string());
    };

    let types = {
        let mut types_macro: Vec<TokenStream2> = Vec::new();

        for type_item in func_params.into_iter() {
            let type_item = get_type(type_item)?;

            let type_item = convert_tokens_to_stream(type_item);
            types_macro.push(type_item);
        }

        types_macro
    };

    let body: TokenStream2 = convert_tokens_to_stream(body.as_slice());

    Ok(quote!{
        {
            let func: std::rc::Rc::<dyn Fn(#(#types,)*) -> _> = std::rc::Rc::new(vertigo::bind!(#(#bind_params,)* #body));
            func
        }
    }.into())
}
