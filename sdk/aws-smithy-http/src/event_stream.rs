/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Provides Sender/Receiver implementations for Event Stream codegen.

use std::error::Error as StdError;

mod input;
mod output;

pub type BoxError = Box<dyn StdError + Send + Sync + 'static>;

#[doc(inline)]
pub use input::{EventStreamInput, MessageStreamAdapter};

#[doc(inline)]
pub use output::{Error, RawMessage, Receiver};
