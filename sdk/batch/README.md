# aws-sdk-batch

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

Using Batch, you can run batch computing workloads on the Amazon Web Services Cloud. Batch computing is a common means for developers, scientists, and engineers to access large amounts of compute resources. Batch uses the advantages of this computing workload to remove the undifferentiated heavy lifting of configuring and managing required infrastructure. At the same time, it also adopts a familiar batch computing software approach. Given these advantages, Batch can help you to efficiently provision resources in response to jobs submitted, thus effectively helping you to eliminate capacity constraints, reduce compute costs, and deliver your results more quickly.

As a fully managed service, Batch can run batch computing workloads of any scale. Batch automatically provisions compute resources and optimizes workload distribution based on the quantity and scale of your specific workloads. With Batch, there's no need to install or manage batch computing software. This means that you can focus your time and energy on analyzing results and solving your specific problems.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-batch` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.14.0"
aws-sdk-batch = "0.14.0"
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

