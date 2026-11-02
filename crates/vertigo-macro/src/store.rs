use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    Error, FnArg, Ident, Index, ItemFn, Pat, Result, ReturnType, Type, TypePath, ext::IdentExt,
    parse::Parser, spanned::Spanned,
};

pub fn store_inner(attr: TokenStream2, item: TokenStream2) -> Result<TokenStream2> {
    let all = parse_all(attr)?;

    let span = item.span();
    let Ok(input) = syn::parse2::<ItemFn>(item) else {
        return Err(Error::new(span, "The macro can only take functions"));
    };

    let attrs = &input.attrs;
    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;

    let inputs = &sig.inputs;
    let output = &sig.output;

    let ReturnType::Type(_, output_type) = output else {
        return Err(Error::new(
            output.span(),
            "The function should return something",
        ));
    };

    // Without arguments there is one value per thread, so a cell is enough. A map with `()`
    // as its only key would bring a copy of the hash map code into the wasm for each store.
    if inputs.is_empty() {
        if let Some(all) = all {
            return Err(Error::new(
                all.span(),
                "A store without arguments has only one value, the one it returns",
            ));
        }
        return Ok(quote! {
            #(#attrs)*
            #vis #sig {
                thread_local! {
                    static CACHE: std::cell::OnceCell<#output_type> = const { std::cell::OnceCell::new() };
                }

                CACHE.with(|cache| {
                    cache.get_or_init(|| #block).clone()
                })
            }
        });
    }

    let mut arguments = Vec::<(&Ident, &TypePath)>::new();

    for arg in inputs {
        let FnArg::Typed(arg) = arg else {
            return Err(Error::new(arg.span(), "Unsupported type"));
        };

        let Pat::Ident(pat) = &*arg.pat else {
            return Err(Error::new(arg.pat.span(), "Unsupported type"));
        };

        // Variable identifier
        let iden = &pat.ident;

        let arg_type: &TypePath = match &*arg.ty {
            Type::Reference(inner) => {
                let Type::Path(inner) = &*inner.elem else {
                    return Err(Error::new(arg.ty.span(), "Unsupported type"));
                };
                inner
            }
            Type::Path(inner) => inner,
            _ => {
                return Err(Error::new(arg.ty.span(), "Unsupported type"));
            }
        };

        arguments.push((iden, arg_type));
    }

    let names = arguments.iter().map(|(name, _)| name);
    let types = arguments
        .iter()
        .map(|(_, arg_type)| arg_type)
        .collect::<Vec<_>>();
    let indexes = (0..arguments.len()).map(Index::from).collect::<Vec<_>>();

    // All the arguments make one key, and the map keeps the types only in its signature: one
    // generic all the way down would bring a copy of the hash map code for each store, and
    // nested maps one more for each further argument.
    let map_type = quote! { vertigo::dev::StoreMap<( #( #types, )* ), #output_type> };
    let get_or_create = |map: &Ident| {
        quote! {
            let __vertigo_store_key: ( #( &#types, )* ) = ( #( &#names, )* );

            #map.with(|__vertigo_store| {
                __vertigo_store.get_or_create(
                    &__vertigo_store_key,
                    |stored| ( #( &stored.#indexes, )* ) == __vertigo_store_key,
                    || ( #( std::clone::Clone::clone(__vertigo_store_key.#indexes), )* ),
                    || #block,
                )
            })
        }
    };

    let Some(all) = all else {
        let body = get_or_create(&format_ident!("CACHE"));
        return Ok(quote! {
            #(#attrs)*
            #vis #sig {
                thread_local! {
                    static CACHE: #map_type = const { vertigo::dev::StoreMap::new() };
                }

                #body
            }
        });
    };

    // The function giving all the values needs the map as well, so it moves out of the store's
    // body, next to both. Whatever leaves the store out leaves out the other two too.
    let name = &sig.ident;
    let map = format_ident!(
        "__VERTIGO_STORE_{}",
        name.unraw().to_string().to_uppercase()
    );
    let body = get_or_create(&map);
    let cfgs = attrs
        .iter()
        .filter(|attr| attr.path().is_ident("cfg"))
        .collect::<Vec<_>>();
    let all_doc = format!(
        "All the values [`{}`] has made so far, in no particular order.",
        name.unraw()
    );

    Ok(quote! {
        #(#cfgs)*
        thread_local! {
            static #map: #map_type = const { vertigo::dev::StoreMap::new() };
        }

        #(#attrs)*
        #vis #sig {
            #body
        }

        #[doc = #all_doc]
        #(#cfgs)*
        #vis fn #all() -> std::vec::Vec<#output_type> {
            #map.with(vertigo::dev::StoreMap::values)
        }
    })
}

