# aws-sdk-ssm

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

Amazon Web Services Systems Manager is a collection of capabilities that helps you automate management tasks such as collecting system inventory, applying operating system (OS) patches, automating the creation of Amazon Machine Images (AMIs), and configuring operating systems (OSs) and applications at scale. Systems Manager lets you remotely and securely manage the configuration of your managed nodes. A _managed node_ is any Amazon Elastic Compute Cloud (Amazon EC2) instance, edge device, or on-premises server or virtual machine (VM) that has been configured for Systems Manager.

This reference is intended to be used with the [Amazon Web Services Systems Manager User Guide](https://docs.aws.amazon.com/systems-manager/latest/userguide/).

To get started, verify prerequisites and configure managed nodes. For more information, see [Setting up Amazon Web Services Systems Manager](https://docs.aws.amazon.com/systems-manager/latest/userguide/systems-manager-setting-up.html) in the _Amazon Web Services Systems Manager User Guide_.

__Related resources__
  - For information about how to use a Query API, see [Making API requests](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/making-api-requests.html).
  - For information about other API operations you can perform on EC2 instances, see the [Amazon EC2 API Reference](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/).
  - For information about AppConfig, a capability of Systems Manager, see the [AppConfig User Guide](https://docs.aws.amazon.com/appconfig/latest/userguide/) and the [AppConfig API Reference](https://docs.aws.amazon.com/appconfig/2019-10-09/APIReference/).
  - For information about Incident Manager, a capability of Systems Manager, see the [Incident Manager User Guide](https://docs.aws.amazon.com/incident-manager/latest/userguide/) and the [Incident Manager API Reference](https://docs.aws.amazon.com/incident-manager/latest/APIReference/).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-ssm` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.14.0"
aws-sdk-ssm = "0.14.0"
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

