# aws-sdk-health

**Please Note: The SDK is currently released as an alpha and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

The AWS Health API provides programmatic access to the AWS Health information that appears in the [AWS Personal Health Dashboard](https://phd.aws.amazon.com/phd/home#/). You can use the API operations to get information about AWS Health events that affect your AWS services and resources.

For authentication of requests, AWS Health uses the [Signature Version 4 Signing Process](https://docs.aws.amazon.com/general/latest/gr/signature-version-4.html).

If your AWS account is part of AWS Organizations, you can use the AWS Health organizational view feature. This feature provides a centralized view of AWS Health events across all accounts in your organization. You can aggregate AWS Health events in real time to identify accounts in your organization that are affected by an operational event or get notified of security vulnerabilities. Use the organizational view API operations to enable this feature and return event information. For more information, see [Aggregating AWS Health events](https://docs.aws.amazon.com/health/latest/ug/aggregate-events.html) in the _AWS Health User Guide_.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/sdk/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-health` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.0.23-alpha"
aws-sdk-health = "0.0.23-alpha"
tokio = { version = "1", features = ["full"] }
```

## Using the SDK

Until the SDK is released, we will be adding information about using the SDK to the
[Guide](https://github.com/awslabs/aws-sdk-rust/blob/main/Guide.md). Feel free to suggest
additional sections for the guide by opening an issue and describing what you are trying to do.

## Getting Help

* [GitHub discussions](https://github.com/awslabs/aws-sdk-rust/discussions) - For ideas, RFCs & general questions
* [GitHub issues](https://github.com/awslabs/aws-sdk-rust/issues/new/choose) – For bug reports & feature requests
* [Generated Docs (latest version)](https://awslabs.github.io/aws-sdk-rust/)
* [Usage examples](https://github.com/awslabs/aws-sdk-rust/tree/main/sdk/examples)

## License

This project is licensed under the Apache-2.0 License.

