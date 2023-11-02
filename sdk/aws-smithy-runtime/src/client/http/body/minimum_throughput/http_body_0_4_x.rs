/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use super::{BoxError, Error, MinimumThroughputBody};
use aws_smithy_async::rt::sleep::AsyncSleep;
use http_body_0_4::Body;
use std::future::Future;
use std::pin::{pin, Pin};
use std::task::{Context, Poll};

impl<B> Body for MinimumThroughputBody<B>
where
    B: Body<Data = bytes::Bytes, Error = BoxError>,
{
    type Data = bytes::Bytes;
    type Error = BoxError;

    fn poll_data(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let now = self.time_source.now();
        // Attempt to read the data from the inner body, then update the
        // throughput logs.
        let mut this = self.as_mut().project();
        // Push a start log if we haven't already done so.
        if this.throughput_logs.is_empty() {
            this.throughput_logs.push((now, 0));
        }
        let poll_res = match this.inner.poll_data(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                this.throughput_logs.push((now, bytes.len() as u64));
                Poll::Ready(Some(Ok(bytes)))
            }
            Poll::Pending => Poll::Pending,
            // If we've read all the data or an error occurred, then return that result.
            res => return res,
        };

        // Check the sleep future to see if it needs refreshing.
        let mut sleep_fut = this.sleep_fut.take().unwrap_or_else(|| {
            this.async_sleep
                .sleep(this.minimum_throughput.per_time_elapsed())
        });
        if let Poll::Ready(()) = pin!(&mut sleep_fut).poll(cx) {
            // Whenever the sleep future expires, we replace it.
            sleep_fut = this
                .async_sleep
                .sleep(this.minimum_throughput.per_time_elapsed());

            // We also schedule a wake up for current task to ensure that
            // it gets polled at least one more time.
            cx.waker().wake_by_ref();
        };
        this.sleep_fut.replace(sleep_fut);

        // Calculate the current throughput and emit an error if it's too low.
        let actual_throughput = this.throughput_logs.calculate_throughput(now);
        let is_below_minimum_throughput = actual_throughput
            .map(|t| t < self.minimum_throughput)
            .unwrap_or_default();
        if is_below_minimum_throughput {
            Poll::Ready(Some(Err(Box::new(Error::ThroughputBelowMinimum {
                expected: self.minimum_throughput,
                actual: actual_throughput.unwrap(),
            }))))
        } else {
            poll_res
        }
    }

    fn poll_trailers(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<http::HeaderMap>, Self::Error>> {
        let this = self.as_mut().project();
        this.inner.poll_trailers(cx)
    }
}

// These tests use `hyper::body::Body::wrap_stream`
#[cfg(all(test, feature = "connector-hyper-0-14-x"))]
mod test {
    use super::{super::Throughput, Error, MinimumThroughputBody};
    use aws_smithy_async::rt::sleep::AsyncSleep;
    use aws_smithy_async::test_util::{instant_time_and_sleep, InstantSleep, ManualTimeSource};
    use aws_smithy_types::body::SdkBody;
    use aws_smithy_types::byte_stream::{AggregatedBytes, ByteStream};
    use aws_smithy_types::error::display::DisplayErrorContext;
    use bytes::{BufMut, Bytes, BytesMut};
    use http::HeaderMap;
    use http_body_0_4::Body;
    use once_cell::sync::Lazy;
    use pretty_assertions::assert_eq;
    use std::convert::Infallible;
    use std::error::Error as StdError;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use std::time::{Duration, UNIX_EPOCH};

    struct NeverBody;

    impl Body for NeverBody {
        type Data = Bytes;
        type Error = Box<(dyn StdError + Send + Sync + 'static)>;

        fn poll_data(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
            Poll::Pending
        }

        fn poll_trailers(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
            unreachable!("body can't be read, so this won't be called")
        }
    }

    #[tokio::test()]
    async fn test_self_waking() {
        let (time_source, async_sleep) = instant_time_and_sleep(UNIX_EPOCH);
        let mut body = MinimumThroughputBody::new(
            time_source.clone(),
            async_sleep.clone(),
            NeverBody,
            (1, Duration::from_secs(1)),
        );
        time_source.advance(Duration::from_secs(1));
        let actual_err = body.data().await.expect("next chunk exists").unwrap_err();
        let expected_err = Error::ThroughputBelowMinimum {
            expected: (1, Duration::from_secs(1)).into(),
            actual: (0, Duration::from_secs(1)).into(),
        };

        assert_eq!(expected_err.to_string(), actual_err.to_string());
    }

    fn create_test_stream(
        async_sleep: impl AsyncSleep + Clone,
    ) -> impl futures_util::Stream<Item = Result<Bytes, Infallible>> {
        futures_util::stream::unfold(1, move |state| {
            let async_sleep = async_sleep.clone();
            async move {
                if state > 255 {
                    None
                } else {
                    async_sleep.sleep(Duration::from_secs(1)).await;
                    Some((
                        Result::<_, Infallible>::Ok(Bytes::from_static(b"00000000")),
                        state + 1,
                    ))
                }
            }
        })
    }

    static EXPECTED_BYTES: Lazy<Vec<u8>> =
        Lazy::new(|| (1..=255).flat_map(|_| b"00000000").copied().collect());

