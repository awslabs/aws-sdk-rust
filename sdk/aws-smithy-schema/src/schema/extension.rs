/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Schema extensions: derived data that runtime components compute once per
//! schema and cache on the schema itself.
//!
//! A codec, protocol, or any other runtime component declares a
//! [`SchemaExtensionKey`] as a `static`, pairing the type of the data with the
//! function that derives it from a schema. [`Schema::extension`] runs that
//! function the first time a given schema is asked for that key and returns the
//! cached value on every later call.
//!
//! ```
//! use aws_smithy_schema::extension::SchemaExtensionKey;
//! use aws_smithy_schema::{shape_id, Schema, ShapeType};
//!
//! /// A pre-encoded `"name":` JSON object key.
//! struct FieldKey(Box<str>);
//!
//! static FIELD_KEY: SchemaExtensionKey<FieldKey> = SchemaExtensionKey::new(|schema| {
//!     FieldKey(format!("\"{}\":", schema.member_name().unwrap_or_default()).into())
//! });
//!
//! static MEMBER: Schema<'static> =
//!     Schema::new_member(shape_id!("ns", "S", "name"), ShapeType::String, "name", 0);
//!
//! assert_eq!(&*MEMBER.extension(&FIELD_KEY).0, "\"name\":");
//! // The second lookup returns the same cached value.
//! assert!(std::ptr::eq(MEMBER.extension(&FIELD_KEY), MEMBER.extension(&FIELD_KEY)));
//! ```
//!
//! # Every schema has storage
//!
//! The storage is a field of [`Schema`], so a schema emitted by any version of the
//! code generator, a hand-written schema, and one materialized from a model at
//! runtime all cache identically. No component needs a fallback path for schemas
//! that lack storage.
//!
//! A schema declared as a `const` rather than a `static` is copied at each use, so
//! values cached on it are discarded with each copy. Declare schemas as `static`s.
//!
//! Nothing is allocated until a schema is first asked for an extension. That
//! allocates room for eight values, and each cached value is boxed. The cache is
//! freed when the schema is dropped, so a schema built from a model at runtime
//! does not leak its cached values.
//!
//! # Providers
//!
//! A provider must derive its value from the schema it is given and the schemas
//! reachable from it, through ordinary accessors. It must not request extensions:
//! a recursive shape could lead back to the schema being initialized, which would
//! otherwise wait on itself forever. A provider that requests an extension that is
//! not yet cached panics instead, for any key and any schema. Ask for a member's
//! extension at the point it is needed, outside the provider.
//!
//! Values must not depend on per-instance codec settings, because a schema is shared
//! by every codec instance in the process. Cache every variant a setting can select
//! (for example, both the `@jsonName` and the member name) and choose at the use
//! site.
//!
//! If a provider panics, the panic propagates to the caller and nothing is cached;
//! the next request for that key runs the provider again.
//!
//! Values are `'static`, so they cannot borrow from the schema. Store owned data,
//! or member indices rather than member schema references.

use crate::Schema;
use std::any::Any;
use std::cell::Cell;
use std::fmt;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

