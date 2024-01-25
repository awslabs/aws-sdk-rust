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
//! __Introduction__
//!
//! The Amazon Interactive Video Service (IVS) real-time API is REST compatible, using a standard HTTP API and an AWS EventBridge event stream for responses. JSON is used for both requests and responses, including errors.
//!
//! Terminology:
//!   - A _stage_ is a virtual space where participants can exchange video in real time.
//!   - A _participant token_ is a token that authenticates a participant when they join a stage.
//!   - A _participant object_ represents participants (people) in the stage and contains information about them. When a token is created, it includes a participant ID; when a participant uses that token to join a stage, the participant is associated with that participant ID. There is a 1:1 mapping between participant tokens and participants.
//!   - Server-side composition: The _composition_ process composites participants of a stage into a single video and forwards it to a set of outputs (e.g., IVS channels). Composition endpoints support this process.
//!   - Server-side composition: A _composition_ controls the look of the outputs, including how participants are positioned in the video.
//!
//! __Resources__
//!
//! The following resources contain information about your IVS live stream (see [Getting Started with Amazon IVS Real-Time Streaming](https://docs.aws.amazon.com/ivs/latest/RealTimeUserGuide/getting-started.html)):
//!   - __Stage__ — A stage is a virtual space where participants can exchange video in real time.
//!
//! __Tagging__
//!
//! A _tag_ is a metadata label that you assign to an AWS resource. A tag comprises a _key_ and a _value_, both set by you. For example, you might set a tag as topic:nature to label a particular video category. See [Tagging AWS Resources](https://docs.aws.amazon.com/general/latest/gr/aws_tagging.html) for more information, including restrictions that apply to tags and "Tag naming limits and requirements"; Amazon IVS stages has no service-specific constraints beyond what is documented there.
//!
//! Tags can help you identify and organize your AWS resources. For example, you can use the same tag for different resources to indicate that they are related. You can also use tags to manage access (see [Access Tags](https://docs.aws.amazon.com/IAM/latest/UserGuide/access_tags.html)).
//!
//! The Amazon IVS real-time API has these tag-related endpoints: TagResource, UntagResource, and ListTagsForResource. The following resource supports tagging: Stage.
//!
//! At most 50 tags can be applied to a resource.
//!
//! __Stages Endpoints__
//!   - CreateParticipantToken — Creates an additional token for a specified stage. This can be done after stage creation or when tokens expire.
//!   - CreateStage — Creates a new stage (and optionally participant tokens).
//!   - DeleteStage — Shuts down and deletes the specified stage (disconnecting all participants).
//!   - DisconnectParticipant — Disconnects a specified participant and revokes the participant permanently from a specified stage.
//!   - GetParticipant — Gets information about the specified participant token.
//!   - GetStage — Gets information for the specified stage.
//!   - GetStageSession — Gets information for the specified stage session.
//!   - ListParticipantEvents — Lists events for a specified participant that occurred during a specified stage session.
//!   - ListParticipants — Lists all participants in a specified stage session.
//!   - ListStages — Gets summary information about all stages in your account, in the AWS region where the API request is processed.
//!   - ListStageSessions — Gets all sessions for a specified stage.
//!   - UpdateStage — Updates a stage’s configuration.
//!
//! __Composition Endpoints__
//!   - GetComposition — Gets information about the specified Composition resource.
//!   - ListCompositions — Gets summary information about all Compositions in your account, in the AWS region where the API request is processed.
//!   - StartComposition — Starts a Composition from a stage based on the configuration provided in the request.
//!   - StopComposition — Stops and deletes a Composition resource. Any broadcast from the Composition resource is stopped.
//!
//! __EncoderConfiguration Endpoints__
//!   - CreateEncoderConfiguration — Creates an EncoderConfiguration object.
//!   - DeleteEncoderConfiguration — Deletes an EncoderConfiguration resource. Ensures that no Compositions are using this template; otherwise, returns an error.
//!   - GetEncoderConfiguration — Gets information about the specified EncoderConfiguration resource.
//!   - ListEncoderConfigurations — Gets summary information about all EncoderConfigurations in your account, in the AWS region where the API request is processed.
//!
//! __StorageConfiguration Endpoints__
//!   - CreateStorageConfiguration — Creates a new storage configuration, used to enable recording to Amazon S3.
//!   - DeleteStorageConfiguration — Deletes the storage configuration for the specified ARN.
//!   - GetStorageConfiguration — Gets the storage configuration for the specified ARN.
//!   - ListStorageConfigurations — Gets summary information about all storage configurations in your account, in the AWS region where the API request is processed.
//!
//! __Tags Endpoints__
//!   - ListTagsForResource — Gets information about AWS tags for the specified ARN.
//!   - TagResource — Adds or updates tags for the AWS resource with the specified ARN.
//!   - UntagResource — Removes tags from the resource with the specified ARN.
//!
//! ## Getting Started
//!
//! > Examples are available for many services and operations, check out the
//! > [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).
//!
//! The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
//! as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-ivsrealtime` to
//! your project, add the following to your **Cargo.toml** file:
//!
//! ```toml
//! [dependencies]
//! aws-config = { version = "1.1.4", features = ["behavior-version-latest"] }
//! aws-sdk-ivsrealtime = "1.12.0"
//! tokio = { version = "1", features = ["full"] }
//! ```
//!
//! Then in code, a client can be created with the following:
//!
//! ```rust,no_run
//! use aws_sdk_ivsrealtime as ivsrealtime;
//!
//! #[::tokio::main]
//! async fn main() -> Result<(), ivsrealtime::Error> {
//!     let config = aws_config::load_from_env().await;
//!     let client = aws_sdk_ivsrealtime::Client::new(&config);
//!
//!     // ... make some calls with the client
//!
//!     Ok(())
//! }
//! ```
//!
//! See the [client documentation](https://docs.rs/aws-sdk-ivsrealtime/latest/aws_sdk_ivsrealtime/client/struct.Client.html)
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
//! offered by Amazon Interactive Video Service RealTime. The return value of each of these methods is a "fluent builder",
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

/// Client for calling Amazon Interactive Video Service RealTime.
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
/// let client = aws_sdk_ivsrealtime::Client::new(&config);
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
/// let config = aws_sdk_ivsrealtime::config::Builder::from(&sdk_config)
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
/// For example, the [`CreateEncoderConfiguration`](crate::operation::create_encoder_configuration) operation has
/// a [`Client::create_encoder_configuration`], function which returns a builder for that operation.
/// The fluent builder ultimately has a `send()` function that returns an async future that
/// returns a result, as illustrated below:
///
/// ```rust,ignore
/// let result = client.create_encoder_configuration()
///     .name("example")
///     .send()
///     .await;
/// ```
///
/// The underlying HTTP requests that get made by this can be modified with the `customize_operation`
/// function on the fluent builder. See the [`customize`](crate::client::customize) module for more
/// information.
pub mod client;

/// Configuration for Amazon Interactive Video Service RealTime.
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
