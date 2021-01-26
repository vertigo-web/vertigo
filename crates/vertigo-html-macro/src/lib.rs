#[macro_use] extern crate pest_derive;
#[macro_use] extern crate proc_macro_error;

mod html_parser;
mod css_parser;

use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro::TokenStream;

use crate::html_parser::HtmlParser;
use crate::css_parser::CssParser;

#[proc_macro]
#[proc_macro_error]
pub fn html_component(input: TokenStream) -> TokenStream {
    let call_site = Span::call_site();
    let result = HtmlParser::parse_stream(call_site, &unformat(input), true);
    // emit_warning!(call_site, "HTML: output: {}", result);
    result.into()
}

#[proc_macro]
#[proc_macro_error]
pub fn html_element(input: TokenStream) -> TokenStream {
    let call_site = Span::call_site();
    let result = HtmlParser::parse_stream(call_site, &unformat(input), false);
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
        quote! { vertigo::Css::new(#css_str) }
    } else {
        quote! { vertigo::Css::one(#css_str) }
    };
    result.into()
}

fn generate_css_string(input: TokenStream) -> (TokenStream2, bool) {
    let call_site = Span::call_site();
    let result = CssParser::parse_stream(call_site, &unformat(input));
    // emit_warning!(call_site, "CSS: output: {}", result.0); // FIXME: deleteme
    result
}

fn unformat(input: TokenStream) -> String {
    // TokenStream breaks html tags (f. ex. "< \n div >"), so we need to remove all newlines.
    input.to_string().replace("\n", " ")
}
