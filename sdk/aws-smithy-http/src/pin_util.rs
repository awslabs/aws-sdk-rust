/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/// Pins a value on the stack.
///
/// # Examples
///
/// ```rust
/// # use core::pin::Pin;
/// # struct Foo {}
/// # use aws_smithy_http::pin_mut;
/// let foo = Foo { /* ... */ };
/// pin_mut!(foo);
/// let _: Pin<&mut Foo> = foo;
/// ```
#[macro_export]
macro_rules! pin_mut {
    ($($x:ident),* $(,)?) => { $(
        // Move the value to ensure that it is owned
        let mut $x = $x;
        // Shadow the original binding so that it can't be directly accessed
        // ever again.
        #[allow(unused_mut)]
        let mut $x = unsafe {
            core::pin::Pin::new_unchecked(&mut $x)
        };
    )* }
}
