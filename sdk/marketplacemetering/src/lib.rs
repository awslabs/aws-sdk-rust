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
//! This reference provides descriptions of the low-level AWS Marketplace Metering Service API.
//! 
//! AWS Marketplace sellers can use this API to submit usage data for custom usage dimensions.
//! 
//! For information on the permissions you need to use this API, see [AWS Marketplace metering and entitlement API permissions](https://docs.aws.amazon.com/marketplace/latest/userguide/iam-user-policy-for-aws-marketplace-actions.html) in the _AWS Marketplace Seller Guide._
//! 
//! __Submitting Metering Records__
//!   - _MeterUsage_ - Submits the metering record for an AWS Marketplace product. MeterUsage is called from an EC2 instance or a container running on EKS or ECS.
//!   - _BatchMeterUsage_ - Submits the metering record for a set of customers. BatchMeterUsage is called from a software-as-a-service (SaaS) application.
//! 
//! __Accepting New Customers__
//!   - _ResolveCustomer_ - Called by a SaaS application during the registration process. When a buyer visits your website during the registration process, the buyer submits a Registration Token through the browser. The Registration Token is resolved through this API to obtain a CustomerIdentifier along with the CustomerAWSAccountId and ProductCode.
//! 
//! __Entitlement and Metering for Paid Container Products__
//!   - Paid container software products sold through AWS Marketplace must integrate with the AWS Marketplace Metering Service and call the RegisterUsage operation for software entitlement and metering. Free and BYOL products for Amazon ECS or Amazon EKS aren't required to call RegisterUsage, but you can do so if you want to receive usage data in your seller reports. For more information on using the RegisterUsage operation, see [Container-Based Products](https://docs.aws.amazon.com/marketplace/latest/userguide/container-based-products.html).
//! 
//! BatchMeterUsage API calls are captured by AWS CloudTrail. You can use Cloudtrail to verify that the SaaS metering records that you sent are accurate by searching for records with the eventName of BatchMeterUsage. You can also use CloudTrail to audit records over time. For more information, see the _ [AWS CloudTrail User Guide](http://docs.aws.amazon.com/awscloudtrail/latest/userguide/cloudtrail-concepts.html)._
//! 
//! ## Getting Started
//! 
//! > Examples are available for many services and operations, check out the
//! > [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).
//! 
//! The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
//! as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-marketplacemetering` to
//! your project, add the following to your **Cargo.toml** file:
//! 
//! ```toml
//! [dependencies]
//! aws-config = "0.0.0-smithy-rs-head"
//! aws-sdk-marketplacemetering = "0.25.0"
//! tokio = { version = "1", features = ["full"] }
//! ```
//! 
//! Then in code, a client can be created with the following:
//! 
//! ```rust,no_run
//! use aws_sdk_marketplacemetering as marketplacemetering;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), marketplacemetering::Error> {
//!     let config = aws_config::load_from_env().await;
//!     let client = marketplacemetering::Client::new(&config);
//! 
//!     // ... make some calls with the client
//! 
//!     Ok(())
//! }
//! ```
//! 
//! See the [client documentation](https://docs.rs/aws-sdk-marketplacemetering/latest/aws_sdk_marketplacemetering/client/struct.Client.html)
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
                    aws_http::user_agent::ApiMetadata::new("marketplacemetering", crate::PKG_VERSION);

pub use aws_types::app_name::AppName;

pub use aws_smithy_http::endpoint::Endpoint;


/// Crate version number.
                pub static PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Client and fluent builders for calling AWSMarketplace Metering.
pub mod client;

/// Configuration for AWSMarketplace Metering.
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