/// Identifies one kind of derived data cached on schemas, and computes it.
///
/// Declare keys as `static`s. Each key is assigned a process-unique slot the first
/// time it is used, and slots are never reused.
///
/// [`Schema::extension`] takes a `&'static` key, so a `const` key, which would be a
/// new key with a new slot at every use, is rejected at compile time:
///
/// ```compile_fail
/// use aws_smithy_schema::extension::SchemaExtensionKey;
/// use aws_smithy_schema::prelude;
///
/// const KEY: SchemaExtensionKey<u8> = SchemaExtensionKey::new(|_| 0);
/// let _ = prelude::STRING.extension(&KEY);
/// ```
pub struct SchemaExtensionKey<T> {
    /// `0` until assigned; afterwards the slot index plus one.
    slot: AtomicUsize,
    provider: fn(&Schema<'_>) -> T,
    _value: PhantomData<fn() -> T>,
}

impl<T> SchemaExtensionKey<T> {
    /// Creates a key whose values are derived by `provider`.
    pub const fn new(provider: fn(&Schema<'_>) -> T) -> Self {
        Self {
            slot: AtomicUsize::new(0),
            provider,
            _value: PhantomData,
        }
    }

    #[inline]
    fn slot(&self) -> usize {
        match self.slot.load(Ordering::Relaxed) {
            0 => self.assign_slot(),
            encoded => encoded - 1,
        }
    }

    #[cold]
    fn assign_slot(&self) -> usize {
        // The slot number is the only shared state, and the value cells it indexes
        // synchronize themselves, so relaxed ordering is enough. A thread that loses
        // the race discards its candidate; slots are plentiful, so the gap is harmless.
        static NEXT_SLOT: AtomicUsize = AtomicUsize::new(0);
        let candidate = NEXT_SLOT.fetch_add(1, Ordering::Relaxed) + 1;
        match self
            .slot
            .compare_exchange(0, candidate, Ordering::Relaxed, Ordering::Relaxed)
        {
            Ok(_) => candidate - 1,
            Err(existing) => existing - 1,
        }
    }
}

impl<T> fmt::Debug for SchemaExtensionKey<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SchemaExtensionKey")
            .field("value_type", &std::any::type_name::<T>())
            .finish_non_exhaustive()
    }
}

/// Number of value cells per chunk.
///
/// Slots are assigned per process in order of first use, so a key that lands past the
/// first chunk costs every schema that reads it a second chunk and an extra dependent
/// load on each lookup. The first chunk should therefore hold every key the runtime
/// crates declare: today the `@httpHeader` name in this crate, the JSON member keys,
/// the XML list and map names, and the awsQuery collection names, which a process
/// using restXml, a JSON protocol and awsQuery clients together uses all at once.
/// Raise this when adding keys; the cost is paid only by schemas that read an extension.
const CHUNK_LEN: usize = 8;

type ExtensionCell = OnceLock<Box<dyn Any + Send + Sync>>;

struct Chunk {
    cells: [ExtensionCell; CHUNK_LEN],
    next: OnceLock<Box<Chunk>>,
}

impl Chunk {
    fn new() -> Box<Self> {
        Box::new(Self {
            cells: [const { OnceLock::new() }; CHUNK_LEN],
            next: OnceLock::new(),
        })
    }
}

/// Per-schema extension storage. Empty until the schema is first asked for an
/// extension, so untouched schemas cost one word plus the cell's state.
pub(crate) struct ExtensionStorage(OnceLock<Box<Chunk>>);

impl ExtensionStorage {
    pub(crate) const fn new() -> Self {
        Self(OnceLock::new())
    }

    #[inline]
    pub(crate) fn get_or_compute<'s, T: Send + Sync + 'static>(
        &'s self,
        schema: &Schema<'_>,
        key: &SchemaExtensionKey<T>,
    ) -> &'s T {
        let mut slot = key.slot();
        let mut chunk: &Chunk = self.0.get_or_init(Chunk::new);
        while slot >= CHUNK_LEN {
            chunk = chunk.next.get_or_init(Chunk::new);
            slot -= CHUNK_LEN;
        }
        let cell = &chunk.cells[slot];
        let value: &(dyn Any + Send + Sync) = match cell.get() {
            // Deref the box: `Box<dyn Any>` is itself `Any`, so downcasting the box
            // would test the box's type rather than the value's.
            Some(value) => &**value,
            None => initialize(cell, schema, key),
        };
        value
            .downcast_ref::<T>()
            .expect("an extension slot is only ever filled by the key that owns it")
    }
}

thread_local! {
    /// Whether this thread is running an extension provider.
    static IN_PROVIDER: Cell<bool> = const { Cell::new(false) };
}

/// Clears [`IN_PROVIDER`] when the provider returns or panics.
struct ProviderGuard;

impl Drop for ProviderGuard {
    fn drop(&mut self) {
        IN_PROVIDER.with(|in_provider| in_provider.set(false));
    }
}

