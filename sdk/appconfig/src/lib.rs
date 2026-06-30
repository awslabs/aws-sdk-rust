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
#![allow(clippy::unnecessary_map_on_constructor)]
#![allow(clippy::useless_conversion)]
#![allow(clippy::deprecated_semver)]
#![allow(rustdoc::bare_urls)]
#![allow(rustdoc::redundant_explicit_links)]
#![allow(rustdoc::broken_intra_doc_links)]
#![allow(rustdoc::invalid_html_tags)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
//! AppConfig helps you safely change application behavior in production without redeploying code. Using feature flags and dynamic free-form configurations, you can control how your application runs in real time. This approach reduces risk, accelerates releases, and enables faster responses to issues. You can gradually roll out new features to specific users, monitor their impact, and expand availability with confidence. You can also update block lists, allow lists, throttling limits, and logging levels instantly, allowing you to mitigate issues and fine-tune performance without a deployment.
//!
//! AppConfig supports a broad spectrum of use cases:
//!   - __Feature flags and toggles__ – Gradually release new capabilities to targeted users, monitor impact, and instantly roll back changes if issues occur.
//!   - __Application tuning__ – Introduce changes safely in production, measure their effects, and refine behavior without redeploying code.
//!   - __Allow list or block list__ – Control access to features or restrict specific users in real time, without modifying application code.
//!   - __Centralized configuration storage__ – Manage configuration data consistently across workloads. AppConfig can deploy configuration from the AppConfig hosted configuration store, Secrets Manager, Systems Manager, Systems Manager Parameter Store, or Amazon S3.
//!
//! __How AppConfig works__
//!
//! This section provides a high-level description of how AppConfig works and how you get started.
//!
//! __1. Identify configuration data to manage in AppConfig__
//!
//! Before creating a configuration profile, identify the configuration data in your code that you want to manage dynamically using AppConfig. Common examples include feature flags, allow and block lists, logging levels, service limits, and throttling rules. These values tend to change frequently and can cause issues if misconfigured. If your configuration data already exists in cloud services such as Systems Manager Parameter Store or Amazon S3, you can use AppConfig to validate, deploy, and manage that data more effectively.
//!
//! __2. Create a configuration profile in AppConfig__
//!
//! A configuration profile defines how AppConfig locates and manages your configuration data. It includes a URI that points to the data source and a profile type. AppConfig supports two profile types   - __Feature flags__ – Enable controlled feature releases, gradual rollouts, and testing in production.
//!   - __Free-form configurations__ – Store and retrieve configuration data from external sources and update it without redeploying code.
//! Both profile types help decouple configuration from code, support continuous delivery, and reduce deployment risk. You can also add optional validators to ensure that configuration data is syntactically and semantically correct. During deployment, AppConfig evaluates these validators and automatically rolls back changes if validation fails. Each configuration profile is associated with an application, which acts as a logical container for your configuration resources. For more information about creating a configuration profile, see [Creating a configuration profile in AppConfig](http://docs.aws.amazon.com/appconfig/latest/userguide/appconfig-creating-configuration-profile.html) in the the _AppConfig User Guide_.
//!
//! __3. Deploy configuration data__
//!
//! When you start a deployment, AppConfig:   1. Retrieves configuration data from the source defined in the configuration profile
//!   1. Validates the data using the configured validators
//!   1. Delivers the validated configuration to AppConfig Agent
//! The delivered configuration becomes the deployed version used by your application. For more information about deploying a configuration, see [Deploying feature flags and configuration data in AppConfig](http://docs.aws.amazon.com/appconfig/latest/userguide/deploying-feature-flags.html).
//!
//! __4. Retrieve configuration data__
//!
//! Your application retrieves configuration data by calling a local endpoint exposed by AppConfig Agent, which caches the deployed configuration. Retrieving data is a metered event. AppConfig Agent supports a variety of use cases, as described in [How to use AppConfig Agent to retrieve configuration data](http://docs.aws.amazon.com/appconfig/latest/userguide/appconfig-agent-how-to-use.html). If the agent is not suitable for your use case, your application can retrieve configuration data directly from AppConfig by calling the [StartConfigurationSession](https://docs.aws.amazon.com/appconfig/2019-10-09/APIReference/API_appconfigdata_StartConfigurationSession.html) and [GetLatestConfiguration](https://docs.aws.amazon.com/appconfig/2019-10-09/APIReference/API_appconfigdata_GetLatestConfiguration.html) API actions. For more information about retrieving a configuration, see [Retrieving feature flags and configuration data in AppConfig](http://docs.aws.amazon.com/appconfig/latest/userguide/retrieving-feature-flags.html).
//!
//!
//! This reference is intended to be used with the [AppConfig User Guide](http://docs.aws.amazon.com/appconfig/latest/userguide/what-is-appconfig.html).
//!
//! ## Getting Started
//!
//! > Examples are available for many services and operations, check out the
//! > [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).
//!
//! The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
//! as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-appconfig` to
//! your project, add the following to your **Cargo.toml** file:
//!
//! ```toml
//! [dependencies]
//! aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
//! aws-sdk-appconfig = "1.108.0"
//! tokio = { version = "1", features = ["full"] }
//! ```
//!
//! Then in code, a client can be created with the following:
//!
//! ```rust,no_run
//! use aws_sdk_appconfig as appconfig;
//!
//! #[::tokio::main]
//! async fn main() -> Result<(), appconfig::Error> {
//!     let config = aws_config::load_from_env().await;
//!     let client = aws_sdk_appconfig::Client::new(&config);
//!
//!     // ... make some calls with the client
//!
//!     Ok(())
//! }
//! ```
//!
//! See the [client documentation](https://docs.rs/aws-sdk-appconfig/latest/aws_sdk_appconfig/client/struct.Client.html)
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
//! * [Usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1)
//!
//!
//! # Crate Organization
//!
//! The entry point for most customers will be [`Client`], which exposes one method for each API
//! offered by Amazon AppConfig. The return value of each of these methods is a "fluent builder",
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

