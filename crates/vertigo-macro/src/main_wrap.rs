use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, TokenTree};
use quote::quote;

use crate::trace_tailwind::bundle_tailwind;

pub(crate) fn main_wrap(input: TokenStream) -> TokenStream {
    let input2 = input.clone();

    // Find for tw! and tw= in the input which can't be included during bundle_tailwind()
    let warnings = find_warnings_in_stream(input2.clone().into(), false);

    // Emit warnings outside of find_warnings_in_stream function for easier testing
    for (span, msg) in &warnings {
        proc_macro_error::emit_warning!(span, msg);
    }

    let ast = syn::parse_macro_input!(input as syn::ItemFn);

    let function_name = &ast.sig.ident;

    if function_name == "vertigo_entry_function" {
        emit_error!(
            function_name.span(),
            "Your main function cannot be named 'vertigo_entry_function' - this is reserved for WASM-JS boundary"
        );
        return TokenStream::new();
    }

    let input: proc_macro2::TokenStream = input2.into();

    // Save current vertigo version to be later compared
    // against version injected from vertigo-cli during SSR
    const VERTIGO_VERSION_MAJOR: u32 = pkg_version::pkg_version_major!();
    const VERTIGO_VERSION_MINOR: u32 = pkg_version::pkg_version_minor!();

    let tailwind_bundle_injector = bundle_tailwind();

    quote! {
        #input

        #[unsafe(no_mangle)]
        pub fn vertigo_entry_function(version: (u32, u32)) {
            #tailwind_bundle_injector
            vertigo::start_app(#function_name);
            if version.0 != #VERTIGO_VERSION_MAJOR || version.1 != #VERTIGO_VERSION_MINOR {
                vertigo::log::error!(
                    "Vertigo version mismatch, server {}.{} != client {}.{}",
                    version.0, version.1,
                    #VERTIGO_VERSION_MAJOR, #VERTIGO_VERSION_MINOR
                );
            }
        }
    }
    .into()
}

/// Recursively find for tw! and tw= in the input, generate warning for each case
fn find_warnings_in_stream(
    tokens: TokenStream2,
    in_dom: bool,
) -> Vec<(proc_macro2::Span, &'static str)> {
    let mut iter = tokens.into_iter().peekable();
    let mut warnings = Vec::new();
    while let Some(tt) = iter.next() {
        match tt {
            TokenTree::Group(g) => {
                warnings.extend(find_warnings_in_stream(g.stream(), in_dom));
            }
            TokenTree::Ident(id) => {
                let id_str = id.to_string();
                if id_str == "tw" {
                    if matches!(iter.peek(), Some(TokenTree::Punct(p)) if p.as_char() == '!') {
                        warnings.push((id.span(), "tw! macro is not supported in #[main] body"));
                    } else if in_dom
                        && matches!(iter.peek(), Some(TokenTree::Punct(p)) if p.as_char() == '=')
                    {
                        warnings
                            .push((id.span(), "tw= attribute is not supported in #[main] body"));
                    }
                } else if (id_str == "dom" || id_str == "dom_element" || id_str == "dom_debug")
                    && matches!(iter.peek(), Some(TokenTree::Punct(p)) if p.as_char() == '!')
                {
                    iter.next(); // consume `!`
                    if let Some(TokenTree::Group(g)) = iter.peek() {
                        warnings.extend(find_warnings_in_stream(g.stream(), true));
                        iter.next(); // consume group
                    }
                }
            }
            _ => {}
        }
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_no_warnings() {
        let input = quote! {
            fn main() {
                let x = 1;
                let y = 2;
                let z = x + y;
                println!("{}", z);
            }
        };
        let warnings = find_warnings_in_stream(input, false);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_tw_macro_warning() {
        let input = quote! {
            fn main() {
                tw!("hello");
            }
        };
        // This should emit a warning
        let warnings = find_warnings_in_stream(input, false);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].1, "tw! macro is not supported in #[main] body");
    }

    #[test]
    fn test_tw_attribute_warning() {
        let input = quote! {
            fn main() {
                dom_element!("div", tw="hello");
            }
        };
        // This should emit a warning
        let warnings = find_warnings_in_stream(input, false);
        assert_eq!(warnings.len(), 1);
        assert_eq!(
            warnings[0].1,
            "tw= attribute is not supported in #[main] body"
        );
    }

    #[test]
    fn test_nested_dom_warning() {
        let input = quote! {
            fn main() {
                dom! {
                    div {
                        tw!("hello");
                    }
                }
            }
        };
        // This is invalid code, but even if it somehow compiles, it should emit a warning
        let warnings = find_warnings_in_stream(input, false);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].1, "tw! macro is not supported in #[main] body");
    }

    #[test]
    fn test_dom_debug_no_warning() {
        let input = quote! {
            fn main() {
                dom_debug!("hello");
            }
        };
        // This should NOT emit a warning because dom_debug! is not using tw! or tw=
        let warnings = find_warnings_in_stream(input, false);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_complex_nested_structure() {
        let input = quote! {
            fn main() {
                let x = 1;
                dom! {
                    div {
                        tw!("hello");
                        dom_element!("span", tw="world");
                        dom_debug!("nested");
                    }
                }
            }
        };
        // This is not a valid code, but even if it somehow compiles, it should emit warnings.
        let warnings = find_warnings_in_stream(input, false);
        assert_eq!(warnings.len(), 2);
        assert_eq!(warnings[0].1, "tw! macro is not supported in #[main] body");
        assert_eq!(
            warnings[1].1,
            "tw= attribute is not supported in #[main] body"
        );
    }
}
