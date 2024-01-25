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
//! __Overview__
//!
//! This is the CodePipeline API Reference. This guide provides descriptions of the actions and data types for CodePipeline. Some functionality for your pipeline can only be configured through the API. For more information, see the [CodePipeline User Guide](https://docs.aws.amazon.com/codepipeline/latest/userguide/welcome.html).
//!
//! You can use the CodePipeline API to work with pipelines, stages, actions, and transitions.
//!
//! _Pipelines_ are models of automated release processes. Each pipeline is uniquely named, and consists of stages, actions, and transitions.
//!
//! You can work with pipelines by calling:
//!   - CreatePipeline, which creates a uniquely named pipeline.
//!   - DeletePipeline, which deletes the specified pipeline.
//!   - GetPipeline, which returns information about the pipeline structure and pipeline metadata, including the pipeline Amazon Resource Name (ARN).
//!   - GetPipelineExecution, which returns information about a specific execution of a pipeline.
//!   - GetPipelineState, which returns information about the current state of the stages and actions of a pipeline.
//!   - ListActionExecutions, which returns action-level details for past executions. The details include full stage and action-level details, including individual action duration, status, any errors that occurred during the execution, and input and output artifact location details.
//!   - ListPipelines, which gets a summary of all of the pipelines associated with your account.
//!   - ListPipelineExecutions, which gets a summary of the most recent executions for a pipeline.
//!   - StartPipelineExecution, which runs the most recent revision of an artifact through the pipeline.
//!   - StopPipelineExecution, which stops the specified pipeline execution from continuing through the pipeline.
//!   - UpdatePipeline, which updates a pipeline with edits or changes to the structure of the pipeline.
//!
//! Pipelines include _stages_. Each stage contains one or more actions that must complete before the next stage begins. A stage results in success or failure. If a stage fails, the pipeline stops at that stage and remains stopped until either a new version of an artifact appears in the source location, or a user takes action to rerun the most recent artifact through the pipeline. You can call GetPipelineState, which displays the status of a pipeline, including the status of stages in the pipeline, or GetPipeline, which returns the entire structure of the pipeline, including the stages of that pipeline. For more information about the structure of stages and actions, see [CodePipeline Pipeline Structure Reference](https://docs.aws.amazon.com/codepipeline/latest/userguide/pipeline-structure.html).
//!
//! Pipeline stages include _actions_ that are categorized into categories such as source or build actions performed in a stage of a pipeline. For example, you can use a source action to import artifacts into a pipeline from a source such as Amazon S3. Like stages, you do not work with actions directly in most cases, but you do define and interact with actions when working with pipeline operations such as CreatePipeline and GetPipelineState. Valid action categories are:
//!   - Source
//!   - Build
//!   - Test
//!   - Deploy
//!   - Approval
//!   - Invoke
//!
//! Pipelines also include _transitions_, which allow the transition of artifacts from one stage to the next in a pipeline after the actions in one stage complete.
//!
//! You can work with transitions by calling:
//!   - DisableStageTransition, which prevents artifacts from transitioning to the next stage in a pipeline.
//!   - EnableStageTransition, which enables transition of artifacts between stages in a pipeline.
//!
//! __Using the API to integrate with CodePipeline__
//!
//! For third-party integrators or developers who want to create their own integrations with CodePipeline, the expected sequence varies from the standard API user. To integrate with CodePipeline, developers need to work with the following items:
//!
//! __Jobs__, which are instances of an action. For example, a job for a source action might import a revision of an artifact from a source.
//!
//! You can work with jobs by calling:
//!   - AcknowledgeJob, which confirms whether a job worker has received the specified job.
//!   - GetJobDetails, which returns the details of a job.
//!   - PollForJobs, which determines whether there are any jobs to act on.
//!   - PutJobFailureResult, which provides details of a job failure.
//!   - PutJobSuccessResult, which provides details of a job success.
//!
//! __Third party jobs__, which are instances of an action created by a partner action and integrated into CodePipeline. Partner actions are created by members of the Amazon Web Services Partner Network.
//!
//! You can work with third party jobs by calling:
//!   - AcknowledgeThirdPartyJob, which confirms whether a job worker has received the specified job.
//!   - GetThirdPartyJobDetails, which requests the details of a job for a partner action.
//!   - PollForThirdPartyJobs, which determines whether there are any jobs to act on.
//!   - PutThirdPartyJobFailureResult, which provides details of a job failure.
//!   - PutThirdPartyJobSuccessResult, which provides details of a job success.
//!
//! ## Getting Started
//!
//! > Examples are available for many services and operations, check out the
//! > [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).
//!
//! The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
//! as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-codepipeline` to
//! your project, add the following to your **Cargo.toml** file:
//!
//! ```toml
//! [dependencies]
//! aws-config = { version = "1.1.4", features = ["behavior-version-latest"] }
//! aws-sdk-codepipeline = "1.12.0"
//! tokio = { version = "1", features = ["full"] }
//! ```
//!
//! Then in code, a client can be created with the following:
//!
//! ```rust,no_run
//! use aws_sdk_codepipeline as codepipeline;
//!
//! #[::tokio::main]
//! async fn main() -> Result<(), codepipeline::Error> {
//!     let config = aws_config::load_from_env().await;
//!     let client = aws_sdk_codepipeline::Client::new(&config);
//!
//!     // ... make some calls with the client
//!
//!     Ok(())
//! }
//! ```
//!
//! See the [client documentation](https://docs.rs/aws-sdk-codepipeline/latest/aws_sdk_codepipeline/client/struct.Client.html)
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
//! offered by AWS CodePipeline. The return value of each of these methods is a "fluent builder",
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

/// Client for calling AWS CodePipeline.
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
/// let client = aws_sdk_codepipeline::Client::new(&config);
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
/// let config = aws_sdk_codepipeline::config::Builder::from(&sdk_config)
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
/// For example, the [`AcknowledgeJob`](crate::operation::acknowledge_job) operation has
/// a [`Client::acknowledge_job`], function which returns a builder for that operation.
/// The fluent builder ultimately has a `send()` function that returns an async future that
/// returns a result, as illustrated below:
///
/// ```rust,ignore
/// let result = client.acknowledge_job()
///     .job_id("example")
///     .send()
///     .await;
/// ```
///
/// The underlying HTTP requests that get made by this can be modified with the `customize_operation`
/// function on the fluent builder. See the [`customize`](crate::client::customize) module for more
/// information.
pub mod client;

/// Configuration for AWS CodePipeline.
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
