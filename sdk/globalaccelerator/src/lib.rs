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
//! This is the _Global Accelerator API Reference_. This guide is for developers who need detailed information about Global Accelerator API actions, data types, and errors. For more information about Global Accelerator features, see the [Global Accelerator Developer Guide](https://docs.aws.amazon.com/global-accelerator/latest/dg/what-is-global-accelerator.html).
//! 
//! Global Accelerator is a service in which you create _accelerators_ to improve the performance of your applications for local and global users. Depending on the type of accelerator you choose, you can gain additional benefits.
//!   - By using a standard accelerator, you can improve availability of your internet applications that are used by a global audience. With a standard accelerator, Global Accelerator directs traffic to optimal endpoints over the Amazon Web Services global network.
//!   - For other scenarios, you might choose a custom routing accelerator. With a custom routing accelerator, you can use application logic to directly map one or more users to a specific endpoint among many endpoints.
//! 
//! Global Accelerator is a global service that supports endpoints in multiple Amazon Web Services Regions but you must specify the US West (Oregon) Region to create, update, or otherwise work with accelerators. That is, for example, specify --region us-west-2 on AWS CLI commands.
//! 
//! By default, Global Accelerator provides you with static IP addresses that you associate with your accelerator. The static IP addresses are anycast from the Amazon Web Services edge network. For IPv4, Global Accelerator provides two static IPv4 addresses. For dual-stack, Global Accelerator provides a total of four addresses: two static IPv4 addresses and two static IPv6 addresses. With a standard accelerator for IPv4, instead of using the addresses that Global Accelerator provides, you can configure these entry points to be IPv4 addresses from your own IP address ranges that you bring toGlobal Accelerator (BYOIP).
//! 
//! For a standard accelerator, they distribute incoming application traffic across multiple endpoint resources in multiple Amazon Web Services Regions , which increases the availability of your applications. Endpoints for standard accelerators can be Network Load Balancers, Application Load Balancers, Amazon EC2 instances, or Elastic IP addresses that are located in one Amazon Web Services Region or multiple Amazon Web Services Regions. For custom routing accelerators, you map traffic that arrives to the static IP addresses to specific Amazon EC2 servers in endpoints that are virtual private cloud (VPC) subnets.
//! 
//! The static IP addresses remain assigned to your accelerator for as long as it exists, even if you disable the accelerator and it no longer accepts or routes traffic. However, when you _delete_ an accelerator, you lose the static IP addresses that are assigned to it, so you can no longer route traffic by using them. You can use IAM policies like tag-based permissions with Global Accelerator to limit the users who have permissions to delete an accelerator. For more information, see [Tag-based policies](https://docs.aws.amazon.com/global-accelerator/latest/dg/access-control-manage-access-tag-policies.html).
//! 
//! For standard accelerators, Global Accelerator uses the Amazon Web Services global network to route traffic to the optimal regional endpoint based on health, client location, and policies that you configure. The service reacts instantly to changes in health or configuration to ensure that internet traffic from clients is always directed to healthy endpoints.
//! 
//! For more information about understanding and using Global Accelerator, see the [Global Accelerator Developer Guide](https://docs.aws.amazon.com/global-accelerator/latest/dg/what-is-global-accelerator.html).
//! 
//! ## Getting Started
//! 
//! > Examples are available for many services and operations, check out the
//! > [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).
//! 
//! The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
//! as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-globalaccelerator` to
//! your project, add the following to your **Cargo.toml** file:
//! 
//! ```toml
//! [dependencies]
//! aws-config = "0.0.0-smithy-rs-head"
//! aws-sdk-globalaccelerator = "0.25.0"
//! tokio = { version = "1", features = ["full"] }
//! ```
//! 
//! Then in code, a client can be created with the following:
//! 
//! ```rust,no_run
//! use aws_sdk_globalaccelerator as globalaccelerator;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), globalaccelerator::Error> {
//!     let config = aws_config::load_from_env().await;
//!     let client = globalaccelerator::Client::new(&config);
//! 
//!     // ... make some calls with the client
//! 
//!     Ok(())
//! }
//! ```
//! 
//! See the [client documentation](https://docs.rs/aws-sdk-globalaccelerator/latest/aws_sdk_globalaccelerator/client/struct.Client.html)
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
//! 
//! # Examples
//! Examples can be found [here](https://github.com/awslabs/aws-sdk-rust/tree/main/examples/globalaccelerator).


// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use error_meta::Error;

#[doc(inline)]
pub use config::Config;

pub use aws_credential_types::Credentials;

pub use aws_types::region::Region;

pub(crate) static API_METADATA: aws_http::user_agent::ApiMetadata =
                    aws_http::user_agent::ApiMetadata::new("globalaccelerator", crate::PKG_VERSION);

pub use aws_types::app_name::AppName;

pub use aws_smithy_http::endpoint::Endpoint;


/// Crate version number.
                pub static PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Client and fluent builders for calling AWS Global Accelerator.
pub mod client;

/// Configuration for AWS Global Accelerator.
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

mod idempotency_token;

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

