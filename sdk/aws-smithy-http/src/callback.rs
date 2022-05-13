/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! A module for traits that define callbacks that will be called at specific points in an HTTP request's lifecycle.

use http::{HeaderMap, HeaderValue};

type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// A callback that, when inserted into a request body, will be called for corresponding lifecycle events.
pub trait BodyCallback: Send + Sync {
    /// This lifecycle function is called for each chunk **successfully** read. If an error occurs while reading a chunk,
    /// this method will not be called. This method takes `&mut self` so that implementors may modify an implementing
    /// struct/enum's internal state. Implementors may return an error.
    fn update(&mut self, bytes: &[u8]) -> Result<(), BoxError> {
        // "Use" bytes so that the compiler won't complain.
        let _ = bytes;
        Ok(())
    }

    /// This callback is called once all chunks have been read. If the callback encountered one or more errors
    /// while running `update`s, this is how those errors are raised. Implementors may return a [`HeaderMap`][HeaderMap]
    /// that will be appended to the HTTP body as a trailer. This is only useful to do for streaming requests.
    fn trailers(&self) -> Result<Option<HeaderMap<HeaderValue>>, BoxError> {
        Ok(None)
    }

    /// Create a new `BodyCallback` from an existing one. This is called when a `BodyCallback` needs to be
    /// re-initialized with default state. For example: when a request has a body that needs to be
    /// rebuilt, all callbacks for that body need to be run again but with a fresh internal state.
    fn make_new(&self) -> Box<dyn BodyCallback>;
}

impl BodyCallback for Box<dyn BodyCallback> {
    fn update(&mut self, bytes: &[u8]) -> Result<(), BoxError> {
        self.as_mut().update(bytes)
    }
    fn trailers(&self) -> Result<Option<HeaderMap<HeaderValue>>, BoxError> {
        self.as_ref().trailers()
    }
    fn make_new(&self) -> Box<dyn BodyCallback> {
        self.as_ref().make_new()
    }
}

#[cfg(test)]
mod tests {
    use super::{BodyCallback, BoxError};
    use crate::body::SdkBody;
    use crate::byte_stream::ByteStream;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn callbacks_are_called_for_update() {
        struct CallbackA;
        struct CallbackB;

        impl BodyCallback for CallbackA {
            fn update(&mut self, _bytes: &[u8]) -> Result<(), BoxError> {
                tracing::debug!("callback A was called");

                Ok(())
            }

            fn make_new(&self) -> Box<dyn BodyCallback> {
                Box::new(Self)
            }
        }

        impl BodyCallback for CallbackB {
            fn update(&mut self, _bytes: &[u8]) -> Result<(), BoxError> {
                tracing::debug!("callback B was called");

                Ok(())
            }

            fn make_new(&self) -> Box<dyn BodyCallback> {
                Box::new(Self)
            }
        }

        let mut body = SdkBody::from("test");
        body.with_callback(Box::new(CallbackA))
            .with_callback(Box::new(CallbackB));

        let body = ByteStream::from(body).collect().await.unwrap().into_bytes();
        let body = std::str::from_utf8(&body).unwrap();

        // Assert that the body that went in is the same as the body coming out.
        assert_eq!(body, "test");

        // Test that all callbacks were called.
        assert!(logs_contain("callback A was called"));
        assert!(logs_contain("callback B was called"));
    }

    struct TestCallback {
        times_called: Arc<AtomicUsize>,
    }

    impl BodyCallback for TestCallback {
        fn update(&mut self, _bytes: &[u8]) -> Result<(), BoxError> {
            self.times_called.fetch_add(1, Ordering::SeqCst);

            Ok(())
        }

        fn make_new(&self) -> Box<dyn BodyCallback> {
            Box::new(Self {
                times_called: Arc::new(AtomicUsize::new(0)),
            })
        }
    }

    #[tokio::test]
    async fn callback_for_buffered_body_is_called_once() {
        let times_called = Arc::new(AtomicUsize::new(0));
        let test_text: String = (0..=1000)
            .into_iter()
            .map(|n| format!("line {}\n", n))
            .collect();

        {
            let mut body = SdkBody::from(test_text);
            let callback = TestCallback {
                times_called: times_called.clone(),
            };
            body.with_callback(Box::new(callback));
            let _body = ByteStream::new(body).collect().await.unwrap().into_bytes();
        }

        let times_called = Arc::try_unwrap(times_called).unwrap();
        let times_called = times_called.into_inner();

        // Callback only gets called once because it's not a streaming body
        assert_eq!(times_called, 1);
    }

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn callback_for_streaming_body_is_called_per_chunk() {
        // Include a large body of text for testing
        let times_called = Arc::new(AtomicUsize::new(0));

        {
            let test_stream = tokio_stream::iter(
                (1..=1000)
                    .into_iter()
                    .map(|n| -> Result<String, std::io::Error> { Ok(format!("line {}\n", n)) }),
            );
            let mut body = SdkBody::from(hyper::body::Body::wrap_stream(test_stream));
            tracing::trace!("{:?}", body);
            assert!(logs_contain("Streaming(Body(Streaming))"));

            let callback = TestCallback {
                times_called: times_called.clone(),
            };
            body.with_callback(Box::new(callback));
            let _body = ByteStream::new(body).collect().await.unwrap().into_bytes();
        }

        let times_called = Arc::try_unwrap(times_called).unwrap();
        let times_called = times_called.into_inner();

        // Callback is called once per chunk
        assert_eq!(times_called, 1000);
    }
}
