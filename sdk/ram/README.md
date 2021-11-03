# aws-sdk-ram

**Please Note: The SDK is currently released as an alpha and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

This is the _Resource Access Manager API Reference_. This documentation provides descriptions and syntax for each of the actions and data types in RAM. RAM is a service that helps you securely share your Amazon Web Services resources across Amazon Web Services accounts and within your organization or organizational units (OUs) in Organizations. For supported resource types, you can also share resources with IAM roles and IAM users. If you have multiple Amazon Web Services accounts, you can use RAM to share those resources with other accounts.

To learn more about RAM, see the following resources:
  - [Resource Access Manager product page](http://aws.amazon.com/ram)
  - [Resource Access Manager User Guide](https://docs.aws.amazon.com/ram/latest/userguide/)

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/sdk/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-ram` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.0.23-alpha"
aws-sdk-ram = "0.0.23-alpha"
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

