use darling::FromAttributes;
use proc_macro::TokenStream;
use quote::quote;
use syn::{ext::IdentExt, DataEnum, Fields, Ident};

use crate::jsjson::attributes::{ContainerOpts, FieldOpts};

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
pub(super) fn impl_js_json_enum(
    name: &Ident,
    data: &DataEnum,
    container_opts: ContainerOpts,
) -> Result<TokenStream, String> {
    // Encoding code for every variant
    let mut variant_encodes = vec![];

    // Encoding code for every simple variant (data-less)
    let mut variant_string_decodes = vec![];

    // Envoding code for every compound variant (with data)
    let mut variant_object_decodes = vec![];

    for variant in &data.variants {
        let field_opts = FieldOpts::from_attributes(&variant.attrs).unwrap();
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
