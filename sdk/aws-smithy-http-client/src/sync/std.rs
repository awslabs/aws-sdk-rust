/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Production synchronization primitives backed by the standard library.

use ::std::ops::{Deref, DerefMut};

pub(crate) use ::std::sync::atomic::{AtomicBool, Ordering};
pub(crate) use ::std::sync::Arc;

/// Non-owning reference used by coordination registries.
#[derive(Debug)]
pub(crate) struct Weak<T>(::std::sync::Weak<T>);

impl<T> Weak<T> {
    /// Creates a non-owning reference to `value`.
    pub(crate) fn from_arc(value: &Arc<T>) -> Self {
        Self(Arc::downgrade(value))
    }

    /// Attempts to acquire an owning reference.
    pub(crate) fn upgrade(&self) -> Option<Arc<T>> {
        self.0.upgrade()
    }
}

impl<T> Clone for Weak<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[cfg(any(debug_assertions, test))]
::std::thread_local! {
    static HELD_POOL_LOCKS: ::std::cell::Cell<usize> = const { ::std::cell::Cell::new(0) };
}

#[cfg(any(debug_assertions, test))]
struct LockDepth;

#[cfg(any(debug_assertions, test))]
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

#[cfg(any(debug_assertions, test))]
impl Drop for LockDepth {
    fn drop(&mut self) {
        HELD_POOL_LOCKS.with(|depth| {
            assert_eq!(1, depth.get(), "pool lock-depth tracking became unbalanced");
            depth.set(0);
        });
    }
}

/// A mutex that retains access to protected state after poisoning.
#[derive(Debug)]
pub(crate) struct Mutex<T>(::std::sync::Mutex<T>);

impl<T> Mutex<T> {
    /// Creates a mutex containing `value`.
    pub(crate) fn new(value: T) -> Self {
        Self(::std::sync::Mutex::new(value))
    }

    /// Locks the mutex.
    pub(crate) fn lock(&self) -> MutexGuard<'_, T> {
        #[cfg(any(debug_assertions, test))]
        let depth = LockDepth::enter();
        let inner = self
            .0
            .lock()
            .unwrap_or_else(::std::sync::PoisonError::into_inner);
        MutexGuard {
            inner,
            #[cfg(any(debug_assertions, test))]
            _depth: depth,
        }
    }
}

/// Exclusive access returned by [`Mutex::lock`].
pub(crate) struct MutexGuard<'a, T> {
    inner: ::std::sync::MutexGuard<'a, T>,
    #[cfg(any(debug_assertions, test))]
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

/// A reader-writer lock that retains access to protected state after poisoning.
#[derive(Debug)]
pub(crate) struct RwLock<T>(::std::sync::RwLock<T>);

impl<T> RwLock<T> {
    /// Creates a reader-writer lock containing `value`.
    pub(crate) fn new(value: T) -> Self {
        Self(::std::sync::RwLock::new(value))
    }

    /// Locks with shared read access.
    pub(crate) fn read(&self) -> RwLockReadGuard<'_, T> {
        #[cfg(any(debug_assertions, test))]
        let depth = LockDepth::enter();
        let inner = self
            .0
            .read()
            .unwrap_or_else(::std::sync::PoisonError::into_inner);
        RwLockReadGuard {
            inner,
            #[cfg(any(debug_assertions, test))]
            _depth: depth,
        }
    }

    /// Locks with exclusive write access.
    pub(crate) fn write(&self) -> RwLockWriteGuard<'_, T> {
        #[cfg(any(debug_assertions, test))]
        let depth = LockDepth::enter();
        let inner = self
            .0
            .write()
            .unwrap_or_else(::std::sync::PoisonError::into_inner);
        RwLockWriteGuard {
            inner,
            #[cfg(any(debug_assertions, test))]
            _depth: depth,
        }
    }
}

/// Shared access returned by [`RwLock::read`].
pub(crate) struct RwLockReadGuard<'a, T> {
    inner: ::std::sync::RwLockReadGuard<'a, T>,
    #[cfg(any(debug_assertions, test))]
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
    inner: ::std::sync::RwLockWriteGuard<'a, T>,
    #[cfg(any(debug_assertions, test))]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "pool coordination locks must never be nested")]
    fn nested_pool_locks_are_rejected() {
        let first = Mutex::new(());
        let second = Mutex::new(());
        let _first = first.lock();
        let _second = second.lock();
    }

    #[test]
    fn mutex_retains_state_after_poisoning() {
        let mutex = Arc::new(Mutex::new(7));
        let panicking = mutex.clone();
        let thread = ::std::thread::spawn(move || {
            let _guard = panicking.lock();
            panic!("poison the mutex");
        });

        assert!(thread.join().is_err());
        assert_eq!(7, *mutex.lock());
    }

    #[test]
    fn rw_lock_retains_state_after_poisoning() {
        let lock = Arc::new(RwLock::new(7));
        let panicking = lock.clone();
        let thread = ::std::thread::spawn(move || {
            let mut guard = panicking.write();
            *guard = 8;
            panic!("poison the reader-writer lock");
        });

        assert!(thread.join().is_err());
        assert_eq!(8, *lock.read());
        *lock.write() = 9;
        assert_eq!(9, *lock.read());
    }
}
