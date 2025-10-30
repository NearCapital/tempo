//! Interface parsing and function extraction for contract macro.
//!
//! This module handles parsing the `#[contract(InterfaceName)]` attribute and
//! extracting interface function signatures for trait generation.

use crate::utils::{self, try_extract_type_ident};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

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
            functions: get_itip20_functions(interface_type),
            events: get_itip20_events(interface_type),
            errors: get_itip20_errors(interface_type),
        }),
        "ITestToken" => Ok(Interface {
            functions: get_itest_token_functions(interface_type),
            events: Vec::new(),
            errors: Vec::new(),
        }),
        "IMetadata" => Ok(Interface {
            functions: get_imetadata_functions(interface_type),
            events: Vec::new(),
            errors: Vec::new(),
        }),
        "IMiniToken" => Ok(Interface {
            functions: get_imini_token_functions(interface_type),
            events: get_imini_token_events(interface_type),
            errors: Vec::new(),
        }),
        "IErrorTest" => Ok(Interface {
            functions: get_ierror_test_functions(interface_type),
            events: Vec::new(),
            errors: get_ierror_test_errors(interface_type),
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

// TODO(rusowsky): Implement automatic method discovery from sol! generated interfaces.
fn get_itip20_functions(interface_type: &Type) -> Vec<InterfaceFunction> {
    use syn::parse_quote;

    vec![
        // Metadata functions (view, no parameters)
        InterfaceFunction {
            name: "name",
            params: vec![],
            return_type: parse_quote!(String),
            is_view: true,
            call_type_path: quote!(#interface_type::nameCall),
        },
        InterfaceFunction {
            name: "symbol",
            params: vec![],
            return_type: parse_quote!(String),
            is_view: true,
            call_type_path: quote!(#interface_type::symbolCall),
        },
        InterfaceFunction {
            name: "decimals",
            params: vec![],
            return_type: parse_quote!(u8),
            is_view: true,
            call_type_path: quote!(#interface_type::decimalsCall),
        },
        InterfaceFunction {
            name: "currency",
            params: vec![],
            return_type: parse_quote!(String),
            is_view: true,
            call_type_path: quote!(#interface_type::currencyCall),
        },
        InterfaceFunction {
            name: "total_supply",
            params: vec![],
            return_type: parse_quote!(U256),
            is_view: true,
            call_type_path: quote!(#interface_type::totalSupplyCall),
        },
        InterfaceFunction {
            name: "supply_cap",
            params: vec![],
            return_type: parse_quote!(U256),
            is_view: true,
            call_type_path: quote!(#interface_type::supplyCapCall),
        },
        InterfaceFunction {
            name: "transfer_policy_id",
            params: vec![],
            return_type: parse_quote!(u64),
            is_view: true,
            call_type_path: quote!(#interface_type::transferPolicyIdCall),
        },
        InterfaceFunction {
            name: "paused",
            params: vec![],
            return_type: parse_quote!(bool),
            is_view: true,
            call_type_path: quote!(#interface_type::pausedCall),
        },
        InterfaceFunction {
            name: "quote_token",
            params: vec![],
            return_type: parse_quote!(Address),
            is_view: true,
            call_type_path: quote!(#interface_type::quoteTokenCall),
        },
        InterfaceFunction {
            name: "next_quote_token",
            params: vec![],
            return_type: parse_quote!(Address),
            is_view: true,
            call_type_path: quote!(#interface_type::nextQuoteTokenCall),
        },
        // View functions with parameters
        InterfaceFunction {
            name: "balance_of",
            params: vec![("account", parse_quote!(Address))],
            return_type: parse_quote!(U256),
            is_view: true,
            call_type_path: quote!(#interface_type::balanceOfCall),
        },
        InterfaceFunction {
            name: "allowance",
            params: vec![
                ("owner", parse_quote!(Address)),
                ("spender", parse_quote!(Address)),
            ],
            return_type: parse_quote!(U256),
            is_view: true,
            call_type_path: quote!(#interface_type::allowanceCall),
        },
        // Mutating functions (non-void)
        InterfaceFunction {
            name: "transfer",
            params: vec![
                ("to", parse_quote!(Address)),
                ("amount", parse_quote!(U256)),
            ],
            return_type: parse_quote!(bool),
            is_view: false,
            call_type_path: quote!(#interface_type::transferCall),
        },
        InterfaceFunction {
            name: "transfer_from",
            params: vec![
                ("from", parse_quote!(Address)),
                ("to", parse_quote!(Address)),
                ("amount", parse_quote!(U256)),
            ],
            return_type: parse_quote!(bool),
            is_view: false,
            call_type_path: quote!(#interface_type::transferFromCall),
        },
        InterfaceFunction {
            name: "approve",
            params: vec![
                ("spender", parse_quote!(Address)),
                ("amount", parse_quote!(U256)),
            ],
            return_type: parse_quote!(bool),
            is_view: false,
            call_type_path: quote!(#interface_type::approveCall),
        },
        InterfaceFunction {
            name: "transfer_from_with_memo",
            params: vec![
                ("from", parse_quote!(Address)),
                ("to", parse_quote!(Address)),
                ("amount", parse_quote!(U256)),
                ("memo", parse_quote!(B256)),
            ],
            return_type: parse_quote!(bool),
            is_view: false,
            call_type_path: quote!(#interface_type::transferFromWithMemoCall),
        },
        // Mutating functions (void)
        InterfaceFunction {
            name: "mint",
            params: vec![
                ("to", parse_quote!(Address)),
                ("amount", parse_quote!(U256)),
            ],
            return_type: parse_quote!(()),
            is_view: false,
            call_type_path: quote!(#interface_type::mintCall),
        },
        InterfaceFunction {
            name: "burn",
            params: vec![("amount", parse_quote!(U256))],
            return_type: parse_quote!(()),
            is_view: false,
            call_type_path: quote!(#interface_type::burnCall),
        },
        InterfaceFunction {
            name: "mint_with_memo",
            params: vec![
                ("to", parse_quote!(Address)),
                ("amount", parse_quote!(U256)),
                ("memo", parse_quote!(B256)),
            ],
            return_type: parse_quote!(()),
            is_view: false,
            call_type_path: quote!(#interface_type::mintWithMemoCall),
        },
        InterfaceFunction {
            name: "burn_with_memo",
            params: vec![("amount", parse_quote!(U256)), ("memo", parse_quote!(B256))],
            return_type: parse_quote!(()),
            is_view: false,
            call_type_path: quote!(#interface_type::burnWithMemoCall),
        },
        InterfaceFunction {
            name: "burn_blocked",
            params: vec![
                ("from", parse_quote!(Address)),
                ("amount", parse_quote!(U256)),
            ],
            return_type: parse_quote!(()),
            is_view: false,
            call_type_path: quote!(#interface_type::burnBlockedCall),
        },
        InterfaceFunction {
            name: "transfer_with_memo",
            params: vec![
                ("to", parse_quote!(Address)),
                ("amount", parse_quote!(U256)),
                ("memo", parse_quote!(B256)),
            ],
            return_type: parse_quote!(()),
            is_view: false,
            call_type_path: quote!(#interface_type::transferWithMemoCall),
        },
        // Admin functions (void)
        InterfaceFunction {
            name: "change_transfer_policy_id",
            params: vec![("new_policy_id", parse_quote!(u64))],
            return_type: parse_quote!(()),
            is_view: false,
            call_type_path: quote!(#interface_type::changeTransferPolicyIdCall),
        },
        InterfaceFunction {
            name: "set_supply_cap",
            params: vec![("new_supply_cap", parse_quote!(U256))],
            return_type: parse_quote!(()),
            is_view: false,
            call_type_path: quote!(#interface_type::setSupplyCapCall),
        },
        InterfaceFunction {
            name: "pause",
            params: vec![],
            return_type: parse_quote!(()),
            is_view: false,
            call_type_path: quote!(#interface_type::pauseCall),
        },
        InterfaceFunction {
            name: "unpause",
            params: vec![],
            return_type: parse_quote!(()),
            is_view: false,
            call_type_path: quote!(#interface_type::unpauseCall),
        },
        InterfaceFunction {
            name: "update_quote_token",
            params: vec![("new_quote_token", parse_quote!(Address))],
            return_type: parse_quote!(()),
            is_view: false,
            call_type_path: quote!(#interface_type::updateQuoteTokenCall),
        },
        InterfaceFunction {
            name: "finalize_quote_token_update",
            params: vec![],
            return_type: parse_quote!(()),
            is_view: false,
            call_type_path: quote!(#interface_type::finalizeQuoteTokenUpdateCall),
        },
    ]
}

// TODO(rusowsky): Implement automatic event discovery from sol! generated interfaces.
fn get_itip20_events(interface_type: &Type) -> Vec<InterfaceEvent> {
    use syn::parse_quote;

    vec![
        // Core token events
        InterfaceEvent {
            name: "transfer",
            params: vec![
                ("from", parse_quote!(Address), true),
                ("to", parse_quote!(Address), true),
                ("amount", parse_quote!(U256), false),
            ],
            event_type_path: quote!(#interface_type::Transfer),
        },
        InterfaceEvent {
            name: "approval",
            params: vec![
                ("owner", parse_quote!(Address), true),
                ("spender", parse_quote!(Address), true),
                ("amount", parse_quote!(U256), false),
            ],
            event_type_path: quote!(#interface_type::Approval),
        },
        InterfaceEvent {
            name: "mint",
            params: vec![
                ("to", parse_quote!(Address), true),
                ("amount", parse_quote!(U256), false),
            ],
            event_type_path: quote!(#interface_type::Mint),
        },
        InterfaceEvent {
            name: "burn",
            params: vec![
                ("from", parse_quote!(Address), true),
                ("amount", parse_quote!(U256), false),
            ],
            event_type_path: quote!(#interface_type::Burn),
        },
        InterfaceEvent {
            name: "burn_blocked",
            params: vec![
                ("from", parse_quote!(Address), true),
                ("amount", parse_quote!(U256), false),
            ],
            event_type_path: quote!(#interface_type::BurnBlocked),
        },
        InterfaceEvent {
            name: "transfer_with_memo",
            params: vec![
                ("from", parse_quote!(Address), true),
                ("to", parse_quote!(Address), true),
                ("amount", parse_quote!(U256), false),
                ("memo", parse_quote!(B256), false),
            ],
            event_type_path: quote!(#interface_type::TransferWithMemo),
        },
        // Admin events
        InterfaceEvent {
            name: "transfer_policy_update",
            params: vec![
                ("updater", parse_quote!(Address), true),
                ("new_policy_id", parse_quote!(u64), true),
            ],
            event_type_path: quote!(#interface_type::TransferPolicyUpdate),
        },
        InterfaceEvent {
            name: "supply_cap_update",
            params: vec![
                ("updater", parse_quote!(Address), true),
                ("new_supply_cap", parse_quote!(U256), true),
            ],
            event_type_path: quote!(#interface_type::SupplyCapUpdate),
        },
        InterfaceEvent {
            name: "pause_state_update",
            params: vec![
                ("updater", parse_quote!(Address), true),
                ("is_paused", parse_quote!(bool), false),
            ],
            event_type_path: quote!(#interface_type::PauseStateUpdate),
        },
        InterfaceEvent {
            name: "update_quote_token",
            params: vec![
                ("updater", parse_quote!(Address), true),
                ("new_quote_token", parse_quote!(Address), true),
            ],
            event_type_path: quote!(#interface_type::UpdateQuoteToken),
        },
        InterfaceEvent {
            name: "quote_token_update_finalized",
            params: vec![
                ("updater", parse_quote!(Address), true),
                ("new_quote_token", parse_quote!(Address), true),
            ],
            event_type_path: quote!(#interface_type::QuoteTokenUpdateFinalized),
        },
    ]
}

// TODO(rusowsky): Implement automatic error discovery from sol! generated interfaces.
fn get_itip20_errors(interface_type: &Type) -> Vec<InterfaceError> {
    use syn::parse_quote;

    vec![
        // Balance and allowance errors
        InterfaceError {
            name: "insufficient_balance",
            params: vec![
                ("account", parse_quote!(Address)),
                ("balance", parse_quote!(U256)),
                ("needed", parse_quote!(U256)),
            ],
            error_type_path: quote!(#interface_type::InsufficientBalance),
        },
        InterfaceError {
            name: "insufficient_allowance",
            params: vec![
                ("owner", parse_quote!(Address)),
                ("spender", parse_quote!(Address)),
                ("allowance", parse_quote!(U256)),
                ("needed", parse_quote!(U256)),
            ],
            error_type_path: quote!(#interface_type::InsufficientAllowance),
        },
        // Supply errors
        InterfaceError {
            name: "supply_cap_exceeded",
            params: vec![
                ("supply_cap", parse_quote!(U256)),
                ("total_supply", parse_quote!(U256)),
                ("amount", parse_quote!(U256)),
            ],
            error_type_path: quote!(#interface_type::SupplyCapExceeded),
        },
        InterfaceError {
            name: "invalid_supply_cap",
            params: vec![("supply_cap", parse_quote!(U256))],
            error_type_path: quote!(#interface_type::InvalidSupplyCap),
        },
        // Access control errors
        InterfaceError {
            name: "unauthorized",
            params: vec![("account", parse_quote!(Address))],
            error_type_path: quote!(#interface_type::Unauthorized),
        },
        // State errors
        InterfaceError {
            name: "paused",
            params: vec![],
            error_type_path: quote!(#interface_type::Paused),
        },
        InterfaceError {
            name: "not_paused",
            params: vec![],
            error_type_path: quote!(#interface_type::NotPaused),
        },
        // Transfer policy errors
        InterfaceError {
            name: "invalid_transfer_policy",
            params: vec![("policy_id", parse_quote!(u64))],
            error_type_path: quote!(#interface_type::InvalidTransferPolicy),
        },
        InterfaceError {
            name: "transfer_policy_violation",
            params: vec![
                ("from", parse_quote!(Address)),
                ("to", parse_quote!(Address)),
                ("policy_id", parse_quote!(u64)),
            ],
            error_type_path: quote!(#interface_type::TransferPolicyViolation),
        },
        // Address errors
        InterfaceError {
            name: "invalid_address",
            params: vec![("address", parse_quote!(Address))],
            error_type_path: quote!(#interface_type::InvalidAddress),
        },
        // Amount errors
        InterfaceError {
            name: "invalid_amount",
            params: vec![("amount", parse_quote!(U256))],
            error_type_path: quote!(#interface_type::InvalidAmount),
        },
        // Quote token errors
        InterfaceError {
            name: "no_pending_quote_token_update",
            params: vec![],
            error_type_path: quote!(#interface_type::NoPendingQuoteTokenUpdate),
        },
        InterfaceError {
            name: "quote_token_update_not_ready",
            params: vec![
                ("current_time", parse_quote!(U256)),
                ("ready_time", parse_quote!(U256)),
            ],
            error_type_path: quote!(#interface_type::QuoteTokenUpdateNotReady),
        },
    ]
}

// Test interface for E2E dispatcher tests
fn get_itest_token_functions(interface_type: &Type) -> Vec<InterfaceFunction> {
    use syn::parse_quote;

    vec![
        // Metadata functions (view, no parameters)
        InterfaceFunction {
            name: "name",
            params: vec![],
            return_type: parse_quote!(String),
            is_view: true,
            call_type_path: quote!(#interface_type::nameCall),
        },
        InterfaceFunction {
            name: "symbol",
            params: vec![],
            return_type: parse_quote!(String),
            is_view: true,
            call_type_path: quote!(#interface_type::symbolCall),
        },
        InterfaceFunction {
            name: "decimals",
            params: vec![],
            return_type: parse_quote!(u8),
            is_view: true,
            call_type_path: quote!(#interface_type::decimalsCall),
        },
        // View functions (with parameters)
        InterfaceFunction {
            name: "balance_of",
            params: vec![("account", parse_quote!(Address))],
            return_type: parse_quote!(U256),
            is_view: true,
            call_type_path: quote!(#interface_type::balanceOfCall),
        },
        InterfaceFunction {
            name: "allowance",
            params: vec![
                ("owner", parse_quote!(Address)),
                ("spender", parse_quote!(Address)),
            ],
            return_type: parse_quote!(U256),
            is_view: true,
            call_type_path: quote!(#interface_type::allowanceCall),
        },
        // Mutating functions (non-void)
        InterfaceFunction {
            name: "transfer",
            params: vec![
                ("to", parse_quote!(Address)),
                ("amount", parse_quote!(U256)),
            ],
            return_type: parse_quote!(bool),
            is_view: false,
            call_type_path: quote!(#interface_type::transferCall),
        },
        InterfaceFunction {
            name: "approve",
            params: vec![
                ("spender", parse_quote!(Address)),
                ("amount", parse_quote!(U256)),
            ],
            return_type: parse_quote!(bool),
            is_view: false,
            call_type_path: quote!(#interface_type::approveCall),
        },
        // Mutating functions (void)
        InterfaceFunction {
            name: "mint",
            params: vec![
                ("to", parse_quote!(Address)),
                ("amount", parse_quote!(U256)),
            ],
            return_type: parse_quote!(()),
            is_view: false,
            call_type_path: quote!(#interface_type::mintCall),
        },
        InterfaceFunction {
            name: "burn",
            params: vec![("amount", parse_quote!(U256))],
            return_type: parse_quote!(()),
            is_view: false,
            call_type_path: quote!(#interface_type::burnCall),
        },
    ]
}

// Test interface for multi-interface testing
fn get_imetadata_functions(interface_type: &Type) -> Vec<InterfaceFunction> {
    use syn::parse_quote;

    vec![
        InterfaceFunction {
            name: "version",
            params: vec![],
            return_type: parse_quote!(U256),
            is_view: true,
            call_type_path: quote!(#interface_type::versionCall),
        },
        InterfaceFunction {
            name: "owner",
            params: vec![],
            return_type: parse_quote!(Address),
            is_view: true,
            call_type_path: quote!(#interface_type::ownerCall),
        },
    ]
}

// Mini token test interface for event emission testing
fn get_imini_token_functions(interface_type: &Type) -> Vec<InterfaceFunction> {
    use syn::parse_quote;

    vec![InterfaceFunction {
        name: "mint",
        params: vec![
            ("to", parse_quote!(Address)),
            ("amount", parse_quote!(U256)),
        ],
        return_type: parse_quote!(()),
        is_view: false,
        call_type_path: quote!(#interface_type::mintCall),
    }]
}

fn get_imini_token_events(interface_type: &Type) -> Vec<InterfaceEvent> {
    use syn::parse_quote;

    vec![
        InterfaceEvent {
            name: "transfer",
            params: vec![
                ("from", parse_quote!(Address), true),
                ("to", parse_quote!(Address), true),
                ("amount", parse_quote!(U256), false),
            ],
            event_type_path: quote!(#interface_type::Transfer),
        },
        InterfaceEvent {
            name: "mint",
            params: vec![
                ("to", parse_quote!(Address), true),
                ("amount", parse_quote!(U256), false),
            ],
            event_type_path: quote!(#interface_type::Mint),
        },
    ]
}

// Test interface for error constructor generation
fn get_ierror_test_functions(interface_type: &Type) -> Vec<InterfaceFunction> {
    use syn::parse_quote;

    vec![InterfaceFunction {
        name: "dummy",
        params: vec![],
        return_type: parse_quote!(()),
        is_view: false,
        call_type_path: quote!(#interface_type::dummyCall),
    }]
}

fn get_ierror_test_errors(interface_type: &Type) -> Vec<InterfaceError> {
    use syn::parse_quote;

    vec![
        InterfaceError {
            name: "simple_error",
            params: vec![],
            error_type_path: quote!(#interface_type::SimpleError),
        },
        InterfaceError {
            name: "parameterized_error",
            params: vec![
                ("code", parse_quote!(U256)),
                ("addr", parse_quote!(Address)),
            ],
            error_type_path: quote!(#interface_type::ParameterizedError),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_extract_interface_ident() {
        let ty: Type = parse_quote!(ITIP20);
        let ident = try_extract_type_ident(&ty).unwrap();
        assert_eq!(ident.to_string(), "ITIP20");

        let ty: Type = parse_quote!(crate::ITIP20);
        let ident = try_extract_type_ident(&ty).unwrap();
        assert_eq!(ident.to_string(), "ITIP20");
    }

    #[test]
    fn test_get_event_enum_path() {
        // Simple path
        let ty: Type = parse_quote!(ITIP20);
        let path = get_event_enum_path(&ty).unwrap();
        assert_eq!(path.to_string(), "ITIP20Events");

        // Qualified path
        let ty: Type = parse_quote!(crate::ITIP20);
        let path = get_event_enum_path(&ty).unwrap();
        assert_eq!(path.to_string(), "crate :: ITIP20Events");

        // Test with ITestToken
        let ty: Type = parse_quote!(ITestToken);
        let path = get_event_enum_path(&ty).unwrap();
        assert_eq!(path.to_string(), "ITestTokenEvents");
    }

    #[test]
    fn test_parse_interface_itip20() {
        let ty: Type = parse_quote!(ITIP20);
        let parsed = parse_interface(&ty).unwrap();

        // Should have 28 functions
        assert_eq!(parsed.functions.len(), 28);

        // Should have 11 events (matching sol! interface)
        assert_eq!(parsed.events.len(), 11);

        // Should have 13 errors
        assert_eq!(parsed.errors.len(), 13);

        // Check a few specific functions
        let name_fn = parsed.functions.iter().find(|f| f.name == "name");
        assert!(name_fn.is_some());
        assert!(name_fn.unwrap().is_view);
        assert!(name_fn.unwrap().params.is_empty());

        let balance_of_fn = parsed.functions.iter().find(|f| f.name == "balance_of");
        assert!(balance_of_fn.is_some());
        assert_eq!(balance_of_fn.unwrap().params.len(), 1);

        // Check a few specific events
        let transfer_event = parsed.events.iter().find(|e| e.name == "transfer");
        assert!(transfer_event.is_some());
        assert_eq!(transfer_event.unwrap().params.len(), 3);

        // Check a few specific errors
        let insufficient_balance_error = parsed
            .errors
            .iter()
            .find(|e| e.name == "insufficient_balance");
        assert!(insufficient_balance_error.is_some());
        assert_eq!(insufficient_balance_error.unwrap().params.len(), 3);
    }

    #[test]
    fn test_parse_unknown_interface() {
        let ty: Type = parse_quote!(UnknownInterface);
        let parsed = parse_interface(&ty).unwrap();

        // Should return empty vecs for unknown interfaces
        assert!(parsed.functions.is_empty());
        assert!(parsed.events.is_empty());
        assert!(parsed.errors.is_empty());
    }

    #[test]
    fn test_fn_kind() {
        let new_fn = |name: &'static str,
                      params: Vec<(&'static str, Type)>,
                      return_type: Type,
                      is_view: bool|
         -> InterfaceFunction {
            InterfaceFunction {
                name,
                params,
                return_type,
                is_view,
                call_type_path: quote::quote!(ITIP20::testCall),
            }
        };

        let func = new_fn("name", vec![], parse_quote!(String), true);
        assert_eq!(func.kind(), FunctionKind::Metadata);

        let func = new_fn(
            "balance_of",
            vec![("account", parse_quote!(Address))],
            parse_quote!(U256),
            true,
        );
        assert_eq!(func.kind(), FunctionKind::View);

        let func = new_fn(
            "transfer",
            vec![
                ("to", parse_quote!(Address)),
                ("amount", parse_quote!(U256)),
            ],
            parse_quote!(bool),
            false,
        );
        assert_eq!(func.kind(), FunctionKind::Mutate);

        let func = new_fn(
            "mint",
            vec![
                ("to", parse_quote!(Address)),
                ("amount", parse_quote!(U256)),
            ],
            parse_quote!(()),
            false,
        );
        assert_eq!(func.kind(), FunctionKind::MutateVoid);
    }
}