    fn eight_byte_per_second_stream_with_minimum_throughput_timeout(
        minimum_throughput: (u64, Duration),
    ) -> (
        impl Future<Output = Result<AggregatedBytes, aws_smithy_types::byte_stream::error::Error>>,
        ManualTimeSource,
        InstantSleep,
    ) {
        let (time_source, async_sleep) = instant_time_and_sleep(UNIX_EPOCH);
        let time_clone = time_source.clone();

        // Will send ~8 bytes per second.
        let stream = create_test_stream(async_sleep.clone());
        let body = ByteStream::new(SdkBody::from_body_0_4(hyper_0_14::body::Body::wrap_stream(
            stream,
        )));
        let body = body.map(move |body| {
            let time_source = time_clone.clone();
            // We don't want to log these sleeps because it would duplicate
            // the `sleep` calls being logged by the MTB
            let async_sleep = InstantSleep::unlogged();
            SdkBody::from_body_0_4(MinimumThroughputBody::new(
                time_source,
                async_sleep,
                body,
                minimum_throughput,
            ))
        });

        (body.collect(), time_source, async_sleep)
    }

    async fn expect_error(minimum_throughput: (u64, Duration)) {
        let (res, ..) =
            eight_byte_per_second_stream_with_minimum_throughput_timeout(minimum_throughput);
        let expected_err = Error::ThroughputBelowMinimum {
            expected: minimum_throughput.into(),
            actual: Throughput::new(8.889, Duration::from_secs(1)),
        };
        match res.await {
            Ok(_) => {
                panic!(
                    "response succeeded instead of returning the expected error '{expected_err}'"
                )
            }
            Err(actual_err) => {
                assert_eq!(
                    expected_err.to_string(),
                    // We need to source this so that we don't get the streaming error it's wrapped in.
                    actual_err.source().unwrap().to_string()
                );
            }
        }
    }

    #[tokio::test]
    async fn test_throughput_timeout_less_than() {
        let minimum_throughput = (9, Duration::from_secs(1));
        expect_error(minimum_throughput).await;
    }

    async fn expect_success(minimum_throughput: (u64, Duration)) {
        let (res, time_source, async_sleep) =
            eight_byte_per_second_stream_with_minimum_throughput_timeout(minimum_throughput);
        match res.await {
            Ok(res) => {
                assert_eq!(255.0, time_source.seconds_since_unix_epoch());
                assert_eq!(Duration::from_secs(255), async_sleep.total_duration());
                assert_eq!(*EXPECTED_BYTES, res.to_vec());
            }
            Err(err) => panic!("{}", DisplayErrorContext(err.source().unwrap())),
        }
    }

    #[tokio::test]
    async fn test_throughput_timeout_equal_to() {
        let minimum_throughput = (32, Duration::from_secs(4));
        expect_success(minimum_throughput).await;
    }

    #[tokio::test]
    async fn test_throughput_timeout_greater_than() {
        let minimum_throughput = (20, Duration::from_secs(3));
        expect_success(minimum_throughput).await;
    }

    // A multiplier for the sine wave amplitude; Chosen arbitrarily.
    const BYTE_COUNT_UPPER_LIMIT: f64 = 100.0;

    fn create_shrinking_sine_wave_stream(
        async_sleep: impl AsyncSleep + Clone,
    ) -> impl futures_util::Stream<Item = Result<Bytes, Infallible>> {
        futures_util::stream::unfold(1, move |i| {
            let async_sleep = async_sleep.clone();
            async move {
                if i > 255 {
                    None
                } else {
                    let byte_count = (i as f64).sin().floor().abs() + 1.0 / (i as f64 + 1.0);
                    let byte_count = (byte_count * BYTE_COUNT_UPPER_LIMIT) as u64;
                    let mut bytes = BytesMut::new();
                    bytes.put_u8(i as u8);
                    if byte_count > 0 {
                        for _ in 0..byte_count {
                            bytes.put_u8(0)
                        }
                    }

                    async_sleep.sleep(Duration::from_secs(1)).await;
                    Some((Result::<Bytes, Infallible>::Ok(bytes.into()), i + 1))
                }
            }
        })
    }

    #[tokio::test]
    async fn test_throughput_timeout_shrinking_sine_wave() {
        // Minimum throughput per second will be approx. half of the BYTE_COUNT_UPPER_LIMIT.
        let minimum_throughput = (
            BYTE_COUNT_UPPER_LIMIT as u64 / 2 + 2,
            Duration::from_secs(1),
        );
        let (time_source, async_sleep) = instant_time_and_sleep(UNIX_EPOCH);
        let time_clone = time_source.clone();

        let stream = create_shrinking_sine_wave_stream(async_sleep.clone());
        let body = ByteStream::new(SdkBody::from_body_0_4(hyper_0_14::body::Body::wrap_stream(
            stream,
        )));
        let res = body
            .map(move |body| {
                let time_source = time_clone.clone();
                // We don't want to log these sleeps because it would duplicate
                // the `sleep` calls being logged by the MTB
                let async_sleep = InstantSleep::unlogged();
                SdkBody::from_body_0_4(MinimumThroughputBody::new(
                    time_source,
                    async_sleep,
                    body,
                    minimum_throughput,
                ))
            })
            .collect();

        match res.await {
            Ok(_res) => {
                assert_eq!(255.0, time_source.seconds_since_unix_epoch());
                assert_eq!(Duration::from_secs(255), async_sleep.total_duration());
            }
            Err(err) => panic!(
                "test stopped after {:?} due to {}",
                async_sleep.total_duration(),
                DisplayErrorContext(err.source().unwrap())
            ),
        }
    }
}
