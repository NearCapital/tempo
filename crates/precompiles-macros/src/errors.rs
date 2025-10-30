//! Error constructor helper generation for contract macro.
//!
//! This module generates static constructor methods for each error defined in the
//! contract's interfaces.

use crate::{interface::InterfaceError, utils::try_extract_type_ident};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type, parse2};

/// Extracts the final identifier from an error type path.
fn extract_error_variant_name(error_type_path: &TokenStream) -> Ident {
    let type_path: Type =
        parse2(error_type_path.clone()).expect("error_type_path should be a valid type path");

    try_extract_type_ident(&type_path).expect("Failed to extract error variant name from type path")
}

/// Constructs the error enum type path from an interface type.
/// Convention: ITIP20 -> ITIP20::ITIP20Errors
fn get_error_enum_path(interface_type: &Type) -> syn::Result<TokenStream> {
    let interface_ident = try_extract_type_ident(interface_type)?;
    let error_enum_ident = format!("{interface_ident}Errors");
    let error_enum: Ident = syn::parse_str(&error_enum_ident).expect("Valid identifier");

    Ok(quote!(#interface_type::#error_enum))
}

/// Generates error constructor methods for all errors across all interfaces.
///
/// Creates static constructor methods on each error enum that simplify error creation.
pub(crate) fn gen_error_helpers(interfaces: &[(Type, Vec<InterfaceError>)]) -> TokenStream {
    let impl_blocks: Vec<_> = interfaces
        .iter()
        .filter_map(|(interface_type, errors)| {
            if errors.is_empty() {
                return None;
            }

            // Get the error enum type path (e.g., ITIP20Errors)
            let error_enum_path = match get_error_enum_path(interface_type) {
                Ok(path) => path,
                Err(_) => return None, // Skip if we can't determine the error enum
            };

            let methods: Vec<_> = errors.iter().map(gen_constructor).collect();

            Some(quote! {
                impl #error_enum_path {
                    #(#methods)*
                }
            })
        })
        .collect();

    quote! {
        #(#impl_blocks)*
    }
}

/// Generates a single error constructor method.
fn gen_constructor(error: &InterfaceError) -> TokenStream {
    let (method_name, error_type_path) = (error.name, &error.error_type_path);
    let method_ident: Ident = syn::parse_str(method_name).expect("Valid identifier");
    let variant_ident = extract_error_variant_name(&error.error_type_path);

    if error.params.is_empty() {
        quote! {
            pub const fn #method_ident() -> Self {
                Self::#variant_ident(#error_type_path {})
            }
        }
    } else {
        let params: Vec<_> = error
            .params
            .iter()
            .map(|(param_name, param_type)| {
                let param_ident: Ident = syn::parse_str(param_name).expect("Valid identifier");
                quote! { #param_ident: #param_type }
            })
            .collect();

        let field_assignments: Vec<_> = error
            .params
            .iter()
            .map(|(param_name, _)| {
                let param_ident: Ident = syn::parse_str(param_name).expect("Valid identifier");
                quote! { #param_ident }
            })
            .collect();

        quote! {
            pub const fn #method_ident(#(#params),*) -> Self {
                Self::#variant_ident(#error_type_path {
                    #(#field_assignments),*
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_get_error_enum_path() {
        // Simple path
        let ty: Type = parse_quote!(ITIP20);
        let path = get_error_enum_path(&ty).unwrap();
        assert_eq!(path.to_string(), "ITIP20 :: ITIP20Errors");

        // Qualified path
        let ty: Type = parse_quote!(crate::ITIP20);
        let path = get_error_enum_path(&ty).unwrap();
        assert_eq!(path.to_string(), "crate :: ITIP20 :: ITIP20Errors");

        // Test with ITestToken
        let ty: Type = parse_quote!(ITestToken);
        let path = get_error_enum_path(&ty).unwrap();
        assert_eq!(path.to_string(), "ITestToken :: ITestTokenErrors");
    }
}
