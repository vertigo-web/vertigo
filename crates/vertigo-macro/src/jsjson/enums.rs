use darling::FromAttributes;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::error::Error;
use syn::{DataEnum, Fields, Ident, ext::IdentExt};

use crate::jsjson::attributes::{ContainerOpts, FieldOpts};

// Externally tagged (default):
// {
//   "Somestring": "foobar"
// }
//
// {
//   "Point": { "x": 10, "y": "one" }
// }
//
// {
//   "Tuple": ["two", 20]
// }
//
// "Nothing"
//
// Adjacently tagged (opt-in via `#[js_json(tag = "t", content = "c")]`):
// { "t": "Somestring", "c": "foobar" }
// { "t": "Point", "c": { "x": 10, "y": "one" } }
// { "t": "Tuple", "c": ["two", 20] }
// { "t": "Nothing" }
pub(super) fn impl_js_json_enum(
    name: &Ident,
    data: &DataEnum,
    container_opts: ContainerOpts,
) -> Result<TokenStream, Box<dyn Error>> {
    match (&container_opts.tag, &container_opts.content) {
        (Some(tag), Some(content)) => {
            let tag = tag.clone();
            let content = content.clone();
            impl_js_json_enum_adjacently_tagged(name, data, container_opts, tag, content)
        }
        (None, None) => impl_js_json_enum_externally_tagged(name, data, container_opts),
        _ => {
            Err("`tag` and `content` must be specified together for adjacently-tagged enums".into())
        }
    }
}

