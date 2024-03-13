/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![allow(dead_code)]

#[cfg(target_family = "wasm")]
mod http_client;
#[cfg(all(target_family = "wasm", target_os = "wasi"))]
mod wasi;
