/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod base64;
pub mod body;
pub mod byte_stream;
pub mod endpoint;
pub mod header;
pub mod label;
pub mod middleware;
pub mod operation;
mod pin_util;
pub mod property_bag;
pub mod query;
pub mod response;
pub mod result;
pub mod retry;
mod urlencode;