/// Computes and caches the value of an empty cell.
///
/// A provider that requests an uncached extension panics here, before it waits on any
/// cell. Waiting would deadlock if the request led back, through a recursive schema or
/// another thread's provider, to a cell whose initialization is waiting on this one.
/// Callers outside a provider still wait for a concurrent initialization, so each value
/// is computed once.
#[cold]
#[inline(never)]
fn initialize<'c, T: Send + Sync + 'static>(
    cell: &'c ExtensionCell,
    schema: &Schema<'_>,
    key: &SchemaExtensionKey<T>,
) -> &'c (dyn Any + Send + Sync) {
    if IN_PROVIDER.with(Cell::get) {
        panic!(
            "a schema extension provider requested the uncached extension `{}` for schema `{}`; \
             providers must derive their value from the schema without requesting extensions",
            std::any::type_name::<T>(),
            schema.shape_id().as_str(),
        );
    }
    &**cell.get_or_init(|| {
        IN_PROVIDER.with(|in_provider| in_provider.set(true));
        let _guard = ProviderGuard;
        Box::new((key.provider)(schema))
    })
}

impl fmt::Debug for ExtensionStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Cached values are derived data, so they are left out of `Schema`'s output.
        f.write_str("..")
    }
}

#[cfg(test)]
mod tests {
    use super::SchemaExtensionKey;
    use crate::{shape_id, Schema, ShapeType};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static CALLS: AtomicUsize = AtomicUsize::new(0);

    static LEN: SchemaExtensionKey<usize> = SchemaExtensionKey::new(|schema| {
        CALLS.fetch_add(1, Ordering::SeqCst);
        schema.member_name().map_or(0, str::len)
    });

    static MEMBER: Schema<'static> =
        Schema::new_member(shape_id!("ns", "S", "abc"), ShapeType::String, "abc", 0);

    #[test]
    fn computes_once_per_schema() {
        let before = CALLS.load(Ordering::SeqCst);
        assert_eq!(*MEMBER.extension(&LEN), 3);
        assert_eq!(*MEMBER.extension(&LEN), 3);
        assert_eq!(CALLS.load(Ordering::SeqCst) - before, 1);
    }

    #[test]
    fn works_on_runtime_schemas() {
        let name = String::from("runtime");
        let id = crate::ShapeId::from_parts("ns#S$runtime", "ns", "S");
        let schema = Schema::new_member(id, ShapeType::String, &name, 0);
        assert_eq!(*schema.extension(&LEN), 7);
    }

