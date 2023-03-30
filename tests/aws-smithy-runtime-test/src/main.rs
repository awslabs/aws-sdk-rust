/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod auth;
mod conn;
mod de;
mod endpoints;
mod interceptors;
mod retry;
mod ser;

use aws_sdk_s3::operation::get_object::{GetObjectInput, GetObjectOutput};
use aws_sdk_s3::types::ChecksumMode;
use aws_smithy_http::body::SdkBody;
use aws_smithy_runtime::{invoke, BoxError};
use aws_smithy_runtime_api::config_bag::ConfigBag;
use aws_smithy_runtime_api::interceptors::Interceptors;
use aws_smithy_runtime_api::runtime_plugin::RuntimePlugins;
use std::str::from_utf8;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    tracing_subscriber::fmt::init();

    // Create the config we'll need to send the request + the request itself
    let sdk_config = aws_config::load_from_env().await;
    let _service_config = aws_sdk_s3::Config::from(&sdk_config);

    let input = GetObjectInput::builder()
        .bucket("zhessler-test-bucket")
        .key("1000-lines.txt")
        .checksum_mode(ChecksumMode::Enabled)
        .build()?;

    let mut runtime_plugins = RuntimePlugins::new();

    // TODO(smithy-orchestrator-codegen) Make it so these are added by default for S3
    runtime_plugins
        .with_client_plugin(auth::GetObjectAuthOrc::new())
        .with_client_plugin(conn::HyperConnection::new())
        // TODO(smithy-orchestrator-codegen) Make it so these are added by default for this S3 operation
        .with_operation_plugin(endpoints::GetObjectEndpointOrc::new())
        .with_operation_plugin(retry::GetObjectRetryStrategy::new())
        .with_operation_plugin(de::GetObjectResponseDeserializer::new())
        .with_operation_plugin(ser::GetObjectInputSerializer::new());

    let mut cfg = ConfigBag::base();
    let mut interceptors: Interceptors<
        GetObjectInput,
        http::Request<SdkBody>,
        http::Response<SdkBody>,
        Result<GetObjectOutput, BoxError>,
    > = Interceptors::new();
    let res = invoke(input, &mut interceptors, &runtime_plugins, &mut cfg).await?;

    let body = res.body.collect().await?.to_vec();
    let body_string = from_utf8(&body)?;

    info!("{body_string}");

    Ok(())
}
