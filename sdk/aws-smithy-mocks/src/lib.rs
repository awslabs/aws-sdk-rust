/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![doc = include_str!("../README.md")]
/* Automatically managed default lints */
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
/* End of automatically managed default lints */
#![warn(
    missing_docs,
    rustdoc::missing_crate_level_docs,
    unreachable_pub,
    rust_2018_idioms
)]

mod interceptor;
mod rule;

pub use interceptor::{create_mock_http_client, MockResponseInterceptor};
pub(crate) use rule::MockResponse;
pub use rule::{Rule, RuleBuilder, RuleMode};

// why do we need a macro for this?
// We want customers to be able to provide an ergonomic way to say the method they're looking for,
// `Client::list_buckets`, e.g. But there isn't enough information on that type to recover everything.
// This macro commits a small amount of crimes to recover that type information so we can construct
// a rule that can intercept these operations.

/// `mock!` macro that produces a [`RuleBuilder`] from a client invocation
///
/// See the `examples` folder of this crate for fully worked examples.
///
/// # Examples
///
/// **Mock and return a success response**:
///
/// ```rust,ignore
/// use aws_sdk_s3::operation::get_object::GetObjectOutput;
/// use aws_sdk_s3::Client;
/// use aws_smithy_types::byte_stream::ByteStream;
/// use aws_smithy_mocks::mock;
/// let get_object_happy_path = mock!(Client::get_object)
///   .match_requests(|req|req.bucket() == Some("test-bucket") && req.key() == Some("test-key"))
///   .then_output(||GetObjectOutput::builder().body(ByteStream::from_static(b"12345-abcde")).build());
/// ```
///
/// **Mock and return an error**:
/// ```rust,ignore
/// use aws_sdk_s3::operation::get_object::GetObjectError;
/// use aws_sdk_s3::types::error::NoSuchKey;
/// use aws_sdk_s3::Client;
/// use aws_smithy_mocks::mock;
/// let get_object_error_path = mock!(Client::get_object)
///   .then_error(||GetObjectError::NoSuchKey(NoSuchKey::builder().build()));
/// ```
///
#[macro_export]
macro_rules! mock {
    ($operation: expr) => {
        #[allow(unreachable_code)]
        {
            $crate::RuleBuilder::new_from_mock(
                // We don't actually want to run this code, so we put it in a closure. The closure
                // has the types we want which makes this whole thing type-safe (and the IDE can even
                // figure out the right input/output types in inference!)
                // The code generated here is:
                // `Client::list_buckets(todo!())`
                || $operation(todo!()).as_input().clone().build().unwrap(),
                || $operation(todo!()).send(),
            )
        }
    };
}

// This could be obviated by a reasonable trait, since you can express it with SdkConfig if clients implement From<&SdkConfig>.

/// `mock_client!` macro produces a Client configured with a number of Rules and appropriate test default configuration.
///
/// # Examples
///
/// **Create a client that uses a mock failure and then a success**:
///
/// ```rust,ignore
/// use aws_sdk_s3::operation::get_object::{GetObjectOutput, GetObjectError};
/// use aws_sdk_s3::types::error::NoSuchKey;
/// use aws_sdk_s3::Client;
/// use aws_smithy_types::byte_stream::ByteStream;
/// use aws_smithy_mocks::{mock_client, mock, RuleMode};
/// let get_object_error_path = mock!(Client::get_object)
///   .then_error(||GetObjectError::NoSuchKey(NoSuchKey::builder().build()))
///   .build();
/// let get_object_happy_path = mock!(Client::get_object)
///   .match_requests(|req|req.bucket() == Some("test-bucket") && req.key() == Some("test-key"))
///   .then_output(||GetObjectOutput::builder().body(ByteStream::from_static(b"12345-abcde")).build())
///   .build();
/// let client = mock_client!(aws_sdk_s3, RuleMode::Sequential, &[&get_object_error_path, &get_object_happy_path]);
///
///
/// **Create a client but customize a specific setting**:
/// rust,ignore
/// use aws_sdk_s3::operation::get_object::GetObjectOutput;
/// use aws_sdk_s3::Client;
/// use aws_smithy_types::byte_stream::ByteStream;
/// use aws_smithy_mocks::{mock_client, mock, RuleMode};
/// let get_object_happy_path = mock!(Client::get_object)
///   .match_requests(|req|req.bucket() == Some("test-bucket") && req.key() == Some("test-key"))
///   .then_output(||GetObjectOutput::builder().body(ByteStream::from_static(b"12345-abcde")).build())
///   .build();
/// let client = mock_client!(
///     aws_sdk_s3,
///     RuleMode::Sequential,
///     &[&get_object_happy_path],
///     // Perhaps you need to force path style
///     |client_builder|client_builder.force_path_style(true)
/// );
/// ```
///
#[macro_export]
macro_rules! mock_client {
    ($aws_crate: ident, $rules: expr) => {
        $crate::mock_client!($aws_crate, $crate::RuleMode::Sequential, $rules)
    };
    ($aws_crate: ident, $rule_mode: expr, $rules: expr) => {{
        $crate::mock_client!($aws_crate, $rule_mode, $rules, |conf| conf)
    }};
    ($aws_crate: ident, $rule_mode: expr, $rules: expr, $additional_configuration: expr) => {{
        let mut mock_response_interceptor =
            $crate::MockResponseInterceptor::new().rule_mode($rule_mode);
        for rule in $rules {
            mock_response_interceptor = mock_response_interceptor.with_rule(rule)
        }

        // Create a mock HTTP client
        let mock_http_client = $crate::create_mock_http_client();

        // Allow callers to avoid explicitly specifying the type
        fn coerce<T: Fn($aws_crate::config::Builder) -> $aws_crate::config::Builder>(f: T) -> T {
            f
        }

        $aws_crate::client::Client::from_conf(
            coerce($additional_configuration)(
                $aws_crate::config::Config::builder()
                    .with_test_defaults()
                    .region($aws_crate::config::Region::from_static("us-east-1"))
                    .http_client(mock_http_client)
                    .interceptor(mock_response_interceptor),
            )
            .build(),
        )
    }};
}
