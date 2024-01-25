#![allow(deprecated)]
#![allow(unknown_lints)]
#![allow(clippy::module_inception)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::wrong_self_convention)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::disallowed_names)]
#![allow(clippy::vec_init_then_push)]
#![allow(clippy::type_complexity)]
#![allow(clippy::needless_return)]
#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(clippy::result_large_err)]
#![allow(rustdoc::bare_urls)]
#![allow(rustdoc::redundant_explicit_links)]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
//! Welcome to the _AWS OpsWorks Stacks API Reference_. This guide provides descriptions, syntax, and usage examples for AWS OpsWorks Stacks actions and data types, including common parameters and error codes.
//!
//! AWS OpsWorks Stacks is an application management service that provides an integrated experience for overseeing the complete application lifecycle. For information about this product, go to the [AWS OpsWorks](http://aws.amazon.com/opsworks/) details page.
//!
//! __SDKs and CLI__
//!
//! The most common way to use the AWS OpsWorks Stacks API is by using the AWS Command Line Interface (CLI) or by using one of the AWS SDKs to implement applications in your preferred language. For more information, see:
//!   - [AWS CLI](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-welcome.html)
//!   - [AWS SDK for Java](https://docs.aws.amazon.com/AWSJavaSDK/latest/javadoc/com/amazonaws/services/opsworks/AWSOpsWorksClient.html)
//!   - [AWS SDK for .NET](https://docs.aws.amazon.com/sdkfornet/latest/apidocs/html/N_Amazon_OpsWorks.htm)
//!   - [AWS SDK for PHP 2](https://docs.aws.amazon.com/aws-sdk-php-2/latest/class-Aws.OpsWorks.OpsWorksClient.html)
//!   - [AWS SDK for Ruby](http://docs.aws.amazon.com/sdkforruby/api/)
//!   - [AWS SDK for Node.js](http://aws.amazon.com/documentation/sdkforjavascript/)
//!   - [AWS SDK for Python(Boto)](http://docs.pythonboto.org/en/latest/ref/opsworks.html)
//!
//! __Endpoints__
//!
//! AWS OpsWorks Stacks supports the following endpoints, all HTTPS. You must connect to one of the following endpoints. Stacks can only be accessed or managed within the endpoint in which they are created.
//!   - opsworks.us-east-1.amazonaws.com
//!   - opsworks.us-east-2.amazonaws.com
//!   - opsworks.us-west-1.amazonaws.com
//!   - opsworks.us-west-2.amazonaws.com
//!   - opsworks.ca-central-1.amazonaws.com (API only; not available in the AWS console)
//!   - opsworks.eu-west-1.amazonaws.com
//!   - opsworks.eu-west-2.amazonaws.com
//!   - opsworks.eu-west-3.amazonaws.com
//!   - opsworks.eu-central-1.amazonaws.com
//!   - opsworks.ap-northeast-1.amazonaws.com
//!   - opsworks.ap-northeast-2.amazonaws.com
//!   - opsworks.ap-south-1.amazonaws.com
//!   - opsworks.ap-southeast-1.amazonaws.com
//!   - opsworks.ap-southeast-2.amazonaws.com
//!   - opsworks.sa-east-1.amazonaws.com
//!
//! __Chef Versions__
//!
//! When you call CreateStack, CloneStack, or UpdateStack we recommend you use the ConfigurationManager parameter to specify the Chef version. The recommended and default value for Linux stacks is currently 12. Windows stacks use Chef 12.2. For more information, see [Chef Versions](https://docs.aws.amazon.com/opsworks/latest/userguide/workingcookbook-chef11.html).
//!
//! ## Getting Started
//!
//! > Examples are available for many services and operations, check out the
//! > [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).
//!
//! The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
//! as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-opsworks` to
//! your project, add the following to your **Cargo.toml** file:
//!
//! ```toml
//! [dependencies]
//! aws-config = { version = "1.1.4", features = ["behavior-version-latest"] }
//! aws-sdk-opsworks = "1.12.0"
//! tokio = { version = "1", features = ["full"] }
//! ```
//!
//! Then in code, a client can be created with the following:
//!
//! ```rust,no_run
//! use aws_sdk_opsworks as opsworks;
//!
//! #[::tokio::main]
//! async fn main() -> Result<(), opsworks::Error> {
//!     let config = aws_config::load_from_env().await;
//!     let client = aws_sdk_opsworks::Client::new(&config);
//!
//!     // ... make some calls with the client
//!
//!     Ok(())
//! }
//! ```
//!
//! See the [client documentation](https://docs.rs/aws-sdk-opsworks/latest/aws_sdk_opsworks/client/struct.Client.html)
//! for information on what calls can be made, and the inputs and outputs for each of those calls.
//!
//! ## Using the SDK
//!
//! Until the SDK is released, we will be adding information about using the SDK to the
//! [Developer Guide](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/welcome.html). Feel free to suggest
//! additional sections for the guide by opening an issue and describing what you are trying to do.
//!
//! ## Getting Help
//!
//! * [GitHub discussions](https://github.com/awslabs/aws-sdk-rust/discussions) - For ideas, RFCs & general questions
//! * [GitHub issues](https://github.com/awslabs/aws-sdk-rust/issues/new/choose) - For bug reports & feature requests
//! * [Generated Docs (latest version)](https://awslabs.github.io/aws-sdk-rust/)
//! * [Usage examples](https://github.com/awslabs/aws-sdk-rust/tree/main/examples)
//!
//!
//! # Crate Organization
//!
//! The entry point for most customers will be [`Client`], which exposes one method for each API
//! offered by AWS OpsWorks. The return value of each of these methods is a "fluent builder",
//! where the different inputs for that API are added by builder-style function call chaining,
//! followed by calling `send()` to get a [`Future`](std::future::Future) that will result in
//! either a successful output or a [`SdkError`](crate::error::SdkError).
//!
//! Some of these API inputs may be structs or enums to provide more complex structured information.
//! These structs and enums live in [`types`](crate::types). There are some simpler types for
//! representing data such as date times or binary blobs that live in [`primitives`](crate::primitives).
//!
//! All types required to configure a client via the [`Config`](crate::Config) struct live
//! in [`config`](crate::config).
//!
//! The [`operation`](crate::operation) module has a submodule for every API, and in each submodule
//! is the input, output, and error type for that API, as well as builders to construct each of those.
//!
//! There is a top-level [`Error`](crate::Error) type that encompasses all the errors that the
//! client can return. Any other error type can be converted to this `Error` type via the
//! [`From`](std::convert::From) trait.
//!
//! The other modules within this crate are not required for normal usage.

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use error_meta::Error;

