/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use super::{Error, Input, InterceptorContext, Output};
use crate::client::interceptors::context::{Request, Response};
use crate::client::orchestrator::OrchestratorError;
use std::fmt::Debug;

macro_rules! output {
    (&Option<Result<$o_ty:ty, $e_ty:ty>>) => {
        Option<Result<&$o_ty, &$e_ty>>
    };
    (&Option<$ty:ty>) => {
        Option<&$ty>
    };
    (&mut Option<$ty:ty>) => {
        Option<&mut $ty>
    };
    (&Result<$o_ty:ty, $e_ty:ty>) => {
        Result<&$o_ty, &$e_ty>
    };
    (&$($tt:tt)+) => {
        &$($tt)+
    };
    (&mut $($tt:tt)+) => {
        &mut $($tt)+
    };
}

macro_rules! declare_method {
    (&mut $name:ident, $inner_name:ident, $doc:literal, Option<$ty:ty>) => {
        #[doc=$doc]
        pub fn $name(&mut self) -> Option<&mut $ty> {
            self.inner.$inner_name.as_ref()
        }
    };
    (&$name:ident, $inner_name:ident, $doc:literal, Option<$ty:ty>) => {
        #[doc=$doc]
        pub fn $name(&self) -> Option<$ty> {
            self.inner.$inner_name.as_mut()
        }
    };
    (&mut $name:ident, $doc:literal, $($tt:tt)+) => {
        #[doc=$doc]
        pub fn $name(&mut self) -> output!(&mut $($tt)+) {
            self.inner.$name().expect(concat!("`", stringify!($name), "` wasn't set in the underlying interceptor context. This is a bug."))
        }
    };
    (&$name:ident, $doc:literal, $($tt:tt)+) => {
        #[doc=$doc]
        pub fn $name(&self) -> output!(&$($tt)+) {
            self.inner.$name().expect(concat!("`", stringify!($name), "` wasn't set in the underlying interceptor context. This is a bug."))
        }
    };
}

macro_rules! declare_known_method {
    (output_or_error: &mut $($tt:tt)+) => {
        declare_method!(&mut output_or_error_mut, "Returns a mutable reference to the deserialized output or error.", $($tt)+);
    };
    (output_or_error: &$($tt:tt)+) => {
        declare_method!(&output_or_error, "Returns a reference to the deserialized output or error.", $($tt)+);
    };
    (input: &mut $($tt:tt)+) => {
        declare_method!(&mut input_mut, "Returns a mutable reference to the input.", $($tt)+);
    };
    (input: &$($tt:tt)+) => {
        declare_method!(&input, "Returns a reference to the input.", $($tt)+);
    };
    (request: &mut $($tt:tt)+) => {
        declare_method!(&mut request_mut, "Returns a mutable reference to the transmittable request for the operation being invoked.", $($tt)+);
    };
    (request: &$($tt:tt)+) => {
        declare_method!(&request, "Returns a reference to the transmittable request for the operation being invoked.", $($tt)+);
    };
    (response: &mut $($tt:tt)+) => {
        declare_method!(&mut response_mut, "Returns a mutable reference to the response.", $($tt)+);
    };
    (response: &$($tt:tt)+) => {
        declare_method!(&response, "Returns a reference to the response.", $($tt)+);
    };
}

macro_rules! declare_wrapper {
    (($ref_struct_name:ident readonly)$($tt:tt)+) => {
        pub struct $ref_struct_name<'a, I = Input, O = Output, E = Error> {
            inner: &'a InterceptorContext<I, O, E>,
        }

        impl<'a, I, O, E: Debug> From<&'a InterceptorContext<I, O, E>> for $ref_struct_name<'a, I, O, E>
        {
            fn from(inner: &'a InterceptorContext<I, O, E>) -> Self {
                Self { inner }
            }
        }

        impl<'a, I, O, E: Debug> $ref_struct_name<'a, I, O, E> {
            declare_ref_wrapper_methods!($($tt)+);
        }
    };
    (($ref_struct_name:ident $mut_struct_name:ident)$($tt:tt)+) => {
        declare_wrapper!(($ref_struct_name readonly) $($tt)+);

        pub struct $mut_struct_name<'a, I = Input, O = Output, E = Error> {
            inner: &'a mut InterceptorContext<I, O, E>,
        }

        impl<'a, I, O, E: Debug> From<&'a mut InterceptorContext<I, O, E>> for $mut_struct_name<'a, I, O, E>
        {
            fn from(inner: &'a mut InterceptorContext<I, O, E>) -> Self {
                Self { inner }
            }
        }

        impl<'a, I, O, E: Debug> $mut_struct_name<'a, I, O, E> {
            declare_ref_wrapper_methods!($($tt)+);
            declare_mut_wrapper_methods!($($tt)+);
        }
    };
}

