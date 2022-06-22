# aws-sdk-sms

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

__Product update__

We recommend [Amazon Web Services Application Migration Service](http://aws.amazon.com/application-migration-service) (Amazon Web Services MGN) as the primary migration service for lift-and-shift migrations. If Amazon Web Services MGN is unavailable in a specific Amazon Web Services Region, you can use the Server Migration Service APIs through March 2023.

Server Migration Service (Server Migration Service) makes it easier and faster for you to migrate your on-premises workloads to Amazon Web Services. To learn more about Server Migration Service, see the following resources:
  - [Server Migration Service product page](http://aws.amazon.com/server-migration-service/)
  - [Server Migration Service User Guide](https://docs.aws.amazon.com/server-migration-service/latest/userguide/)

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-sms` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.14.0"
aws-sdk-sms = "0.14.0"
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

