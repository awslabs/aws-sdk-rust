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
//! AppConfig Data provides the data plane APIs your application uses to retrieve configuration data. Here's how it works:
//! 
//! Your application retrieves configuration data by first establishing a configuration session using the AppConfig Data StartConfigurationSession API action. Your session's client then makes periodic calls to GetLatestConfiguration to check for and retrieve the latest data available.
//! 
//! When calling StartConfigurationSession, your code sends the following information:
//!   - Identifiers (ID or name) of an AppConfig application, environment, and configuration profile that the session tracks.
//!   - (Optional) The minimum amount of time the session's client must wait between calls to GetLatestConfiguration.
//! 
//! In response, AppConfig provides an InitialConfigurationToken to be given to the session's client and used the first time it calls GetLatestConfiguration for that session.
//! 
//! When calling GetLatestConfiguration, your client code sends the most recent ConfigurationToken value it has and receives in response:
//!   - NextPollConfigurationToken: the ConfigurationToken value to use on the next call to GetLatestConfiguration.
//!   - NextPollIntervalInSeconds: the duration the client should wait before making its next call to GetLatestConfiguration. This duration may vary over the course of the session, so it should be used instead of the value sent on the StartConfigurationSession call.
//!   - The configuration: the latest data intended for the session. This may be empty if the client already has the latest version of the configuration.
//! 
//! For more information and to view example CLI commands that show how to retrieve a configuration using the AppConfig Data StartConfigurationSession and GetLatestConfiguration API actions, see [Receiving the configuration](http://docs.aws.amazon.com/appconfig/latest/userguide/appconfig-retrieving-the-configuration) in the _AppConfig User Guide_.
//! 
//! ## Getting Started
//! 
//! > Examples are available for many services and operations, check out the
//! > [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).
//! 
//! The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
//! as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-appconfigdata` to
//! your project, add the following to your **Cargo.toml** file:
//! 
//! ```toml
//! [dependencies]
//! aws-config = "0.0.0-smithy-rs-head"
//! aws-sdk-appconfigdata = "0.25.0"
//! tokio = { version = "1", features = ["full"] }
//! ```
//! 
//! Then in code, a client can be created with the following:
//! 
//! ```rust,no_run
//! use aws_sdk_appconfigdata as appconfigdata;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), appconfigdata::Error> {
//!     let config = aws_config::load_from_env().await;
//!     let client = appconfigdata::Client::new(&config);
//! 
//!     // ... make some calls with the client
//! 
//!     Ok(())
//! }
//! ```
//! 
//! See the [client documentation](https://docs.rs/aws-sdk-appconfigdata/latest/aws_sdk_appconfigdata/client/struct.Client.html)
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
                    aws_http::user_agent::ApiMetadata::new("appconfigdata", crate::PKG_VERSION);

pub use aws_types::app_name::AppName;

pub use aws_smithy_http::endpoint::Endpoint;


/// Crate version number.
                pub static PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Client and fluent builders for calling AWS AppConfig Data.
pub mod client;

/// Configuration for AWS AppConfig Data.
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

pub(crate) mod protocol_serde;

mod endpoint_lib;

mod json_errors;

#[doc(inline)]
pub use client::Client;

