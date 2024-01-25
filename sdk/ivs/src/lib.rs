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
//! The Amazon Interactive Video Service (IVS) API is REST compatible, using a standard HTTP API and an Amazon Web Services EventBridge event stream for responses. JSON is used for both requests and responses, including errors.
//!
//! The API is an Amazon Web Services regional service. For a list of supported regions and Amazon IVS HTTPS service endpoints, see the [Amazon IVS page](https://docs.aws.amazon.com/general/latest/gr/ivs.html) in the _Amazon Web Services General Reference_.
//!
//! _ __All API request parameters and URLs are case sensitive. __ _
//!
//! For a summary of notable documentation changes in each release, see [Document History](https://docs.aws.amazon.com/ivs/latest/userguide/doc-history.html).
//!
//! __Allowed Header Values__
//!   - __Accept:__ application/json
//!   - __Accept-Encoding:__ gzip, deflate
//!   - __Content-Type:__ application/json
//!
//! __Resources__
//!
//! The following resources contain information about your IVS live stream (see [Getting Started with Amazon IVS](https://docs.aws.amazon.com/ivs/latest/userguide/getting-started.html)):
//!   - __Channel__ — Stores configuration data related to your live stream. You first create a channel and then use the channel’s stream key to start your live stream. See the Channel endpoints for more information.
//!   - __Stream key__ — An identifier assigned by Amazon IVS when you create a channel, which is then used to authorize streaming. See the StreamKey endpoints for more information. _ __Treat the stream key like a secret, since it allows anyone to stream to the channel.__ _
//!   - __Playback key pair__ — Video playback may be restricted using playback-authorization tokens, which use public-key encryption. A playback key pair is the public-private pair of keys used to sign and validate the playback-authorization token. See the PlaybackKeyPair endpoints for more information.
//!   - __Recording configuration__ — Stores configuration related to recording a live stream and where to store the recorded content. Multiple channels can reference the same recording configuration. See the Recording Configuration endpoints for more information.
//!
//! __Tagging__
//!
//! A _tag_ is a metadata label that you assign to an Amazon Web Services resource. A tag comprises a _key_ and a _value_, both set by you. For example, you might set a tag as topic:nature to label a particular video category. See [Tagging Amazon Web Services Resources](https://docs.aws.amazon.com/general/latest/gr/aws_tagging.html) for more information, including restrictions that apply to tags and "Tag naming limits and requirements"; Amazon IVS has no service-specific constraints beyond what is documented there.
//!
//! Tags can help you identify and organize your Amazon Web Services resources. For example, you can use the same tag for different resources to indicate that they are related. You can also use tags to manage access (see [Access Tags](https://docs.aws.amazon.com/IAM/latest/UserGuide/access_tags.html)).
//!
//! The Amazon IVS API has these tag-related endpoints: TagResource, UntagResource, and ListTagsForResource. The following resources support tagging: Channels, Stream Keys, Playback Key Pairs, and Recording Configurations.
//!
//! At most 50 tags can be applied to a resource.
//!
//! __Authentication versus Authorization__
//!
//! Note the differences between these concepts:
//!   - _Authentication_ is about verifying identity. You need to be authenticated to sign Amazon IVS API requests.
//!   - _Authorization_ is about granting permissions. Your IAM roles need to have permissions for Amazon IVS API requests. In addition, authorization is needed to view [Amazon IVS private channels](https://docs.aws.amazon.com/ivs/latest/userguide/private-channels.html). (Private channels are channels that are enabled for "playback authorization.")
//!
//! __Authentication__
//!
//! All Amazon IVS API requests must be authenticated with a signature. The Amazon Web Services Command-Line Interface (CLI) and Amazon IVS Player SDKs take care of signing the underlying API calls for you. However, if your application calls the Amazon IVS API directly, it’s your responsibility to sign the requests.
//!
//! You generate a signature using valid Amazon Web Services credentials that have permission to perform the requested action. For example, you must sign PutMetadata requests with a signature generated from a user account that has the ivs:PutMetadata permission.
//!
//! For more information:
//!   - Authentication and generating signatures — See [Authenticating Requests (Amazon Web Services Signature Version 4)](https://docs.aws.amazon.com/AmazonS3/latest/API/sig-v4-authenticating-requests.html) in the _Amazon Web Services General Reference_.
//!   - Managing Amazon IVS permissions — See [Identity and Access Management](https://docs.aws.amazon.com/ivs/latest/userguide/security-iam.html) on the Security page of the _Amazon IVS User Guide_.
//!
//! __Amazon Resource Names (ARNs)__
//!
//! ARNs uniquely identify AWS resources. An ARN is required when you need to specify a resource unambiguously across all of AWS, such as in IAM policies and API calls. For more information, see [Amazon Resource Names](https://docs.aws.amazon.com/general/latest/gr/aws-arns-and-namespaces.html) in the _AWS General Reference_.
//!
//! __Channel Endpoints__
//!   - CreateChannel — Creates a new channel and an associated stream key to start streaming.
//!   - GetChannel — Gets the channel configuration for the specified channel ARN.
//!   - BatchGetChannel — Performs GetChannel on multiple ARNs simultaneously.
//!   - ListChannels — Gets summary information about all channels in your account, in the Amazon Web Services region where the API request is processed. This list can be filtered to match a specified name or recording-configuration ARN. Filters are mutually exclusive and cannot be used together. If you try to use both filters, you will get an error (409 Conflict Exception).
//!   - UpdateChannel — Updates a channel's configuration. This does not affect an ongoing stream of this channel. You must stop and restart the stream for the changes to take effect.
//!   - DeleteChannel — Deletes the specified channel.
//!
//! __StreamKey Endpoints__
//!   - CreateStreamKey — Creates a stream key, used to initiate a stream, for the specified channel ARN.
//!   - GetStreamKey — Gets stream key information for the specified ARN.
//!   - BatchGetStreamKey — Performs GetStreamKey on multiple ARNs simultaneously.
//!   - ListStreamKeys — Gets summary information about stream keys for the specified channel.
//!   - DeleteStreamKey — Deletes the stream key for the specified ARN, so it can no longer be used to stream.
//!
//! __Stream Endpoints__
//!   - GetStream — Gets information about the active (live) stream on a specified channel.
//!   - GetStreamSession — Gets metadata on a specified stream.
//!   - ListStreams — Gets summary information about live streams in your account, in the Amazon Web Services region where the API request is processed.
//!   - ListStreamSessions — Gets a summary of current and previous streams for a specified channel in your account, in the AWS region where the API request is processed.
//!   - StopStream — Disconnects the incoming RTMPS stream for the specified channel. Can be used in conjunction with DeleteStreamKey to prevent further streaming to a channel.
//!   - PutMetadata — Inserts metadata into the active stream of the specified channel. At most 5 requests per second per channel are allowed, each with a maximum 1 KB payload. (If 5 TPS is not sufficient for your needs, we recommend batching your data into a single PutMetadata call.) At most 155 requests per second per account are allowed.
//!
//! __Private Channel Endpoints__
//!
//! For more information, see [Setting Up Private Channels](https://docs.aws.amazon.com/ivs/latest/userguide/private-channels.html) in the _Amazon IVS User Guide_.
//!   - ImportPlaybackKeyPair — Imports the public portion of a new key pair and returns its arn and fingerprint. The privateKey can then be used to generate viewer authorization tokens, to grant viewers access to private channels (channels enabled for playback authorization).
//!   - GetPlaybackKeyPair — Gets a specified playback authorization key pair and returns the arn and fingerprint. The privateKey held by the caller can be used to generate viewer authorization tokens, to grant viewers access to private channels.
//!   - ListPlaybackKeyPairs — Gets summary information about playback key pairs.
//!   - DeletePlaybackKeyPair — Deletes a specified authorization key pair. This invalidates future viewer tokens generated using the key pair’s privateKey.
//!   - StartViewerSessionRevocation — Starts the process of revoking the viewer session associated with a specified channel ARN and viewer ID. Optionally, you can provide a version to revoke viewer sessions less than and including that version.
//!   - BatchStartViewerSessionRevocation — Performs StartViewerSessionRevocation on multiple channel ARN and viewer ID pairs simultaneously.
//!
//! __RecordingConfiguration Endpoints__
//!   - CreateRecordingConfiguration — Creates a new recording configuration, used to enable recording to Amazon S3.
//!   - GetRecordingConfiguration — Gets the recording-configuration metadata for the specified ARN.
//!   - ListRecordingConfigurations — Gets summary information about all recording configurations in your account, in the Amazon Web Services region where the API request is processed.
//!   - DeleteRecordingConfiguration — Deletes the recording configuration for the specified ARN.
//!
//! __Amazon Web Services Tags Endpoints__
//!   - TagResource — Adds or updates tags for the Amazon Web Services resource with the specified ARN.
//!   - UntagResource — Removes tags from the resource with the specified ARN.
//!   - ListTagsForResource — Gets information about Amazon Web Services tags for the specified ARN.
//!
//! ## Getting Started
//!
//! > Examples are available for many services and operations, check out the
//! > [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).
//!
//! The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
//! as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-ivs` to
//! your project, add the following to your **Cargo.toml** file:
//!
//! ```toml
//! [dependencies]
//! aws-config = { version = "1.1.4", features = ["behavior-version-latest"] }
//! aws-sdk-ivs = "1.12.0"
//! tokio = { version = "1", features = ["full"] }
//! ```
//!
//! Then in code, a client can be created with the following:
//!
//! ```rust,no_run
//! use aws_sdk_ivs as ivs;
//!
//! #[::tokio::main]
//! async fn main() -> Result<(), ivs::Error> {
//!     let config = aws_config::load_from_env().await;
//!     let client = aws_sdk_ivs::Client::new(&config);
//!
//!     // ... make some calls with the client
//!
//!     Ok(())
//! }
//! ```
//!
//! See the [client documentation](https://docs.rs/aws-sdk-ivs/latest/aws_sdk_ivs/client/struct.Client.html)
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
//! offered by Amazon Interactive Video Service. The return value of each of these methods is a "fluent builder",
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

/// Client for calling Amazon Interactive Video Service.
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
/// let client = aws_sdk_ivs::Client::new(&config);
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
/// let config = aws_sdk_ivs::config::Builder::from(&sdk_config)
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
/// For example, the [`CreateChannel`](crate::operation::create_channel) operation has
/// a [`Client::create_channel`], function which returns a builder for that operation.
/// The fluent builder ultimately has a `send()` function that returns an async future that
/// returns a result, as illustrated below:
///
/// ```rust,ignore
/// let result = client.create_channel()
///     .name("example")
///     .send()
///     .await;
/// ```
///
/// The underlying HTTP requests that get made by this can be modified with the `customize_operation`
/// function on the fluent builder. See the [`customize`](crate::client::customize) module for more
/// information.
pub mod client;

/// Configuration for Amazon Interactive Video Service.
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

mod serde_util;

mod json_errors;

#[doc(inline)]
pub use client::Client;
