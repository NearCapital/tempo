use crate::{
    Type,
    interface::{InterfaceError, InterfaceEvent, InterfaceFunction},
};
use quote::quote;
use syn::parse_quote;

pub(crate) fn get_functions(interface_type: &Type) -> Vec<InterfaceFunction> {
    vec![
        InterfaceFunction {
            name: "create_token",
            params: vec![
                ("name", parse_quote!(String)),
                ("symbol", parse_quote!(String)),
                ("currency", parse_quote!(String)),
                ("quote_token", parse_quote!(Address)),
                ("admin", parse_quote!(Address)),
            ],
            return_type: parse_quote!(U256),
            is_view: false,
            call_type_path: quote!(#interface_type::createTokenCall),
        },
        InterfaceFunction {
            name: "token_id_counter",
            params: vec![],
            return_type: parse_quote!(U256),
            is_view: true,
            call_type_path: quote!(#interface_type::tokenIdCounterCall),
        },
    ]
}

pub(crate) fn get_events(interface_type: &Type) -> Vec<InterfaceEvent> {
    vec![InterfaceEvent {
        name: "token_created",
        params: vec![
            ("token", parse_quote!(Address), true),
            ("token_id", parse_quote!(U256), true),
            ("name", parse_quote!(String), false),
            ("symbol", parse_quote!(String), false),
            ("currency", parse_quote!(String), false),
            ("admin", parse_quote!(Address), false),
        ],
        event_type_path: quote!(#interface_type::TokenCreated),
    }]
}

pub(crate) fn get_errors(_interface_type: &Type) -> Vec<InterfaceError> {
    vec![]
}
