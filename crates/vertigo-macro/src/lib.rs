// Default component props on compilers that have default field values only on nightly,
// see `probe_default_field_values` in build.rs.
#![cfg_attr(vertigo_default_props_unstable, feature(allow_internal_unstable))]
#![cfg_attr(vertigo_default_props_unstable, allow(internal_features))]
// Warnings from the macros, see `probe_proc_macro_diagnostic` in build.rs.
#![cfg_attr(vertigo_macro_warnings, feature(proc_macro_diagnostic))]

#[macro_use]
extern crate pest_derive;
// First, so its macros are in scope in the modules below
#[macro_use]
mod diagnostics;

mod bind;
mod component;
mod css_parser;
mod get_target_dir;
mod html_parser;
mod include_static;
mod js_expression;
mod jsjson;
mod main_wrap;
mod store;
mod trace_tailwind;
mod utils;
mod wasm_path;

use proc_macro::TokenStream;
use quote::quote;

use crate::{
    bind::{bind_inner, bind_rc_inner, bind_spawn_inner},
    component::component_inner,
    css_parser::generate_css_string,
    html_parser::{dom_element_inner, dom_inner},
    include_static::include_static_inner,
    js_expression::js_expression,
    main_wrap::main_wrap,
    store::store_inner,
    trace_tailwind::{trace_tailwind, with_build_tracking},
};

#[proc_macro]
pub fn dom(input: TokenStream) -> TokenStream {
    diagnostics::expand(|| with_build_tracking(|| dom_inner(input)).into())
}

#[proc_macro]
pub fn dom_element(input: TokenStream) -> TokenStream {
    diagnostics::expand(|| with_build_tracking(|| dom_element_inner(input)).into())
}

#[proc_macro]
pub fn dom_debug(input: TokenStream) -> TokenStream {
    diagnostics::expand(|| {
        let stream = with_build_tracking(|| dom_inner(input));
        emit_warning!(proc_macro2::Span::call_site(), "debug: {}", stream);
        stream.into()
    })
}

#[proc_macro]
pub fn css_block(input: TokenStream) -> TokenStream {
    diagnostics::expand(|| {
        let (css_str, _, refs) = generate_css_string(input);
        let result = quote! {{
            #refs
            #css_str
        }};
        result.into()
    })
}

#[proc_macro]
pub fn css(input: TokenStream) -> TokenStream {
    diagnostics::expand(|| {
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
    })
}

#[proc_macro_derive(AutoJsJson, attributes(js_json))]
pub fn auto_js_json(input: TokenStream) -> TokenStream {
    diagnostics::expand(|| {
        let Ok(ast) = syn::parse(input) else {
            emit_call_site_error!("Failed to parse input");
            return quote! {}.into();
        };

        match jsjson::impl_js_json_derive(&ast) {
            Ok(result) => result,
            Err(message) => {
                emit_call_site_error!("{}", message);
                quote! {}.into()
            }
        }
    })
}

#[proc_macro]
pub fn tw(input: TokenStream) -> TokenStream {
    diagnostics::expand(|| {
        with_build_tracking(|| match trace_tailwind(input) {
            Ok(token_stream) => token_stream,
            Err(err) => {
                emit_call_site_error!(err);
                proc_macro2::TokenStream::new()
            }
        })
        .into()
    })
}

#[proc_macro]
pub fn include_static(input: TokenStream) -> TokenStream {
    diagnostics::expand(|| include_static_inner(input))
}

#[proc_macro]
pub fn bind(input: TokenStream) -> TokenStream {
    diagnostics::expand(|| convert_to_tokens(bind_inner(input.into())))
}

#[proc_macro]
pub fn bind_spawn(input: TokenStream) -> TokenStream {
    diagnostics::expand(|| convert_to_tokens(bind_spawn_inner(input.into())))
}

#[proc_macro]
pub fn bind_rc(input: TokenStream) -> TokenStream {
    diagnostics::expand(|| convert_to_tokens(bind_rc_inner(input.into())))
}

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, input: TokenStream) -> TokenStream {
    diagnostics::expand(|| main_wrap(input))
}

#[proc_macro_attribute]
#[cfg_attr(
    vertigo_default_props_unstable,
    allow_internal_unstable(default_field_values)
)]
pub fn component(attrs: TokenStream, input: TokenStream) -> TokenStream {
    diagnostics::expand(|| component_inner(attrs, input))
}

#[proc_macro]
pub fn js_inner(input: TokenStream) -> TokenStream {
    diagnostics::expand(|| js_expression(input))
}

fn convert_to_tokens(input: Option<proc_macro2::TokenStream>) -> TokenStream {
    match input {
        Some(body) => body.into(),
        None => quote! {}.into(),
    }
}

#[proc_macro_attribute]
pub fn store(attr: TokenStream, item: TokenStream) -> TokenStream {
    diagnostics::expand(|| {
        store_inner(attr.into(), item.into())
            .unwrap_or_else(|err| err.to_compile_error())
            .into()
    })
}
