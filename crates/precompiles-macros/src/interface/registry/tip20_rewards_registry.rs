use crate::{
    Type,
    interface::{InterfaceError, InterfaceEvent, InterfaceFunction},
};
use quote::quote;
use syn::parse_quote;

pub(crate) fn get_functions(interface_type: &Type) -> Vec<InterfaceFunction> {
    vec![InterfaceFunction {
        name: "finalize_streams",
        params: vec![],
        return_type: parse_quote!(()),
        is_view: false,
        call_type_path: quote!(#interface_type::finalizeStreamsCall),
    }]
}

pub(crate) fn get_events(_interface_type: &Type) -> Vec<InterfaceEvent> {
    vec![]
}

pub(crate) fn get_errors(interface_type: &Type) -> Vec<InterfaceError> {
    vec![
        InterfaceError {
            name: "unauthorized",
            params: vec![],
            error_type_path: quote!(#interface_type::Unauthorized),
        },
        InterfaceError {
            name: "streams_already_finalized",
            params: vec![],
            error_type_path: quote!(#interface_type::StreamsAlreadyFinalized),
        },
    ]
}
