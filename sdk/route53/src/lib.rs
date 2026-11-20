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
//! Amazon Route 53 is a highly available and scalable Domain Name System (DNS) web service. Route 53 performs four main functions:
//!   - __Domain registration__ – Route 53 lets you register domain names such as example.com.
//!   - __Domain Name System (DNS) service__ – Route 53 translates friendly domain names into IP addresses using a global network of authoritative DNS servers, and provides DNS resolution within your VPCs for private hosted zones.
//!   - __Health checking__ – Route 53 sends automated requests to your application, over the internet or from within your VPC for private endpoints, to verify that it's reachable, available, and functional.
//!   - __Resolver__ – Route 53 Resolver lets you forward DNS queries from a VPC that you created using Amazon VPC to DNS resolvers in your network, and from your network to resolvers in your VPC.
//!
//! You can use the API to create and manage the following resources:
//!
//! __Public and Private Hosted Zones__
//!
//! A _public hosted zone_ is a container that holds information about how you want to route traffic on the internet for a domain, such as example.com, and its subdomains. A _private hosted zone_ is a container that holds information about how you want to route traffic for a domain and its subdomains within one or more VPCs that you created with the Amazon VPC service.
//!
//! __Reusable Delegation Sets__
//!
//! By default, each hosted zone that you create gets a different set of four name servers—a different delegation set. If you create a lot of hosted zones, maintaining different delegation sets can be difficult and time consuming. Route 53 lets you create a delegation set that you can reuse with multiple hosted zones.
//!
//! __Resource Record Sets__
//!
//! After you create a hosted zone for your domain, such as example.com, you create resource record sets to tell the Domain Name System (DNS) how to route traffic for that domain.
//!
//! __Traffic Policies and Traffic Policy Instances__
//!
//! You can create complex routing configurations, known as traffic policies, that use weighted, latency, failover, and geolocation resource record sets. You can then associate a traffic policy with a domain name or subdomain name, such as www.example.com, by creating a traffic policy instance. When users submit DNS queries for the domain or subdomain, Route 53 responds based on the traffic policy that you used to create the traffic policy instance.
//!
//! __Health Checks__
//!
//! Route 53 health checks monitor the health and performance of your web applications, web servers, and other resources. At regular intervals that you specify, Route 53 submits automated requests to your application, server, or other resource, over the internet or from within your VPC for private endpoints, to verify that it's reachable, available, and functional.
//!
//! __Domain Registrations__
//!
//! When you want to get a new domain name, such as example.com, you can register it with Route 53. You can also transfer the registration for existing domains from other registrars to Route 53.
//!
//! __DNS–DNSSEC__
//!
//! You can protect your domain from DNS spoofing or a man-in-the-middle attack, by configuring Domain Name System Security Extensions (DNSSEC).
//!
//! __Query Logs__
//!
//! You can configure Route 53 to log information about the DNS queries that Route 53 receives, such as the domain or subdomain that was requested, the date and time of the request, and the DNS record type (such as A or AAAA). You can also configure Route 53 Resolver to log information about the DNS queries that originate in Amazon VPCs.
//!
//! __Outbound and Inbound Endpoints, and Rules__
//!
//! You can configure Route 53 Resolver to forward DNS queries from your VPC to your network or vice versa. DNS queries pass through an outbound endpoint on their way from a VPC to your network, and they pass through an inbound endpoint on their way from your network to a VPC. For outbound queries, rules let you specify the domain names that you want to forward to your network and the IP addresses of the DNS resolvers in your network.
//!
//! __Tags__
//!
//! A tag is a label that you assign to an Amazon Web Services resource. Each tag consists of a key and a value, both of which you define. You can use tags for a variety of purposes; one common use is to categorize and track your Route 53 costs.
//!
//! __Profiles__
//!
//! A _profile_ is a named container that groups DNS configurations—such as private hosted zone associations, Resolver rules, and DNS Firewall rule groups—into a single reusable unit. You can associate a profile with multiple VPCs and share it across Amazon Web Services accounts so that each VPC inherits the same DNS settings without requiring individual configuration.
//!
//! __DNS Firewall__
//!
//! A _DNS Firewall rule group_ is an ordered collection of rules that each pair a domain list with an action—allow, block, or alert. You associate rule groups with your VPCs to filter outbound DNS queries made by VPC resources, preventing connections to known-malicious domains or restricting access to only approved domains.
//!
//!
//! You can also use the Route 53 API to get the current limit on Route 53 objects that you can create, such as hosted zones and health checks.
//!
//! For information about Route 53 concepts and about how to use the Route 53 console, see the [Amazon Route 53 Developer Guide](https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/).
//!
//! ## Getting Started
//!
//! > Examples are available for many services and operations, check out the
//! > [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).
//!
//! The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
//! as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-route53` to
//! your project, add the following to your **Cargo.toml** file:
//!
//! ```toml
//! [dependencies]
//! aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
//! aws-sdk-route53 = "1.138.0"
//! tokio = { version = "1", features = ["full"] }
//! ```
//!
//! Then in code, a client can be created with the following:
//!
//! ```rust,no_run
//! use aws_sdk_route53 as route53;
//!
//! #[::tokio::main]
//! async fn main() -> Result<(), route53::Error> {
//!     let config = aws_config::load_from_env().await;
//!     let client = aws_sdk_route53::Client::new(&config);
//!
//!     // ... make some calls with the client
//!
//!     Ok(())
//! }
//! ```
//!
//! See the [client documentation](https://docs.rs/aws-sdk-route53/latest/aws_sdk_route53/client/struct.Client.html)
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
//! offered by Amazon Route 53. The return value of each of these methods is a "fluent builder",
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

