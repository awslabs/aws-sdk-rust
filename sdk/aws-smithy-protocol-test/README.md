# Smithy Protocol Tests

This library implements utilities for validating serializers & deserializers
against [Smithy protocol tests](https://awslabs.github.io/smithy/1.0/spec/http-protocol-compliance-tests.html). Specifically, this crate includes support for:

* MediaType-aware comparison for XML, JSON and AWS Query.
* NaN/Infinty supporting floating point comparisons.
* HTTP header & query string validators.

<!-- anchor_start:footer -->
This crate is part of the [AWS SDK for Rust](https://awslabs.github.io/aws-sdk-rust/) and the [smithy-rs](https://github.com/smithy-lang/smithy-rs) code generator. In most cases, it should not be used directly.
<!-- anchor_end:footer -->
