#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate proc_macro_error;

mod css_parser;
mod html_parser;
mod serde_request;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};

use crate::{
    css_parser::generate_css_string,
    html_parser::HtmlParser,
};

#[proc_macro]
#[proc_macro_error]
pub fn html(input: TokenStream) -> TokenStream {
    let call_site = Span::call_site();
    // emit_warning!(call_site, "HTML: input: {}", input.to_string());
    let result = HtmlParser::parse_stream(call_site, &input.to_string(), true);
    // emit_warning!(call_site, "HTML: output: {}", result);
    result.into()
}

#[proc_macro]
#[proc_macro_error]
pub fn css_block(input: TokenStream) -> TokenStream {
    let (css_str, _) = generate_css_string(input);
    let result = quote! { #css_str };
    result.into()
}

#[proc_macro]
#[proc_macro_error]
pub fn css(input: TokenStream) -> TokenStream {
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