#[doc(inline)]
pub use config::Config;

/// Client for calling AWS OpsWorks.
/// ## Constructing a `Client`
///
/// A [`Config`] is required to construct a client. For most use cases, the [`aws-config`]
/// crate should be used to automatically resolve this config using
/// [`aws_config::load_from_env()`], since this will resolve an [`SdkConfig`] which can be shared
/// across multiple different AWS SDK clients. This config resolution process can be customized
/// by calling [`aws_config::from_env()`] instead, which returns a [`ConfigLoader`] that uses
/// the [builder pattern] to customize the default config.
///
/// In the simplest case, creating a client looks as follows:
/// ```rust,no_run
/// # async fn wrapper() {
/// let config = aws_config::load_from_env().await;
/// let client = aws_sdk_opsworks::Client::new(&config);
/// # }
/// ```
///
/// Occasionally, SDKs may have additional service-specific values that can be set on the [`Config`] that
/// is absent from [`SdkConfig`], or slightly different settings for a specific client may be desired.
/// The [`Config`] struct implements `From<&SdkConfig>`, so setting these specific settings can be
/// done as follows:
///
/// ```rust,no_run
/// # async fn wrapper() {
/// let sdk_config = ::aws_config::load_from_env().await;
/// let config = aws_sdk_opsworks::config::Builder::from(&sdk_config)
/// # /*
///     .some_service_specific_setting("value")
/// # */
///     .build();
/// # }
/// ```
///
/// See the [`aws-config` docs] and [`Config`] for more information on customizing configuration.
///
/// _Note:_ Client construction is expensive due to connection thread pool initialization, and should
/// be done once at application start-up.
///
/// [`Config`]: crate::Config
/// [`ConfigLoader`]: https://docs.rs/aws-config/*/aws_config/struct.ConfigLoader.html
/// [`SdkConfig`]: https://docs.rs/aws-config/*/aws_config/struct.SdkConfig.html
/// [`aws-config` docs]: https://docs.rs/aws-config/*
/// [`aws-config`]: https://crates.io/crates/aws-config
/// [`aws_config::from_env()`]: https://docs.rs/aws-config/*/aws_config/fn.from_env.html
/// [`aws_config::load_from_env()`]: https://docs.rs/aws-config/*/aws_config/fn.load_from_env.html
/// [builder pattern]: https://rust-lang.github.io/api-guidelines/type-safety.html#builders-enable-construction-of-complex-values-c-builder
/// # Using the `Client`
///
/// A client has a function for every operation that can be performed by the service.
/// For example, the [`AssignInstance`](crate::operation::assign_instance) operation has
/// a [`Client::assign_instance`], function which returns a builder for that operation.
/// The fluent builder ultimately has a `send()` function that returns an async future that
/// returns a result, as illustrated below:
///
/// ```rust,ignore
/// let result = client.assign_instance()
///     .instance_id("example")
///     .send()
///     .await;
/// ```
///
/// The underlying HTTP requests that get made by this can be modified with the `customize_operation`
/// function on the fluent builder. See the [`customize`](crate::client::customize) module for more
/// information.
pub mod client;

/// Configuration for AWS OpsWorks.
pub mod config;

/// Common errors and error handling utilities.
pub mod error;

mod error_meta;

/// Information about this crate.
pub mod meta;

/// All operations that this crate can perform.
pub mod operation;

/// Primitives such as `Blob` or `DateTime` used by other types.
pub mod primitives;

/// Data structures used by operation inputs/outputs.
pub mod types;

mod auth_plugin;

pub(crate) mod protocol_serde;

mod serialization_settings;

mod endpoint_lib;

mod lens;

mod json_errors;

mod serde_util;

#[doc(inline)]
pub use client::Client;
