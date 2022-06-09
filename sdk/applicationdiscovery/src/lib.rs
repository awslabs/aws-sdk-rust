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
//! <fullname>Amazon Web Services Application Discovery Service</fullname>
//!
//! <p>Amazon Web Services Application Discovery Service helps you plan application migration projects. It
//! automatically identifies servers, virtual machines (VMs), and network dependencies in your
//! on-premises data centers. For more information, see the <a href="http://aws.amazon.com/application-discovery/faqs/">Amazon Web Services Application Discovery Service FAQ</a>.
//! Application Discovery Service offers three ways of performing discovery and
//! collecting data about your on-premises servers:</p>
//!
//! <ul>
//! <li>
//! <p>
//! <b>Agentless discovery</b> is recommended for environments
//! that use VMware vCenter Server. This mode doesn't require you to install an agent on each
//! host. It does not work in non-VMware environments.</p>
//!
//! <ul>
//! <li>
//! <p>Agentless discovery gathers server information regardless of the operating
//! systems, which minimizes the time required for initial on-premises infrastructure
//! assessment.</p>
//! </li>
//! <li>
//! <p>Agentless discovery doesn't collect information about network dependencies, only
//! agent-based discovery collects that information.</p>
//! </li>
//! </ul>
//! </li>
//! </ul>
//!
//! <ul>
//! <li>
//! <p>
//! <b>Agent-based discovery</b> collects a richer set of data
//! than agentless discovery by using the Amazon Web Services Application Discovery Agent, which you install
//! on one or more hosts in your data center.</p>
//!
//! <ul>
//! <li>
//! <p> The agent captures infrastructure and application information, including an
//! inventory of running processes, system performance information, resource utilization,
//! and network dependencies.</p>
//! </li>
//! <li>
//! <p>The information collected by agents is secured at rest and in transit to the
//! Application Discovery Service database in the cloud. </p>
//! </li>
//! </ul>
//! </li>
//! </ul>
//!
//! <ul>
//! <li>
//! <p>
//! <b>Amazon Web Services Partner Network (APN) solutions</b> integrate with
//! Application Discovery Service, enabling you to import details of your on-premises
//! environment directly into Migration Hub without using the discovery connector or discovery
//! agent.</p>
//!
//! <ul>
//! <li>
//! <p>Third-party application discovery tools can query Amazon Web Services Application Discovery
//! Service, and they can write to the Application Discovery Service database using the
//! public API.</p>
//! </li>
//! <li>
//! <p>In this way, you can import data into Migration Hub and view it, so that you can
//! associate applications with servers and track migrations.</p>
//! </li>
//! </ul>
//! </li>
//! </ul>
//!
//!
//! <p>
//! <b>Recommendations</b>
//! </p>
//! <p>We recommend that you use agent-based discovery for non-VMware environments, and
//! whenever you want to collect information about network dependencies. You can run agent-based
//! and agentless discovery simultaneously. Use agentless discovery to complete the initial
//! infrastructure assessment quickly, and then install agents on select hosts to collect
//! additional information.</p>
//!
//! <p>
//! <b>Working With This Guide</b>
//! </p>
//!
//! <p>This API reference provides descriptions, syntax, and usage examples for each of the
//! actions and data types for Application Discovery Service. The topic for each action shows the
//! API request parameters and the response. Alternatively, you can use one of the Amazon Web Services SDKs to
//! access an API that is tailored to the programming language or platform that you're using. For
//! more information, see <a href="http://aws.amazon.com/tools/#SDKs">Amazon Web Services
//! SDKs</a>.</p>
//!
//! <note>
//! <ul>
//! <li>
//! <p>Remember that you must set your Migration Hub home region before you call any of
//! these APIs.</p>
//! </li>
//! <li>
//! <p>You must make API calls for write actions (create, notify, associate, disassociate,
//! import, or put) while in your home region, or a <code>HomeRegionNotSetException</code>
//! error is returned.</p>
//! </li>
//! <li>
//! <p>API calls for read actions (list, describe, stop, and delete) are permitted outside
//! of your home region.</p>
//! </li>
//! <li>
//! <p>Although it is unlikely, the Migration Hub home region could change. If you call
//! APIs outside the home region, an <code>InvalidInputException</code> is returned.</p>
//! </li>
//! <li>
//! <p>You must call <code>GetHomeRegion</code> to obtain the latest Migration Hub home
//! region.</p>
//! </li>
//! </ul>
//! </note>
//!
//! <p>This guide is intended for use with the <a href="http://docs.aws.amazon.com/application-discovery/latest/userguide/">Amazon Web Services Application
//! Discovery Service User Guide</a>.</p>
//!
//! <important>
//! <p>All data is handled according to the <a href="http://aws.amazon.com/privacy/">Amazon Web Services
//! Privacy Policy</a>. You can operate Application Discovery Service offline to inspect
//! collected data before it is shared with the service.</p>
//! </important>
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
    aws_http::user_agent::ApiMetadata::new("applicationdiscoveryservice", PKG_VERSION);
pub use aws_smithy_http::endpoint::Endpoint;
pub use aws_smithy_types::retry::RetryConfig;
pub use aws_types::app_name::AppName;
pub use aws_types::region::Region;
pub use aws_types::Credentials;
#[doc(inline)]
pub use client::Client;
