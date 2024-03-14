# aws-smithy-experimental

Staging ground for experimental new features in the smithy-rs ecosystem.

### Hyper 1.0 Support
This crate adds support for Hyper 1.0 (see [examples](./examples)). There a few blockers before stablization:
1. Moving to `rustls` 0.23 to take advantage of FIPS support. This is blocked on `hyper-rustls` being upgraded.
2. Expose an API for providing a custom connector. Currently that API is not exposed because a shim layer is needed to avoid taking a hard dependency on hyper-util.
3. Add support for poisoning connections in the connection pool. This API needs to be either backported into hyper-util or we need to establish our own client.


<!-- anchor_start:footer -->
This crate is part of the [AWS SDK for Rust](https://awslabs.github.io/aws-sdk-rust/) and the [smithy-rs](https://github.com/smithy-lang/smithy-rs) code generator.
<!-- anchor_end:footer -->
