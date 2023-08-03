/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Various utility macros to aid runtime crate writers.

/// Define a new builder struct, along with a method to create it, and setters.
///
/// ## Examples
///
/// The builder macro takes a list of field definitions, each with four elements:
/// ```txt
/// set_optional_field, optional_field,   String,     "An optional field which may or may not be set when `.build()` is called.",
/// ^ The setter name,  ^ The field name, ^ The type, ^ The documentation for the field and setter methods.
/// ```
///
/// The following example creates a new builder struct, along with a method to create it, and setters
/// for a struct `MyConfig` with three fields:
///
/// ```
/// use std::collections::HashMap;
/// use std::sync::{Arc, Mutex};
/// use aws_smithy_runtime_api::{builder, builder_methods, builder_struct};
///
/// struct MyConfig {
///     optional_field: Option<String>,
///     optional_field_with_a_default: f64,
///     required_field_with_no_default: Arc<Mutex<HashMap<String, String>>>,
/// }
///
/// impl MyConfig {
///     pub fn builder() -> Builder {
///         Builder::new()
///     }
/// }
///
/// builder!(
///     set_optional_field, optional_field, String, "An optional field which may or may not be set when `.build()` is called.",
///     set_optional_field_with_a_default, optional_field_with_a_default, f64, "An optional field that will default to `f64::MAX` if it's unset when `.build()` is called.",
///     set_required_field_with_no_default, required_field_with_no_default, HashMap<String, String>, "A required field that will cause the builder to panic if it's unset when `.build()` is called."
/// );
///
/// impl Builder {
///     fn build(self) -> MyConfig {
///         MyConfig {
///             optional_field: self.optional_field,
///             optional_field_with_a_default: self.optional_field_with_a_default.unwrap_or(f64::MAX),
///             required_field_with_no_default: Arc::new(Mutex::new(
///                 self.required_field_with_no_default.expect("'required_field_with_no_default' is required")
///             )),
///        }
///    }
/// }
/// ```
///
/// In this example, the result of macro expansion would look like this:
///
/// ```
/// # use std::collections::HashMap;
/// # use std::sync::{Arc, Mutex};
/// #[derive(Clone, Debug, Default)]
/// pub struct Builder {
///     #[doc = "An optional field which may or may not be set when `.build()` is called."]
///     optional_field: Option<String>,
///     #[doc = "An optional field that will default to `f64::MAX` if it's unset when `.build()` is called."]
///     optional_field_with_a_default: Option<f64>,
///     #[doc = "A required field that will cause the builder to panic if it's unset when `.build()` is called."]
///     required_field_with_no_default: Option<HashMap<String, String>>,
/// }
///
/// impl Builder {
///     pub fn new() -> Self {
///         Builder::default()
///     }
///
///     #[doc = "An optional field which may or may not be set when `.build()` is called."]
///     pub fn set_optional_field(&mut self, optional_field: Option<String>) -> &mut Self {
///         self.optional_field = optional_field;
///         self
///     }
///
///     #[doc = "An optional field which may or may not be set when `.build()` is called."]
///     pub fn optional_field(mut self, optional_field: String) -> Self {
///         self.optional_field = Some(optional_field);
///         self
///     }
///
///     #[doc = "An optional field that will default to `f64::MAX` if it's unset when `.build()` is called."]
///     pub fn set_optional_field_with_a_default(&mut self, optional_field_with_a_default: Option<f64>) -> &mut Self {
///         self.optional_field_with_a_default = optional_field_with_a_default;
///         self
///     }
///
///     #[doc = "An optional field that will default to `f64::MAX` if it's unset when `.build()` is called."]
///     pub fn optional_field_with_a_default(mut self, optional_field_with_a_default: f64) -> Self {
///         self.optional_field_with_a_default = Some(optional_field_with_a_default);
///         self
///     }
///
///     #[doc = "A required field that will cause the builder to panic if it's unset when `.build()` is called."]
///     pub fn set_required_field_with_no_default(&mut self, required_field_with_no_default: Option<HashMap<String, String>>) -> &mut Self {
///         self.required_field_with_no_default = required_field_with_no_default;
///         self
///     }
///
///     #[doc = "A required field that will cause the builder to panic if it's unset when `.build()` is called."]
///     pub fn required_field_with_no_default(mut self, required_field_with_no_default: HashMap<String, String>) -> Self {
///         self.required_field_with_no_default = Some(required_field_with_no_default);
///         self
///     }
/// }
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! builder {
    ($($tt:tt)+) => {
        builder_struct!($($tt)+);

        impl Builder {
            pub fn new() -> Self {
                Builder::default()
            }

            builder_methods!($($tt)+);
        }
    }
}

/// Define a new builder struct, its fields, and their docs. This macro is intended to be called
/// by the `builder!` macro and should not be called directly.
#[doc(hidden)]
#[macro_export]
macro_rules! builder_struct {
    ($($_setter_name:ident, $field_name:ident, $ty:ty, $doc:literal $(,)?)+) => {
        #[derive(Clone, Debug, Default)]
        pub struct Builder {
            $(
            #[doc = $doc]
            $field_name: Option<$ty>,
            )+
        }
    }
}

/// Define setter methods for a builder struct. Must be called from within an `impl` block. This
/// macro is intended to be called by the `builder!` macro and should not be called directly.
#[doc(hidden)]
#[macro_export]
macro_rules! builder_methods {
    ($fn_name:ident, $arg_name:ident, $ty:ty, $doc:literal, $($tail:tt)+) => {
        builder_methods!($fn_name, $arg_name, $ty, $doc);
        builder_methods!($($tail)+);
    };
    ($fn_name:ident, $arg_name:ident, $ty:ty, $doc:literal) => {
        #[doc = $doc]
        pub fn $fn_name(&mut self, $arg_name: Option<$ty>) -> &mut Self {
            self.$arg_name = $arg_name;
            self
        }

        #[doc = $doc]
        pub fn $arg_name(mut self, $arg_name: $ty) -> Self {
            self.$arg_name = Some($arg_name);
            self
        }
    };
}
