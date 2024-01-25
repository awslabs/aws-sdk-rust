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
//! This is the _AWS HealthImaging API Reference_. AWS HealthImaging is a HIPAA-eligible service that helps health care providers and their medical imaging ISV partners store, transform, and apply machine learning to medical images. For an introduction to the service, see the [_AWS HealthImaging Developer Guide_](https://docs.aws.amazon.com/healthimaging/latest/devguide/what-is.html).
//!
//! The following sections list AWS HealthImaging API actions categorized according to functionality. Links are provided to actions within this Reference, along with links back to corresponding sections in the _AWS HealthImaging Developer Guide_ where you can view console procedures and CLI/SDK code examples.
//!
//! __Data store actions__
//!   - [CreateDatastore](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_CreateDatastore.html) – See [Creating a data store](https://docs.aws.amazon.com/healthimaging/latest/devguide/create-data-store.html).
//!   - [GetDatastore](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_GetDatastore.html) – See [Getting data store properties](https://docs.aws.amazon.com/healthimaging/latest/devguide/get-data-store.html).
//!   - [ListDatastores](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_ListDatastores.html) – See [Listing data stores](https://docs.aws.amazon.com/healthimaging/latest/devguide/list-data-stores.html).
//!   - [DeleteDatastore](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_DeleteDatastore.html) – See [Deleting a data store](https://docs.aws.amazon.com/healthimaging/latest/devguide/delete-data-store.html).
//!
//! __Import job actions__
//!   - [StartDICOMImportJob](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_StartDICOMImportJob.html) – See [Starting an import job](https://docs.aws.amazon.com/healthimaging/latest/devguide/start-dicom-import-job.html).
//!   - [GetDICOMImportJob](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_GetDICOMImportJob.html) – See [Getting import job properties](https://docs.aws.amazon.com/healthimaging/latest/devguide/get-dicom-import-job.html).
//!   - [ListDICOMImportJobs](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_ListDICOMImportJobs.html) – See [Listing import jobs](https://docs.aws.amazon.com/healthimaging/latest/devguide/list-dicom-import-jobs.html).
//!
//! __Image set access actions__
//!   - [SearchImageSets](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_SearchImageSets.html) – See [Searching image sets](https://docs.aws.amazon.com/healthimaging/latest/devguide/search-image-sets.html).
//!   - [GetImageSet](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_GetImageSet.html) – See [Getting image set properties](https://docs.aws.amazon.com/healthimaging/latest/devguide/get-image-set-properties.html).
//!   - [GetImageSetMetadata](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_GetImageSetMetadata.html) – See [Getting image set metadata](https://docs.aws.amazon.com/healthimaging/latest/devguide/get-image-set-metadata.html).
//!   - [GetImageFrame](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_GetImageFrame.html) – See [Getting image set pixel data](https://docs.aws.amazon.com/healthimaging/latest/devguide/get-image-frame.html).
//!
//! __Image set modification actions__
//!   - [ListImageSetVersions](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_ListImageSetVersions.html) – See [Listing image set versions](https://docs.aws.amazon.com/healthimaging/latest/devguide/list-image-set-versions.html).
//!   - [UpdateImageSetMetadata](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_UpdateImageSetMetadata.html) – See [Updating image set metadata](https://docs.aws.amazon.com/healthimaging/latest/devguide/update-image-set-metadata.html).
//!   - [CopyImageSet](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_CopyImageSet.html) – See [Copying an image set](https://docs.aws.amazon.com/healthimaging/latest/devguide/copy-image-set.html).
//!   - [DeleteImageSet](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_DeleteImageSet.html) – See [Deleting an image set](https://docs.aws.amazon.com/healthimaging/latest/devguide/delete-image-set.html).
//!
//! __Tagging actions__
//!   - [TagResource](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_TagResource.html) – See [Tagging a data store](https://docs.aws.amazon.com/healthimaging/latest/devguide/tag-list-untag-data-store.html) and [Tagging an image set](https://docs.aws.amazon.com/healthimaging/latest/devguide/tag-list-untag-image-set.html).
//!   - [ListTagsForResource](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_ListTagsForResource.html) – See [Tagging a data store](https://docs.aws.amazon.com/healthimaging/latest/devguide/tag-list-untag-data-store.html) and [Tagging an image set](https://docs.aws.amazon.com/healthimaging/latest/devguide/tag-list-untag-image-set.html).
//!   - [UntagResource](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_UntagResource.html) – See [Tagging a data store](https://docs.aws.amazon.com/healthimaging/latest/devguide/tag-list-untag-data-store.html) and [Tagging an image set](https://docs.aws.amazon.com/healthimaging/latest/devguide/tag-list-untag-image-set.html).
//!
//! ## Getting Started
//!
//! > Examples are available for many services and operations, check out the
//! > [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).
//!
//! The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
//! as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-medicalimaging` to
//! your project, add the following to your **Cargo.toml** file:
//!
//! ```toml
//! [dependencies]
//! aws-config = { version = "1.1.4", features = ["behavior-version-latest"] }
//! aws-sdk-medicalimaging = "1.12.0"
//! tokio = { version = "1", features = ["full"] }
//! ```
//!
//! Then in code, a client can be created with the following:
//!
//! ```rust,no_run
//! use aws_sdk_medicalimaging as medicalimaging;
//!
//! #[::tokio::main]
//! async fn main() -> Result<(), medicalimaging::Error> {
//!     let config = aws_config::load_from_env().await;
//!     let client = aws_sdk_medicalimaging::Client::new(&config);
//!
//!     // ... make some calls with the client
//!
//!     Ok(())
//! }
//! ```
//!
//! See the [client documentation](https://docs.rs/aws-sdk-medicalimaging/latest/aws_sdk_medicalimaging/client/struct.Client.html)
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
//! offered by AWS Health Imaging. The return value of each of these methods is a "fluent builder",
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

/// Client for calling AWS Health Imaging.
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
/// let client = aws_sdk_medicalimaging::Client::new(&config);
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
/// let config = aws_sdk_medicalimaging::config::Builder::from(&sdk_config)
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
/// For example, the [`CopyImageSet`](crate::operation::copy_image_set) operation has
/// a [`Client::copy_image_set`], function which returns a builder for that operation.
/// The fluent builder ultimately has a `send()` function that returns an async future that
/// returns a result, as illustrated below:
///
/// ```rust,ignore
/// let result = client.copy_image_set()
///     .datastore_id("example")
///     .send()
///     .await;
/// ```
///
/// The underlying HTTP requests that get made by this can be modified with the `customize_operation`
/// function on the fluent builder. See the [`customize`](crate::client::customize) module for more
/// information.
pub mod client;

/// Configuration for AWS Health Imaging.
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

pub(crate) mod client_idempotency_token;

mod idempotency_token;

pub(crate) mod protocol_serde;

mod serialization_settings;

mod endpoint_lib;

mod lens;

mod serde_util;

mod json_errors;

#[doc(inline)]
pub use client::Client;
