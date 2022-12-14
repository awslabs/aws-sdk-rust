/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Utility to drive a stream with an async function and a channel.

use crate::future::rendezvous;
use futures_util::StreamExt;
use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_stream::{Iter, Once, Stream};

pin_project! {
    /// Utility to drive a stream with an async function and a channel.
    ///
    /// The closure is passed a reference to a `Sender` which acts as a rendezvous channel. Messages
    /// sent to the sender will be emitted to the stream. Because the stream is 1-bounded, the function
    /// will not proceed until the stream is read.
    ///
    /// This utility is used by generated paginators to generate a stream of paginated results.
    ///
    /// If `tx.send` returns an error, the function MUST return immediately.
    ///
    /// # Examples
    /// ```no_run
    /// use tokio_stream::StreamExt;
    /// # async fn docs() {
    /// use aws_smithy_async::future::fn_stream::FnStream;
    /// let stream = FnStream::new(|tx| Box::pin(async move {
    ///     if let Err(_) = tx.send("Hello!").await {
    ///         return;
    ///     }
    ///     if let Err(_) = tx.send("Goodbye!").await {
    ///         return;
    ///     }
    /// }));
    /// assert_eq!(stream.collect::<Vec<_>>().await, vec!["Hello!", "Goodbye!"]);
    /// # }
    pub struct FnStream<Item, F> {
        #[pin]
        rx: rendezvous::Receiver<Item>,
        #[pin]
        generator: Option<F>,
    }
}

impl<Item, F> FnStream<Item, F> {
    /// Creates a new function based stream driven by `generator`.
    ///
    /// For examples, see the documentation for [`FnStream`]
    pub fn new<T>(generator: T) -> Self
    where
        T: FnOnce(rendezvous::Sender<Item>) -> F,
    {
        let (tx, rx) = rendezvous::channel::<Item>();
        Self {
            rx,
            generator: Some(generator(tx)),
        }
    }
}

impl<Item, F> Stream for FnStream<Item, F>
where
    F: Future<Output = ()>,
{
    type Item = Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut me = self.project();
        match me.rx.poll_recv(cx) {
            Poll::Ready(item) => Poll::Ready(item),
            Poll::Pending => {
                if let Some(generator) = me.generator.as_mut().as_pin_mut() {
                    if generator.poll(cx).is_ready() {
                        // if the generator returned ready we MUST NOT poll it again—doing so
                        // will cause a panic.
                        me.generator.set(None);
                    }
                }
                Poll::Pending
            }
        }
    }
}

/// Utility wrapper to flatten paginated results
///
/// When flattening paginated results, it's most convenient to produce an iterator where the `Result`
/// is present in each item. This provides `items()` which can wrap an stream of `Result<Page, Err>`
/// and produce a stream of `Result<Item, Err>`.
#[derive(Debug)]
pub struct TryFlatMap<I>(I);

impl<I> TryFlatMap<I> {
    /// Create a `TryFlatMap` that wraps the input
    pub fn new(i: I) -> Self {
        Self(i)
    }

    /// Produce a new [`Stream`] by mapping this stream with `map` then flattening the result
    pub fn flat_map<M, Item, Iter, Page, Err>(self, map: M) -> impl Stream<Item = Result<Item, Err>>
    where
        I: Stream<Item = Result<Page, Err>>,
        M: Fn(Page) -> Iter,
        Iter: IntoIterator<Item = Item>,
    {
        self.0.flat_map(move |page| match page {
            Ok(page) => OnceOrMany::Many {
                many: tokio_stream::iter(map(page).into_iter().map(Ok)),
            },
            Err(e) => OnceOrMany::Once {
                once: tokio_stream::once(Err(e)),
            },
        })
    }
}

