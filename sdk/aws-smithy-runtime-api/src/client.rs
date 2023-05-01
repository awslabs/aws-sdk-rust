/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/// Smithy identity used by auth and signing.
pub mod identity;

/// Smithy interceptors for smithy clients.
///
/// Interceptors are lifecycle hooks that can read/modify requests and responses.
pub mod interceptors;

pub mod orchestrator;

/// Smithy code related to retry handling and token bucket.
///
/// This code defines when and how failed requests should be retried. It also defines the behavior
/// used to limit the rate that requests are sent.
pub mod retries;
/// Runtime plugin type definitions.
pub mod runtime_plugin;

/// Smithy endpoint resolution runtime plugins
pub mod endpoints;

/// Smithy auth runtime plugins
pub mod auth;
