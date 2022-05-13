/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Provides an [`AsyncSleep`] trait that returns a future that sleeps for a given duration,
//! and implementations of `AsyncSleep` for different async runtimes.

use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

/// Async trait with a `sleep` function.
pub trait AsyncSleep: std::fmt::Debug + Send + Sync {
    /// Returns a future that sleeps for the given `duration` of time.
    fn sleep(&self, duration: Duration) -> Sleep;
}

impl<T> AsyncSleep for Box<T>
where
    T: AsyncSleep,
    T: ?Sized,
{
    fn sleep(&self, duration: Duration) -> Sleep {
        T::sleep(self, duration)
    }
}

impl<T> AsyncSleep for Arc<T>
where
    T: AsyncSleep,
    T: ?Sized,
{
    fn sleep(&self, duration: Duration) -> Sleep {
        T::sleep(self, duration)
    }
}

#[cfg(feature = "rt-tokio")]
/// Returns a default sleep implementation based on the features enabled
pub fn default_async_sleep() -> Option<Arc<dyn AsyncSleep>> {
    Some(sleep_tokio())
}

#[cfg(not(feature = "rt-tokio"))]
/// Returns a default sleep implementation based on the features enabled
pub fn default_async_sleep() -> Option<Arc<dyn AsyncSleep>> {
    None
}

/// Future returned by [`AsyncSleep`].
#[non_exhaustive]
pub struct Sleep(Pin<Box<dyn Future<Output = ()> + Send + 'static>>);

impl Debug for Sleep {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sleep")
    }
}

impl Sleep {
    /// Create a new [`Sleep`] future
    ///
    /// The provided future will be Boxed.
    pub fn new(future: impl Future<Output = ()> + Send + 'static) -> Sleep {
        Sleep(Box::pin(future))
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.as_mut().poll(cx)
    }
}

/// Implementation of [`AsyncSleep`] for Tokio.
#[non_exhaustive]
#[cfg(feature = "rt-tokio")]
#[derive(Debug, Default)]
pub struct TokioSleep;

#[cfg(feature = "rt-tokio")]
impl TokioSleep {
    /// Create a new [`AsyncSleep`] implementation using the Tokio hashed wheel sleep implementation
    pub fn new() -> TokioSleep {
        Default::default()
    }
}

#[cfg(feature = "rt-tokio")]
impl AsyncSleep for TokioSleep {
    fn sleep(&self, duration: Duration) -> Sleep {
        Sleep::new(tokio::time::sleep(duration))
    }
}

#[cfg(feature = "rt-tokio")]
fn sleep_tokio() -> Arc<dyn AsyncSleep> {
    Arc::new(TokioSleep::new())
}
