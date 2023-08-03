/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::ops::DerefMut;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use bytes::{Bytes, BytesMut};
use http::{Request, Version};
use http_body::Body;
use tokio::task::JoinHandle;

use aws_smithy_http::body::SdkBody;
use aws_smithy_http::result::ConnectorError;
use aws_smithy_protocol_test::MediaType;
use aws_smithy_types::error::display::DisplayErrorContext;

use crate::dvr::{Action, ConnectionId, Direction, Event, NetworkTraffic};

/// Wrapper type to enable optionally waiting for a future to complete
#[derive(Debug)]
enum Waitable<T> {
    Loading(JoinHandle<T>),
    Value(T),
}

impl<T> Waitable<T> {
    /// Consumes the future and returns the value
    async fn take(self) -> T {
        match self {
            Waitable::Loading(f) => f.await.expect("join failed"),
            Waitable::Value(value) => value,
        }
    }

    /// Waits for the future to be ready
    async fn wait(&mut self) {
        match self {
            Waitable::Loading(f) => *self = Waitable::Value(f.await.expect("join failed")),
            Waitable::Value(_) => {}
        }
    }
}

/// Replay traffic recorded by a [`RecordingConnection`](super::RecordingConnection)
#[derive(Clone, Debug)]
pub struct ReplayingConnection {
    live_events: Arc<Mutex<HashMap<ConnectionId, VecDeque<Event>>>>,
    verifiable_events: Arc<HashMap<ConnectionId, Request<Bytes>>>,
    num_events: Arc<AtomicUsize>,
    recorded_requests: Arc<Mutex<HashMap<ConnectionId, Waitable<http::Request<Bytes>>>>>,
}

impl ReplayingConnection {
    fn next_id(&self) -> ConnectionId {
        ConnectionId(self.num_events.fetch_add(1, Ordering::Relaxed))
    }

    /// Validate all headers and bodies
    pub async fn full_validate(self, media_type: MediaType) -> Result<(), Box<dyn Error>> {
        self.validate_body_and_headers(None, media_type).await
    }

    /// Validate actual requests against expected requests
    pub async fn validate(
        self,
        checked_headers: &[&str],
        body_comparer: impl Fn(&[u8], &[u8]) -> Result<(), Box<dyn Error>>,
    ) -> Result<(), Box<dyn Error>> {
        self.validate_base(Some(checked_headers), body_comparer)
            .await
    }

    /// Validate that the bodies match, using a given [`MediaType`] for comparison
    ///
    /// The specified headers are also validated
    pub async fn validate_body_and_headers(
        self,
        checked_headers: Option<&[&str]>,
        media_type: MediaType,
    ) -> Result<(), Box<dyn Error>> {
        self.validate_base(checked_headers, |b1, b2| {
            aws_smithy_protocol_test::validate_body(
                b1,
                std::str::from_utf8(b2).unwrap(),
                media_type.clone(),
            )
            .map_err(|e| Box::new(e) as _)
        })
        .await
    }

    async fn validate_base(
        self,
        checked_headers: Option<&[&str]>,
        body_comparer: impl Fn(&[u8], &[u8]) -> Result<(), Box<dyn Error>>,
    ) -> Result<(), Box<dyn Error>> {
        let mut actual_requests =
            std::mem::take(self.recorded_requests.lock().unwrap().deref_mut());
        for conn_id in 0..self.verifiable_events.len() {
            let conn_id = ConnectionId(conn_id);
            let expected = self.verifiable_events.get(&conn_id).unwrap();
            let actual = actual_requests
                .remove(&conn_id)
                .ok_or(format!(
                    "expected connection {:?} but request was never sent",
                    conn_id
                ))?
                .take()
                .await;
            aws_smithy_protocol_test::assert_uris_match(expected.uri(), actual.uri());
            body_comparer(expected.body().as_ref(), actual.body().as_ref())?;
            let expected_headers = expected
                .headers()
                .keys()
                .map(|k| k.as_str())
                .filter(|k| match checked_headers {
                    Some(list) => list.contains(k),
                    None => true,
                })
                .flat_map(|key| {
                    let _ = expected.headers().get(key)?;
                    Some((
                        key,
                        expected
                            .headers()
                            .get_all(key)
                            .iter()
                            .map(|h| h.to_str().unwrap())
                            .collect::<Vec<_>>()
                            .join(", "),
                    ))
                })
                .collect::<Vec<_>>();
            aws_smithy_protocol_test::validate_headers(actual.headers(), expected_headers)
                .map_err(|err| {
                    format!(
                        "event {} validation failed with: {}",
                        conn_id.0,
                        DisplayErrorContext(&err)
                    )
                })?;
        }
        Ok(())
    }

    /// Return all the recorded requests for further analysis
    pub async fn take_requests(self) -> Vec<http::Request<Bytes>> {
        let mut recorded_requests =
            std::mem::take(self.recorded_requests.lock().unwrap().deref_mut());
        let mut out = Vec::with_capacity(recorded_requests.len());
        for conn_id in 0..recorded_requests.len() {
            out.push(
                recorded_requests
                    .remove(&ConnectionId(conn_id))
                    .expect("should exist")
                    .take()
                    .await,
            )
        }
        out
    }

