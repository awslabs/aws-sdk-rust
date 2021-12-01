# AWS Default Middleware

This crate defines the default middleware stack used by AWS services. It also provides a re-export
of `aws_smithy_client::Client` with the middleware type preset.

_Note:_ this crate will be removed in the future in favor of defining the middlewares directly in the service clients.

<!-- anchor_start:footer -->
This crate is part of the [AWS SDK for Rust](https://awslabs.github.io/aws-sdk-rust/) and the [smithy-rs](https://github.com/awslabs/smithy-rs) code generator. In most cases, it should not be used directly.
<!-- anchor_end:footer -->
