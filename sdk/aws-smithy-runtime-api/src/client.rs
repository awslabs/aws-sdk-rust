/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

macro_rules! new_type_future {
    (
        doc = $type_docs:literal,
        pub struct $future_name:ident<$output:ty, $err:ty>,
    ) => {
        pin_project_lite::pin_project! {
            #[allow(clippy::type_complexity)]
            #[doc = $type_docs]
            pub struct $future_name {
                #[pin]
                inner: aws_smithy_async::future::now_or_later::NowOrLater<
                    Result<$output, $err>,
                    aws_smithy_async::future::BoxFuture<$output, $err>
                >,
            }
        }

        impl $future_name {
            #[doc = concat!("Create a new `", stringify!($future_name), "` with the given future.")]
            pub fn new<F>(future: F) -> Self
            where
                F: std::future::Future<Output = Result<$output, $err>> + Send + 'static,
            {
                Self {
                    inner: aws_smithy_async::future::now_or_later::NowOrLater::new(Box::pin(future)),
                }
            }

            #[doc = concat!("
            Create a new `", stringify!($future_name), "` with the given boxed future.

            Use this if you already have a boxed future to avoid double boxing it.
            ")]
            pub fn new_boxed(
                future: std::pin::Pin<
                    Box<dyn std::future::Future<Output = Result<$output, $err>> + Send>,
                >,
            ) -> Self {
                Self {
                    inner: aws_smithy_async::future::now_or_later::NowOrLater::new(future),
                }
            }

            #[doc = concat!("Create a `", stringify!($future_name), "` that is immediately ready with the given result.")]
            pub fn ready(result: Result<$output, $err>) -> Self {
                Self {
                    inner: aws_smithy_async::future::now_or_later::NowOrLater::ready(result),
                }
            }
        }

        impl std::future::Future for $future_name {
            type Output = Result<$output, $err>;

            fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
                let this = self.project();
                this.inner.poll(cx)
            }
        }
    };
}

pub mod dns;

pub mod endpoint;

/// Smithy identity used by auth and signing.
pub mod identity;

pub mod interceptors;

pub mod orchestrator;

pub mod retries;

pub mod runtime_components;

pub mod runtime_plugin;

pub mod auth;

pub mod http;

pub mod ser_de;
