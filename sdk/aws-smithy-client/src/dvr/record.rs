/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::{Context, Poll};

use http_body::Body;
use tokio::task::JoinHandle;
use tower::Service;

use aws_smithy_http::body::SdkBody;

use crate::dvr::{self, Action, BodyData, ConnectionId, Direction, Error, NetworkTraffic, Version};

use super::Event;
use std::fmt::Display;
use std::io;
use std::path::Path;

/// Recording Connection Wrapper
///
/// RecordingConnection wraps an inner connection and records all traffic, enabling traffic replay.
#[derive(Clone, Debug)]
pub struct RecordingConnection<S> {
    pub(crate) data: Arc<Mutex<Vec<Event>>>,
    pub(crate) num_events: Arc<AtomicUsize>,
    pub(crate) inner: S,
}

#[cfg(all(feature = "rustls", feature = "client-hyper"))]
impl RecordingConnection<crate::hyper_ext::Adapter<crate::conns::Https>> {
    /// Construct a recording connection wrapping a default HTTPS implementation
    pub fn https() -> Self {
        Self {
            data: Default::default(),
            inner: crate::hyper_ext::Adapter::builder().build(crate::conns::https()),
            num_events: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl<S> RecordingConnection<S> {
    /// Create a new recording connection from a connection
    pub fn new(connection: S) -> Self {
        Self {
            data: Default::default(),
            inner: connection,
            num_events: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Return the traffic recorded by this connection
    pub fn events(&self) -> MutexGuard<'_, Vec<Event>> {
        self.data.lock().unwrap()
    }

    /// NetworkTraffic struct suitable for serialization
    pub fn network_traffic(&self) -> NetworkTraffic {
        NetworkTraffic {
            events: self.events().clone(),
            docs: Some("todo docs".into()),
            version: Version::V0,
        }
    }

    /// Dump the network traffic to a file
    pub fn dump_to_file(&self, path: impl AsRef<Path>) -> Result<(), io::Error> {
        std::fs::write(
            path,
            serde_json::to_string(&self.network_traffic()).unwrap(),
        )
    }

    fn next_id(&self) -> ConnectionId {
        ConnectionId(self.num_events.fetch_add(1, Ordering::Relaxed))
    }
}

fn record_body(
    body: &mut SdkBody,
    event_id: ConnectionId,
    direction: Direction,
    event_bus: Arc<Mutex<Vec<Event>>>,
) -> JoinHandle<()> {
    let (sender, output_body) = hyper::Body::channel();
    let real_body = std::mem::replace(body, SdkBody::from(output_body));
    tokio::spawn(async move {
        let mut real_body = real_body;
        let mut sender = sender;
        loop {
            let data = real_body.data().await;
            match data {
                Some(Ok(data)) => {
                    event_bus.lock().unwrap().push(Event {
                        connection_id: event_id,
                        action: Action::Data {
                            data: BodyData::from(data.clone()),
                            direction,
                        },
                    });
                    // This happens if the real connection is closed during recording.
                    // Need to think more carefully if this is the correct thing to log in this
                    // case.
                    if sender.send_data(data).await.is_err() {
                        event_bus.lock().unwrap().push(Event {
                            connection_id: event_id,
                            action: Action::Eof {
                                direction: direction.opposite(),
                                ok: false,
                            },
                        })
                    };
                }
                None => {
                    event_bus.lock().unwrap().push(Event {
                        connection_id: event_id,
                        action: Action::Eof {
                            ok: true,
                            direction,
                        },
                    });
                    drop(sender);
                    break;
                }
                Some(Err(_err)) => {
                    event_bus.lock().unwrap().push(Event {
                        connection_id: event_id,
                        action: Action::Eof {
                            ok: false,
                            direction,
                        },
                    });
                    sender.abort();
                    break;
                }
            }
        }
    })
}

impl<S, ResponseBody> tower::Service<http::Request<SdkBody>> for RecordingConnection<S>
where
    S: Service<http::Request<SdkBody>, Response = http::Response<ResponseBody>>
        + Send
        + Clone
        + 'static,
    S::Error: Display + Send + Sync + 'static,
    S::Future: Send + 'static,
    ResponseBody: Into<SdkBody>,
{
    type Response = http::Response<SdkBody>;
    type Error = S::Error;
    #[allow(clippy::type_complexity)]
    type Future =
        Pin<Box<dyn Future<Output = Result<http::Response<SdkBody>, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: http::Request<SdkBody>) -> Self::Future {
        let event_id = self.next_id();
        // A request has two 3 phases:
        // 1. A "Request" phase. This is initial HTTP request, headers, & URI
        // 2. A body phase. This may contain multiple data segments.
        // 3. A finalization phase. An EOF of some sort is sent on the body to indicate that
        // the channel should be closed.

        // Phase 1: the initial http request
        self.data.lock().unwrap().push(Event {
            connection_id: event_id,
            action: Action::Request {
                request: dvr::Request::from(&req),
            },
        });

        // Phase 2: Swap out the real request body for one that will log all traffic that passes
        // through it
        // This will also handle phase three when the request body runs out of data.
        record_body(
            req.body_mut(),
            event_id,
            Direction::Request,
            self.data.clone(),
        );
        let events = self.data.clone();
        // create a channel we'll use to stream the data while reading it
        let resp_fut = self.inner.call(req);
        let fut = async move {
            let resp = resp_fut.await;
            match resp {
                Ok(resp) => {
                    // wrap the hyper body in an SDK body
                    let mut resp = resp.map(|body| body.into());

                    // push the initial response event
                    events.lock().unwrap().push(Event {
                        connection_id: event_id,
                        action: Action::Response {
                            response: Ok(dvr::Response::from(&resp)),
                        },
                    });

                    // instrument the body and record traffic
                    record_body(resp.body_mut(), event_id, Direction::Response, events);
                    Ok(resp)
                }
                Err(e) => {
                    events.lock().unwrap().push(Event {
                        connection_id: event_id,
                        action: Action::Response {
                            response: Err(Error(format!("{}", &e))),
                        },
                    });
                    Err(e)
                }
            }
        };
        Box::pin(fut)
    }
}
