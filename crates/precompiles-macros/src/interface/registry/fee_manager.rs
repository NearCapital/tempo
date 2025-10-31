use crate::{
    Type,
    interface::{InterfaceError, InterfaceEvent, InterfaceFunction},
};
use quote::quote;
use syn::parse_quote;

pub(crate) fn get_functions(interface_type: &Type) -> Vec<InterfaceFunction> {
    vec![
        // Constants (pure functions)
        InterfaceFunction {
            name: "basis_points",
            params: vec![],
            return_type: parse_quote!(U256),
            is_view: true,
            call_type_path: quote!(#interface_type::BASIS_POINTSCall),
        },
        InterfaceFunction {
            name: "fee_bps",
            params: vec![],
            return_type: parse_quote!(U256),
            is_view: true,
            call_type_path: quote!(#interface_type::FEE_BPSCall),
        },
        // User preference view functions
        InterfaceFunction {
            name: "user_tokens",
            params: vec![("user", parse_quote!(Address))],
            return_type: parse_quote!(Address),
            is_view: true,
            call_type_path: quote!(#interface_type::userTokensCall),
        },
        InterfaceFunction {
            name: "validator_tokens",
            params: vec![("validator", parse_quote!(Address))],
            return_type: parse_quote!(Address),
            is_view: true,
            call_type_path: quote!(#interface_type::validatorTokensCall),
        },
        // Fee view function
        InterfaceFunction {
            name: "get_fee_token_balance",
            params: vec![
                ("sender", parse_quote!(Address)),
                ("validator", parse_quote!(Address)),
            ],
            return_type: parse_quote!((Address, U256)),
            is_view: true,
            call_type_path: quote!(#interface_type::getFeeTokenBalanceCall),
        },
        // Mutating functions (void)
        InterfaceFunction {
            name: "set_user_token",
            params: vec![("token", parse_quote!(Address))],
            return_type: parse_quote!(()),
            is_view: false,
            call_type_path: quote!(#interface_type::setUserTokenCall),
        },
        InterfaceFunction {
            name: "set_validator_token",
            params: vec![("token", parse_quote!(Address))],
            return_type: parse_quote!(()),
            is_view: false,
            call_type_path: quote!(#interface_type::setValidatorTokenCall),
        },
        InterfaceFunction {
            name: "execute_block",
            params: vec![],
            return_type: parse_quote!(()),
            is_view: false,
            call_type_path: quote!(#interface_type::executeBlockCall),
        },
    ]
}

pub(crate) fn get_events(interface_type: &Type) -> Vec<InterfaceEvent> {
    vec![
        InterfaceEvent {
            name: "user_token_set",
            params: vec![
                ("user", parse_quote!(Address), true),
                ("token", parse_quote!(Address), true),
            ],
            event_type_path: quote!(#interface_type::UserTokenSet),
        },
        InterfaceEvent {
            name: "validator_token_set",
            params: vec![
                ("validator", parse_quote!(Address), true),
                ("token", parse_quote!(Address), true),
            ],
            event_type_path: quote!(#interface_type::ValidatorTokenSet),
        },
    ]
}

pub(crate) fn get_errors(interface_type: &Type) -> Vec<InterfaceError> {
    vec![
        InterfaceError {
            name: "only_validator",
            params: vec![],
            error_type_path: quote!(#interface_type::OnlyValidator),
        },
        InterfaceError {
            name: "only_system_contract",
            params: vec![],
            error_type_path: quote!(#interface_type::OnlySystemContract),
        },
        InterfaceError {
            name: "invalid_token",
            params: vec![],
            error_type_path: quote!(#interface_type::InvalidToken),
        },
        InterfaceError {
            name: "pool_does_not_exist",
            params: vec![],
            error_type_path: quote!(#interface_type::PoolDoesNotExist),
        },
        InterfaceError {
            name: "insufficient_liquidity",
            params: vec![],
            error_type_path: quote!(#interface_type::InsufficientLiquidity),
        },
        InterfaceError {
            name: "insufficient_fee_token_balance",
            params: vec![],
            error_type_path: quote!(#interface_type::InsufficientFeeTokenBalance),
        },
        InterfaceError {
            name: "internal_error",
            params: vec![],
            error_type_path: quote!(#interface_type::InternalError),
        },
        InterfaceError {
            name: "cannot_change_within_block",
            params: vec![],
            error_type_path: quote!(#interface_type::CannotChangeWithinBlock),
        },
        InterfaceError {
            name: "token_policy_forbids",
            params: vec![],
            error_type_path: quote!(#interface_type::TokenPolicyForbids),
        },
    ]
}
