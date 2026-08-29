use darling::FromAttributes;
use proc_macro::TokenStream;
use quote::quote;
use std::error::Error;
use syn::{DataStruct, Ident, ext::IdentExt};

use crate::jsjson::{
    attributes::{ContainerOpts, FieldOpts},
    is_vec_u8, js_json_object, object_ident,
};

pub(super) fn impl_js_json_struct(
    name: &Ident,
    data: &DataStruct,
    container_opts: ContainerOpts,
) -> Result<TokenStream, Box<dyn Error>> {
    let mut field_list = Vec::new();

    for field in &data.fields {
        let Some(field_name) = &field.ident else {
            return super::newtypes::impl_js_json_newtype(name, data);
        };

        let attrs = &field.attrs;

        field_list.push((field_name, attrs, &field.ty));
    }

    let object = object_ident();

    let mut list_to_json: Vec<(String, proc_macro2::TokenStream)> = Vec::new();
    let mut list_from_json = Vec::new();

    for (field_name, attrs, field_ty) in field_list {
        let field_unraw = field_name.unraw().to_string();
        let field_opts = FieldOpts::from_attributes(attrs)?;

        if field_opts.skip {
            list_from_json.push(quote! {
                #field_name: Default::default(),
            });
            continue;
        }

        let json_key = match field_opts.rename {
            Some(json_key) => json_key,
            None => match container_opts.rename_all {
                Some(rule) => rule.rename(&field_unraw),
                None => field_unraw.clone(),
            },
        };

        let unpack_expr = match field_opts.default {
            Some(darling::util::Override::Explicit(default_expr)) => quote! {
                .unwrap_or_else(|_| #default_expr)
            },
            Some(darling::util::Override::Inherit) => quote! {
                .unwrap_or_default()
            },
            None => quote! { ? },
        };

        if is_vec_u8(field_ty) {
            list_to_json.push((json_key.clone(), quote! {
                vertigo::object_insert(&mut #object, #json_key, vertigo::JsJson::Vec(self.#field_name));
            }));

            list_from_json.push(quote! {
                #field_name: json.get_property_jsjson(&context, #json_key).and_then(|item| {
                    match item {
                        vertigo::JsJson::Vec(v) => Ok(v),
                        other => {
                            let message = ["Vec<u8> expected, received ", other.typename()].concat();
                            Err(context.add(message))
                        }
                    }
                })#unpack_expr,
            })
        } else if field_opts.stringify {
            let is_option = match field_ty {
                syn::Type::Path(ty_path) => ty_path
                    .path
                    .segments
                    .last()
                    .is_some_and(|last| last.ident == "Option"),
                _ => false,
            };

            if is_option {
                list_to_json.push((
                    json_key.clone(),
                    quote! {
                        vertigo::object_insert(&mut #object, #json_key, match &self.#field_name {
                            Some(val) => vertigo::JsJson::String(format!("{}", val)),
                            None => vertigo::JsJson::Null,
                        });
                    },
                ));

                list_from_json.push(quote! {
                    #field_name: json.get_property(&context, #json_key).and_then(|item| {
                        match item {
                            vertigo::JsJson::String(v) => {
                                match v.parse() {
                                    Ok(v) => Ok(Some(v)),
                                    Err(e) => {
                                        let message = format!("Error parsing string '{}': {}", v, e);
                                        Err(context.add(message))
                                    }
                                }
                            },
                            vertigo::JsJson::Null => Ok(None),
                            other => {
                                let message = ["String or null expected, received ", other.typename()].concat();
                                Err(context.add(message))
                            }
                        }
                    })#unpack_expr,
                })
            } else {
                list_to_json.push((json_key.clone(), quote! {
                    vertigo::object_insert(&mut #object, #json_key, vertigo::JsJson::String(format!("{}", self.#field_name)));
                }));

                list_from_json.push(quote! {
                    #field_name: json.get_property(&context, #json_key).and_then(|item| {
                        match item {
                            vertigo::JsJson::String(v) => {
                                match v.parse() {
                                    Ok(v) => Ok(v),
                                    Err(e) => {
                                        let message = format!("Error parsing string '{}': {}", v, e);
                                        Err(context.add(message))
                                    }
                                }
                            },
                            other => {
                                let message = ["String expected, received ", other.typename()].concat();
                                Err(context.add(message))
                            }
                        }
                    })#unpack_expr,
                })
            }
        } else {
            list_to_json.push((
                json_key.clone(),
                quote! {
                    vertigo::object_insert(&mut #object, #json_key, self.#field_name.to_json());
                },
            ));

            list_from_json.push(quote! {
                #field_name: json.get_property(&context, #json_key)#unpack_expr,
            })
        }
    }

    // Ascending key order: a `BTreeMap` insert that appends past everything already there
    // does no shifting, and the derive knows the names at compile time, so the sort is free.
    list_to_json.sort_by(|(left, _), (right, _)| left.cmp(right));
    let list_to_json = list_to_json
        .into_iter()
        .map(|(_, tokens)| tokens)
        .collect::<Vec<_>>();

    let to_json_body = js_json_object(&list_to_json);

    let result = quote! {
        impl vertigo::JsJsonSerialize for #name {
            fn to_json(self) -> vertigo::JsJson {
                #to_json_body
            }
        }

        impl vertigo::JsJsonDeserialize for #name {
            fn from_json(context: vertigo::JsJsonContext, mut json: vertigo::JsJson) -> Result<Self, vertigo::JsJsonContext> {
                Ok(Self {
                    #(#list_from_json)*
                })
            }
        }
    };

    Ok(result.into())
}
