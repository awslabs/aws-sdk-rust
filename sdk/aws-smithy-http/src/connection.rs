/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//TODO(runtimeCratesVersioningCleanup): Keep the following deprecated type alias for at least
// one release since 0.56.1 and then remove this module.

//! Types related to connection monitoring and management.

/// Metadata that tracks the state of an active connection.
#[deprecated(note = "Moved to `aws_smithy_runtime_api::client::connection::ConnectionMetadata`.")]
pub type ConnectionMetadata = aws_smithy_runtime_api::client::connection::ConnectionMetadata;
