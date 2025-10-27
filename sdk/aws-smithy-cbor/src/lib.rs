/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! CBOR abstractions for Smithy.

/* Automatically managed default lints */
#![cfg_attr(docsrs, feature(doc_cfg))]
/* End of automatically managed default lints */

pub mod data;
pub mod decode;
pub mod encode;

pub use decode::Decoder;
pub use encode::Encoder;