    #[test]
    fn values_are_dropped_with_their_schema() {
        static DROPS: AtomicUsize = AtomicUsize::new(0);
        struct Tracked;
        impl Drop for Tracked {
            fn drop(&mut self) {
                DROPS.fetch_add(1, Ordering::SeqCst);
            }
        }
        static KEY: SchemaExtensionKey<Tracked> = SchemaExtensionKey::new(|_| Tracked);

        let schema = Schema::new(shape_id!("ns", "Dropped"), ShapeType::String);
        let _ = schema.extension(&KEY);
        assert_eq!(DROPS.load(Ordering::SeqCst), 0);
        drop(schema);
        assert_eq!(DROPS.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn a_panicking_provider_caches_nothing() {
        static ATTEMPTS: AtomicUsize = AtomicUsize::new(0);
        static KEY: SchemaExtensionKey<usize> = SchemaExtensionKey::new(|_| {
            if ATTEMPTS.fetch_add(1, Ordering::SeqCst) == 0 {
                panic!("first attempt fails");
            }
            7
        });
        static S: Schema<'static> = Schema::new(shape_id!("ns", "Panics"), ShapeType::String);

        let first = std::panic::catch_unwind(|| *S.extension(&KEY));
        assert!(first.is_err());
        assert_eq!(*S.extension(&KEY), 7);
        assert_eq!(*S.extension(&KEY), 7);
        assert_eq!(ATTEMPTS.load(Ordering::SeqCst), 2);
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn chunk_size() {
        // Eight 24-byte cells and the 16-byte link: the heap cost of the first extension
        // a schema reads.
        assert_eq!(std::mem::size_of::<super::Chunk>(), 208);
    }

    #[test]
    fn many_keys_spill_into_later_chunks() {
        static KEYS: [SchemaExtensionKey<usize>; 9] = [
            SchemaExtensionKey::new(|_| 0),
            SchemaExtensionKey::new(|_| 1),
            SchemaExtensionKey::new(|_| 2),
            SchemaExtensionKey::new(|_| 3),
            SchemaExtensionKey::new(|_| 4),
            SchemaExtensionKey::new(|_| 5),
            SchemaExtensionKey::new(|_| 6),
            SchemaExtensionKey::new(|_| 7),
            SchemaExtensionKey::new(|_| 8),
        ];
        static S: Schema<'static> = Schema::new(shape_id!("ns", "T"), ShapeType::String);
        for (i, key) in KEYS.iter().enumerate() {
            assert_eq!(*S.extension(key), i);
        }
        for (i, key) in KEYS.iter().enumerate().rev() {
            assert_eq!(*S.extension(key), i);
        }
    }

    #[test]
    fn concurrent_first_access_computes_once() {
        static N: AtomicUsize = AtomicUsize::new(0);
        static KEY: SchemaExtensionKey<u64> = SchemaExtensionKey::new(|_| {
            N.fetch_add(1, Ordering::SeqCst);
            42
        });
        static S: Schema<'static> = Schema::new(shape_id!("ns", "U"), ShapeType::String);
        std::thread::scope(|scope| {
            for _ in 0..8 {
                scope.spawn(|| assert_eq!(*S.extension(&KEY), 42));
            }
        });
        assert_eq!(N.load(Ordering::SeqCst), 1);
    }

    /// Runs `f` and returns its panic message, failing if it did not panic.
    fn panic_message(f: impl FnOnce() + std::panic::UnwindSafe) -> String {
        let payload = std::panic::catch_unwind(f).expect_err("expected a panic");
        payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default()
    }

    #[test]
    fn a_provider_requesting_its_own_key_panics() {
        static KEY: SchemaExtensionKey<u8> =
            SchemaExtensionKey::new(|schema| *schema.extension(&KEY));
        static S: Schema<'static> = Schema::new(shape_id!("ns", "SelfRef"), ShapeType::String);

        let message = panic_message(|| {
            let _ = S.extension(&KEY);
        });
        assert!(message.contains("uncached extension"), "{message}");
        assert!(message.contains("ns#SelfRef"), "{message}");
        // Nothing is cached and the guard is cleared, so the next attempt fails the same way.
        let message = panic_message(|| {
            let _ = S.extension(&KEY);
        });
        assert!(message.contains("uncached extension"), "{message}");
    }

    /// The example from review: DynamoDB `AttributeValue` refers to itself through a list
    /// and a map, and the provider recurses into members.
    #[test]
    fn a_recursive_provider_over_a_recursive_schema_panics() {
        use crate::prelude;

        static AV_S: Schema<'static> = Schema::new_member(
            shape_id!("com.amazonaws.dynamodb", "AttributeValue", "S"),
            ShapeType::String,
            "S",
            0,
        );
        static AV_L: Schema<'static> = Schema::new_member(
            shape_id!("com.amazonaws.dynamodb", "AttributeValue", "L"),
            ShapeType::List,
            "L",
            1,
        )
        .with_list_member(&ATTRIBUTE_VALUE);
        static AV_M: Schema<'static> = Schema::new_member(
            shape_id!("com.amazonaws.dynamodb", "AttributeValue", "M"),
            ShapeType::Map,
            "M",
            2,
        )
        .with_map_members(&prelude::STRING, &ATTRIBUTE_VALUE);
        static ATTRIBUTE_VALUE: Schema<'static> = Schema::new_struct(
            shape_id!("com.amazonaws.dynamodb", "AttributeValue"),
            ShapeType::Union,
            &[&AV_S, &AV_L, &AV_M],
        );
        static CONTAINS_SENSITIVE: SchemaExtensionKey<bool> = SchemaExtensionKey::new(|schema| {
            schema.sensitive().is_some()
                || schema
                    .members()
                    .iter()
                    .any(|m| *m.extension(&CONTAINS_SENSITIVE))
                || schema
                    .member()
                    .is_some_and(|t| *t.extension(&CONTAINS_SENSITIVE))
                || schema
                    .key()
                    .is_some_and(|t| *t.extension(&CONTAINS_SENSITIVE))
        });

