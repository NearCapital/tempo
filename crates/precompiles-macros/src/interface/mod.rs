//! Interface parsing and function extraction for contract macro.
//!
//! This module handles parsing the `#[contract(InterfaceName)]` attribute and
//! extracting interface function signatures for trait generation.

use crate::utils::{self, try_extract_type_ident};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

// TODO(rusowsky): Implement automatic method discovery from sol! generated interfaces.
mod tests;
mod tip20;

/// Represents a single function from a sol! interface.
#[derive(Debug, Clone)]
pub(crate) struct InterfaceFunction {
    /// Function name, normalized to snake_case
    pub name: &'static str,
    /// Function parameters as (name, type) pairs
    pub params: Vec<(&'static str, Type)>,
    /// Return type of the function
    pub return_type: Type,
    /// Whether this is a view function
    pub is_view: bool,
    /// Path to the Call struct for this function
    pub call_type_path: TokenStream,
}

/// Represents a single event from a sol! interface.
#[derive(Debug, Clone)]
pub(crate) struct InterfaceEvent {
    /// Event name, normalized to snake_case
    pub name: &'static str,
    /// Event parameters as (name, type, indexed) tuples
    pub params: Vec<(&'static str, Type, bool)>,
    /// Path to the Event struct for this event
    pub event_type_path: TokenStream,
}

/// Represents a single error from a sol! interface.
#[derive(Debug, Clone)]
pub(crate) struct InterfaceError {
    /// Error name
    pub name: &'static str,
    /// Error parameters as (name, type) pairs
    pub params: Vec<(&'static str, Type)>,
    /// Path to the Error struct for this error
    pub error_type_path: TokenStream,
}

/// Complete interface metadata including functions, events, and errors.
#[derive(Debug, Clone)]
pub(crate) struct Interface {
    /// Function definitions
    pub functions: Vec<InterfaceFunction>,
    /// Event definitions
    pub events: Vec<InterfaceEvent>,
    /// Error definitions
    pub errors: Vec<InterfaceError>,
}

/// Classification of function types for dispatcher routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FunctionKind {
    /// View function with no parameters
    Metadata,
    /// View function with parameters
    View,
    /// Mutating function returning
    Mutate,
    /// Mutating function returning
    MutateVoid,
}

impl InterfaceFunction {
    pub(crate) fn kind(&self) -> FunctionKind {
        match self.is_view {
            true if self.params.is_empty() => FunctionKind::Metadata,
            true => FunctionKind::View,
            false if utils::is_unit(&self.return_type) => FunctionKind::MutateVoid,
            false => FunctionKind::Mutate,
        }
    }
}

/// Constructs the event enum type path from an interface type.
/// Convention: ITIP20 -> ITIP20Events
pub(crate) fn get_event_enum_path(interface_type: &Type) -> syn::Result<TokenStream> {
    let interface_ident = try_extract_type_ident(interface_type)?;
    let event_enum_ident = format!("{interface_ident}Events");
    let event_enum: Ident = syn::parse_str(&event_enum_ident).expect("Valid identifier");

    if let Type::Path(type_path) = interface_type {
        // Preserve the path prefix if it exists (e.g., crate::ITIP20 -> crate::ITIP20Events)
        let mut path = type_path.path.clone();
        if let Some(last_segment) = path.segments.last_mut() {
            last_segment.ident = event_enum;
        }
        Ok(quote!(#path))
    } else {
        Ok(quote!(#event_enum))
    }
}

// TODO(rusowsky): Implement automatic method discovery from sol! generated interfaces.
pub(crate) fn parse_interface(interface_type: &Type) -> syn::Result<Interface> {
    let interface_ident = try_extract_type_ident(interface_type)?;
    get_interface_metadata(&interface_ident, interface_type)
}

// TODO(rusowsky): Implement automatic method discovery from sol! generated interfaces.
fn get_interface_metadata(
    interface_ident: &Ident,
    interface_type: &Type,
) -> syn::Result<Interface> {
    let interface_name = interface_ident.to_string();
    match interface_name.as_str() {
        "ITIP20" => Ok(Interface {
            functions: tip20::get_itip20_functions(interface_type),
            events: tip20::get_itip20_events(interface_type),
            errors: tip20::get_itip20_errors(interface_type),
        }),
        // Test interfaces
        "ITestToken" => Ok(Interface {
            functions: tests::get_itest_token_functions(interface_type),
            events: Vec::new(),
            errors: Vec::new(),
        }),
        "IMetadata" => Ok(Interface {
            functions: tests::get_imetadata_functions(interface_type),
            events: Vec::new(),
            errors: Vec::new(),
        }),
        "IMiniToken" => Ok(Interface {
            functions: tests::get_imini_token_functions(interface_type),
            events: tests::get_imini_token_events(interface_type),
            errors: Vec::new(),
        }),
        "IErrorTest" => Ok(Interface {
            functions: tests::get_ierror_test_functions(interface_type),
            events: Vec::new(),
            errors: tests::get_ierror_test_errors(interface_type),
        }),
        _ => {
            eprintln!(
                "Warning: Interface '{interface_name}' not in registry. No trait methods will be generated."
            );
            Ok(Interface {
                functions: Vec::new(),
                events: Vec::new(),
                errors: Vec::new(),
            })
        }
    }
}
