/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_http::body::SdkBody;
use aws_smithy_runtime_api::client::orchestrator::HttpResponse;
use bytes::{Buf, Bytes};
use http_body::Body;
use pin_utils::pin_mut;

async fn body_to_bytes(body: SdkBody) -> Result<Bytes, <SdkBody as Body>::Error> {
    let mut output = Vec::new();
    pin_mut!(body);
    while let Some(buf) = body.data().await {
        let mut buf = buf?;
        while buf.has_remaining() {
            output.extend_from_slice(buf.chunk());
            buf.advance(buf.chunk().len())
        }
    }

    Ok(Bytes::from(output))
}

pub(crate) async fn read_body(response: &mut HttpResponse) -> Result<(), <SdkBody as Body>::Error> {
    let mut body = SdkBody::taken();
    std::mem::swap(&mut body, response.body_mut());

    let bytes = body_to_bytes(body).await?;
    let mut body = SdkBody::from(bytes);
    std::mem::swap(&mut body, response.body_mut());

    Ok(())
}
