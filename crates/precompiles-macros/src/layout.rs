use crate::{
    FieldKind,
    packing::{self, LayoutField, PackingConstants, SlotAssignment},
};
use quote::quote;
use syn::{Ident, Visibility};

/// Generate the transformed struct with handler fields
pub(crate) fn gen_struct(
    name: &Ident,
    vis: &Visibility,
    allocated_fields: &[LayoutField<'_>],
) -> proc_macro2::TokenStream {
    // Generate handler field for each storage variable
    let handler_fields = allocated_fields.iter().map(|field| {
        let field_name = field.name;
        let handler_type = match &field.kind {
            FieldKind::Slot(ty) => quote! { crate::storage::Slot<#ty> },
            FieldKind::Mapping { key, value } => {
                quote! { crate::storage::Mapping<#key, #value> }
            }
            FieldKind::NestedMapping { key1, key2, value } => {
                quote! { crate::storage::NestedMapping<#key1, #key2, #value> }
            }
        };

        quote! {
            pub #field_name: #handler_type
        }
    });

    quote! {
        #vis struct #name {
            #(#handler_fields,)*
            address: ::std::rc::Rc<::alloy::primitives::Address>,
        }
    }
}

/// Generate the constructor method
pub(crate) fn gen_constructor(
    name: &Ident,
    allocated_fields: &[LayoutField<'_>],
) -> proc_macro2::TokenStream {
    let consts = PackingConstants::new;

    // Generate handler initializations for each field
    let field_inits = allocated_fields.iter().enumerate().map(|(idx, field)| {
        let field_name = field.name;
        let slot_const = consts(field_name).slot();
        let offset_const = consts(field_name).offset();

        // Calculate neighbor slot references for packing detection
        let prev_slot_const_ref = if idx > 0 {
            let prev = &allocated_fields[idx - 1];
            let prev_slot = consts(prev.name).slot();
            Some(quote! { slots::#prev_slot })
        } else {
            None
        };

        let next_slot_const_ref = if idx + 1 < allocated_fields.len() {
            let next = &allocated_fields[idx + 1];
            let next_slot = consts(next.name).slot();
            Some(quote! { slots::#next_slot })
        } else {
            None
        };

        let handler_init = match &field.kind {
            FieldKind::Slot(ty) => {
                // Generate LayoutCtx expression based on packing
                let layout_ctx = packing::gen_layout_ctx_expr_inefficient(
                    ty,
                    matches!(field.assigned_slot, SlotAssignment::Manual(_)),
                    quote! { slots::#slot_const },
                    quote! { slots::#offset_const },
                    prev_slot_const_ref,
                    next_slot_const_ref,
                );

                quote! {
                    #field_name: crate::storage::Slot::new_with_ctx(
                        slots::#slot_const,
                        #layout_ctx,
                        ::std::rc::Rc::clone(&address_rc)
                    )
                }
            }
            FieldKind::Mapping { .. } => {
                quote! {
                    #field_name: crate::storage::Mapping::new(
                        slots::#slot_const,
                        ::std::rc::Rc::clone(&address_rc)
                    )
                }
            }
            FieldKind::NestedMapping { .. } => {
                quote! {
                    #field_name: crate::storage::NestedMapping::new(
                        slots::#slot_const,
                        ::std::rc::Rc::clone(&address_rc)
                    )
                }
            }
        };

        handler_init
    });

    quote! {
        impl #name {
            #[inline(always)]
            fn _new(address: ::alloy::primitives::Address) -> Self {
                // Run collision detection checks in debug builds
                #[cfg(debug_assertions)]
                {
                    slots::__check_all_collisions();
                }

                let address_rc = ::std::rc::Rc::new(address);

                Self {
                    #(#field_inits,)*
                    address: address_rc,
                }
            }
        }
    }
}

/// Generate the `trait ContractStorage` implementation
pub(crate) fn gen_contract_storage_impl(name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        impl crate::storage::ContractStorage for #name {
            #[inline(always)]
            fn address(&self) -> ::alloy::primitives::Address {
                *self.address
            }
        }
    }
}

/// Generate the `slots` module with constants and collision checks
///
/// Returns the slots module containing only constants and collision detection functions
pub(crate) fn gen_slots_module(allocated_fields: &[LayoutField<'_>]) -> proc_macro2::TokenStream {
    // Generate constants and collision check functions
    let constants = packing::gen_constants_from_ir(allocated_fields, false);
    let collision_checks = gen_collision_checks(allocated_fields);

    quote! {
        pub mod slots {
            use super::*;

            #constants
            #collision_checks
        }
    }
}

/// Generate collision check functions for all fields
fn gen_collision_checks(allocated_fields: &[LayoutField<'_>]) -> proc_macro2::TokenStream {
    let mut generated = proc_macro2::TokenStream::new();
    let mut check_fn_calls = Vec::new();

    // Generate collision detection check functions
    for (idx, allocated) in allocated_fields.iter().enumerate() {
        if let Some((check_fn_name, check_fn)) =
            packing::gen_collision_check_fn(idx, allocated, allocated_fields)
        {
            generated.extend(check_fn);
            check_fn_calls.push(check_fn_name);
        }
    }

    // Generate a module initializer that calls all check functions
    // Always generate the function, even if empty, so the constructor can call it
    generated.extend(quote! {
        #[cfg(debug_assertions)]
        #[inline(always)]
        pub(super) fn __check_all_collisions() {
            #(#check_fn_calls();)*
        }
    });

    generated
}