fn impl_js_json_enum_externally_tagged(
    name: &Ident,
    data: &DataEnum,
    container_opts: ContainerOpts,
) -> Result<TokenStream, Box<dyn Error>> {
    // Encoding code for every variant
    let mut variant_encodes = vec![];

    // Encoding code for every simple variant (data-less)
    let mut variant_string_decodes = vec![];

    // Envoding code for every compound variant (with data)
    let mut variant_object_decodes = vec![];

    for variant in &data.variants {
        let field_opts = FieldOpts::from_attributes(&variant.attrs)?;
        let variant_ident = &variant.ident;
        let variant_name = variant.ident.unraw().to_string();

        let json_key = match field_opts.rename {
            Some(json_key) => json_key,
            None => match container_opts.rename_all {
                Some(rule) => rule.rename(&variant_name),
                None => variant_name.clone(),
            },
        };

        match &variant.fields {
            // Simple variant
            // Enum::Variant <-> "Variant"
            Fields::Unit => {
                variant_encodes.push(quote! { Self::#variant_ident => #json_key.to_json(), });
                variant_string_decodes.push(quote! { #json_key => Ok(Self::#variant_ident), });
            }

            // Compound variant with unnamed field(s) (tuple)
            // Enum::Variant(...) <-> "Variant": ...
            Fields::Unnamed(fields) => {
                // Enum::Variant(T) <-> "Variant": T
                if fields.unnamed.len() == 1 {
                    // Encode
                    variant_encodes.push(quote! {
                        Self::#variant_ident(value) => {
                            vertigo::JsJson::Object(::std::collections::BTreeMap::from([
                                (
                                    #json_key.to_string(),
                                    value.to_json(),
                                ),
                            ]))
                        }
                    });

                    // Decode
                    variant_object_decodes.push(quote! {
                        if let Some(value) = compound_variant.get_mut(#json_key) {
                            return Ok(Self::#variant_ident(
                                vertigo::JsJsonDeserialize::from_json(ctx.clone(), value.to_owned())?
                            ))
                        }
                    });

                // Enum::Variant(T1, T2...) <-> "Variant": [T1, T2, ...]
                } else {
                    // Encode
                    let (field_idents, field_encodes) =
                        super::tuple_fields::get_encodes(fields.unnamed.iter());

                    variant_encodes.push(quote! {
                        Self::#variant_ident(#(#field_idents,)*) => {
                            vertigo::JsJson::Object(::std::collections::BTreeMap::from([
                                (
                                    #json_key.to_string(),
                                    vertigo::JsJson::List(vec![
                                        #(#field_encodes)*
                                    ])
                                ),
                            ]))
                        }
                    });

                    // Decode
                    let fields_number = field_idents.len();
                    let field_decodes = super::tuple_fields::get_decodes(field_idents);

                    variant_object_decodes.push(quote! {
                        if let Some(value) = compound_variant.get_mut(#json_key) {
                            match value.to_owned() {
                                vertigo::JsJson::List(fields) => {
                                    if fields.len() != #fields_number {
                                        return Err(ctx.add(
                                            format!("Wrong unmber of fields in tuple for variant {}. Expected {}, got {}", #variant_name, #fields_number, fields.len())
                                        ));
                                    }
                                    let mut fields_rev = fields.into_iter().rev().collect::<Vec<_>>();
                                    return Ok(Self::#variant_ident (
                                        #(#field_decodes)*
                                    ))
                                },
                                x => return Err(ctx.add(
                                    format!("Invalid type {} while decoding enum tuple, expected list", x.typename())
                                )),
                            }
                        }
                    });
                }
            }

            // Compound variant with named field(s) (anonymous struct)
            // Enum::Variant { x: X, y: Y, ...) <-> "Variant": { x: X, y: Y, ... }
            Fields::Named(fields) => {
                // Encode
                let field_idents = fields
                    .named
                    .iter()
                    .filter_map(|field| field.ident.clone())
                    .collect::<Vec<_>>();

                let field_encodes = field_idents
                    .iter()
                    .map(|field_ident| {
                        let field_name = field_ident.unraw().to_string();
                        quote! {
                            (#field_name.to_string(), #field_ident.to_json()),
                        }
                    })
                    .collect::<Vec<_>>();

                variant_encodes.push(quote! {
                    Self::#variant_ident {#(#field_idents,)*} => {
                        vertigo::JsJson::Object(::std::collections::BTreeMap::from([
                            (
                                #json_key.to_string(),
                                vertigo::JsJson::Object(::std::collections::BTreeMap::from([
                                    #(#field_encodes)*
                                ]))
                            ),
                        ]))
                    }
                });

                // Decode
                let field_decodes = field_idents
                    .iter()
                    .map(|field_ident| {
                        let field_name = field_ident.unraw().to_string();
                        quote! {
                            #field_ident: value.get_property(&ctx, #field_name)?,
                        }
                    })
                    .collect::<Vec<_>>();

                variant_object_decodes.push(quote! {
                    if let Some(value) = compound_variant.get_mut(#json_key) {
                        return Ok(Self::#variant_ident {
                            #(#field_decodes)*
                        })
                    }
                });
            }
        }
    }

    let result = quote! {
        impl vertigo::JsJsonSerialize for #name {
            fn to_json(self) -> vertigo::JsJson {
                match self {
                    #(#variant_encodes)*
                }
            }
        }

        impl vertigo::JsJsonDeserialize for #name {
            fn from_json(
                ctx: vertigo::JsJsonContext,
                json: vertigo::JsJson,
            ) -> Result<Self, vertigo::JsJsonContext> {
                match json {
                    vertigo::JsJson::String(simple_variant) => {
                        match simple_variant.as_str() {
                            #(#variant_string_decodes)*
                            x => Err(ctx.add(format!("Invalid simple variant {x}"))),
                        }
                    }
                    vertigo::JsJson::Object(mut compound_variant) => {
                        #(#variant_object_decodes)*
                        Err(ctx.add("Value not matched with any variant".to_string()))
                    }
                    x => Err(ctx.add(
                        format!("Invalid type {} while decoding enum, expected string or object", x.typename())
                    )),
                }
            }
        }
    };

    Ok(result.into())
}

