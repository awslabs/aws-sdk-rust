/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::any::Any;
use std::error::Error as StdError;
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// A [`TypeErasedBox`] with type information tracked via generics at compile-time
///
/// `TypedBox` is used to transition to/from a `TypeErasedBox`. A `TypedBox<T>` can only
/// be created from a `T` or from a `TypeErasedBox` value that _is a_ `T`. Therefore, it can
/// be assumed to be a `T` even though the underlying storage is still a `TypeErasedBox`.
/// Since the `T` is only used in `PhantomData`, it gets compiled down to just a `TypeErasedBox`.
///
/// The orchestrator uses `TypeErasedBox` to avoid the complication of six or more generic parameters
/// and to avoid the monomorphization that brings with it. This `TypedBox` will primarily be useful
/// for operation-specific or service-specific interceptors that need to operate on the actual
/// input/output/error types.
pub struct TypedBox<T> {
    inner: TypeErasedBox,
    _phantom: PhantomData<T>,
}

impl<T> TypedBox<T>
where
    T: fmt::Debug + Send + Sync + 'static,
{
    // Creates a new `TypedBox`.
    pub fn new(inner: T) -> Self {
        Self {
            inner: TypeErasedBox::new(inner),
            _phantom: Default::default(),
        }
    }

    // Tries to create a `TypedBox<T>` from a `TypeErasedBox`.
    //
    // If the `TypedBox<T>` can't be created due to the `TypeErasedBox`'s value consisting
    // of another type, then the original `TypeErasedBox` will be returned in the `Err` variant.
    pub fn assume_from(type_erased: TypeErasedBox) -> Result<TypedBox<T>, TypeErasedBox> {
        if type_erased.downcast_ref::<T>().is_some() {
            Ok(TypedBox {
                inner: type_erased,
                _phantom: Default::default(),
            })
        } else {
            Err(type_erased)
        }
    }

    /// Converts the `TypedBox<T>` back into `T`.
    pub fn unwrap(self) -> T {
        *self.inner.downcast::<T>().expect("type checked")
    }

    /// Converts the `TypedBox<T>` into a `TypeErasedBox`.
    pub fn erase(self) -> TypeErasedBox {
        self.inner
    }
}

impl<T> TypedBox<T>
where
    T: StdError + fmt::Debug + Send + Sync + 'static,
{
    /// Converts `TypedBox<T>` to a `TypeErasedError` where `T` implements `Error`.
    pub fn erase_error(self) -> TypeErasedError {
        let inner = self.inner.downcast::<T>().expect("typechecked");
        TypeErasedError::new(inner)
    }
}

impl<T> fmt::Debug for TypedBox<T>
where
    T: Send + Sync + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TypedBox:")?;
        (self.inner.debug)(&self.inner.field, f)
    }
}

impl<T: fmt::Debug + Send + Sync + 'static> Deref for TypedBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.downcast_ref().expect("type checked")
    }
}

impl<T: fmt::Debug + Send + Sync + 'static> DerefMut for TypedBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.downcast_mut().expect("type checked")
    }
}

/// A new-type around `Box<dyn Debug + Send + Sync>`
pub struct TypeErasedBox {
    field: Box<dyn Any + Send + Sync>,
    #[allow(clippy::type_complexity)]
    debug: Box<
        dyn Fn(&Box<dyn Any + Send + Sync>, &mut fmt::Formatter<'_>) -> fmt::Result + Send + Sync,
    >,
}

impl fmt::Debug for TypeErasedBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TypeErasedBox:")?;
        (self.debug)(&self.field, f)
    }
}

impl TypeErasedBox {
    pub fn new<T: Send + Sync + fmt::Debug + 'static>(value: T) -> Self {
        let debug = |value: &Box<dyn Any + Send + Sync>, f: &mut fmt::Formatter<'_>| {
            fmt::Debug::fmt(value.downcast_ref::<T>().expect("typechecked"), f)
        };
        Self {
            field: Box::new(value),
            debug: Box::new(debug),
        }
    }

    // Downcast into a `Box<T>`, or return `Self` if it is not a `T`.
    pub fn downcast<T: fmt::Debug + Send + Sync + 'static>(self) -> Result<Box<T>, Self> {
        let TypeErasedBox { field, debug } = self;
        field.downcast().map_err(|field| Self { field, debug })
    }

    /// Downcast as a `&T`, or return `None` if it is not a `T`.
    pub fn downcast_ref<T: fmt::Debug + Send + Sync + 'static>(&self) -> Option<&T> {
        self.field.downcast_ref()
    }

    /// Downcast as a `&mut T`, or return `None` if it is not a `T`.
    pub fn downcast_mut<T: fmt::Debug + Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.field.downcast_mut()
    }
}

impl From<TypeErasedError> for TypeErasedBox {
    fn from(value: TypeErasedError) -> Self {
        TypeErasedBox {
            field: value.field,
            debug: value.debug,
        }
    }
}

