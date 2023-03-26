#![feature(proc_macro_span)]

#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate proc_macro_error;

mod css_parser;
mod js_json_derive;
mod html_parser;
mod bind;
mod include_static;

mod wasm_path;

use html_parser::{dom_inner, dom_element_inner};
use proc_macro::{TokenStream, Span};
use syn::{Visibility, __private::ToTokens};

use crate::{
    css_parser::generate_css_string,
};
use bind::{bind_macro_fn, bind_spawn_fn, bind_rc_fn};

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

#[proc_macro_derive(AutoJsJson)]
#[proc_macro_error]
pub fn auto_js_json(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    match js_json_derive::impl_js_json_derive(&ast) {
        Ok(result) => {
            result
        },
        Err(message) => {
            emit_error!(Span::call_site(), "{}", message);
            let empty = "";
            quote! { #empty }.into()
        }
    }
}

fn convert_to_tokens(input: Result<TokenStream, String>) -> TokenStream {
    match input {
        Ok(body) => {
            body
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
pub fn include_static(input: TokenStream) -> TokenStream {
    let path = input.to_string();
    let file_path = Span::call_site().source_file().path();

    match include_static::include_static(file_path, path) {
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
    convert_to_tokens(bind_macro_fn(input))
}

#[proc_macro]
#[proc_macro_error]
pub fn bind_spawn(input: TokenStream) -> TokenStream {
    convert_to_tokens(bind_spawn_fn(input))
}

#[proc_macro]
#[proc_macro_error]
pub fn bind_rc(input: TokenStream) -> TokenStream {
    convert_to_tokens(bind_rc_fn(input))
}

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input2 = input.clone();

    let ast = syn::parse_macro_input!(input as syn::ItemFn);

    //function name
    let name = &ast.sig.ident;

    let input: proc_macro2::TokenStream = input2.into();

    quote! {
        #input

        #[no_mangle]
        pub fn vertigo_entry_function() {
            vertigo::start_app(#name);
        }
    }.into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn component(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::ItemFn);

    //function name
    let name = &ast.sig.ident;

    if ast.sig.output.to_token_stream().to_string() != "" {
        emit_error!(
            Span::call_site(),
            "{} => \"{}\"", "remove the information about the returned type. A component always returns DomNode",
            ast.sig.output.to_token_stream().to_string()
        );
        return quote! { }.into();
    }

    let mut struct_fields = Vec::new();

    for aaa in ast.sig.inputs.iter() {
        struct_fields.push(quote!{
            pub #aaa
        })
    }

    let body = ast.block;

    let mut param_names = Vec::new();
    for param in &ast.sig.inputs {
        if let syn::FnArg::Typed(pat_type) = param {
            if let syn::Pat::Ident(ident) = &*pat_type.pat {
                let param_name = ident.ident.clone();

                param_names.push(quote! {
                    let #param_name = self.#param_name;
                })
            }
        }
    }

    let visibility = &ast.vis;

    let visibility2 = match visibility {
        Visibility::Public(_) => {
            quote! {
                pub
            }
        }
        _ => {
            quote! {}
        }
    };

    let result = quote! {
        #visibility2 struct #name {
            #(#struct_fields,)*
        }

        impl #name {
            pub fn mount(self) -> vertigo::DomNode {
                #(#param_names)*

                (#body).into()
            }
        }
    };

    result.into()
}
