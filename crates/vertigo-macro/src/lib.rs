#![feature(proc_macro_span)]

#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate proc_macro_error;

mod bind;
mod component;
mod css_parser;
mod get_target_dir;
mod html_parser;
mod include_static;
mod js_expression;
mod jsjson;
mod main_wrap;
mod trace_tailwind;
mod utils;
mod wasm_path;

use proc_macro::{Span, TokenStream};
use quote::quote;

use crate::{
    bind::{bind_inner, bind_rc_inner, bind_spawn_inner},
    component::component_inner,
    css_parser::generate_css_string,
    html_parser::{dom_element_inner, dom_inner},
    include_static::include_static_inner,
    js_expression::js_expression,
    main_wrap::main_wrap,
    trace_tailwind::trace_tailwind,
};

#[proc_macro]
#[proc_macro_error]
pub fn dom(input: TokenStream) -> TokenStream {
    dom_inner(input)
}

#[proc_macro]
#[proc_macro_error]
pub fn dom_element(input: TokenStream) -> TokenStream {
    dom_element_inner(input)
}

#[proc_macro]
#[proc_macro_error]
pub fn dom_debug(input: TokenStream) -> TokenStream {
    let stream = dom_inner(input);
    emit_warning!("debug: {:?}", stream);
    stream
}

#[proc_macro]
#[proc_macro_error]
pub fn css_block(input: TokenStream) -> TokenStream {
    let (css_str, _, refs) = generate_css_string(input);
    let result = quote! {{
        #refs
        #css_str
    }};
    result.into()
}

#[proc_macro]
#[proc_macro_error]
pub fn css(input: TokenStream) -> TokenStream {
    let (css_str, is_dynamic, refs) = generate_css_string(input);

    let result = if is_dynamic {
        quote! {{
            #refs
            vertigo::Css::string(#css_str)
        }}
    } else {
        quote! {
            vertigo::Css::str(#css_str)
        }
    };
    result.into()
}

#[proc_macro_derive(AutoJsJson, attributes(js_json))]
#[proc_macro_error]
pub fn auto_js_json(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    match jsjson::impl_js_json_derive(&ast) {
        Ok(result) => result,
        Err(message) => {
            emit_error!(Span::call_site(), "{}", message);
            let empty = "";
            quote! { #empty }.into()
        }
    }
}

#[proc_macro]
#[proc_macro_error]
pub fn tw(input: TokenStream) -> TokenStream {
    trace_tailwind(input)
}

#[proc_macro]
#[proc_macro_error]
pub fn include_static(input: TokenStream) -> TokenStream {
    include_static_inner(input)
}

#[proc_macro]
#[proc_macro_error]
pub fn bind(input: TokenStream) -> TokenStream {
    convert_to_tokens(bind_inner(input))
}

#[proc_macro]
#[proc_macro_error]
pub fn bind_spawn(input: TokenStream) -> TokenStream {
    convert_to_tokens(bind_spawn_inner(input))
}

#[proc_macro]
#[proc_macro_error]
pub fn bind_rc(input: TokenStream) -> TokenStream {
    convert_to_tokens(bind_rc_inner(input))
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn main(_attr: TokenStream, input: TokenStream) -> TokenStream {
    main_wrap(input)
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn component(attrs: TokenStream, input: TokenStream) -> TokenStream {
    component_inner(attrs, input)
}

#[proc_macro]
#[proc_macro_error]
pub fn js_inner(input: TokenStream) -> TokenStream {
    js_expression(input)
}

fn convert_to_tokens(input: Result<TokenStream, String>) -> TokenStream {
    match input {
        Ok(body) => body,
        Err(message) => {
            emit_error!(Span::call_site(), "{}", message);
            let empty = "";
            quote! { #empty }.into()
        }
    }
}
