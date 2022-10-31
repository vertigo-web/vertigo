#![feature(proc_macro_span)]

#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate proc_macro_error;


mod logs;
mod css_parser;
mod serde_request;
mod html_parser;
mod build;
mod bind;

use html_parser::dom_inner;
use proc_macro::{TokenStream, Span};
use proc_macro2::{TokenStream as TokenStream2};

use crate::{
    css_parser::generate_css_string,
};
use bind::bind_macro_fn;

#[proc_macro]
#[proc_macro_error]
pub fn dom(input: TokenStream) -> TokenStream {
    crate::build::build_static();
    dom_inner(input)
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
    crate::build::build_static();
    let (css_str, _) = generate_css_string(input);
    let result = quote! { #css_str };
    result.into()
}

#[proc_macro]
#[proc_macro_error]
pub fn css(input: TokenStream) -> TokenStream {
    crate::build::build_static();
    let (css_str, is_dynamic) = generate_css_string(input);
    let result = if is_dynamic {
        quote! { vertigo::Css::string(#css_str) }
    } else {
        quote! { vertigo::Css::str(#css_str) }
    };
    result.into()
}

#[proc_macro_derive(SerdeSingleRequest)]
#[proc_macro_error]
pub fn serde_single_request_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    serde_request::impl_single_request_trait_macro(&ast)
}

#[proc_macro_derive(SerdeListRequest)]
#[proc_macro_error]
pub fn serde_list_request_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    serde_request::impl_list_request_trait_macro(&ast)
}

#[proc_macro_derive(SerdeRequest)]
#[proc_macro_error]
pub fn serde_request_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let single: TokenStream2 = serde_request::impl_single_request_trait_macro(&ast).into();
    let list: TokenStream2 = serde_request::impl_list_request_trait_macro(&ast).into();
    let result = quote! {
        #single
        #list
    };
    result.into()
}

#[proc_macro]
#[proc_macro_error]
pub fn include_static(input: TokenStream) -> TokenStream {
    let path = input.to_string();
    let file_path = Span::call_site().source_file().path();

    match build::include_static(file_path, path) {
        Ok(hash) => {
            quote! { #hash }.into()
        },
        Err(message) => {
            emit_error!(Span::call_site(), "{}", message);
            let empty = "";
            quote! { #empty }.into()
        }
    }
}


#[proc_macro]
#[proc_macro_error]
pub fn bind(input: TokenStream) -> TokenStream {
    bind_macro_fn(input)
}
