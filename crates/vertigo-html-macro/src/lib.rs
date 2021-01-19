#[macro_use] extern crate pest_derive;
#[macro_use] extern crate proc_macro_error;

mod parser;

use proc_macro::TokenStream;
use proc_macro2::Span;

use crate::parser::HtmlParser;

#[proc_macro]
#[proc_macro_error]
pub fn html_component(input: TokenStream) -> TokenStream {
    let call_site = Span::call_site();
    let result = HtmlParser::parse_stream(call_site, &unformat(input), true);
    result.into()
}

#[proc_macro]
#[proc_macro_error]
pub fn html_element(input: TokenStream) -> TokenStream {
    let call_site = Span::call_site();
    let result = HtmlParser::parse_stream(call_site, &unformat(input), false);
    result.into()
}

fn unformat(input: TokenStream) -> String {
    // TokenStream breaks html tags (f. ex. "< \n div >"), so we need to remove all newlines.
    input.to_string().replace("\n", " ")
}


#[proc_macro]
#[proc_macro_error]
pub fn debug_html_component(input: TokenStream) -> TokenStream {
    let call_site = Span::call_site();
    let result = HtmlParser::parse_stream(call_site, &unformat(input), true);
    emit_warning!(call_site, "HTML: output: {}", result);
    result.into()
}
