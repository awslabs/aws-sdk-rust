/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Records what happened during one call to a generated client.
//!
//! A *handle* is a value that records one call. You keep the handle, add it to a [`Capture`] with
//! [`Capture::with`], and attach the `Capture` to the call as an interceptor. The `Capture` holds
//! a clone of the handle, and clones share one record, so the handle you kept reads what the call
//! reported. In this release the only handle is [`ChecksumCapture`], which records whether the
//! response body was validated against its checksum.
//!
//! ```
//! use aws_smithy_observability::unit_of_work::checksums::{ChecksumCapture, ChecksumOutcome};
//! use aws_smithy_observability::unit_of_work::Capture;
//!
//! let checksum = ChecksumCapture::new();
//! let capture = Capture::new().with(&checksum);
//! // Attach `capture` to one call with `.customize().interceptor(capture)`, send the call, and
//! // read the response body to its end. Then:
//! match checksum.try_outcome() {
//!     Some(ChecksumOutcome::Validated { algorithm, .. }) => {
//!         println!("validated with {}", algorithm.as_str())
//!     }
//!     Some(ChecksumOutcome::Mismatch { .. }) => println!("the body does not match its checksum"),
//!     Some(other) => println!("not validated: {other:?}"),
//!     None => println!("the call or its body has not ended"),
//! }
//! ```

pub mod checksums;

use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::interceptors::context::{
    BeforeSerializationInterceptorContextRef, BeforeTransmitInterceptorContextRef,
    FinalizerInterceptorContextRef,
};
use aws_smithy_runtime_api::client::interceptors::{dyn_dispatch_hint, Intercept};
use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
use aws_smithy_types::config_bag::ConfigBag;
use checksums::ChecksumCapture;

/// Carries handles into one call to a generated client.
///
/// Attach it to a call with `.customize().interceptor(..)`, which consumes it. A handle records
/// one call: attached to a second call, it reads
/// [`NotValidatedReason::Reused`](checksums::NotValidatedReason::Reused) instead.
#[derive(Debug, Default)]
pub struct Capture {
    checksums: Vec<ChecksumCapture>,
}

impl Capture {
    /// Creates a `Capture` that carries no handles.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a handle, which records the call this `Capture` is attached to.
    pub fn with(mut self, handle: &impl AsHandle) -> Self {
        handle.add_to(&mut self);
        self
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        for handle in &self.checksums {
            handle.capture_dropped();
        }
    }
}

#[dyn_dispatch_hint]
impl Intercept for Capture {
    fn name(&self) -> &'static str {
        "Capture"
    }

    fn read_before_execution(
        &self,
        _context: &BeforeSerializationInterceptorContextRef<'_>,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        for handle in &self.checksums {
            handle.bind(cfg);
        }
        Ok(())
    }

    fn read_before_attempt(
        &self,
        _context: &BeforeTransmitInterceptorContextRef<'_>,
        _runtime_components: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        for handle in &self.checksums {
            handle.attempt_started();
        }
        Ok(())
    }

    fn read_after_execution(
        &self,
        context: &FinalizerInterceptorContextRef<'_>,
        _runtime_components: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        let succeeded = matches!(context.output_or_error(), Some(Ok(_)));
        for handle in &self.checksums {
            handle.call_ended(succeeded);
        }
        Ok(())
    }
}

/// A handle that a [`Capture`] can carry into a call.
///
/// This trait is sealed. [`ChecksumCapture`] is the only handle in this release.
pub trait AsHandle: sealed::Sealed {}

mod sealed {
    /// Keeps [`AsHandle`](super::AsHandle) implemented only by this crate's handles.
    pub trait Sealed {
        /// Adds this handle to `capture`.
        fn add_to(&self, capture: &mut super::Capture);
    }
}
