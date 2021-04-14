/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::hash::{BuildHasherDefault, Hasher};

type AnyMap = HashMap<TypeId, Box<dyn Any + Send + Sync>, BuildHasherDefault<IdHasher>>;

// With TypeIds as keys, there's no need to hash them. They are already hashes
// themselves, coming from the compiler. The IdHasher just holds the u64 of
// the TypeId, and then returns it, instead of doing any bit fiddling.
#[derive(Default)]
struct IdHasher(u64);

impl Hasher for IdHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _: &[u8]) {
        unreachable!("TypeId calls write_u64");
    }

    #[inline]
    fn write_u64(&mut self, id: u64) {
        self.0 = id;
    }
}

/// A type map of protocol extensions.
///
/// `PropertyBag` can be used by `Request` and `Response` to store
/// extra data derived from the underlying protocol.
///
/// TODO: We should consider if we want to require members of the property to be "resettable" in some
/// way to reset any state prior to a retry. I think this is worth delaying until we need it, but
/// is worth keeping in mind.
#[derive(Default)]
pub struct PropertyBag {
    // In http where this property bag is usually empty, this makes sense. We will almost always put
    // something in the bag, so we could consider removing the layer of indirection.
    // If extensions are never used, no need to carry around an empty HashMap.
    // That's 3 words. Instead, this is only 1 word.
    map: Option<Box<AnyMap>>,
}

impl PropertyBag {
    /// Create an empty `PropertyBag`.
    #[inline]
    pub fn new() -> PropertyBag {
        PropertyBag { map: None }
    }

    /// Insert a type into this `PropertyBag`.
    ///
    /// If a extension of this type already existed, it will
    /// be returned.
    ///
    /// Generally, this method should not be called directly. The best practice is
    /// calling this method via an extension trait on `PropertyBag`.
    ///
    /// # Example
    ///
    /// ```
    /// # use smithy_http::property_bag::PropertyBag;
    /// let mut ext = PropertyBag::new();
    /// #[derive(Debug, Eq, PartialEq)]
    /// struct Endpoint(&'static str);
    /// assert!(ext.insert(Endpoint("dynamo.amazon.com")).is_none());
    /// assert_eq!(ext.insert(Endpoint("kinesis.amazon.com")), Some(Endpoint("dynamo.amazon.com")));
    /// ```
    pub fn insert<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T> {
        self.map
            .get_or_insert_with(|| Box::new(HashMap::default()))
            .insert(TypeId::of::<T>(), Box::new(val))
            .and_then(|boxed| {
                (boxed as Box<dyn Any + 'static>)
                    .downcast()
                    .ok()
                    .map(|boxed| *boxed)
            })
    }

    /// Get a reference to a type previously inserted on this `PropertyBag`.
    ///
    /// # Example
    ///
    /// ```
    /// # use smithy_http::property_bag::PropertyBag;
    /// let mut ext = PropertyBag::new();
    /// assert!(ext.get::<i32>().is_none());
    /// ext.insert(5i32);
    ///
    /// assert_eq!(ext.get::<i32>(), Some(&5i32));
    /// ```
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.map
            .as_ref()
            .and_then(|map| map.get(&TypeId::of::<T>()))
            .and_then(|boxed| (&**boxed as &(dyn Any + 'static)).downcast_ref())
    }

    /// Get a mutable reference to a type previously inserted on this `PropertyBag`.
    ///
    /// # Example
    ///
    /// ```
    /// # use smithy_http::property_bag::PropertyBag;
    /// let mut ext = PropertyBag::new();
    /// ext.insert(String::from("Hello"));
    /// ext.get_mut::<String>().unwrap().push_str(" World");
    ///
    /// assert_eq!(ext.get::<String>().unwrap(), "Hello World");
    /// ```
    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.map
            .as_mut()
            .and_then(|map| map.get_mut(&TypeId::of::<T>()))
            .and_then(|boxed| (&mut **boxed as &mut (dyn Any + 'static)).downcast_mut())
    }

    /// Remove a type from this `PropertyBag`.
    ///
    /// If a extension of this type existed, it will be returned.
    ///
    /// # Example
    ///
    /// ```
    /// # use smithy_http::property_bag::PropertyBag;
    /// let mut ext = PropertyBag::new();
    /// ext.insert(5i32);
    /// assert_eq!(ext.remove::<i32>(), Some(5i32));
    /// assert!(ext.get::<i32>().is_none());
    /// ```
    pub fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.map
            .as_mut()
            .and_then(|map| map.remove(&TypeId::of::<T>()))
            .and_then(|boxed| {
                (boxed as Box<dyn Any + 'static>)
                    .downcast()
                    .ok()
                    .map(|boxed| *boxed)
            })
    }

    /// Clear the `PropertyBag` of all inserted extensions.
    ///
    /// # Example
    ///
    /// ```
    /// # use smithy_http::property_bag::PropertyBag;
    /// let mut ext = PropertyBag::new();
    /// ext.insert(5i32);
    /// ext.clear();
    ///
    /// assert!(ext.get::<i32>().is_none());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        if let Some(ref mut map) = self.map {
            map.clear();
        }
    }
}

impl fmt::Debug for PropertyBag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PropertyBag").finish()
    }
}

#[test]
fn test_extensions() {
    #[derive(Debug, PartialEq)]
    struct MyType(i32);

    let mut extensions = PropertyBag::new();

    extensions.insert(5i32);
    extensions.insert(MyType(10));

    assert_eq!(extensions.get(), Some(&5i32));
    assert_eq!(extensions.get_mut(), Some(&mut 5i32));

    assert_eq!(extensions.remove::<i32>(), Some(5i32));
    assert!(extensions.get::<i32>().is_none());

    assert_eq!(extensions.get::<bool>(), None);
    assert_eq!(extensions.get(), Some(&MyType(10)));
}
