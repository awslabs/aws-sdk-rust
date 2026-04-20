/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/* Automatically managed default lints */
#![cfg_attr(docsrs, feature(doc_cfg))]
/* End of automatically managed default lints */

//! Proc macros for `aws-smithy-runtime-api`.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ImplItem, ItemImpl};

// If you update this list, also update:
//   - `OverriddenHooks` constants in `aws-smithy-runtime-api/src/client/interceptors.rs`
//   - Hook methods on the `Intercept` trait in the same file
const KNOWN_HOOKS: &[&str] = &[
    "read_before_execution",
    "modify_before_serialization",
    "read_before_serialization",
    "read_after_serialization",
    "modify_before_retry_loop",
    "read_before_attempt",
    "modify_before_signing",
    "read_before_signing",
    "read_after_signing",
    "modify_before_transmit",
    "read_before_transmit",
    "read_after_transmit",
    "modify_before_deserialization",
    "read_before_deserialization",
    "read_after_deserialization",
    "modify_before_attempt_completion",
    "read_after_attempt",
    "modify_before_completion",
    "read_after_execution",
];

const _: () = assert!(
    KNOWN_HOOKS.len() <= 32,
    "OverriddenHooks uses a u32 bitmask; widen to u64 in interceptors.rs if more hooks are needed"
);

/// Automatically generates an `overridden_hooks()` method on an `impl Intercept` block
/// based on which hook methods are overridden.
///
/// This attribute must be placed on an `impl Intercept for T` block. It inspects
/// which hook methods are overridden and generates a corresponding
/// `overridden_hooks()` method that returns the correct `OverriddenHooks` bitmask.
///
/// # Example
/// ```ignore
/// #[dyn_dispatch_hint]
/// impl Intercept for MyInterceptor {
///     fn name(&self) -> &'static str { "MyInterceptor" }
///     fn modify_before_signing(...) -> ... { ... }
/// }
/// // Generates: fn overridden_hooks(&self) -> OverriddenHooks { OverriddenHooks::MODIFY_BEFORE_SIGNING }
/// ```
#[proc_macro_attribute]
pub fn dyn_dispatch_hint(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut impl_block = parse_macro_input!(item as ItemImpl);

    let overridden: Vec<String> = impl_block
        .items
        .iter()
        .filter_map(|item| {
            if let ImplItem::Fn(method) = item {
                let name = method.sig.ident.to_string();
                if KNOWN_HOOKS.contains(&name.as_str()) {
                    Some(name)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    let flags: Vec<proc_macro2::TokenStream> = overridden
        .iter()
        .map(|name| {
            let upper = name.to_uppercase();
            let ident = syn::Ident::new(&upper, proc_macro2::Span::call_site());
            quote! { ::aws_smithy_runtime_api::client::interceptors::OverriddenHooks::#ident }
        })
        .collect();

    let body = if flags.is_empty() {
        quote! { ::aws_smithy_runtime_api::client::interceptors::OverriddenHooks::none() }
    } else {
        quote! { #(#flags)|* }
    };

    let method: ImplItem = syn::parse_quote! {
        fn overridden_hooks(&self) -> ::aws_smithy_runtime_api::client::interceptors::OverriddenHooks {
            #body
        }
    };
    impl_block.items.push(method);

    quote! { #impl_block }.into()
}
