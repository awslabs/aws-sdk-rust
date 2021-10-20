/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod body;
pub mod byte_stream;
pub mod endpoint;
pub mod header;
pub mod label;
pub mod middleware;
pub mod operation;
pub mod property_bag;
pub mod query;
pub mod response;
pub mod result;
pub mod retry;

#[cfg(feature = "event-stream")]
pub mod event_stream;

mod pin_util;
mod urlencode;
