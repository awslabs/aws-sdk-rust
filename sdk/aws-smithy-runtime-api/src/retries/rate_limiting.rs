/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Code for rate-limiting smithy clients.

pub mod error;
pub mod token;
pub mod token_bucket;

pub use token::Token;
pub use token_bucket::TokenBucket;
