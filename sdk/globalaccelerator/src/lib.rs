#![allow(deprecated)]
#![allow(clippy::module_inception)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::wrong_self_convention)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::blacklisted_name)]
#![allow(clippy::vec_init_then_push)]
#![allow(clippy::type_complexity)]
#![allow(rustdoc::bare_urls)]
#![warn(missing_docs)]
//! <fullname>Global Accelerator</fullname>
//! <p>This is the <i>Global Accelerator API Reference</i>. This guide is for developers who need detailed information about
//! Global Accelerator API actions, data types, and errors. For more information about Global Accelerator features, see the
//! <a href="https://docs.aws.amazon.com/global-accelerator/latest/dg/what-is-global-accelerator.html">Global Accelerator Developer Guide</a>.</p>
//! <p>Global Accelerator is a service in which you create <i>accelerators</i> to improve the performance
//! of your applications for local and global users. Depending on the type of accelerator you choose, you can
//! gain additional benefits. </p>
//! <ul>
//! <li>
//! <p>By using a standard accelerator, you can improve availability of your internet applications
//! that are used by a global audience. With a standard accelerator, Global Accelerator directs traffic to optimal endpoints over the Amazon Web Services
//! global network. </p>
//! </li>
//! <li>
//! <p>For other scenarios, you might choose a custom routing accelerator. With a custom routing accelerator, you
//! can use application logic to directly map one or more users to a specific endpoint among many endpoints.</p>
//! </li>
//! </ul>
//! <important>
//! <p>Global Accelerator is a global service that supports endpoints in multiple Amazon Web Services Regions but you must specify the
//! US West (Oregon) Region to create, update, or otherwise work with accelerators.  That is, for example, specify <code>--region us-west-2</code>
//! on AWS CLI commands.</p>
//! </important>
//!
//!
//! <p>By default, Global Accelerator provides you with static IP addresses that you associate with your accelerator. The static IP addresses
//! are anycast from the Amazon Web Services edge network. For IPv4, Global Accelerator provides two static IPv4 addresses. For dual-stack,
//! Global Accelerator provides a total of four addresses: two static IPv4 addresses and two static IPv6 addresses.
//! With a standard accelerator for IPv4, instead of using the addresses that Global Accelerator provides, you can configure
//! these entry points to be IPv4 addresses from your own IP address ranges that you bring toGlobal Accelerator (BYOIP). </p>
//!
//!
//! <p>For a standard accelerator,
//! they distribute incoming application traffic across multiple endpoint resources in multiple Amazon Web Services Regions , which increases
//! the availability of your applications. Endpoints for standard accelerators can be Network Load Balancers, Application Load Balancers,
//! Amazon EC2 instances, or Elastic IP addresses that are located in one Amazon Web Services Region or multiple Amazon Web Services Regions. For custom routing
//! accelerators, you map traffic that arrives to the static IP addresses to specific Amazon EC2 servers in endpoints that
//! are virtual private cloud (VPC) subnets.</p>
//!
//! <important>
//! <p>The static IP addresses remain assigned to your accelerator for as long as it exists, even if you
//! disable the accelerator and it no longer accepts or routes traffic. However, when you
//! <i>delete</i> an accelerator, you lose the static IP addresses that
//! are assigned to it, so you can no longer route traffic by using them. You can use
//! IAM policies like tag-based permissions with Global Accelerator to limit the users who have
//! permissions to delete an accelerator. For more information, see <a href="https://docs.aws.amazon.com/global-accelerator/latest/dg/access-control-manage-access-tag-policies.html">Tag-based policies</a>.</p>
//! </important>
//! <p>For standard accelerators, Global Accelerator uses the Amazon Web Services global network to route traffic to the optimal regional endpoint based
//! on health, client location, and policies that you configure. The service reacts instantly to
//! changes in health or configuration to ensure that internet traffic from clients is always
//! directed to healthy endpoints.</p>
//! <p>For more information about understanding and using Global Accelerator, see the
//! <a href="https://docs.aws.amazon.com/global-accelerator/latest/dg/what-is-global-accelerator.html">Global Accelerator Developer Guide</a>.</p>
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

mod aws_endpoint;
/// Client and fluent builders for calling the service.
pub mod client;
/// Configuration for the service.
pub mod config;
/// Errors that can occur when calling the service.
pub mod error;
mod error_meta;
mod idempotency_token;
/// Input structures for operations.
pub mod input;
mod json_deser;
mod json_errors;
mod json_ser;
/// Generated accessors for nested fields
pub mod lens;
pub mod middleware;
/// Data structures used by operation inputs/outputs.
pub mod model;
mod no_credentials;
/// All operations that this crate can perform.
pub mod operation;
mod operation_deser;
mod operation_ser;
/// Output structures for operations.
pub mod output;
/// Paginators for the service
pub mod paginator;
/// Crate version number.
pub static PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Re-exported types from supporting crates.
pub mod types {
    pub use aws_smithy_http::result::SdkError;
    pub use aws_smithy_types::DateTime;
}
static API_METADATA: aws_http::user_agent::ApiMetadata =
    aws_http::user_agent::ApiMetadata::new("globalaccelerator", PKG_VERSION);
pub use aws_smithy_http::endpoint::Endpoint;
pub use aws_smithy_types::retry::RetryConfig;
pub use aws_types::app_name::AppName;
pub use aws_types::region::Region;
pub use aws_types::Credentials;
#[doc(inline)]
pub use client::Client;
