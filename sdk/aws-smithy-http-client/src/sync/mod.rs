/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Synchronization primitives with standard-library and Loom backends.

#[cfg(all(test, smithy_http_client_loom))]
mod loom;
#[cfg(not(all(test, smithy_http_client_loom)))]
mod std;

#[cfg(all(test, smithy_http_client_loom))]
pub(crate) use self::loom::*;
#[cfg(not(all(test, smithy_http_client_loom)))]
pub(crate) use self::std::*;
