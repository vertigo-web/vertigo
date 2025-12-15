use proc_macro::TokenStream;
use quote::quote;
use std::error::Error;
use syn::{DataStruct, Ident};

pub(super) fn impl_js_json_newtype(
    name: &Ident,
    data: &DataStruct,
) -> Result<TokenStream, Box<dyn Error>> {
    let mut encodes = Vec::new();
    let mut decodes = Vec::new();

    // Struct(T) <-> T
    if data.fields.len() == 1 {
        // Encode
        encodes.push(quote! {
            self.0.to_json()
        });

        // Decode
        decodes.push(quote! {
            vertigo::JsJsonDeserialize::from_json(ctx, json).map(Self)
        });

    // Struct(T1, T2...) <-> [T1, T2, ...]
    } else {
        // Encode
        let (field_idents, field_encodes) = super::tuple_fields::get_encodes(data.fields.iter());

        encodes.push(quote! {
            let #name (#(#field_idents,)*) = self;
            vertigo::JsJson::List(vec![
                #(#field_encodes)*
            ])
        });

        // Decode
        let fields_number = field_idents.len();
        let field_decodes = super::tuple_fields::get_decodes(field_idents);
        let name_str = name.to_string();

        decodes.push(quote! {
            match json {
                vertigo::JsJson::List(fields) => {
                    if fields.len() != #fields_number {
                        return Err(ctx.add(
                            format!("Wrong number of fields in tuple for newtype {}. Expected {}, got {}", #name_str, #fields_number, fields.len())
                        ));
                    }
                    let mut fields_rev = fields.into_iter().rev().collect::<Vec<_>>();
                    return Ok(#name(
                        #(#field_decodes)*
                    ))
                }
                x => return Err(ctx.add(
                    format!("Invalid type {} while decoding newtype tuple, expected list", x.typename())
                )),
            }
        });
    }

    let result = quote! {
        impl vertigo::JsJsonSerialize for #name {
            fn to_json(self) -> vertigo::JsJson {
                #(#encodes)*
            }
        }

        impl vertigo::JsJsonDeserialize for #name {
            fn from_json(
                ctx: vertigo::JsJsonContext,
                json: vertigo::JsJson,
            ) -> Result<Self, vertigo::JsJsonContext> {
                #(#decodes)*
            }
        }
    };

    Ok(result.into())
}