macro_rules! declare_ref_wrapper_methods {
    (($field:ident: $($head:tt)+)$($tail:tt)+) => {
        declare_known_method!($field: &$($head)+);
        declare_ref_wrapper_methods!($($tail)+);
    };
    (($field:ident: $($tt:tt)+)) => {
        declare_known_method!($field: &$($tt)+);
    };
}

macro_rules! declare_mut_wrapper_methods {
    (($field:ident: $($head:tt)+)$($tail:tt)+) => {
        declare_known_method!($field: &mut $($head)+);
        declare_mut_wrapper_methods!($($tail)+);
    };
    (($field:ident: $($tt:tt)+)) => {
        declare_known_method!($field: &mut $($tt)+);
    };
}

declare_wrapper!(
    (BeforeSerializationInterceptorContextRef BeforeSerializationInterceptorContextMut)
    (input: I)
);

declare_wrapper!(
    (BeforeTransmitInterceptorContextRef BeforeTransmitInterceptorContextMut)
    (request: Request)
);

declare_wrapper!(
    (BeforeDeserializationInterceptorContextRef BeforeDeserializationInterceptorContextMut)
    (input: I)
    (request: Request)
    (response: Response)
);

impl<'a, I, O, E: Debug> BeforeDeserializationInterceptorContextMut<'a, I, O, E> {
    #[doc(hidden)]
    /// Downgrade this helper struct, returning the underlying InterceptorContext. There's no good
    /// reason to use this unless you're writing tests or you have to interact with an API that
    /// doesn't support the helper structs.
    pub fn into_inner(&mut self) -> &'_ mut InterceptorContext<I, O, E> {
        self.inner
    }
}

declare_wrapper!(
    (AfterDeserializationInterceptorContextRef readonly)
    (input: I)
    (request: Request)
    (response: Response)
    (output_or_error: Result<O, OrchestratorError<E>>
));

// Why are all the rest of these defined with a macro but these last two aren't? I simply ran out of
// time. Consider updating the macros to support these last two if you're looking for a challenge.
// - Zelda

pub struct FinalizerInterceptorContextRef<'a, I = Input, O = Output, E = Error> {
    inner: &'a InterceptorContext<I, O, E>,
}

impl<'a, I, O, E: Debug> From<&'a InterceptorContext<I, O, E>>
    for FinalizerInterceptorContextRef<'a, I, O, E>
{
    fn from(inner: &'a InterceptorContext<I, O, E>) -> Self {
        Self { inner }
    }
}

impl<'a, I, O, E: Debug> FinalizerInterceptorContextRef<'a, I, O, E> {
    pub fn input(&self) -> Option<&I> {
        self.inner.input.as_ref()
    }

    pub fn request(&self) -> Option<&Request> {
        self.inner.request.as_ref()
    }

    pub fn response(&self) -> Option<&Response> {
        self.inner.response.as_ref()
    }

    pub fn output_or_error(&self) -> Option<Result<&O, &OrchestratorError<E>>> {
        self.inner.output_or_error.as_ref().map(|o| o.as_ref())
    }
}

pub struct FinalizerInterceptorContextMut<'a, I = Input, O = Output, E = Error> {
    inner: &'a mut InterceptorContext<I, O, E>,
}

impl<'a, I, O, E: Debug> From<&'a mut InterceptorContext<I, O, E>>
    for FinalizerInterceptorContextMut<'a, I, O, E>
{
    fn from(inner: &'a mut InterceptorContext<I, O, E>) -> Self {
        Self { inner }
    }
}

impl<'a, I, O, E: Debug> FinalizerInterceptorContextMut<'a, I, O, E> {
    pub fn input(&self) -> Option<&I> {
        self.inner.input.as_ref()
    }

    pub fn request(&self) -> Option<&Request> {
        self.inner.request.as_ref()
    }

    pub fn response(&self) -> Option<&Response> {
        self.inner.response.as_ref()
    }

    pub fn output_or_error(&self) -> Option<Result<&O, &OrchestratorError<E>>> {
        self.inner.output_or_error.as_ref().map(|o| o.as_ref())
    }

    pub fn input_mut(&mut self) -> Option<&mut I> {
        self.inner.input.as_mut()
    }

    pub fn request_mut(&mut self) -> Option<&mut Request> {
        self.inner.request.as_mut()
    }

    pub fn response_mut(&mut self) -> Option<&mut Response> {
        self.inner.response.as_mut()
    }

    pub fn output_or_error_mut(&mut self) -> Option<&mut Result<O, OrchestratorError<E>>> {
        self.inner.output_or_error.as_mut()
    }
}
