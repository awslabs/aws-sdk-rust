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
        // this code is called quite frequently in production—one every millisecond or so when downloading
        // a stream. However, SystemTime::now is on the order of nanoseconds
        let now = self.time_source.now();
        // Attempt to read the data from the inner body, then update the
        // throughput logs.
        let mut this = self.as_mut().project();
        let poll_res = match this.inner.poll_data(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                tracing::trace!("received data: {}", bytes.len());
                this.throughput_logs.push((now, bytes.len() as u64));
                Poll::Ready(Some(Ok(bytes)))
            }
            Poll::Pending => {
                tracing::trace!("received poll pending");
                this.throughput_logs.push((now, 0));
                Poll::Pending
            }
            // If we've read all the data or an error occurred, then return that result.
            res => return res,
        };

        // Check the sleep future to see if it needs refreshing.
        let mut sleep_fut = this
            .sleep_fut
            .take()
            .unwrap_or_else(|| this.async_sleep.sleep(this.options.check_interval()));
        if let Poll::Ready(()) = pin!(&mut sleep_fut).poll(cx) {
            tracing::trace!("sleep future triggered—triggering a wakeup");
            // Whenever the sleep future expires, we replace it.
            sleep_fut = this.async_sleep.sleep(this.options.check_interval());

            // We also schedule a wake up for current task to ensure that
            // it gets polled at least one more time.
            cx.waker().wake_by_ref();
        };
        this.sleep_fut.replace(sleep_fut);
        let calculated_tpt = match this
            .throughput_logs
            .calculate_throughput(now, this.options.check_window())
        {
            Some(tpt) => tpt,
            None => {
                tracing::trace!("calculated throughput is None!");
                return poll_res;
            }
        };
        tracing::trace!(
            "calculated throughput {:?} (window: {:?})",
            calculated_tpt,
            this.options.check_window()
        );

        // Calculate the current throughput and emit an error if it's too low and
        // the grace period has elapsed.
        let is_below_minimum_throughput = calculated_tpt <= this.options.minimum_throughput();
        if is_below_minimum_throughput {
            // Check the grace period future to see if it needs creating.
            tracing::trace!(
                in_grace_period = this.grace_period_fut.is_some(),
                observed_throughput = ?calculated_tpt,
                minimum_throughput = ?this.options.minimum_throughput(),
                "below minimum throughput"
            );
            let mut grace_period_fut = this
                .grace_period_fut
                .take()
                .unwrap_or_else(|| this.async_sleep.sleep(this.options.grace_period()));
            if let Poll::Ready(()) = pin!(&mut grace_period_fut).poll(cx) {
                // The grace period has ended!
                return Poll::Ready(Some(Err(Box::new(Error::ThroughputBelowMinimum {
                    expected: self.options.minimum_throughput(),
                    actual: calculated_tpt,
                }))));
            };
            this.grace_period_fut.replace(grace_period_fut);
        } else {
            // Ensure we don't have an active grace period future if we're not
            // currently below the minimum throughput.
            let _ = this.grace_period_fut.take();
        }

        poll_res
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
#[cfg(all(test, feature = "connector-hyper-0-14-x", feature = "test-util"))]
mod test {
    use super::{super::Throughput, Error, MinimumThroughputBody};
    use crate::client::http::body::minimum_throughput::options::MinimumThroughputBodyOptions;
    use crate::test_util::capture_test_logs::capture_test_logs;
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
    use std::future::{poll_fn, Future};
    use std::pin::{pin, Pin};
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
            Default::default(),
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
        minimum_throughput: Throughput,
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
                MinimumThroughputBodyOptions::builder()
                    .minimum_throughput(minimum_throughput)
                    .build(),
            ))
        });

        (body.collect(), time_source, async_sleep)
    }

    async fn expect_error(minimum_throughput: Throughput) {
        let (res, ..) =
            eight_byte_per_second_stream_with_minimum_throughput_timeout(minimum_throughput);
        let expected_err = Error::ThroughputBelowMinimum {
            expected: minimum_throughput,
            actual: Throughput::new(8, Duration::from_secs(1)),
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
        let minimum_throughput = Throughput::new_bytes_per_second(9);
        expect_error(minimum_throughput).await;
    }

    async fn expect_success(minimum_throughput: Throughput) {
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
        let (_guard, _) = capture_test_logs();
        // a tiny bit less. To capture 0-throughput properly, we need to allow 0 to be 0
        let minimum_throughput = Throughput::new(31, Duration::from_secs(4));
        expect_success(minimum_throughput).await;
    }

    #[tokio::test]
    async fn test_throughput_timeout_greater_than() {
        let minimum_throughput = Throughput::new(20, Duration::from_secs(3));
        expect_success(minimum_throughput).await;
    }

    // A multiplier for the sine wave amplitude; Chosen arbitrarily.
    const BYTE_COUNT_UPPER_LIMIT: u64 = 1000;

    /// emits 1000B/S for 5 seconds then suddenly stops
    fn sudden_stop(
        async_sleep: impl AsyncSleep + Clone,
    ) -> impl futures_util::Stream<Item = Result<Bytes, Infallible>> {
        let sleep_dur = Duration::from_millis(50);
        fastrand::seed(0);
        futures_util::stream::unfold(1, move |i| {
            let async_sleep = async_sleep.clone();
            async move {
                let number_seconds = (i * sleep_dur).as_secs_f64();
                async_sleep.sleep(sleep_dur).await;
                if number_seconds > 5.0 {
                    Some((Result::<Bytes, Infallible>::Ok(Bytes::new()), i + 1))
                } else {
                    let mut bytes = BytesMut::new();
                    let bytes_per_segment =
                        (BYTE_COUNT_UPPER_LIMIT as f64) * sleep_dur.as_secs_f64();
                    for _ in 0..bytes_per_segment as usize {
                        bytes.put_u8(0)
                    }

                    Some((Result::<Bytes, Infallible>::Ok(bytes.into()), i + 1))
                }
            }
        })
    }

    #[tokio::test]
    async fn test_stalled_stream_detection() {
        test_suddenly_stopping_stream(0, Duration::from_secs(6)).await
    }

    #[tokio::test]
    async fn test_slow_stream_detection() {
        test_suddenly_stopping_stream(BYTE_COUNT_UPPER_LIMIT / 2, Duration::from_secs_f64(5.50))
            .await
    }

    #[tokio::test]
    async fn test_check_interval() {
        let (_guard, _) = capture_test_logs();
        let (ts, sleep) = instant_time_and_sleep(UNIX_EPOCH);
        let mut body = MinimumThroughputBody::new(
            ts,
            sleep.clone(),
            NeverBody,
            MinimumThroughputBodyOptions::builder()
                .check_interval(Duration::from_millis(1234))
                .grace_period(Duration::from_millis(456))
                .build(),
        );
        let mut body = pin!(body);
        let _ = poll_fn(|cx| body.as_mut().poll_data(cx)).await;
        assert_eq!(
            sleep.logs(),
            vec![
                // sleep, by second sleep we know we have no data, then the grace period
                Duration::from_millis(1234),
                Duration::from_millis(1234),
                Duration::from_millis(456)
            ]
        );
    }

    async fn test_suddenly_stopping_stream(throughput_limit: u64, time_until_timeout: Duration) {
        let (_guard, _) = capture_test_logs();
        let options = MinimumThroughputBodyOptions::builder()
            // Minimum throughput per second will be approx. half of the BYTE_COUNT_UPPER_LIMIT.
            .minimum_throughput(Throughput::new_bytes_per_second(throughput_limit))
            .build();
        let (time_source, async_sleep) = instant_time_and_sleep(UNIX_EPOCH);
        let time_clone = time_source.clone();

        let stream = sudden_stop(async_sleep.clone());
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
                    options.clone(),
                ))
            })
            .collect();

        match res.await {
            Ok(_res) => {
                panic!("stream should have timed out");
            }
            Err(err) => {
                dbg!(err);
                assert_eq!(
                    async_sleep.total_duration(),
                    time_until_timeout,
                    "With throughput limit {:?} expected timeout after {:?} (stream starts sending 0's at 5 seconds.",
                    throughput_limit, time_until_timeout
                );
            }
        }
    }
}
