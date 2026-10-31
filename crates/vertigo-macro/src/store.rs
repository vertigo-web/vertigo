use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Error, FnArg, Ident, Index, ItemFn, Pat, Result, ReturnType, Type, TypePath, spanned::Spanned,
};

pub fn store_inner(_attr: TokenStream2, item: TokenStream2) -> Result<TokenStream2> {
    let span = item.span();
    let Ok(input) = syn::parse2::<ItemFn>(item) else {
        return Err(Error::new(span, "The macro can only take functions"));
    };

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
        return Ok(quote! {
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
    Ok(quote! {
        #vis #sig {
            thread_local! {
                static CACHE: vertigo::dev::StoreMap<( #( #types, )* ), #output_type>
                    = const { vertigo::dev::StoreMap::new() };
            }

            let __vertigo_store_key: ( #( &#types, )* ) = ( #( &#names, )* );

            CACHE.with(|__vertigo_store| {
                __vertigo_store.get_or_create(
                    &__vertigo_store_key,
                    |stored| ( #( &stored.#indexes, )* ) == __vertigo_store_key,
                    || ( #( std::clone::Clone::clone(__vertigo_store_key.#indexes), )* ),
                    || #block,
                )
            })
        }
    })
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