    /// Build a replay connection from a JSON file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let events: NetworkTraffic =
            serde_json::from_str(&std::fs::read_to_string(path.as_ref())?)?;
        Ok(Self::new(events.events))
    }

    /// Build a replay connection from a sequence of events
    pub fn new(events: Vec<Event>) -> Self {
        let mut event_map: HashMap<_, VecDeque<_>> = HashMap::new();
        for event in events {
            let event_buffer = event_map.entry(event.connection_id).or_default();
            event_buffer.push_back(event);
        }
        let verifiable_events = event_map
            .iter()
            .map(|(id, events)| {
                let mut body = BytesMut::new();
                for event in events {
                    if let Action::Data {
                        direction: Direction::Request,
                        data,
                    } = &event.action
                    {
                        body.extend_from_slice(&data.copy_to_vec());
                    }
                }
                let initial_request = events.iter().next().expect("must have one event");
                let request = match &initial_request.action {
                    Action::Request { request } => {
                        http::Request::from(request).map(|_| Bytes::from(body))
                    }
                    _ => panic!("invalid first event"),
                };
                (*id, request)
            })
            .collect();
        let verifiable_events = Arc::new(verifiable_events);

        ReplayingConnection {
            live_events: Arc::new(Mutex::new(event_map)),
            num_events: Arc::new(AtomicUsize::new(0)),
            recorded_requests: Default::default(),
            verifiable_events,
        }
    }
}

async fn replay_body(events: VecDeque<Event>, mut sender: hyper::body::Sender) {
    for event in events {
        match event.action {
            Action::Request { .. } => panic!(),
            Action::Response { .. } => panic!(),
            Action::Data {
                data,
                direction: Direction::Response,
            } => {
                sender
                    .send_data(Bytes::from(data.into_bytes()))
                    .await
                    .expect("this is in memory traffic that should not fail to send");
            }
            Action::Data {
                data: _data,
                direction: Direction::Request,
            } => {}
            Action::Eof {
                direction: Direction::Request,
                ..
            } => {}
            Action::Eof {
                direction: Direction::Response,
                ok: true,
                ..
            } => {
                drop(sender);
                break;
            }
            Action::Eof {
                direction: Direction::Response,
                ok: false,
                ..
            } => {
                sender.abort();
                break;
            }
        }
    }
}

fn convert_version(version: &str) -> Version {
    match version {
        "HTTP/1.1" => Version::HTTP_11,
        "HTTP/2.0" => Version::HTTP_2,
        _ => panic!("unsupported: {}", version),
    }
}

impl tower::Service<http::Request<SdkBody>> for ReplayingConnection {
    type Response = http::Response<SdkBody>;
    type Error = ConnectorError;

    #[allow(clippy::type_complexity)]
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>,
    >;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: Request<SdkBody>) -> Self::Future {
        let event_id = self.next_id();
        tracing::debug!("received event {}: {req:?}", event_id.0);
        let mut events = match self.live_events.lock().unwrap().remove(&event_id) {
            Some(traffic) => traffic,
            None => {
                return Box::pin(std::future::ready(Err(ConnectorError::other(
                    format!("no data for event {}. req: {:?}", event_id.0, req).into(),
                    None,
                ))));
            }
        };

        let _initial_request = events.pop_front().unwrap();
        let (sender, response_body) = hyper::Body::channel();
        let body = SdkBody::from(response_body);
        let recording = self.recorded_requests.clone();
        let recorded_request = tokio::spawn(async move {
            let mut data_read = vec![];
            while let Some(data) = req.body_mut().data().await {
                data_read
                    .extend_from_slice(data.expect("in memory request should not fail").as_ref())
            }
            req.map(|_| Bytes::from(data_read))
        });
        let mut recorded_request = Waitable::Loading(recorded_request);
        let fut = async move {
            let resp = loop {
                let event = events
                    .pop_front()
                    .expect("no events, needed a response event");
                match event.action {
                    // to ensure deterministic behavior if the request EOF happens first in the log,
                    // wait for the request body to be done before returning a response.
                    Action::Eof {
                        direction: Direction::Request,
                        ..
                    } => {
                        recorded_request.wait().await;
                    }
                    Action::Request { .. } => panic!("invalid"),
                    Action::Response {
                        response: Err(error),
                    } => break Err(ConnectorError::other(error.0.into(), None)),
                    Action::Response {
                        response: Ok(response),
                    } => {
                        let mut builder = http::Response::builder()
                            .status(response.status)
                            .version(convert_version(&response.version));
                        for (name, values) in response.headers {
                            for value in values {
                                builder = builder.header(&name, &value);
                            }
                        }
                        tokio::spawn(async move {
                            replay_body(events, sender).await;
                            // insert the finalized body into
                        });
                        break Ok(builder.body(body).expect("valid builder"));
                    }

                    Action::Data {
                        direction: Direction::Request,
                        data: _data,
                    } => {
                        tracing::info!("get request data");
                    }
                    Action::Eof {
                        direction: Direction::Response,
                        ..
                    } => panic!("got eof before response"),

                    Action::Data {
                        data: _,
                        direction: Direction::Response,
                    } => panic!("got response data before response"),
                }
            };
            recording.lock().unwrap().insert(event_id, recorded_request);
            resp
        };
        Box::pin(fut)
    }
}
