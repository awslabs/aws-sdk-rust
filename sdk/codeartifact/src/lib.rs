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
//! CodeArtifact is a fully managed artifact repository compatible with language-native package managers and build tools such as npm, Apache Maven, pip, and dotnet. You can use CodeArtifact to share packages with development teams and pull packages. Packages can be pulled from both public and CodeArtifact repositories. You can also create an upstream relationship between a CodeArtifact repository and another repository, which effectively merges their contents from the point of view of a package manager client.
//!
//! __CodeArtifact Components__
//!
//! Use the information in this guide to help you work with the following CodeArtifact components:
//!   - __Repository__: A CodeArtifact repository contains a set of [package versions](https://docs.aws.amazon.com/codeartifact/latest/ug/welcome.html#welcome-concepts-package-version), each of which maps to a set of assets, or files. Repositories are polyglot, so a single repository can contain packages of any supported type. Each repository exposes endpoints for fetching and publishing packages using tools like the __ npm __ CLI, the Maven CLI (__ mvn __), Python CLIs (__ pip __ and twine), and NuGet CLIs (nuget and dotnet).
//!   - __Domain__: Repositories are aggregated into a higher-level entity known as a _domain_. All package assets and metadata are stored in the domain, but are consumed through repositories. A given package asset, such as a Maven JAR file, is stored once per domain, no matter how many repositories it's present in. All of the assets and metadata in a domain are encrypted with the same customer master key (CMK) stored in Key Management Service (KMS). Each repository is a member of a single domain and can't be moved to a different domain. The domain allows organizational policy to be applied across multiple repositories, such as which accounts can access repositories in the domain, and which public repositories can be used as sources of packages. Although an organization can have multiple domains, we recommend a single production domain that contains all published artifacts so that teams can find and share packages across their organization.
//!   - __Package__: A _package_ is a bundle of software and the metadata required to resolve dependencies and install the software. CodeArtifact supports [npm](https://docs.aws.amazon.com/codeartifact/latest/ug/using-npm.html), [PyPI](https://docs.aws.amazon.com/codeartifact/latest/ug/using-python.html), [Maven](https://docs.aws.amazon.com/codeartifact/latest/ug/using-maven), and [NuGet](https://docs.aws.amazon.com/codeartifact/latest/ug/using-nuget) package formats. In CodeArtifact, a package consists of:
//!     - A _name_ (for example, webpack is the name of a popular npm package)
//!     - An optional namespace (for example, @types in @types/node)
//!     - A set of versions (for example, 1.0.0, 1.0.1, 1.0.2, etc.)
//!     - Package-level metadata (for example, npm tags)
//!
//!   - __Package version__: A version of a package, such as @types/node 12.6.9. The version number format and semantics vary for different package formats. For example, npm package versions must conform to the [Semantic Versioning specification](https://semver.org/). In CodeArtifact, a package version consists of the version identifier, metadata at the package version level, and a set of assets.
//!   - __Upstream repository__: One repository is _upstream_ of another when the package versions in it can be accessed from the repository endpoint of the downstream repository, effectively merging the contents of the two repositories from the point of view of a client. CodeArtifact allows creating an upstream relationship between two repositories.
//!   - __Asset__: An individual file stored in CodeArtifact associated with a package version, such as an npm .tgz file or Maven POM and JAR files.
//!
//! CodeArtifact supports these operations:
//!   - AssociateExternalConnection: Adds an existing external connection to a repository.
//!   - CopyPackageVersions: Copies package versions from one repository to another repository in the same domain.
//!   - CreateDomain: Creates a domain
//!   - CreateRepository: Creates a CodeArtifact repository in a domain.
//!   - DeleteDomain: Deletes a domain. You cannot delete a domain that contains repositories.
//!   - DeleteDomainPermissionsPolicy: Deletes the resource policy that is set on a domain.
//!   - DeletePackage: Deletes a package and all associated package versions.
//!   - DeletePackageVersions: Deletes versions of a package. After a package has been deleted, it can be republished, but its assets and metadata cannot be restored because they have been permanently removed from storage.
//!   - DeleteRepository: Deletes a repository.
//!   - DeleteRepositoryPermissionsPolicy: Deletes the resource policy that is set on a repository.
//!   - DescribeDomain: Returns a DomainDescription object that contains information about the requested domain.
//!   - DescribePackage: Returns a [PackageDescription](https://docs.aws.amazon.com/codeartifact/latest/APIReference/API_PackageDescription.html) object that contains details about a package.
//!   - DescribePackageVersion: Returns a [PackageVersionDescription](https://docs.aws.amazon.com/codeartifact/latest/APIReference/API_PackageVersionDescription.html) object that contains details about a package version.
//!   - DescribeRepository: Returns a RepositoryDescription object that contains detailed information about the requested repository.
//!   - DisposePackageVersions: Disposes versions of a package. A package version with the status Disposed cannot be restored because they have been permanently removed from storage.
//!   - DisassociateExternalConnection: Removes an existing external connection from a repository.
//!   - GetAuthorizationToken: Generates a temporary authorization token for accessing repositories in the domain. The token expires the authorization period has passed. The default authorization period is 12 hours and can be customized to any length with a maximum of 12 hours.
//!   - GetDomainPermissionsPolicy: Returns the policy of a resource that is attached to the specified domain.
//!   - GetPackageVersionAsset: Returns the contents of an asset that is in a package version.
//!   - GetPackageVersionReadme: Gets the readme file or descriptive text for a package version.
//!   - GetRepositoryEndpoint: Returns the endpoint of a repository for a specific package format. A repository has one endpoint for each package format:
//!     - maven
//!     - npm
//!     - nuget
//!     - pypi
//!
//!   - GetRepositoryPermissionsPolicy: Returns the resource policy that is set on a repository.
//!   - ListDomains: Returns a list of DomainSummary objects. Each returned DomainSummary object contains information about a domain.
//!   - ListPackages: Lists the packages in a repository.
//!   - ListPackageVersionAssets: Lists the assets for a given package version.
//!   - ListPackageVersionDependencies: Returns a list of the direct dependencies for a package version.
//!   - ListPackageVersions: Returns a list of package versions for a specified package in a repository.
//!   - ListRepositories: Returns a list of repositories owned by the Amazon Web Services account that called this method.
//!   - ListRepositoriesInDomain: Returns a list of the repositories in a domain.
//!   - PublishPackageVersion: Creates a new package version containing one or more assets.
//!   - PutDomainPermissionsPolicy: Attaches a resource policy to a domain.
//!   - PutPackageOriginConfiguration: Sets the package origin configuration for a package, which determine how new versions of the package can be added to a specific repository.
//!   - PutRepositoryPermissionsPolicy: Sets the resource policy on a repository that specifies permissions to access it.
//!   - UpdatePackageVersionsStatus: Updates the status of one or more versions of a package.
//!   - UpdateRepository: Updates the properties of a repository.
//!
//! ## Getting Started
//!
//! > Examples are available for many services and operations, check out the
//! > [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).
//!
//! The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
//! as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-codeartifact` to
//! your project, add the following to your **Cargo.toml** file:
//!
//! ```toml
//! [dependencies]
//! aws-config = { version = "1.1.4", features = ["behavior-version-latest"] }
//! aws-sdk-codeartifact = "1.12.0"
//! tokio = { version = "1", features = ["full"] }
//! ```
//!
//! Then in code, a client can be created with the following:
//!
//! ```rust,no_run
//! use aws_sdk_codeartifact as codeartifact;
//!
//! #[::tokio::main]
//! async fn main() -> Result<(), codeartifact::Error> {
//!     let config = aws_config::load_from_env().await;
//!     let client = aws_sdk_codeartifact::Client::new(&config);
//!
//!     // ... make some calls with the client
//!
//!     Ok(())
//! }
//! ```
//!
//! See the [client documentation](https://docs.rs/aws-sdk-codeartifact/latest/aws_sdk_codeartifact/client/struct.Client.html)
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
//! offered by CodeArtifact. The return value of each of these methods is a "fluent builder",
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

/// Client for calling CodeArtifact.
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
/// let client = aws_sdk_codeartifact::Client::new(&config);
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
/// let config = aws_sdk_codeartifact::config::Builder::from(&sdk_config)
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
/// For example, the [`AssociateExternalConnection`](crate::operation::associate_external_connection) operation has
/// a [`Client::associate_external_connection`], function which returns a builder for that operation.
/// The fluent builder ultimately has a `send()` function that returns an async future that
/// returns a result, as illustrated below:
///
/// ```rust,ignore
/// let result = client.associate_external_connection()
///     .domain("example")
///     .send()
///     .await;
/// ```
///
/// The underlying HTTP requests that get made by this can be modified with the `customize_operation`
/// function on the fluent builder. See the [`customize`](crate::client::customize) module for more
/// information.
pub mod client;

/// Configuration for CodeArtifact.
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

pub(crate) mod protocol_serde;

mod serialization_settings;

mod endpoint_lib;

mod lens;

mod serde_util;

mod json_errors;

#[doc(inline)]
pub use client::Client;