/// Client for calling Amazon AppConfig.
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
/// let client = aws_sdk_appconfig::Client::new(&config);
/// # }
/// ```
///
/// Occasionally, SDKs may have additional service-specific values that can be set on the [`Config`] that
/// is absent from [`SdkConfig`], or slightly different settings for a specific client may be desired.
/// The [`Builder`](crate::config::Builder) struct implements `From<&SdkConfig>`, so setting these specific settings can be
/// done as follows:
///
/// ```rust,no_run
/// # async fn wrapper() {
/// let sdk_config = ::aws_config::load_from_env().await;
/// let config = aws_sdk_appconfig::config::Builder::from(&sdk_config)
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
/// For example, the [`CreateApplication`](crate::operation::create_application) operation has
/// a [`Client::create_application`], function which returns a builder for that operation.
/// The fluent builder ultimately has a `send()` function that returns an async future that
/// returns a result, as illustrated below:
///
/// ```rust,ignore
/// let result = client.create_application()
///     .name("example")
///     .send()
///     .await;
/// ```
///
/// The underlying HTTP requests that get made by this can be modified with the `customize_operation`
/// function on the fluent builder. See the [`customize`](crate::client::customize) module for more
/// information.
/// # Waiters
///
/// This client provides `wait_until` methods behind the [`Waiters`](crate::client::Waiters) trait.
/// To use them, simply import the trait, and then call one of the `wait_until` methods. This will
/// return a waiter fluent builder that takes various parameters, which are documented on the builder
/// type. Once parameters have been provided, the `wait` method can be called to initiate waiting.
///
/// For example, if there was a `wait_until_thing` method, it could look like:
/// ```rust,ignore
/// let result = client.wait_until_thing()
///     .thing_id("someId")
///     .wait(Duration::from_secs(120))
///     .await;
/// ```
pub mod client;

/// Configuration for Amazon AppConfig.
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

mod observability_feature;

pub(crate) mod protocol_serde;

mod sdk_feature_tracker;

mod serialization_settings;

mod endpoint_lib;

mod lens;

/// Supporting types for waiters.
///
/// Note: to use waiters, import the [`Waiters`](crate::client::Waiters) trait, which adds methods prefixed with `wait_until` to the client.
pub mod waiters;

mod json_errors;

mod serde_util;

#[doc(inline)]
pub use client::Client;
