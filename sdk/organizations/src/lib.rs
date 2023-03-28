#![allow(deprecated)]
#![allow(clippy::module_inception)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::wrong_self_convention)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::blacklisted_name)]
#![allow(clippy::vec_init_then_push)]
#![allow(clippy::type_complexity)]
#![allow(clippy::needless_return)]
#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(rustdoc::bare_urls)]
#![warn(missing_docs)]
//! **Please Note: The SDK is currently in Developer Preview and is intended strictly for
//! feedback purposes only. Do not use this SDK for production workloads.**
//!
//! Organizations is a web service that enables you to consolidate your multiple Amazon Web Services accounts into an _organization_ and centrally manage your accounts and their resources.
//!
//! This guide provides descriptions of the Organizations operations. For more information about using this service, see the [Organizations User Guide](https://docs.aws.amazon.com/organizations/latest/userguide/orgs_introduction.html).
//!
//! __Support and feedback for Organizations__
//!
//! We welcome your feedback. Send your comments to [feedback-awsorganizations@amazon.com](mailto:feedback-awsorganizations@amazon.com) or post your feedback and questions in the [Organizations support forum](http://forums.aws.amazon.com/forum.jspa?forumID=219). For more information about the Amazon Web Services support forums, see [Forums Help](http://forums.aws.amazon.com/help.jspa).
//!
//! __Endpoint to call When using the CLI or the Amazon Web Services SDK__
//!
//! For the current release of Organizations, specify the us-east-1 region for all Amazon Web Services API and CLI calls made from the commercial Amazon Web Services Regions outside of China. If calling from one of the Amazon Web Services Regions in China, then specify cn-northwest-1. You can do this in the CLI by using these parameters and commands:
//!   - Use the following parameter with each command to specify both the endpoint and its region: --endpoint-url https://organizations.us-east-1.amazonaws.com _(from commercial Amazon Web Services Regions outside of China)_ or --endpoint-url https://organizations.cn-northwest-1.amazonaws.com.cn _(from Amazon Web Services Regions in China)_
//!   - Use the default endpoint, but configure your default region with this command: aws configure set default.region us-east-1 _(from commercial Amazon Web Services Regions outside of China)_ or aws configure set default.region cn-northwest-1 _(from Amazon Web Services Regions in China)_
//!   - Use the following parameter with each command to specify the endpoint: --region us-east-1 _(from commercial Amazon Web Services Regions outside of China)_ or --region cn-northwest-1 _(from Amazon Web Services Regions in China)_
//!
//! __Recording API Requests__
//!
//! Organizations supports CloudTrail, a service that records Amazon Web Services API calls for your Amazon Web Services account and delivers log files to an Amazon S3 bucket. By using information collected by CloudTrail, you can determine which requests the Organizations service received, who made the request and when, and so on. For more about Organizations and its support for CloudTrail, see [Logging Organizations Events with CloudTrail](https://docs.aws.amazon.com/organizations/latest/userguide/orgs_incident-response.html#orgs_cloudtrail-integration) in the _Organizations User Guide_. To learn more about CloudTrail, including how to turn it on and find your log files, see the [CloudTrail User Guide](https://docs.aws.amazon.com/awscloudtrail/latest/userguide/what_is_cloud_trail_top_level.html).
//!
//! ## Getting Started
//!
//! > Examples are available for many services and operations, check out the
//! > [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).
//!
//! The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
//! as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-organizations` to
//! your project, add the following to your **Cargo.toml** file:
//!
//! ```toml
//! [dependencies]
//! aws-config = "0.0.0-smithy-rs-head"
//! aws-sdk-organizations = "0.59.0"
//! tokio = { version = "1", features = ["full"] }
//! ```
//!
//! Then in code, a client can be created with the following:
//!
//! ```rust,no_run
//! use aws_sdk_organizations as organizations;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), organizations::Error> {
//!     let config = aws_config::load_from_env().await;
//!     let client = organizations::Client::new(&config);
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
//! * [Usage examples](https://github.com/awslabs/aws-sdk-rust/tree/main/examples)
//!
//!
//! # Crate Organization
//!
//! The entry point for most customers will be [`Client`]. [`Client`] exposes one method for each API offered
//! by the service.
//!
//! Some APIs require complex or nested arguments. These exist in [`model`](crate::model).
//!
//! Lastly, errors that can be returned by the service are contained within [`error`]. [`Error`] defines a meta
//! error encompassing all possible errors that can be returned by the service.
//!
//! The other modules within this crate are not required for normal usage.

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use error_meta::Error;

#[doc(inline)]
pub use config::Config;

pub use aws_credential_types::Credentials;

pub use aws_types::region::Region;

pub(crate) static API_METADATA: aws_http::user_agent::ApiMetadata =
    aws_http::user_agent::ApiMetadata::new("organizations", crate::PKG_VERSION);

pub use aws_types::app_name::AppName;

pub use aws_smithy_http::endpoint::Endpoint;

/// Crate version number.
pub static PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Client and fluent builders for calling AWS Organizations.
pub mod client;

/// Configuration for AWS Organizations.
pub mod config;

/// Endpoint resolution functionality.
pub mod endpoint;

/// All error types that operations can return. Documentation on these types is copied from the model.
pub mod error;

mod error_meta;

/// Input structures for operations. Documentation on these types is copied from the model.
pub mod input;

/// Data structures used by operation inputs/outputs.
pub mod model;

/// All operations that this crate can perform.
pub mod operation;

/// Output structures for operations. Documentation on these types is copied from the model.
pub mod output;

/// Data primitives referenced by other data types.
pub mod types;

///
pub mod middleware;

///
mod no_credentials;

/// Paginators for the service
pub mod paginator;

mod lens;

pub(crate) mod protocol_serde;

mod endpoint_lib;

mod json_errors;

#[doc(inline)]
pub use client::Client;
