# aws-smithy-types-convert

This crate provides utilities for converting between the types defined
in [aws-smithy-types](https://docs.rs/aws-smithy-types) and types commonly used in other libraries.

## Crate Features

By default, no features are enabled. Using the conversions requires enabling one or more features:

```toml
[dependencies]
aws-smithy-types-convert = { version = "VERSION", features = ["convert-chrono"] }
```

Currently, the following conversions are supported:

* `convert-chrono`: Conversions between `DateTime` and [chrono](https://docs.rs/chrono/latest/chrono/).
* `convert-time`: Conversions between `DateTime` and [time](https://docs.rs/time/latest/time/).

_Note:_ Conversions to and from [`SystemTime`](https://doc.rust-lang.org/std/time/struct.SystemTime.html) are built
into [`aws-smithy-types`](https://docs.rs/aws-smithy-types/0.30.0-alpha/aws_smithy_types/date_time/struct.DateTime.html#impl-From%3CSystemTime%3E).

<!-- anchor_start:footer -->
This crate is part of the [AWS SDK for Rust](https://awslabs.github.io/aws-sdk-rust/) and the [smithy-rs](https://github.com/smithy-lang/smithy-rs) code generator. In most cases, it should not be used directly.
<!-- anchor_end:footer -->