/// A new-type around `Box<dyn Error + Debug + Send + Sync>` that also implements `Error`
pub struct TypeErasedError {
    field: Box<dyn Any + Send + Sync>,
    #[allow(clippy::type_complexity)]
    debug: Box<
        dyn Fn(&Box<dyn Any + Send + Sync>, &mut fmt::Formatter<'_>) -> fmt::Result + Send + Sync,
    >,
    #[allow(clippy::type_complexity)]
    as_error: Box<dyn for<'a> Fn(&'a TypeErasedError) -> &'a (dyn StdError) + Send + Sync>,
}

impl fmt::Debug for TypeErasedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TypeErasedError:")?;
        (self.debug)(&self.field, f)
    }
}

impl fmt::Display for TypeErasedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt((self.as_error)(self), f)
    }
}

impl StdError for TypeErasedError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        (self.as_error)(self).source()
    }
}

impl TypeErasedError {
    pub fn new<T: StdError + Send + Sync + fmt::Debug + 'static>(value: T) -> Self {
        let debug = |value: &Box<dyn Any + Send + Sync>, f: &mut fmt::Formatter<'_>| {
            fmt::Debug::fmt(value.downcast_ref::<T>().expect("typechecked"), f)
        };
        Self {
            field: Box::new(value),
            debug: Box::new(debug),
            as_error: Box::new(|value: &TypeErasedError| {
                value.downcast_ref::<T>().expect("typechecked") as _
            }),
        }
    }

    // Downcast into a `Box<T>`, or return `Self` if it is not a `T`.
    pub fn downcast<T: StdError + fmt::Debug + Send + Sync + 'static>(
        self,
    ) -> Result<Box<T>, Self> {
        let TypeErasedError {
            field,
            debug,
            as_error,
        } = self;
        field.downcast().map_err(|field| Self {
            field,
            debug,
            as_error,
        })
    }

    /// Downcast as a `&T`, or return `None` if it is not a `T`.
    pub fn downcast_ref<T: StdError + fmt::Debug + Send + Sync + 'static>(&self) -> Option<&T> {
        self.field.downcast_ref()
    }

    /// Downcast as a `&mut T`, or return `None` if it is not a `T`.
    pub fn downcast_mut<T: StdError + fmt::Debug + Send + Sync + 'static>(
        &mut self,
    ) -> Option<&mut T> {
        self.field.downcast_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::{TypeErasedError, TypedBox};
    use std::fmt;

    #[derive(Debug)]
    struct Foo(&'static str);
    #[derive(Debug)]
    struct Bar(isize);

    #[test]
    fn test_typed_boxes() {
        let foo = TypedBox::new(Foo("1"));
        let bar = TypedBox::new(Bar(2));

        assert_eq!("TypedBox:Foo(\"1\")", format!("{foo:?}"));
        assert_eq!("TypedBox:Bar(2)", format!("{bar:?}"));

        let mut foo_erased = foo.erase();
        foo_erased
            .downcast_mut::<Foo>()
            .expect("I know its a Foo")
            .0 = "3";

        let bar_erased = bar.erase();
        assert_eq!("TypeErasedBox:Foo(\"3\")", format!("{foo_erased:?}"));
        assert_eq!("TypeErasedBox:Bar(2)", format!("{bar_erased:?}"));

        let bar_erased = TypedBox::<Foo>::assume_from(bar_erased).expect_err("it's not a Foo");
        let mut bar = TypedBox::<Bar>::assume_from(bar_erased).expect("it's a Bar");
        assert_eq!(2, bar.0);
        bar.0 += 1;

        let bar = bar.unwrap();
        assert_eq!(3, bar.0);

        assert!(foo_erased.downcast_ref::<Bar>().is_none());
        assert!(foo_erased.downcast_mut::<Bar>().is_none());
        let mut foo_erased = foo_erased.downcast::<Bar>().expect_err("it's not a Bar");

        assert_eq!("3", foo_erased.downcast_ref::<Foo>().expect("it's a Foo").0);
        foo_erased.downcast_mut::<Foo>().expect("it's a Foo").0 = "4";
        let foo = *foo_erased.downcast::<Foo>().expect("it's a Foo");
        assert_eq!("4", foo.0);
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestErr {
        inner: &'static str,
    }

    impl TestErr {
        fn new(inner: &'static str) -> Self {
            Self { inner }
        }
    }

    impl fmt::Display for TestErr {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Error: {}", self.inner)
        }
    }

    impl std::error::Error for TestErr {}

    #[test]
    fn test_typed_erased_errors_can_be_downcast() {
        let test_err = TestErr::new("something failed!");
        let type_erased_test_err = TypeErasedError::new(test_err.clone());
        let actual = type_erased_test_err
            .downcast::<TestErr>()
            .expect("type erased error can be downcast into original type");
        assert_eq!(test_err, *actual);
    }
}
