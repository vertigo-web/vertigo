use proc_macro2::TokenStream;
use quote::quote;

pub fn build_profile() -> &'static str {
    env!("VERTIGO_PROFILE")
}

pub fn release_build() -> bool {
    build_profile() == "release"
}

/// Makes cargo re-expand the crate the tokens land in on every vertigo-cli build.
///
/// Cargo can't see that the bundling macros (`#[vertigo::main]`, `include_static!`, tailwind
/// classes in `tw!` and `dom!`) read VERTIGO_BUNDLE and write asset files, so it would
/// happily call a crate fresh and bundle nothing - and vertigo-cli wipes `static/included`
/// and `tailwind` before each build, so a library crate taken from the cache leaves its
/// assets missing. Referencing an env variable through
/// `option_env!` puts it in the crate's dep-info, which cargo does track - so vertigo-cli
/// can force a re-expansion just by passing a new value. Cheaper than dropping the
/// artifacts: the incremental cache survives.
pub fn track_build_id() -> TokenStream {
    quote! {
        const _: ::core::option::Option<&str> = ::core::option_env!("VERTIGO_BUILD_ID");
    }
}
