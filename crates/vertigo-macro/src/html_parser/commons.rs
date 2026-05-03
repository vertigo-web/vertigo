use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, TokenStreamExt, quote};
use rstml::node::{KVAttributeValue, KeyedAttributeValue};
use syn::{Block, Expr, ExprBlock, ExprLit, Stmt, spanned::Spanned};

use crate::utils::release_build;

/// Out of attribute value takes ExprBlock or ExprLit and ignores everything else
pub(super) fn take_block_or_literal_expr<'a>(
    expr: &'a KeyedAttributeValue,
    msg: &str,
) -> (Option<&'a ExprBlock>, Option<&'a ExprLit>) {
    match expr {
        KeyedAttributeValue::Binding(_fn_binding) => {
            emit_error!(expr.span(), "Invalid attribute binding {}", msg);
            (None, None)
        }
        KeyedAttributeValue::Value(attribute_value_expr) => match &attribute_value_expr.value {
            KVAttributeValue::Expr(expr) => {
                match expr {
                    Expr::Block(block) => (Some(block), None),
                    Expr::Lit(lit) => (None, Some(lit)),
                    // TODO: Possibly others needs to be supported,
                    // so this function should in fact return Option<Expr> in future.
                    _ => {
                        emit_error!(expr.span(), "Invalid attribute {}", msg);
                        (None, None)
                    }
                }
            }
            KVAttributeValue::InvalidBraced(invalid) => {
                emit_error!(invalid.span(), "Invalid attribute braces {}", msg);
                (None, None)
            }
        },
        KeyedAttributeValue::None => {
            emit_error!(expr.span(), "Missing attribute value {}", msg);
            (None, None)
        }
    }
}

/// Tags starting with uppercase letter are considered components
pub(super) fn is_component_name(name: &str) -> bool {
    name.split("::")
        .last()
        .unwrap_or_default()
        .chars()
        .next()
        .map(char::is_uppercase)
        .unwrap_or_default()
}

