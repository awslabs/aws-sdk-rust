# aws-sdk-connect

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

Amazon Connect is a cloud-based contact center solution that you use to set up and manage a customer contact center and provide reliable customer engagement at any scale.

Amazon Connect provides metrics and real-time reporting that enable you to optimize contact routing. You can also resolve customer issues more efficiently by getting customers in touch with the appropriate agents.

There are limits to the number of Amazon Connect resources that you can create. There are also limits to the number of requests that you can make per second. For more information, see [Amazon Connect Service Quotas](https://docs.aws.amazon.com/connect/latest/adminguide/amazon-connect-service-limits.html) in the _Amazon Connect Administrator Guide_.

You can connect programmatically to an Amazon Web Services service by using an endpoint. For a list of Amazon Connect endpoints, see [Amazon Connect Endpoints](https://docs.aws.amazon.com/general/latest/gr/connect_region.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-connect` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.14.0"
aws-sdk-connect = "0.14.0"
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

