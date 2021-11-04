# aws-sdk-lambda

**Please Note: The SDK is currently released as an alpha and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

__Overview__

This is the _Lambda API Reference_. The Lambda Developer Guide provides additional information. For the service overview, see [What is Lambda](https://docs.aws.amazon.com/lambda/latest/dg/welcome.html), and for information about how the service works, see [Lambda: How it Works](https://docs.aws.amazon.com/lambda/latest/dg/lambda-introduction.html) in the __Lambda Developer Guide__.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/sdk/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-lambda` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.0.23-alpha"
aws-sdk-lambda = "0.0.23-alpha"
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

