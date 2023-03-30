/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/// Smithy interceptors for smithy clients.
///
/// Interceptors are lifecycle hooks that can read/modify requests and responses.
pub mod interceptors;
/// Smithy code related to retry handling and token bucket.
///
/// This code defines when and how failed requests should be retried. It also defines the behavior
/// used to limit the rate that requests are sent.
pub mod retries;
