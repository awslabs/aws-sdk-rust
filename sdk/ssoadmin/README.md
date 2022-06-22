# aws-sdk-ssoadmin

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

Amazon Web Services Single Sign On (SSO) is a cloud SSO service that makes it easy to centrally manage SSO access to multiple Amazon Web Services accounts and business applications. This guide provides information on SSO operations which could be used for access management of Amazon Web Services accounts. For information about Amazon Web Services SSO features, see the [Amazon Web Services Single Sign-On User Guide](https://docs.aws.amazon.com/singlesignon/latest/userguide/what-is.html).

Many operations in the SSO APIs rely on identifiers for users and groups, known as principals. For more information about how to work with principals and principal IDs in Amazon Web Services SSO, see the [Amazon Web Services SSO Identity Store API Reference](https://docs.aws.amazon.com/singlesignon/latest/IdentityStoreAPIReference/welcome.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-ssoadmin` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.14.0"
aws-sdk-ssoadmin = "0.14.0"
tokio = { version = "1", features = ["full"] }
```

## Using the SDK

Until the SDK is released, we will be adding information about using the SDK to the
[Developer Guide](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/welcome.html). Feel free to suggest
additional sections for the guide by opening an issue and describing what you are trying to do.

## Getting Help

* [GitHub discussions](https://github.com/awslabs/aws-sdk-rust/discussions) - For ideas, RFCs & general questions
* [GitHub issues](https://github.com/awslabs/aws-sdk-rust/issues/new/choose) – For bug reports & feature requests
* [Generated Docs (latest version)](https://awslabs.github.io/aws-sdk-rust/)
* [Usage examples](https://github.com/awslabs/aws-sdk-rust/tree/main/examples)

## License

This project is licensed under the Apache-2.0 License.

