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
//! <fullname>Key Management Service</fullname>
//! <p>Key Management Service (KMS) is an encryption and key management web service. This guide describes
//! the KMS operations that you can call programmatically. For general information about KMS,
//! see the <a href="https://docs.aws.amazon.com/kms/latest/developerguide/">
//! <i>Key Management Service Developer Guide</i>
//! </a>.</p>
//! <note>
//! <p>KMS is replacing the term <i>customer master key (CMK)</i> with <i>KMS key</i> and <i>KMS key</i>. The concept has not changed. To prevent breaking changes, KMS is keeping some variations of this term.</p>
//! <p>Amazon Web Services provides SDKs that consist of libraries and sample code for various programming
//! languages and platforms (Java, Ruby, .Net, macOS, Android, etc.). The SDKs provide a
//! convenient way to create programmatic access to KMS and other Amazon Web Services services. For example,
//! the SDKs take care of tasks such as signing requests (see below), managing errors, and
//! retrying requests automatically. For more information about the Amazon Web Services SDKs, including how to
//! download and install them, see <a href="http://aws.amazon.com/tools/">Tools for Amazon Web
//! Services</a>.</p>
//! </note>
//! <p>We recommend that you use the Amazon Web Services SDKs to make programmatic API calls to KMS. </p>
//! <p>If you need to use FIPS 140-2 validated cryptographic modules when communicating with
//! Amazon Web Services, use the FIPS endpoint in your preferred Amazon Web Services Region. For more information about the
//! available FIPS endpoints, see <a href="https://docs.aws.amazon.com/general/latest/gr/kms.html#kms_region">Service endpoints</a> in the Key Management Service topic of the <i>Amazon Web Services General Reference</i>.</p>
//! <p>All KMS API calls must be signed and be transmitted using Transport Layer Security (TLS).
//! KMS recommends you always use the latest supported TLS version. Clients
//! must also support cipher suites with Perfect Forward Secrecy (PFS) such as Ephemeral
//! Diffie-Hellman (DHE) or Elliptic Curve Ephemeral Diffie-Hellman (ECDHE). Most modern systems
//! such as Java 7 and later support these modes.</p>
//! <p>
//! <b>Signing Requests</b>
//! </p>
//! <p>Requests must be signed by using an access key ID and a secret access key. We strongly
//! recommend that you <i>do not</i> use your Amazon Web Services account (root) access key ID and
//! secret key for everyday work with KMS. Instead, use the access key ID and secret access key
//! for an IAM user. You can also use the Amazon Web Services Security Token Service to generate temporary
//! security credentials that you can use to sign requests.</p>
//! <p>All KMS operations require <a href="https://docs.aws.amazon.com/general/latest/gr/signature-version-4.html">Signature Version 4</a>.</p>
//! <p>
//! <b>Logging API Requests</b>
//! </p>
//! <p>KMS supports CloudTrail, a service that logs Amazon Web Services API calls and related events for your
//! Amazon Web Services account and delivers them to an Amazon S3 bucket that you specify. By using the
//! information collected by CloudTrail, you can determine what requests were made to KMS, who made
//! the request, when it was made, and so on. To learn more about CloudTrail, including how to turn it
//! on and find your log files, see the <a href="https://docs.aws.amazon.com/awscloudtrail/latest/userguide/">CloudTrail User Guide</a>.</p>
//! <p>
//! <b>Additional Resources</b>
//! </p>
//! <p>For more information about credentials and request signing, see the following:</p>
//! <ul>
//! <li>
//! <p>
//! <a href="https://docs.aws.amazon.com/general/latest/gr/aws-security-credentials.html">Amazon Web Services
//! Security Credentials</a> - This topic provides general information about the types
//! of credentials used to access Amazon Web Services.</p>
//! </li>
//! <li>
//! <p>
//! <a href="https://docs.aws.amazon.com/IAM/latest/UserGuide/id_credentials_temp.html">Temporary
//! Security Credentials</a> - This section of the <i>IAM User Guide</i>
//! describes how to create and use temporary security credentials.</p>
//! </li>
//! <li>
//! <p>
//! <a href="https://docs.aws.amazon.com/general/latest/gr/signature-version-4.html">Signature Version
//! 4 Signing Process</a> - This set of topics walks you through the process of signing
//! a request using an access key ID and a secret access key.</p>
//! </li>
//! </ul>
//! <p>
//! <b>Commonly Used API Operations</b>
//! </p>
//! <p>Of the API operations discussed in this guide, the following will prove the most useful
//! for most applications. You will likely perform operations other than these, such as creating
//! keys and assigning policies, by using the console.</p>
//! <ul>
//! <li>
//! <p>
//! <a>Encrypt</a>
//! </p>
//! </li>
//! <li>
//! <p>
//! <a>Decrypt</a>
//! </p>
//! </li>
//! <li>
//! <p>
//! <a>GenerateDataKey</a>
//! </p>
//! </li>
//! <li>
//! <p>
//! <a>GenerateDataKeyWithoutPlaintext</a>
//! </p>
//! </li>
//! </ul>
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
//! Examples can be found [here](https://github.com/awslabs/aws-sdk-rust/tree/main/examples/kms).

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
    pub use aws_smithy_types::Blob;
    pub use aws_smithy_types::DateTime;
}
static API_METADATA: aws_http::user_agent::ApiMetadata =
    aws_http::user_agent::ApiMetadata::new("kms", PKG_VERSION);
pub use aws_smithy_http::endpoint::Endpoint;
pub use aws_smithy_types::retry::RetryConfig;
pub use aws_types::app_name::AppName;
pub use aws_types::region::Region;
pub use aws_types::Credentials;
#[doc(inline)]
pub use client::Client;
