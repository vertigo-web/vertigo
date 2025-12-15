use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{spanned::Spanned, FnArg, Ident, ItemFn, Pat, ReturnType, Type, TypePath};

pub fn store_inner(_attr: TokenStream2, item: TokenStream2) -> TokenStream2 {
    let span = item.span();
    let Ok(input) = syn::parse2::<ItemFn>(item) else {
        emit_error!(span, "The macro can only take functions");
        return quote! {};
    };

    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;

    let inputs = &sig.inputs;
    let output = &sig.output;

    let ReturnType::Type(_, output_type) = output else {
        emit_error!(output.span(), "The function should return something",);
        return quote! {};
    };

    if inputs.is_empty() {
        return quote! {
            #vis #sig {
                thread_local! {
                    static CACHE: std::rc::Rc<vertigo::dev::HashMapMut<(), #output_type>>
                        = std::rc::Rc::new(vertigo::dev::HashMapMut::new());
                }

                CACHE.with(|cache| {
                    cache.get_or_create(&(), || #block)
                })
            }
        };
    }

    let mut arguments = Vec::<(&Ident, &TypePath)>::new();

    for arg in inputs {
        let FnArg::Typed(arg) = arg else {
            emit_error!(arg.span(), "Unsupported type");
            return quote! {};
        };

        let Pat::Ident(pat) = &*arg.pat else {
            emit_error!(arg.pat.span(), "Unsupported type");
            return quote! {};
        };

        // Variable identifier
        let iden = &pat.ident;

        let arg_type: &TypePath = match &*arg.ty {
            Type::Reference(inner) => {
                let Type::Path(inner) = &*inner.elem else {
                    emit_error!(arg.ty.span(), "Unsupported type");
                    return quote! {};
                };
                inner
            }
            Type::Path(inner) => inner,
            _ => {
                emit_error!(arg.ty.span(), "Unsupported type");
                return quote! {};
            }
        };

        arguments.push((iden, arg_type));
    }

    let mut types = Vec::new();

    types.push(quote! {
        type Cache0Type = #output_type;
    });

    for (index, (_, arg_type)) in arguments.iter().rev().enumerate() {
        let type_name_current = format_ident!("Cache{}Type", index + 1);
        let type_name_prev = format_ident!("Cache{}Type", index);

        types.push(quote! {
            type #type_name_current = std::rc::Rc<vertigo::dev::HashMapMut<#arg_type, #type_name_prev>>;
        });
    }

    let type_name_last = format_ident!("Cache{}Type", arguments.len());

    types.push(quote! {
        type CacheType = #type_name_last;
    });

    let mut call_list = Vec::new();
    let arguments_len = arguments.len();

    for (index, (arg_name, _)) in arguments.iter().enumerate() {
        let is_last = index == arguments_len - 1;

        if is_last {
            call_list.push(quote! {
                .get_or_create(&#arg_name, || #block)
            })
        } else {
            call_list.push(quote! {
                .get_or_default(&#arg_name)
            });
        }
    }

    quote! {
        #vis #sig {
            #( #types )*

            thread_local! {
                static CACHE: CacheType = std::rc::Rc::new(vertigo::dev::HashMapMut::new());
            }

            CACHE.with(|cache| {
                cache
                    #( #call_list )*
            })
        }
    }
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
    fn function_without_arguments() {
        let input: TokenStream2 = quote! {
            pub fn get_state() -> FakeState {
                FakeState {}
            }
        };

        let output = store_inner(quote!(), input.clone());

        let expected = quote! {
            pub fn get_state() -> FakeState {
                thread_local! {
                    static CACHE: std::rc::Rc<vertigo::dev::HashMapMut<(), FakeState>>
                        = std::rc::Rc::new(vertigo::dev::HashMapMut::new());
                }

                CACHE.with(|cache| {
                    cache.get_or_create(&(), || {
                        FakeState {}
                    })
                })
            }
        };

        pretty_assertions::assert_eq!(pretty_format(&output), pretty_format(&expected));
    }

    #[test]
    fn with_arguments() {
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

        let output = store_inner(quote!(), input.clone());

        let expected = quote! {
            pub fn get_comments(
                id4: u8,
                post_id: u32,
                url: &String
            ) -> LazyCache<Vec<CommentModel>> {
                type Cache0Type = LazyCache<Vec<CommentModel>>;
                type Cache1Type = std::rc::Rc<vertigo::dev::HashMapMut<String, Cache0Type>>;
                type Cache2Type = std::rc::Rc<vertigo::dev::HashMapMut<u32, Cache1Type>>;
                type Cache3Type = std::rc::Rc<vertigo::dev::HashMapMut<u8, Cache2Type>>;
                type CacheType = Cache3Type;

                thread_local! {
                    static CACHE: CacheType = std::rc::Rc::new(vertigo::dev::HashMapMut::new());
                }

                CACHE.with(|cache| {
                    cache
                        .get_or_default(&id4)
                        .get_or_default(&post_id)
                        .get_or_create(&url, || {
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
                        })
                })
            }
        };

        pretty_assertions::assert_eq!(pretty_format(&output), pretty_format(&expected));
    }
}