pub(super) fn generate_debug_class_name(value: &TokenStream2) -> TokenStream2 {
    let debug_class_name = value
        .to_string()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();

    let debug_info = quote! {
        Some(#debug_class_name.to_string())
    };

    if release_build() {
        quote! {{
            #[cfg(test)] { #debug_info }
            #[cfg(not(test))] { None }
        }}
    } else {
        debug_info
    }
}

pub(super) fn is_default_block(block: &Block) -> bool {
    if block.stmts.is_empty() {
        return true;
    }
    let stmt_str = block.stmts[0].to_token_stream().to_string();
    stmt_str == "Default :: default()" || stmt_str == "Default :: default ()"
}

pub(super) fn extract_spread_block(
    block: &Block,
    map_inmost: impl FnOnce(&Expr) -> TokenStream2,
) -> Option<TokenStream2> {
    if let Some(Stmt::Expr(Expr::Range(range), _)) = block.stmts.last()
        && range.start.is_none()
        && let Some(inmost_value) = &range.end
    {
        let mut block = block.clone();
        block.stmts.pop();

        let mut new_block = quote! {};
        for stmt in block.stmts {
            new_block.append_all(stmt.to_token_stream());
        }
        let mapped = map_inmost(inmost_value);
        new_block.append_all(mapped);

        Some(new_block)
    } else {
        None
    }
}

pub(super) fn dereference_maybe(block: &Block) -> Option<TokenStream2> {
    if let Some(Stmt::Expr(Expr::Reference(inner), _)) = block.stmts.last() {
        Some(inner.expr.to_token_stream())
    } else {
        None
    }
}

pub(super) fn dereferenced_assignment(
    key: TokenStream2,
    value: &ExprBlock,
) -> Option<TokenStream2> {
    if let Some(value) = dereference_maybe(&value.block) {
        Some(quote! { #key: #value.clone(), })
    } else {
        Some(quote! { #key: #value.into(), })
    }
}

/// If block has only one statement, return it, otherwise return the block intact
pub(super) fn unwrap_block_if_single(block: &Block) -> TokenStream2 {
    if block.stmts.len() == 1 {
        block.stmts[0].to_token_stream()
    } else {
        block.to_token_stream()
    }
}

/// When no explicit attr key provided, create kay-value-method triple out of the block of statements
pub(super) fn parse_block_of_statements(
    block: &Block,
) -> (TokenStream2, TokenStream2, TokenStream2) {
    let dereferenced = dereference_maybe(block);

    // Last (or the only) statement is the key
    let key = dereferenced
        .clone()
        .unwrap_or_else(|| block.stmts.last().to_token_stream());

    let method = if dereferenced.is_some() {
        quote! { clone() }
    } else {
        quote! { into() }
    };

    // For multi-statement blocks ending with a reference, keep the whole block so that
    // preceding statements (e.g. `let mut x = ...; x += 1; &x`) are not lost.
    // For single-statement reference blocks (e.g. `{&my_var}`), use the inner expression
    // directly so `.clone()` applies to the value, not to a double-reference.
    // For non-reference blocks, unwrap single-statement blocks or use the whole block.
    let value = match &dereferenced {
        Some(inner) if block.stmts.len() > 1 => block.to_token_stream(),
        Some(inner) => inner.clone(),
        None => unwrap_block_if_single(block),
    };

    (key, value, method)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_component_name() {
        assert!(is_component_name("Component"));
        assert!(!is_component_name("div"));
        assert!(is_component_name("my::path::Component"));
        assert!(!is_component_name("my::path::div"));
        assert!(is_component_name("::Component"));
        assert!(!is_component_name(""));
    }

    #[test]
    fn test_unwrap_block_if_single() {
        let block: Block = syn::parse_quote! {
            {
                let a = 1;
            }
        };
        let stream = unwrap_block_if_single(&block);
        assert_eq!(stream.to_string(), "let a = 1 ;");

        let block2: Block = syn::parse_quote! {
            {
                let a = 1;
                let b = 2;
            }
        };
        let stream2 = unwrap_block_if_single(&block2);
        assert_eq!(stream2.to_string(), "{ let a = 1 ; let b = 2 ; }");
    }

    #[test]
    fn test_dereference_maybe() {
        let block: Block = syn::parse_quote! {
            {
                &my_var
            }
        };
        let stmt = dereference_maybe(&block);
        if let Some(stmt) = stmt {
            assert_eq!(stmt.to_string(), "my_var");
        } else {
            unreachable!("Expected Some");
        }

        let block2: Block = syn::parse_quote! {
            {
                my_var
            }
        };
        let stmt2 = dereference_maybe(&block2);
        assert!(stmt2.is_none());
    }

    #[test]
    fn test_is_default_block() {
        let block1: Block = syn::parse_quote! { {} };
        assert!(is_default_block(&block1));

        let block2: Block = syn::parse_quote! { { Default::default() } };
        assert!(is_default_block(&block2));

        let block3: Block = syn::parse_quote! { { 123 } };
        assert!(!is_default_block(&block3));
    }

    #[test]
    fn test_parse_block_of_statements_simple_value() {
        // Single value: both key and value are the expr; method is into()
        let block: Block = syn::parse_quote! { { my_var } };
        let (key, value, method) = parse_block_of_statements(&block);
        assert_eq!(key.to_string(), "my_var");
        assert_eq!(value.to_string(), "my_var");
        assert_eq!(method.to_string(), "into ()");
    }

    #[test]
    fn test_parse_block_of_statements_reference_value() {
        // Single-statement reference: inner expr used as value so `.clone()` applies correctly
        let block: Block = syn::parse_quote! { { &my_var } };
        let (key, value, method) = parse_block_of_statements(&block);
        assert_eq!(key.to_string(), "my_var");
        assert_eq!(value.to_string(), "my_var");
        assert_eq!(method.to_string(), "clone ()");
    }

    #[test]
    fn test_parse_block_of_statements_reference_with_preceding_stmts() {
        // Multi-statement block ending with a reference: whole block preserved as value
        let block: Block = syn::parse_quote! { { let mut x = 1; x += 1; &x } };
        let (key, value, method) = parse_block_of_statements(&block);
        assert_eq!(key.to_string(), "x");
        assert_eq!(value.to_string(), "{ let mut x = 1 ; x += 1 ; & x }");
        assert_eq!(method.to_string(), "clone ()");
    }

    #[test]
    fn test_extract_spread_block_with_spread() {
        // Spread block: last stmt is a range like `..expr`
        let block: Block = syn::parse_quote! { { ..my_items } };
        let result = extract_spread_block(&block, |v| quote::quote! { spread(#v) });
        if let Some(tokens) = result {
            let s = tokens.to_string();
            assert!(s.contains("spread"), "Expected spread call, got: {s}");
            assert!(s.contains("my_items"), "Expected items ident, got: {s}");
        } else {
            unreachable!("Expected Some for spread block");
        }
    }

    #[test]
    fn test_extract_spread_block_without_spread() {
        // Non-spread block: should return None
        let block: Block = syn::parse_quote! { { my_items } };
        let result = extract_spread_block(&block, |v| quote::quote! { spread(#v) });
        assert!(result.is_none(), "Expected None for non-spread block");
    }
}
