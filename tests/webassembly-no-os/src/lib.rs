/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![cfg(target_family = "wasm")]

// Note the tests here are not gated since they should pass successfully in both
// wasm32-unknown-unknown and wasm32-wasip2
mod wasm32_unknown_unknown;

#[cfg(all(target_arch = "wasm32", target_env = "p2"))]
mod wasm32_wasip2;