/// Client for calling Amazon Route 53.
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
/// let client = aws_sdk_route53::Client::new(&config);
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
/// let config = aws_sdk_route53::config::Builder::from(&sdk_config)
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
/// For example, the [`ActivateKeySigningKey`](crate::operation::activate_key_signing_key) operation has
/// a [`Client::activate_key_signing_key`], function which returns a builder for that operation.
/// The fluent builder ultimately has a `send()` function that returns an async future that
/// returns a result, as illustrated below:
///
/// ```rust,ignore
/// let result = client.activate_key_signing_key()
///     .hosted_zone_id("example")
///     .send()
///     .await;
/// ```
///
/// The underlying HTTP requests that get made by this can be modified with the `customize_operation`
/// function on the fluent builder. See the [`customize`](crate::client::customize) module for more
/// information.
/// # Waiters
///
/// This client provides `wait_until` methods behind the [`Waiters`](crate::client::Waiters) trait.
/// To use them, simply import the trait, and then call one of the `wait_until` methods. This will
/// return a waiter fluent builder that takes various parameters, which are documented on the builder
/// type. Once parameters have been provided, the `wait` method can be called to initiate waiting.
///
/// For example, if there was a `wait_until_thing` method, it could look like:
/// ```rust,ignore
/// let result = client.wait_until_thing()
///     .thing_id("someId")
///     .wait(Duration::from_secs(120))
///     .await;
/// ```
pub mod client;

/// Configuration for Amazon Route 53.
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

pub(crate) mod client_idempotency_token;

mod idempotency_token;

mod observability_feature;

pub(crate) mod protocol_serde;

mod route53_resource_id_preprocessor;

mod sdk_feature_tracker;

mod serialization_settings;

mod endpoint_lib;

mod lens;

mod serde_util;

/// Supporting types for waiters.
///
/// Note: to use waiters, import the [`Waiters`](crate::client::Waiters) trait, which adds methods prefixed with `wait_until` to the client.
pub mod waiters;

mod rest_xml_wrapped_errors;

#[doc(inline)]
pub use client::Client;
