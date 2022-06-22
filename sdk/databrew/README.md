# aws-sdk-databrew

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

Glue DataBrew is a visual, cloud-scale data-preparation service. DataBrew simplifies data preparation tasks, targeting data issues that are hard to spot and time-consuming to fix. DataBrew empowers users of all technical levels to visualize the data and perform one-click data transformations, with no coding required.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-databrew` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.14.0"
aws-sdk-databrew = "0.14.0"
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