/// The name in `#[store(all = name)]`, if given.
fn parse_all(attr: TokenStream2) -> Result<Option<Ident>> {
    let mut all = None;
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("all") {
            all = Some(meta.value()?.parse::<Ident>()?);
            Ok(())
        } else {
            Err(meta.error("Unsupported store attribute, expected `all = name`"))
        }
    });
    parser.parse2(attr)?;
    Ok(all)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pretty_format(output: &TokenStream2) -> String {
        use syn::parse2;

        let Ok(syntax_tree) = parse2::<syn::File>(output.clone()) else {
            emit_error!(output.span(), "Failed to parse output");
            return "".to_string();
        };
        prettyplease::unparse(&syntax_tree)
    }

    #[test]
    fn function_without_arguments() -> Result<()> {
        let input: TokenStream2 = quote! {
            pub fn get_state() -> FakeState {
                FakeState {}
            }
        };

        let output = store_inner(quote!(), input.clone())?;

        let expected = quote! {
            pub fn get_state() -> FakeState {
                thread_local! {
                    static CACHE: std::cell::OnceCell<FakeState> = const { std::cell::OnceCell::new() };
                }

                CACHE.with(|cache| {
                    cache.get_or_init(|| {
                        FakeState {}
                    }).clone()
                })
            }
        };

        pretty_assertions::assert_eq!(pretty_format(&output), pretty_format(&expected));

        Ok(())
    }

    #[test]
    fn with_arguments() -> Result<()> {
        let input: TokenStream2 = quote! {
            pub fn get_comments(id4: u8, post_id: u32, url: &String) -> LazyCache<Vec<CommentModel>> {
                vertigo::fetch::RequestBuilder
                    ::get(format!("https://jsonplaceholder.typicode.com/posts/{post_id}/comments"))
                    .ttl_minutes(10)
                    .lazy_cache(|status, body| {
                        if status == 200 {
                            Some(body.into::<Vec<CommentModel>>())
                        } else {
                            None
                        }
                    })
            }
        };

        let output = store_inner(quote!(), input.clone())?;

        let expected = quote! {
            pub fn get_comments(
                id4: u8,
                post_id: u32,
                url: &String
            ) -> LazyCache<Vec<CommentModel>> {
                thread_local! {
                    static CACHE: vertigo::dev::StoreMap<(u8, u32, String,), LazyCache<Vec<CommentModel> > >
                        = const { vertigo::dev::StoreMap::new() };
                }

                let __vertigo_store_key: (&u8, &u32, &String,) = (&id4, &post_id, &url,);

                CACHE.with(|__vertigo_store| {
                    __vertigo_store.get_or_create(
                        &__vertigo_store_key,
                        |stored| (&stored.0, &stored.1, &stored.2,) == __vertigo_store_key,
                        || (
                            std::clone::Clone::clone(__vertigo_store_key.0),
                            std::clone::Clone::clone(__vertigo_store_key.1),
                            std::clone::Clone::clone(__vertigo_store_key.2),
                        ),
                        || {
                            vertigo::fetch::RequestBuilder
                                ::get(format!("https://jsonplaceholder.typicode.com/posts/{post_id}/comments"))
                                .ttl_minutes(10)
                                .lazy_cache(|status, body| {
                                    if status == 200 {
                                        Some(body.into::<Vec<CommentModel>>())
                                    } else {
                                        None
                                    }
                                })
                        },
                    )
                })
            }
        };

        pretty_assertions::assert_eq!(pretty_format(&output), pretty_format(&expected));

        Ok(())
    }

    #[test]
    fn keeps_attributes() -> Result<()> {
        let input: TokenStream2 = quote! {
            /// The state.
            #[cfg(feature = "state")]
            pub fn get_state() -> FakeState {
                FakeState {}
            }
        };

        let output = store_inner(quote!(), input.clone())?;

        let expected = quote! {
            /// The state.
            #[cfg(feature = "state")]
            pub fn get_state() -> FakeState {
                thread_local! {
                    static CACHE: std::cell::OnceCell<FakeState> = const { std::cell::OnceCell::new() };
                }

                CACHE.with(|cache| {
                    cache.get_or_init(|| {
                        FakeState {}
                    }).clone()
                })
            }
        };

        pretty_assertions::assert_eq!(pretty_format(&output), pretty_format(&expected));

        Ok(())
    }

    #[test]
    fn with_all() -> Result<()> {
        let input: TokenStream2 = quote! {
            /// The comments.
            #[cfg(feature = "comments")]
            pub fn get_comments(post_id: u32, r#type: &String) -> LazyCache<Vec<CommentModel>> {
                comments(post_id, r#type)
            }
        };

        let output = store_inner(quote!(all = all_comments), input.clone())?;

        let expected = quote! {
            #[cfg(feature = "comments")]
            thread_local! {
                static __VERTIGO_STORE_GET_COMMENTS: vertigo::dev::StoreMap<(u32, String,), LazyCache<Vec<CommentModel> > >
                    = const { vertigo::dev::StoreMap::new() };
            }

            /// The comments.
            #[cfg(feature = "comments")]
            pub fn get_comments(post_id: u32, r#type: &String) -> LazyCache<Vec<CommentModel>> {
                let __vertigo_store_key: (&u32, &String,) = (&post_id, &r#type,);

                __VERTIGO_STORE_GET_COMMENTS.with(|__vertigo_store| {
                    __vertigo_store.get_or_create(
                        &__vertigo_store_key,
                        |stored| (&stored.0, &stored.1,) == __vertigo_store_key,
                        || (
                            std::clone::Clone::clone(__vertigo_store_key.0),
                            std::clone::Clone::clone(__vertigo_store_key.1),
                        ),
                        || {
                            comments(post_id, r#type)
                        },
                    )
                })
            }

            #[doc = "All the values [`get_comments`] has made so far, in no particular order."]
            #[cfg(feature = "comments")]
            pub fn all_comments() -> std::vec::Vec<LazyCache<Vec<CommentModel> > > {
                __VERTIGO_STORE_GET_COMMENTS.with(vertigo::dev::StoreMap::values)
            }
        };

        pretty_assertions::assert_eq!(pretty_format(&output), pretty_format(&expected));

        Ok(())
    }

    #[test]
    fn all_needs_arguments() -> core::result::Result<(), String> {
        let input: TokenStream2 = quote! {
            pub fn get_state() -> FakeState {
                FakeState {}
            }
        };

        match store_inner(quote!(all = all_states), input) {
            Ok(_) => Err("Expected an error".into()),
            Err(err) => {
                assert_eq!(
                    err.to_string(),
                    "A store without arguments has only one value, the one it returns"
                );
                Ok(())
            }
        }
    }

    #[test]
    fn unknown_attribute() -> core::result::Result<(), String> {
        let input: TokenStream2 = quote! {
            pub fn get_post(id: u32) -> Post {
                Post::new(id)
            }
        };

        match store_inner(quote!(each = posts), input) {
            Ok(_) => Err("Expected an error".into()),
            Err(err) => {
                assert_eq!(
                    err.to_string(),
                    "Unsupported store attribute, expected `all = name`"
                );
                Ok(())
            }
        }
    }

    #[test]
    fn no_return_type() -> core::result::Result<(), String> {
        let input: TokenStream2 = quote! {
            pub fn my_store() {
            }
        };

        let output = store_inner(quote!(), input.clone());

        match output {
            Ok(_) => Err("Expected an error".into()),
            Err(err) => {
                assert_eq!(err.to_string(), "The function should return something");
                Ok(())
            }
        }
    }

    #[test]
    fn not_a_function() -> core::result::Result<(), String> {
        let input: TokenStream2 = quote! {
            struct MyStruct {
                field: u32,
            }
        };

        let output = store_inner(quote!(), input.clone());

        match output {
            Ok(_) => Err("Expected an error".into()),
            Err(err) => {
                assert_eq!(err.to_string(), "The macro can only take functions");
                Ok(())
            }
        }
    }
}