fn impl_js_json_enum_adjacently_tagged(
    name: &Ident,
    data: &DataEnum,
    container_opts: ContainerOpts,
    tag_key: String,
    content_key: String,
) -> Result<TokenStream, Box<dyn Error>> {
    let mut variant_encodes: Vec<TokenStream2> = vec![];
    let mut variant_tag_arms: Vec<TokenStream2> = vec![];

    for variant in &data.variants {
        let field_opts = FieldOpts::from_attributes(&variant.attrs)?;
        let variant_ident = &variant.ident;
        let variant_name = variant.ident.unraw().to_string();

        let json_key = match field_opts.rename {
            Some(json_key) => json_key,
            None => match container_opts.rename_all {
                Some(rule) => rule.rename(&variant_name),
                None => variant_name.clone(),
            },
        };

        match &variant.fields {
            // Unit -> { tag: "Variant" }
            Fields::Unit => {
                variant_encodes.push(quote! {
                    Self::#variant_ident => {
                        vertigo::JsJson::Object(::std::collections::BTreeMap::from([
                            (#tag_key.to_string(), #json_key.to_json()),
                        ]))
                    }
                });

                variant_tag_arms.push(quote! {
                    #json_key => Ok(Self::#variant_ident),
                });
            }

            Fields::Unnamed(fields) => {
                // Single-field tuple -> { tag: "Variant", content: <value> }
                if fields.unnamed.len() == 1 {
                    variant_encodes.push(quote! {
                        Self::#variant_ident(value) => {
                            vertigo::JsJson::Object(::std::collections::BTreeMap::from([
                                (#tag_key.to_string(), #json_key.to_json()),
                                (#content_key.to_string(), value.to_json()),
                            ]))
                        }
                    });

                    variant_tag_arms.push(quote! {
                        #json_key => {
                            let content_value = json.get_property_jsjson(&ctx, #content_key)?;
                            Ok(Self::#variant_ident(
                                vertigo::JsJsonDeserialize::from_json(ctx.clone(), content_value)?
                            ))
                        }
                    });

                // Multi-field tuple -> { tag: "Variant", content: [v1, v2, ...] }
                } else {
                    let (field_idents, field_encodes) =
                        super::tuple_fields::get_encodes(fields.unnamed.iter());

                    variant_encodes.push(quote! {
                        Self::#variant_ident(#(#field_idents,)*) => {
                            vertigo::JsJson::Object(::std::collections::BTreeMap::from([
                                (#tag_key.to_string(), #json_key.to_json()),
                                (
                                    #content_key.to_string(),
                                    vertigo::JsJson::List(vec![
                                        #(#field_encodes)*
                                    ])
                                ),
                            ]))
                        }
                    });

                    let fields_number = field_idents.len();
                    let field_decodes = super::tuple_fields::get_decodes(field_idents);

                    variant_tag_arms.push(quote! {
                        #json_key => {
                            let content_value = json.get_property_jsjson(&ctx, #content_key)?;
                            match content_value {
                                vertigo::JsJson::List(fields) => {
                                    if fields.len() != #fields_number {
                                        return Err(ctx.add(
                                            format!("Wrong unmber of fields in tuple for variant {}. Expected {}, got {}", #variant_name, #fields_number, fields.len())
                                        ));
                                    }
                                    let mut fields_rev = fields.into_iter().rev().collect::<Vec<_>>();
                                    Ok(Self::#variant_ident (
                                        #(#field_decodes)*
                                    ))
                                },
                                x => Err(ctx.add(
                                    format!("Invalid type {} while decoding enum tuple, expected list", x.typename())
                                )),
                            }
                        }
                    });
                }
            }

            // Struct -> { tag: "Variant", content: { x, y, ... } }
            Fields::Named(fields) => {
                let field_idents = fields
                    .named
                    .iter()
                    .filter_map(|field| field.ident.clone())
                    .collect::<Vec<_>>();

                let field_encodes = field_idents
                    .iter()
                    .map(|field_ident| {
                        let field_name = field_ident.unraw().to_string();
                        quote! {
                            (#field_name.to_string(), #field_ident.to_json()),
                        }
                    })
                    .collect::<Vec<_>>();

                variant_encodes.push(quote! {
                    Self::#variant_ident {#(#field_idents,)*} => {
                        vertigo::JsJson::Object(::std::collections::BTreeMap::from([
                            (#tag_key.to_string(), #json_key.to_json()),
                            (
                                #content_key.to_string(),
                                vertigo::JsJson::Object(::std::collections::BTreeMap::from([
                                    #(#field_encodes)*
                                ]))
                            ),
                        ]))
                    }
                });

                let field_decodes = field_idents
                    .iter()
                    .map(|field_ident| {
                        let field_name = field_ident.unraw().to_string();
                        quote! {
                            #field_ident: value.get_property(&ctx, #field_name)?,
                        }
                    })
                    .collect::<Vec<_>>();

                variant_tag_arms.push(quote! {
                    #json_key => {
                        let mut value = json.get_property_jsjson(&ctx, #content_key)?;
                        Ok(Self::#variant_ident {
                            #(#field_decodes)*
                        })
                    }
                });
            }
        }
    }

    let result = quote! {
        impl vertigo::JsJsonSerialize for #name {
            fn to_json(self) -> vertigo::JsJson {
                match self {
                    #(#variant_encodes)*
                }
            }
        }

        impl vertigo::JsJsonDeserialize for #name {
            fn from_json(
                ctx: vertigo::JsJsonContext,
                mut json: vertigo::JsJson,
            ) -> Result<Self, vertigo::JsJsonContext> {
                if !matches!(json, vertigo::JsJson::Object(_)) {
                    return Err(ctx.add(
                        format!("Invalid type {} while decoding enum, expected object", json.typename())
                    ));
                }
                let tag: String = json.get_property(&ctx, #tag_key)?;
                match tag.as_str() {
                    #(#variant_tag_arms)*
                    other => Err(ctx.add(format!("Unknown variant tag {other}"))),
                }
            }
        }
    };

    Ok(result.into())
}
