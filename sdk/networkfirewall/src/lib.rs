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
#![allow(clippy::deprecated_semver)]
#![allow(rustdoc::bare_urls)]
#![allow(rustdoc::redundant_explicit_links)]
#![allow(rustdoc::invalid_html_tags)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
//! This is the API Reference for Network Firewall. This guide is for developers who need detailed information about the Network Firewall API actions, data types, and errors.
//!
//! The REST API requires you to handle connection details, such as calculating signatures, handling request retries, and error handling. For general information about using the Amazon Web Services REST APIs, see [Amazon Web Services APIs](https://docs.aws.amazon.com/general/latest/gr/aws-apis.html).
//!
//! To view the complete list of Amazon Web Services Regions where Network Firewall is available, see [Service endpoints and quotas](https://docs.aws.amazon.com/general/latest/gr/network-firewall.html) in the _Amazon Web Services General Reference_.
//!
//! To access Network Firewall using the IPv4 REST API endpoint: https://network-firewall..amazonaws.com
//!
//! To access Network Firewall using the Dualstack (IPv4 and IPv6) REST API endpoint: https://network-firewall..aws.api
//!
//! Alternatively, you can use one of the Amazon Web Services SDKs to access an API that's tailored to the programming language or platform that you're using. For more information, see [Amazon Web Services SDKs](http://aws.amazon.com/tools/#SDKs).
//!
//! For descriptions of Network Firewall features, including and step-by-step instructions on how to use them through the Network Firewall console, see the [Network Firewall Developer Guide](https://docs.aws.amazon.com/network-firewall/latest/developerguide/).
//!
//! Network Firewall is a stateful, managed, network firewall and intrusion detection and prevention service for Amazon Virtual Private Cloud (Amazon VPC). With Network Firewall, you can filter traffic at the perimeter of your VPC. This includes filtering traffic going to and coming from an internet gateway, NAT gateway, or over VPN or Direct Connect. Network Firewall uses rules that are compatible with Suricata, a free, open source network analysis and threat detection engine. Network Firewall supports Suricata version 7.0.3. For information about Suricata, see the [Suricata website](https://suricata.io/) and the [Suricata User Guide](https://suricata.readthedocs.io/en/suricata-7.0.3/).
//!
//! You can use Network Firewall to monitor and protect your VPC traffic in a number of ways. The following are just a few examples:
//!   - Allow domains or IP addresses for known Amazon Web Services service endpoints, such as Amazon S3, and block all other forms of traffic.
//!   - Use custom lists of known bad domains to limit the types of domain names that your applications can access.
//!   - Perform deep packet inspection on traffic entering or leaving your VPC.
//!   - Use stateful protocol detection to filter protocols like HTTPS, regardless of the port used.
//!
//! To enable Network Firewall for your VPCs, you perform steps in both Amazon VPC and in Network Firewall. For information about using Amazon VPC, see [Amazon VPC User Guide](https://docs.aws.amazon.com/vpc/latest/userguide/).
//!
//! To start using Network Firewall, do the following:
//!   1. (Optional) If you don't already have a VPC that you want to protect, create it in Amazon VPC.
//!   1. In Amazon VPC, in each Availability Zone where you want to have a firewall endpoint, create a subnet for the sole use of Network Firewall.
//!   1. In Network Firewall, define the firewall behavior as follows:
//!     1. Create stateless and stateful rule groups, to define the components of the network traffic filtering behavior that you want your firewall to have.
//!     1. Create a firewall policy that uses your rule groups and specifies additional default traffic filtering behavior.
//!
//!   1. In Network Firewall, create a firewall and specify your new firewall policy and VPC subnets. Network Firewall creates a firewall endpoint in each subnet that you specify, with the behavior that's defined in the firewall policy.
//!   1. In Amazon VPC, use ingress routing enhancements to route traffic through the new firewall endpoints.
//!
//! After your firewall is established, you can add firewall endpoints for new Availability Zones by following the prior steps for the Amazon VPC setup and firewall subnet definitions. You can also add endpoints to Availability Zones that you're using in the firewall, either for the same VPC or for another VPC, by following the prior steps for the Amazon VPC setup, and defining the new VPC subnets as VPC endpoint associations.
//!
//! ## Getting Started
//!
//! > Examples are available for many services and operations, check out the
//! > [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).
//!
//! The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
//! as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-networkfirewall` to
//! your project, add the following to your **Cargo.toml** file:
//!
//! ```toml
//! [dependencies]
//! aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
//! aws-sdk-networkfirewall = "1.96.0"
//! tokio = { version = "1", features = ["full"] }
//! ```
//!
//! Then in code, a client can be created with the following:
//!
//! ```rust,no_run
//! use aws_sdk_networkfirewall as networkfirewall;
//!
//! #[::tokio::main]
//! async fn main() -> Result<(), networkfirewall::Error> {
//!     let config = aws_config::load_from_env().await;
//!     let client = aws_sdk_networkfirewall::Client::new(&config);
//!
//!     // ... make some calls with the client
//!
//!     Ok(())
//! }
//! ```
//!
//! See the [client documentation](https://docs.rs/aws-sdk-networkfirewall/latest/aws_sdk_networkfirewall/client/struct.Client.html)
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
//! offered by AWS Network Firewall. The return value of each of these methods is a "fluent builder",
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

/// Client for calling AWS Network Firewall.
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
/// let client = aws_sdk_networkfirewall::Client::new(&config);
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
/// let config = aws_sdk_networkfirewall::config::Builder::from(&sdk_config)
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
/// For example, the [`AcceptNetworkFirewallTransitGatewayAttachment`](crate::operation::accept_network_firewall_transit_gateway_attachment) operation has
/// a [`Client::accept_network_firewall_transit_gateway_attachment`], function which returns a builder for that operation.
/// The fluent builder ultimately has a `send()` function that returns an async future that
/// returns a result, as illustrated below:
///
/// ```rust,ignore
/// let result = client.accept_network_firewall_transit_gateway_attachment()
///     .transit_gateway_attachment_id("example")
///     .send()
///     .await;
/// ```
///
/// The underlying HTTP requests that get made by this can be modified with the `customize_operation`
/// function on the fluent builder. See the [`customize`](crate::client::customize) module for more
/// information.
pub mod client;

/// Configuration for AWS Network Firewall.
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

pub(crate) mod protocol_serde;

mod sdk_feature_tracker;

mod serialization_settings;

mod endpoint_lib;

mod lens;

mod serde_util;

mod json_errors;

#[doc(inline)]
pub use client::Client;
