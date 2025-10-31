use crate::{
    Type,
    interface::{InterfaceError, InterfaceEvent, InterfaceFunction},
};
use quote::quote;
use syn::parse_quote;

// Test interface for E2E dispatcher tests
pub(crate) fn get_itest_token_functions(interface_type: &Type) -> Vec<InterfaceFunction> {
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
pub(crate) fn get_imetadata_functions(interface_type: &Type) -> Vec<InterfaceFunction> {
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
pub(crate) fn get_imini_token_functions(interface_type: &Type) -> Vec<InterfaceFunction> {
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

pub(crate) fn get_imini_token_events(interface_type: &Type) -> Vec<InterfaceEvent> {
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
pub(crate) fn get_ierror_test_functions(interface_type: &Type) -> Vec<InterfaceFunction> {
    vec![InterfaceFunction {
        name: "dummy",
        params: vec![],
        return_type: parse_quote!(()),
        is_view: false,
        call_type_path: quote!(#interface_type::dummyCall),
    }]
}

pub(crate) fn get_ierror_test_errors(interface_type: &Type) -> Vec<InterfaceError> {
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
    use crate::{interface::*, utils::try_extract_type_ident};

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

        // Should have 34 functions
        assert_eq!(parsed.functions.len(), 34);

        // Should have 11 events (matching sol! interface)
        assert_eq!(parsed.events.len(), 14);

        // Should have 13 errors
        assert_eq!(parsed.errors.len(), 16);

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
        assert_eq!(insufficient_balance_error.unwrap().params.len(), 0);
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
