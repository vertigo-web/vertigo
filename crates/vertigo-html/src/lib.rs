#[macro_use] extern crate pest_derive;
#[macro_use] extern crate proc_macro_error;

mod parser;

use proc_macro::TokenStream;
use proc_macro2::Span;

use crate::parser::HtmlParser;

#[proc_macro]
#[proc_macro_error]
pub fn html(input: TokenStream) -> TokenStream {
    let call_site = Span::call_site();

    let string_stream = input.to_string().replace("\n", " ");

    let result = HtmlParser::parse_stream(call_site, &string_stream);

    result.into()
}
