# aws-sdk-devicefarm

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

Welcome to the AWS Device Farm API documentation, which contains APIs for:
  - Testing on desktop browsers Device Farm makes it possible for you to test your web applications on desktop browsers using Selenium. The APIs for desktop browser testing contain TestGrid in their names. For more information, see [Testing Web Applications on Selenium with Device Farm](https://docs.aws.amazon.com/devicefarm/latest/testgrid/).
  - Testing on real mobile devices Device Farm makes it possible for you to test apps on physical phones, tablets, and other devices in the cloud. For more information, see the [Device Farm Developer Guide](https://docs.aws.amazon.com/devicefarm/latest/developerguide/).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-devicefarm` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.14.0"
aws-sdk-devicefarm = "0.14.0"
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

