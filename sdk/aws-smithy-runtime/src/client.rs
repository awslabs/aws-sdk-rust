/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub mod auth;

pub mod orchestrator;

/// Smithy connector runtime plugins
pub mod connections;

/// Smithy code related to retry handling and token buckets.
///
/// This code defines when and how failed requests should be retried. It also defines the behavior
/// used to limit the rate at which requests are sent.
pub mod retries;
