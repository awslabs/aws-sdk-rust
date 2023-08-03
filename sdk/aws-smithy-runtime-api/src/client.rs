/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub mod runtime_components;

/// Client orchestrator configuration accessors for the [`ConfigBag`](aws_smithy_types::config_bag::ConfigBag).
pub mod config_bag_accessors;

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

/// Smithy auth runtime plugins
pub mod auth;

/// A type to track the number of requests sent by the orchestrator for a given operation.
pub mod request_attempts;

/// Smithy connectors and related code.
pub mod connectors;
