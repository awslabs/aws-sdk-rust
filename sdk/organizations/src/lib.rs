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
//! Organizations is a web service that enables you to consolidate your multiple Amazon Web Services accounts into an _organization_ and centrally manage your accounts and their resources.
//!
//! This guide provides descriptions of the Organizations API. For more information about using this service, see the [Organizations User Guide](https://docs.aws.amazon.com/organizations/latest/userguide/orgs_introduction.html).
//!
//! __API version__
//!
//! This version of the Organizations API Reference documents the Organizations API version 2016-11-28.
//!
//! We recommend that you use the Amazon Web Services SDKs to make programmatic API calls to Organizations. However, you also can use the Organizations Query API to make direct calls to the Organizations web service. To learn more about the Organizations Query API, see [Calling the API by making HTTP Query requests](https://docs.aws.amazon.com/organizations/latest/userguide/orgs_query-requests.html) in the _Organizations User Guide_. Organizations supports GET and POST requests for all actions. That is, the API doesn't require you to use GET for some actions and POST for others. However, GET requests are subject to the limitation size of a URL. Therefore, for operations that require larger sizes, use a POST request.
//!
//! __Signing requests__
//!
//! When you send HTTP requests to Amazon Web Services, sign the requests so that Amazon Web Services can identify who sent them. You sign requests with your Amazon Web Services access key, which consists of an access key ID and a secret access key. We strongly recommend that you don't create an access key for your root account. Anyone who has the access key for your root account has unrestricted access to all the resources in your account. Instead, create an access key for an IAM user that has administrative permissions. As another option, use Amazon Web Services Security Token Service (Amazon Web Services STS) to generate temporary security credentials, and use those credentials to sign requests.
//!
//! To sign requests, we recommend that you use [Signature Version 4](https://docs.aws.amazon.com/general/latest/gr/signature-version-4.html). If you have an existing application that uses Signature Version 2, you don't have to update it to use Signature Version 4. However, some operations now require Signature Version 4. The documentation for operations that require version 4 indicate this requirement.
//!
//! When you use the Command Line Interface (CLI) or one of the Amazon Web Services SDKs to make requests to Amazon Web Services, these tools automatically sign the requests for you with the access key that you specify when you configure the tools.
//!
//! In this release, each organization can have only one root.
//!
//! __Support and feedback for Organizations__
//!
//! We welcome your feedback. You can post your feedback and questions in the [Organizations support forum](https://forums.aws.amazon.com/forum.jspa?forumID=219). For more information about the Amazon Web Services Support forums, see [Forums Help](https://forums.aws.amazon.com/help.jspa).
//!
//! __Endpoint to call When using the CLI or the Amazon Web Services SDK__
//!
//! For the current release of Organizations, specify the us-east-1 region for all Amazon Web Services API and CLI calls made from the commercial Amazon Web Services Regions outside of China. If calling from one of the Amazon Web Services Regions in China, then specify cn-northwest-1. You can do this in the CLI by using these parameters and commands:
//!   - Use the following parameter with each command to specify both the endpoint and its region: --endpoint-url https://organizations.us-east-1.amazonaws.com _(from commercial Amazon Web Services Regions outside of China)_ or --endpoint-url https://organizations.cn-northwest-1.amazonaws.com.cn _(from Amazon Web Services Regions in China)_
//!   - Use the default endpoint, but configure your default region with this command: aws configure set default.region us-east-1 _(from commercial Amazon Web Services Regions outside of China)_ or aws configure set default.region cn-northwest-1 _(from Amazon Web Services Regions in China)_
//!   - Use the following parameter with each command to specify the endpoint: --region us-east-1 _(from commercial Amazon Web Services Regions outside of China)_ or --region cn-northwest-1 _(from Amazon Web Services Regions in China)_
//!
//! __How examples are presented__
//!
//! The JSON returned by the Organizations service as response to your requests arrives as a single long string without line breaks or formatting whitespace. The examples in this guide include both line breaks and whitespace to improve readability. When example input parameters also would result in long strings that would extend beyond the screen, we insert line breaks to enhance readability. Always submit the input as a single JSON text string.
//!
//! __Recording API Requests__
//!
//! Organizations supports CloudTrail, a service that records Amazon Web Services API calls for your Amazon Web Services account and delivers log files to an Amazon S3 bucket. By using information collected by CloudTrail, you can determine which requests the Organizations service received, who made the request and when, and so on. For more about Organizations and its support for CloudTrail, see [Logging Organizations API calls with CloudTrail](https://docs.aws.amazon.com/organizations/latest/userguide/orgs_security_incident-response.html#orgs_cloudtrail-integration) in the _Organizations User Guide_. To learn more about CloudTrail, including how to turn it on and find your log files, see the [CloudTrail User Guide](https://docs.aws.amazon.com/awscloudtrail/latest/userguide/what_is_cloud_trail_top_level.html).
//!
//! ## Getting Started
//!
//! > Examples are available for many services and operations, check out the
//! > [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).
//!
//! The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
//! as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-organizations` to
//! your project, add the following to your **Cargo.toml** file:
//!
//! ```toml
//! [dependencies]
//! aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
//! aws-sdk-organizations = "1.119.1"
//! tokio = { version = "1", features = ["full"] }
//! ```
//!
//! Then in code, a client can be created with the following:
//!
//! ```rust,no_run
//! use aws_sdk_organizations as organizations;
//!
//! #[::tokio::main]
//! async fn main() -> Result<(), organizations::Error> {
//!     let config = aws_config::load_from_env().await;
//!     let client = aws_sdk_organizations::Client::new(&config);
//!
//!     // ... make some calls with the client
//!
//!     Ok(())
//! }
//! ```
//!
//! See the [client documentation](https://docs.rs/aws-sdk-organizations/latest/aws_sdk_organizations/client/struct.Client.html)
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
//! offered by AWS Organizations. The return value of each of these methods is a "fluent builder",
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

/// Client for calling AWS Organizations.
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
/// let client = aws_sdk_organizations::Client::new(&config);
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
/// let config = aws_sdk_organizations::config::Builder::from(&sdk_config)
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
/// For example, the [`AcceptHandshake`](crate::operation::accept_handshake) operation has
/// a [`Client::accept_handshake`], function which returns a builder for that operation.
/// The fluent builder ultimately has a `send()` function that returns an async future that
/// returns a result, as illustrated below:
///
/// ```rust,ignore
/// let result = client.accept_handshake()
///     .handshake_id("example")
///     .send()
///     .await;
/// ```
///
/// The underlying HTTP requests that get made by this can be modified with the `customize_operation`
/// function on the fluent builder. See the [`customize`](crate::client::customize) module for more
/// information.
pub mod client;

/// Configuration for AWS Organizations.
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

mod json_errors;

mod serde_util;

#[doc(inline)]
pub use client::Client;
