/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Conversions from Stream-like structs to implementors of `futures::Stream`

use futures_core::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

use aws_smithy_async::future::pagination_stream::PaginationStream;

/// Stream implementor wrapping `PaginationStream`
pub struct PaginationStreamImplStream<Item> {
    pagination_stream: PaginationStream<Item>,
}

impl<Item> PaginationStreamImplStream<Item> {
    /// Create a new Stream object wrapping a `PaginationStream`
    pub fn new(pagination_stream: PaginationStream<Item>) -> Self {
        PaginationStreamImplStream { pagination_stream }
    }
}

impl<Item> Stream for PaginationStreamImplStream<Item> {
    type Item = Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.pagination_stream.poll_next(cx)
    }
}

/// Trait to convert PaginationStream into implementor of `Stream`
pub trait PaginationStreamExt<Item> {
    /// Convert PaginationStream into implementor of `Stream`
    ///
    /// # Example
    /// ```no_run
    /// # use aws_smithy_async::future::pagination_stream::PaginationStream;
    /// use aws_smithy_types_convert::stream::PaginationStreamExt;
    /// // Assuming you have obtained a pagination stream, by something like:
    /// // ```
    /// // let pagination_stream = s3_client
    /// //     .list_objects_v2()
    /// //     .bucket(bucket)
    /// //     .into_paginator()
    /// //     .send();
    /// // ```
    /// # let pagination_stream: PaginationStream<i32> = unimplemented!();
    /// let futures_stream = pagination_stream.into_stream_03x();
    /// ```
    fn into_stream_03x(self) -> PaginationStreamImplStream<Item>;
}

impl<Item> PaginationStreamExt<Item> for PaginationStream<Item> {
    fn into_stream_03x(self) -> PaginationStreamImplStream<Item> {
        PaginationStreamImplStream {
            pagination_stream: self,
        }
    }
}
