/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Loom-backed synchronization primitives used to run production pool state
//! machines under enumerated thread schedules.

use ::std::ops::{Deref, DerefMut};

pub(crate) use loom::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
pub(crate) use loom::sync::Arc;

loom::thread_local! {
    static HELD_POOL_LOCKS: ::std::cell::Cell<usize> = ::std::cell::Cell::new(0);
}

struct LockDepth;

impl LockDepth {
    fn enter() -> Self {
        HELD_POOL_LOCKS.with(|depth| {
            assert_eq!(
                0,
                depth.get(),
                "pool coordination locks must never be nested"
            );
            depth.set(1);
        });
        Self
    }
}

impl Drop for LockDepth {
    fn drop(&mut self) {
        HELD_POOL_LOCKS.with(|depth| {
            assert_eq!(1, depth.get(), "pool lock-depth tracking became unbalanced");
            depth.set(0);
        });
    }
}

/// Loom substitute for a non-owning registry reference.
///
/// Loom's modeled `Arc` has no weak-reference API. Keeping a strong reference
/// in Loom tests preserves lookup behavior but cannot model target expiry.
/// Ordinary tests cover the expired-target delivery fallback.
#[derive(Debug)]
pub(crate) struct Weak<T>(Arc<T>);

impl<T> Weak<T> {
    /// Creates the modeled registry reference.
    pub(crate) fn from_arc(value: &Arc<T>) -> Self {
        Self(value.clone())
    }

    /// Acquires an owning reference.
    pub(crate) fn upgrade(&self) -> Option<Arc<T>> {
        Some(self.0.clone())
    }
}

impl<T> Clone for Weak<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// Loom-backed mutex.
#[derive(Debug)]
pub(crate) struct Mutex<T>(loom::sync::Mutex<T>);

impl<T> Mutex<T> {
    /// Creates a mutex containing `value`.
    pub(crate) fn new(value: T) -> Self {
        Self(loom::sync::Mutex::new(value))
    }

    /// Locks the mutex.
    pub(crate) fn lock(&self) -> MutexGuard<'_, T> {
        let depth = LockDepth::enter();
        MutexGuard {
            inner: self.0.lock().expect("Loom mutex poisoned"),
            _depth: depth,
        }
    }
}

/// Exclusive access returned by [`Mutex::lock`].
pub(crate) struct MutexGuard<'a, T> {
    inner: loom::sync::MutexGuard<'a, T>,
    _depth: LockDepth,
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Loom-backed reader-writer lock.
#[derive(Debug)]
pub(crate) struct RwLock<T>(loom::sync::RwLock<T>);

impl<T> RwLock<T> {
    /// Creates a reader-writer lock containing `value`.
    pub(crate) fn new(value: T) -> Self {
        Self(loom::sync::RwLock::new(value))
    }

    /// Locks with shared read access.
    pub(crate) fn read(&self) -> RwLockReadGuard<'_, T> {
        let depth = LockDepth::enter();
        RwLockReadGuard {
            inner: self.0.read().expect("Loom RwLock poisoned"),
            _depth: depth,
        }
    }

    /// Locks with exclusive write access.
    pub(crate) fn write(&self) -> RwLockWriteGuard<'_, T> {
        let depth = LockDepth::enter();
        RwLockWriteGuard {
            inner: self.0.write().expect("Loom RwLock poisoned"),
            _depth: depth,
        }
    }
}

/// Shared access returned by [`RwLock::read`].
pub(crate) struct RwLockReadGuard<'a, T> {
    inner: loom::sync::RwLockReadGuard<'a, T>,
    _depth: LockDepth,
}

impl<T> Deref for RwLockReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Exclusive access returned by [`RwLock::write`].
pub(crate) struct RwLockWriteGuard<'a, T> {
    inner: loom::sync::RwLockWriteGuard<'a, T>,
    _depth: LockDepth,
}

impl<T> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for RwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
