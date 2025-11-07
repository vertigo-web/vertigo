use proc_macro::TokenStream;
use quote::quote;

use crate::trace_tailwind::bundle_tailwind;

pub(crate) fn main_wrap(input: TokenStream) -> TokenStream {
    let input2 = input.clone();

    let ast = syn::parse_macro_input!(input as syn::ItemFn);

    let function_name = &ast.sig.ident;

    let input: proc_macro2::TokenStream = input2.into();

    const VERTIGO_VERSION_MAJOR: u32 = pkg_version::pkg_version_major!();
    const VERTIGO_VERSION_MINOR: u32 = pkg_version::pkg_version_minor!();

    // Bundle final tailwind output (only during build)
    let tailwind_bundle_injector = bundle_tailwind();

    quote! {
        #input

        #[no_mangle]
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
