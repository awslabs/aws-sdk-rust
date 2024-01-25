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
//! This is the Proton Service API Reference. It provides descriptions, syntax and usage examples for each of the [actions](https://docs.aws.amazon.com/proton/latest/APIReference/API_Operations.html) and [data types](https://docs.aws.amazon.com/proton/latest/APIReference/API_Types.html) for the Proton service.
//!
//! The documentation for each action shows the Query API request parameters and the XML response.
//!
//! Alternatively, you can use the Amazon Web Services CLI to access an API. For more information, see the [Amazon Web Services Command Line Interface User Guide](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-welcome.html).
//!
//! The Proton service is a two-pronged automation framework. Administrators create service templates to provide standardized infrastructure and deployment tooling for serverless and container based applications. Developers, in turn, select from the available service templates to automate their application or service deployments.
//!
//! Because administrators define the infrastructure and tooling that Proton deploys and manages, they need permissions to use all of the listed API operations.
//!
//! When developers select a specific infrastructure and tooling set, Proton deploys their applications. To monitor their applications that are running on Proton, developers need permissions to the service _create_, _list_, _update_ and _delete_ API operations and the service instance _list_ and _update_ API operations.
//!
//! To learn more about Proton, see the [Proton User Guide](https://docs.aws.amazon.com/proton/latest/userguide/Welcome.html).
//!
//! __Ensuring Idempotency__
//!
//! When you make a mutating API request, the request typically returns a result before the asynchronous workflows of the operation are complete. Operations might also time out or encounter other server issues before they're complete, even if the request already returned a result. This might make it difficult to determine whether the request succeeded. Moreover, you might need to retry the request multiple times to ensure that the operation completes successfully. However, if the original request and the subsequent retries are successful, the operation occurs multiple times. This means that you might create more resources than you intended.
//!
//! _Idempotency_ ensures that an API request action completes no more than one time. With an idempotent request, if the original request action completes successfully, any subsequent retries complete successfully without performing any further actions. However, the result might contain updated information, such as the current creation status.
//!
//! The following lists of APIs are grouped according to methods that ensure idempotency.
//!
//! __Idempotent create APIs with a client token__
//!
//! The API actions in this list support idempotency with the use of a _client token_. The corresponding Amazon Web Services CLI commands also support idempotency using a client token. A client token is a unique, case-sensitive string of up to 64 ASCII characters. To make an idempotent API request using one of these actions, specify a client token in the request. We recommend that you _don't_ reuse the same client token for other API requests. If you don’t provide a client token for these APIs, a default client token is automatically provided by SDKs.
//!
//! Given a request action that has succeeded:
//!
//! If you retry the request using the same client token and the same parameters, the retry succeeds without performing any further actions other than returning the original resource detail data in the response.
//!
//! If you retry the request using the same client token, but one or more of the parameters are different, the retry throws a ValidationException with an IdempotentParameterMismatch error.
//!
//! Client tokens expire eight hours after a request is made. If you retry the request with the expired token, a new resource is created.
//!
//! If the original resource is deleted and you retry the request, a new resource is created.
//!
//! Idempotent create APIs with a client token:
//!   - CreateEnvironmentTemplateVersion
//!   - CreateServiceTemplateVersion
//!   - CreateEnvironmentAccountConnection
//!
//! __Idempotent create APIs__
//!
//! Given a request action that has succeeded:
//!
//! If you retry the request with an API from this group, and the original resource _hasn't_ been modified, the retry succeeds without performing any further actions other than returning the original resource detail data in the response.
//!
//! If the original resource has been modified, the retry throws a ConflictException.
//!
//! If you retry with different input parameters, the retry throws a ValidationException with an IdempotentParameterMismatch error.
//!
//! Idempotent create APIs:
//!   - CreateEnvironmentTemplate
//!   - CreateServiceTemplate
//!   - CreateEnvironment
//!   - CreateService
//!
//! __Idempotent delete APIs__
//!
//! Given a request action that has succeeded:
//!
//! When you retry the request with an API from this group and the resource was deleted, its metadata is returned in the response.
//!
//! If you retry and the resource doesn't exist, the response is empty.
//!
//! In both cases, the retry succeeds.
//!
//! Idempotent delete APIs:
//!   - DeleteEnvironmentTemplate
//!   - DeleteEnvironmentTemplateVersion
//!   - DeleteServiceTemplate
//!   - DeleteServiceTemplateVersion
//!   - DeleteEnvironmentAccountConnection
//!
//! __Asynchronous idempotent delete APIs__
//!
//! Given a request action that has succeeded:
//!
//! If you retry the request with an API from this group, if the original request delete operation status is DELETE_IN_PROGRESS, the retry returns the resource detail data in the response without performing any further actions.
//!
//! If the original request delete operation is complete, a retry returns an empty response.
//!
//! Asynchronous idempotent delete APIs:
//!   - DeleteEnvironment
//!   - DeleteService
//!
//! ## Getting Started
//!
//! > Examples are available for many services and operations, check out the
//! > [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).
//!
//! The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
//! as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-proton` to
//! your project, add the following to your **Cargo.toml** file:
//!
//! ```toml
//! [dependencies]
//! aws-config = { version = "1.1.4", features = ["behavior-version-latest"] }
//! aws-sdk-proton = "1.12.0"
//! tokio = { version = "1", features = ["full"] }
//! ```
//!
//! Then in code, a client can be created with the following:
//!
//! ```rust,no_run
//! use aws_sdk_proton as proton;
//!
//! #[::tokio::main]
//! async fn main() -> Result<(), proton::Error> {
//!     let config = aws_config::load_from_env().await;
//!     let client = aws_sdk_proton::Client::new(&config);
//!
//!     // ... make some calls with the client
//!
//!     Ok(())
//! }
//! ```
//!
//! See the [client documentation](https://docs.rs/aws-sdk-proton/latest/aws_sdk_proton/client/struct.Client.html)
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
//! offered by AWS Proton. The return value of each of these methods is a "fluent builder",
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

/// Client for calling AWS Proton.
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
/// let client = aws_sdk_proton::Client::new(&config);
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
/// let config = aws_sdk_proton::config::Builder::from(&sdk_config)
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
/// For example, the [`CancelComponentDeployment`](crate::operation::cancel_component_deployment) operation has
/// a [`Client::cancel_component_deployment`], function which returns a builder for that operation.
/// The fluent builder ultimately has a `send()` function that returns an async future that
/// returns a result, as illustrated below:
///
/// ```rust,ignore
/// let result = client.cancel_component_deployment()
///     .component_name("example")
///     .send()
///     .await;
/// ```
///
/// The underlying HTTP requests that get made by this can be modified with the `customize_operation`
/// function on the fluent builder. See the [`customize`](crate::client::customize) module for more
/// information.
pub mod client;

/// Configuration for AWS Proton.
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

pub(crate) mod client_idempotency_token;

mod idempotency_token;

pub(crate) mod protocol_serde;

mod serialization_settings;

mod endpoint_lib;

mod lens;

mod serde_util;

mod json_errors;

#[doc(inline)]
pub use client::Client;