        let message = panic_message(|| {
            let _ = ATTRIBUTE_VALUE.extension(&CONTAINS_SENSITIVE);
        });
        assert!(message.contains("uncached extension"), "{message}");
    }

    #[test]
    fn a_cycle_through_two_keys_panics() {
        static A: SchemaExtensionKey<u8> = SchemaExtensionKey::new(|schema| *schema.extension(&B));
        static B: SchemaExtensionKey<u8> = SchemaExtensionKey::new(|schema| *schema.extension(&A));
        static S: Schema<'static> = Schema::new(shape_id!("ns", "TwoKeys"), ShapeType::String);

        let message = panic_message(|| {
            let _ = S.extension(&A);
        });
        assert!(message.contains("uncached extension"), "{message}");
    }

    /// Two threads each initialize one cell whose provider requests the other's cell.
    /// Waiting would deadlock; both providers must panic instead.
    #[test]
    fn a_cycle_across_threads_panics_instead_of_deadlocking() {
        use std::sync::Barrier;

        static BOTH_IN_PROVIDERS: Barrier = Barrier::new(2);
        static S1: Schema<'static> = Schema::new(shape_id!("ns", "Thread1"), ShapeType::String);
        static S2: Schema<'static> = Schema::new(shape_id!("ns", "Thread2"), ShapeType::String);
        static A: SchemaExtensionKey<u8> = SchemaExtensionKey::new(|_| {
            BOTH_IN_PROVIDERS.wait();
            *S2.extension(&B)
        });
        static B: SchemaExtensionKey<u8> = SchemaExtensionKey::new(|_| {
            BOTH_IN_PROVIDERS.wait();
            *S1.extension(&A)
        });

        std::thread::scope(|scope| {
            let first = scope.spawn(|| *S1.extension(&A));
            let second = scope.spawn(|| *S2.extension(&B));
            assert!(first.join().is_err());
            assert!(second.join().is_err());
        });
    }

    /// The guard is checked only when a cell is initialized, keeping it off the cached
    /// lookup path. This is not a supported provider pattern: whether the value is cached
    /// depends on access order.
    #[test]
    fn a_provider_may_read_an_already_cached_extension() {
        static BASE: SchemaExtensionKey<u8> = SchemaExtensionKey::new(|_| 2);
        static DERIVED: SchemaExtensionKey<u8> =
            SchemaExtensionKey::new(|schema| *schema.extension(&BASE) * 3);
        static S: Schema<'static> = Schema::new(shape_id!("ns", "Cached"), ShapeType::String);

        assert_eq!(*S.extension(&BASE), 2);
        assert_eq!(*S.extension(&DERIVED), 6);
    }

    #[test]
    fn a_panicking_provider_clears_the_guard() {
        static FAILS: SchemaExtensionKey<u8> =
            SchemaExtensionKey::new(|_| panic!("provider failed"));
        static OK: SchemaExtensionKey<u8> = SchemaExtensionKey::new(|_| 5);
        static S: Schema<'static> = Schema::new(shape_id!("ns", "Cleared"), ShapeType::String);

        assert_eq!(
            panic_message(|| {
                let _ = S.extension(&FAILS);
            }),
            "provider failed"
        );
        // Not treated as nested: this thread is no longer inside a provider.
        assert_eq!(*S.extension(&OK), 5);
    }
}
