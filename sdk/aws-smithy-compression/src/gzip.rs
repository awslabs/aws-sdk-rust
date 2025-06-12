/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::{Compress, CompressionOptions};
use aws_smithy_runtime_api::box_error::BoxError;
use flate2::write::GzEncoder;
use std::io::prelude::*;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Gzip {
    compression: flate2::Compression,
}

impl Gzip {
    fn compress_bytes(&self, bytes: &[u8], writer: impl Write) -> Result<(), BoxError> {
        let mut encoder = GzEncoder::new(writer, self.compression);
        encoder.write_all(bytes)?;
        encoder.try_finish()?;

        Ok(())
    }
}

impl Compress for Gzip {
    fn compress_bytes(&mut self, bytes: &[u8], writer: &mut dyn Write) -> Result<(), BoxError> {
        Gzip::compress_bytes(self, bytes, writer)
    }
}

#[cfg(feature = "http-body-0-4-x")]
mod http_body_0_4_x {
    use crate::http::http_body_0_4_x::CompressRequest;

    impl CompressRequest for super::Gzip {
        fn header_value(&self) -> http_0_2::HeaderValue {
            http_0_2::HeaderValue::from_static("gzip")
        }
    }
}

#[cfg(feature = "http-body-1-x")]
mod http_body_1_x {
    use crate::http::http_body_1_x::CompressRequest;

    impl CompressRequest for super::Gzip {
        fn header_value(&self) -> http_1_0::HeaderValue {
            http_1_0::HeaderValue::from_static("gzip")
        }
    }
}

impl From<&CompressionOptions> for Gzip {
    fn from(options: &CompressionOptions) -> Self {
        Gzip {
            compression: flate2::Compression::new(options.level),
        }
    }
}

impl From<CompressionOptions> for Gzip {
    fn from(options: CompressionOptions) -> Self {
        Gzip {
            compression: flate2::Compression::new(options.level),
        }
    }
}

// Windows line-endings will cause the compression test to fail.
#[cfg(all(test, not(windows)))]
mod tests {
    use super::Gzip;
    use crate::CompressionOptions;
    use flate2::read::GzDecoder;
    use pretty_assertions::assert_eq;
    use std::io::Read;

    fn gettysburg_address() -> &'static [u8] {
        include_bytes!("../test-data/gettysburg_address.txt")
    }

    fn gzip_compressed_gettysburg_address() -> &'static [u8] {
        // This file was compressed using Apple gzip with the following command:
        // `gzip -k gettysburg_address.txt -6`
        include_bytes!("../test-data/gettysburg_address.txt.gz")
    }

    #[test]
    fn test_gzip_compression() {
        let gzip = Gzip::from(&CompressionOptions::default());
        let mut compressed_output = Vec::new();
        gzip.compress_bytes(gettysburg_address(), &mut compressed_output)
            .expect("compression succeeds");

        let uncompressed_expected = {
            let mut s = String::new();
            GzDecoder::new(gzip_compressed_gettysburg_address())
                .read_to_string(&mut s)
                .unwrap();
            s
        };
        let uncompressed_actual = {
            let mut s = String::new();
            GzDecoder::new(&compressed_output[..])
                .read_to_string(&mut s)
                .unwrap();
            s
        };

        assert_eq!(uncompressed_expected, uncompressed_actual);
    }
}
