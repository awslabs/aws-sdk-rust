/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Checksum support for HTTP requests and responses.

/// Support for the `http-body-0-4` and `http-0-2` crates.
#[cfg(feature = "http-body-0-4-x")]
pub mod http_body_0_4_x {
    use crate::Compress;
    use http_0_2::header::{HeaderName, HeaderValue};

    /// Implementors of this trait can be used to compress HTTP requests.
    pub trait CompressRequest: Compress + CloneCompressRequest {
        /// Return the header name for the content-encoding header.
        fn header_name(&self) -> HeaderName {
            HeaderName::from_static("content-encoding")
        }

        /// Return the header value for the content-encoding header.
        fn header_value(&self) -> HeaderValue;
    }

    /// Enables CompressRequest implementors to be cloned.
    pub trait CloneCompressRequest {
        /// Clone this request compressor.
        fn clone_request_compressor(&self) -> Box<dyn CompressRequest>;
    }

    impl<T> CloneCompressRequest for T
    where
        T: CompressRequest + Clone + 'static,
    {
        fn clone_request_compressor(&self) -> Box<dyn CompressRequest> {
            Box::new(self.clone())
        }
    }

    impl Clone for Box<dyn CompressRequest> {
        fn clone(&self) -> Self {
            self.clone_request_compressor()
        }
    }
}

/// Support for the `http-body-1-0` and `http-1-0` crates.
#[cfg(feature = "http-body-1-x")]
pub mod http_body_1_x {
    use crate::Compress;
    use http_1_0::header::{HeaderName, HeaderValue};

    /// Implementors of this trait can be used to compress HTTP requests.
    pub trait CompressRequest: Compress + CloneCompressRequest {
        /// Return the header name for the content-encoding header.
        fn header_name(&self) -> HeaderName {
            HeaderName::from_static("content-encoding")
        }

        /// Return the header value for the content-encoding header.
        fn header_value(&self) -> HeaderValue;
    }

    /// Enables CompressRequest implementors to be cloned.
    pub trait CloneCompressRequest {
        /// Clone this request compressor.
        fn clone_request_compressor(&self) -> Box<dyn CompressRequest>;
    }

    impl<T> CloneCompressRequest for T
    where
        T: CompressRequest + Clone + 'static,
    {
        fn clone_request_compressor(&self) -> Box<dyn CompressRequest> {
            Box::new(self.clone())
        }
    }

    impl Clone for Box<dyn CompressRequest> {
        fn clone(&self) -> Self {
            self.clone_request_compressor()
        }
    }
}
