/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Extremely Experimental Test Connection
//!
//! Warning: Extremely experimental, API likely to change.
//!
//! DVR is an extremely experimental record & replay framework that supports multi-frame HTTP request / response traffic.

use std::collections::HashMap;

use bytes::Bytes;
use serde::{Deserialize, Serialize};

pub use aws_smithy_protocol_test::MediaType;
use aws_smithy_types::base64;
pub use record::RecordingConnection;
pub use replay::ReplayingConnection;

mod record;
mod replay;

/// A complete traffic recording
///
/// A traffic recording can be replayed with [`RecordingConnection`](RecordingConnection)
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkTraffic {
    events: Vec<Event>,
    docs: Option<String>,
    version: Version,
}

impl NetworkTraffic {
    /// Network events
    pub fn events(&self) -> &Vec<Event> {
        &self.events
    }
}

/// Serialization version of DVR data
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Version {
    /// Initial network traffic version
    V0,
}

/// A network traffic recording may contain multiple different connections occurring simultaneously
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct ConnectionId(usize);

/// A network event
///
/// Network events consist of a connection identifier and an action. An event is sufficient to
/// reproduce traffic later during replay
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Event {
    connection_id: ConnectionId,
    action: Action,
}

/// An initial HTTP request, roughly equivalent to `http::Request<()>`
///
/// The initial request phase of an HTTP request. The body will be
/// sent later as a separate action.
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Request {
    uri: String,
    headers: HashMap<String, Vec<String>>,
    method: String,
}

/// An initial HTTP response roughly equivalent to `http::Response<()>`
///
/// The initial response phase of an HTTP request. The body will be
/// sent later as a separate action.
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Response {
    status: u16,
    version: String,
    headers: HashMap<String, Vec<String>>,
}

impl From<&Request> for http::Request<()> {
    fn from(request: &Request) -> Self {
        let mut builder = http::Request::builder().uri(request.uri.as_str());
        for (k, values) in request.headers.iter() {
            for v in values {
                builder = builder.header(k, v);
            }
        }
        builder.method(request.method.as_str()).body(()).unwrap()
    }
}

impl<'a, B> From<&'a http::Request<B>> for Request {
    fn from(req: &'a http::Request<B>) -> Self {
        let uri = req.uri().to_string();
        let headers = headers_to_map(req.headers());
        let method = req.method().to_string();
        Self {
            uri,
            headers,
            method,
        }
    }
}

fn headers_to_map(headers: &http::HeaderMap<http::HeaderValue>) -> HashMap<String, Vec<String>> {
    let mut out: HashMap<_, Vec<_>> = HashMap::new();
    for (header_name, header_value) in headers.iter() {
        let entry = out.entry(header_name.to_string()).or_default();
        entry.push(header_value.to_str().unwrap().to_string());
    }
    out
}

impl<'a, B> From<&'a http::Response<B>> for Response {
    fn from(resp: &'a http::Response<B>) -> Self {
        let status = resp.status().as_u16();
        let version = format!("{:?}", resp.version());
        let headers = headers_to_map(resp.headers());
        Self {
            status,
            version,
            headers,
        }
    }
}

/// Error response wrapper
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Error(String);

/// Network Action
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum Action {
    /// Initial HTTP Request
    Request {
        /// HTTP Request headers, method, and URI
        request: Request,
    },

    /// Initial HTTP response or failure
    Response {
        /// HTTP response or failure
        response: Result<Response, Error>,
    },

    /// Data segment
    Data {
        /// Body Data
        data: BodyData,
        /// Direction: request vs. response
        direction: Direction,
    },

    /// End of data
    Eof {
        /// Succesful vs. failed termination
        ok: bool,
        /// Direction: request vs. response
        direction: Direction,
    },
}

/// Event direction
///
/// During replay, this is used to replay data in the right direction
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Direction {
    /// Request phase
    Request,
    /// Response phase
    Response,
}

impl Direction {
    /// The opposite of a given direction
    pub fn opposite(self) -> Self {
        match self {
            Direction::Request => Direction::Response,
            Direction::Response => Direction::Request,
        }
    }
}

/// HTTP Body Data Abstraction
///
/// When the data is a UTF-8 encoded string, it will be serialized as a string for readability.
/// Otherwise, it will be base64 encoded.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[non_exhaustive]
pub enum BodyData {
    /// UTF-8 encoded data
    Utf8(String),

    /// Base64 encoded binary data
    Base64(String),
}

impl BodyData {
    /// Convert [`BodyData`](BodyData) into Bytes
    pub fn into_bytes(self) -> Vec<u8> {
        match self {
            BodyData::Utf8(string) => string.into_bytes(),
            BodyData::Base64(string) => base64::decode(string).unwrap(),
        }
    }

    /// Copy [`BodyData`](BodyData) into a `Vec<u8>`
    pub fn copy_to_vec(&self) -> Vec<u8> {
        match self {
            BodyData::Utf8(string) => string.as_bytes().into(),
            BodyData::Base64(string) => base64::decode(string).unwrap(),
        }
    }
}

impl From<Bytes> for BodyData {
    fn from(data: Bytes) -> Self {
        match std::str::from_utf8(data.as_ref()) {
            Ok(string) => BodyData::Utf8(string.to_string()),
            Err(_) => BodyData::Base64(base64::encode(data)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;

    use aws_smithy_http::body::SdkBody;
    use aws_smithy_http::byte_stream::ByteStream;

    use crate::dvr::{NetworkTraffic, RecordingConnection, ReplayingConnection};
    use bytes::Bytes;
    use http::Uri;

    #[tokio::test]
    async fn turtles_all_the_way_down() -> Result<(), Box<dyn Error>> {
        // create a replaying connection from a recording, wrap a recording connection around it,
        // make a request, then verify that the same traffic was recorded.
        let network_traffic = fs::read_to_string("test-data/example.com.json")?;
        let network_traffic: NetworkTraffic = serde_json::from_str(&network_traffic)?;
        let inner = ReplayingConnection::new(network_traffic.events.clone());
        let mut connection = RecordingConnection::new(inner.clone());
        let req = http::Request::post("https://www.example.com")
            .body(SdkBody::from("hello world"))
            .unwrap();
        use tower::Service;
        let mut resp = connection.call(req).await.expect("ok");
        let body = std::mem::replace(resp.body_mut(), SdkBody::taken());
        let data = ByteStream::new(body).collect().await.unwrap().into_bytes();
        assert_eq!(
            String::from_utf8(data.to_vec()).unwrap(),
            "hello from example.com"
        );
        assert_eq!(
            connection.events().as_slice(),
            network_traffic.events.as_slice()
        );
        let requests = inner.take_requests().await;
        assert_eq!(
            requests[0].uri(),
            &Uri::from_static("https://www.example.com")
        );
        assert_eq!(
            requests[0].body(),
            &Bytes::from_static("hello world".as_bytes())
        );
        Ok(())
    }
}
