use crate::{
    Type,
    interface::{InterfaceError, InterfaceEvent, InterfaceFunction},
};
use quote::quote;
use syn::parse_quote;

pub(crate) fn get_functions(interface_type: &Type) -> Vec<InterfaceFunction> {
    vec![InterfaceFunction {
        name: "get_currency_decimals",
        params: vec![("currency", parse_quote!(String))],
        return_type: parse_quote!(u8),
        is_view: true,
        call_type_path: quote!(#interface_type::getCurrencyDecimalsCall),
    }]
}

pub(crate) fn get_events(_interface_type: &Type) -> Vec<InterfaceEvent> {
    vec![]
}

pub(crate) fn get_errors(_interface_type: &Type) -> Vec<InterfaceError> {
    vec![]
}