pin_project! {
    /// Helper enum to to support returning `Once` and `Iter` from `Items::items`
    #[project = OnceOrManyProj]
    enum OnceOrMany<Item, Many> {
        Many { #[pin] many: Iter<Many> },
        Once { #[pin] once: Once<Item> },
    }
}

impl<Item, Iter> Stream for OnceOrMany<Item, Iter>
where
    Iter: Iterator<Item = Item>,
{
    type Item = Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let me = self.project();
        match me {
            OnceOrManyProj::Many { many } => many.poll_next(cx),
            OnceOrManyProj::Once { once } => once.poll_next(cx),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::future::fn_stream::{FnStream, TryFlatMap};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tokio_stream::StreamExt;

    /// basic test of FnStream functionality
    #[tokio::test]
    async fn fn_stream_returns_results() {
        tokio::time::pause();
        let mut stream = FnStream::new(|tx| {
            Box::pin(async move {
                tx.send("1").await.expect("failed to send");
                tokio::time::sleep(Duration::from_secs(1)).await;
                tokio::time::sleep(Duration::from_secs(1)).await;
                tx.send("2").await.expect("failed to send");
                tokio::time::sleep(Duration::from_secs(1)).await;
                tx.send("3").await.expect("failed to send");
            })
        });
        let mut out = vec![];
        while let Some(value) = stream.next().await {
            out.push(value);
        }
        assert_eq!(out, vec!["1", "2", "3"]);
    }

    // smithy-rs#1902: there was a bug where we could continue to poll the generator after it
    // had returned Poll::Ready. This test case leaks the tx half so that the channel stays open
    // but the send side generator completes. By calling `poll` multiple times on the resulting future,
    // we can trigger the bug and validate the fix.
    #[tokio::test]
    async fn fn_stream_doesnt_poll_after_done() {
        let mut stream = FnStream::new(|tx| {
            Box::pin(async move {
                assert!(tx.send("blah").await.is_ok());
                Box::leak(Box::new(tx));
            })
        });
        assert_eq!(stream.next().await, Some("blah"));
        let mut test_stream = tokio_test::task::spawn(stream);
        assert!(test_stream.poll_next().is_pending());
        assert!(test_stream.poll_next().is_pending());
    }

    /// Tests that the generator will not advance until demand exists
    #[tokio::test]
    async fn waits_for_reader() {
        let progress = Arc::new(Mutex::new(0));
        let mut stream = FnStream::new(|tx| {
            let progress = progress.clone();
            Box::pin(async move {
                *progress.lock().unwrap() = 1;
                tx.send("1").await.expect("failed to send");
                *progress.lock().unwrap() = 2;
                tx.send("2").await.expect("failed to send");
                *progress.lock().unwrap() = 3;
                tx.send("3").await.expect("failed to send");
                *progress.lock().unwrap() = 4;
            })
        });
        assert_eq!(*progress.lock().unwrap(), 0);
        stream.next().await.expect("ready");
        assert_eq!(*progress.lock().unwrap(), 1);

        assert_eq!(stream.next().await.expect("ready"), "2");
        assert_eq!(*progress.lock().unwrap(), 2);

        let _ = stream.next().await.expect("ready");
        assert_eq!(*progress.lock().unwrap(), 3);
        assert_eq!(stream.next().await, None);
        assert_eq!(*progress.lock().unwrap(), 4);
    }

    #[tokio::test]
    async fn generator_with_errors() {
        let mut stream = FnStream::new(|tx| {
            Box::pin(async move {
                for i in 0..5 {
                    if i != 2 {
                        if tx.send(Ok(i)).await.is_err() {
                            return;
                        }
                    } else {
                        tx.send(Err(i)).await.unwrap();
                        return;
                    }
                }
            })
        });
        let mut out = vec![];
        while let Some(Ok(value)) = stream.next().await {
            out.push(value);
        }
        assert_eq!(out, vec![0, 1]);
    }

    #[tokio::test]
    async fn flatten_items_ok() {
        #[derive(Debug)]
        struct Output {
            items: Vec<u8>,
        }
        let stream = FnStream::new(|tx| {
            Box::pin(async move {
                tx.send(Ok(Output {
                    items: vec![1, 2, 3],
                }))
                .await
                .unwrap();
                tx.send(Ok(Output {
                    items: vec![4, 5, 6],
                }))
                .await
                .unwrap();
            })
        });
        assert_eq!(
            TryFlatMap(stream)
                .flat_map(|output| output.items.into_iter())
                .collect::<Result<Vec<_>, &str>>()
                .await,
            Ok(vec![1, 2, 3, 4, 5, 6])
        )
    }

    #[tokio::test]
    async fn flatten_items_error() {
        #[derive(Debug)]
        struct Output {
            items: Vec<u8>,
        }
        let stream = FnStream::new(|tx| {
            Box::pin(async move {
                tx.send(Ok(Output {
                    items: vec![1, 2, 3],
                }))
                .await
                .unwrap();
                tx.send(Err("bummer")).await.unwrap();
            })
        });
        assert_eq!(
            TryFlatMap(stream)
                .flat_map(|output| output.items.into_iter())
                .collect::<Result<Vec<_>, &str>>()
                .await,
            Err("bummer")
        )
    }
}
