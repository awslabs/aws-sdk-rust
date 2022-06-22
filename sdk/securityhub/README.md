# aws-sdk-securityhub

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

Security Hub provides you with a comprehensive view of the security state of your Amazon Web Services environment and resources. It also provides you with the readiness status of your environment based on controls from supported security standards. Security Hub collects security data from Amazon Web Services accounts, services, and integrated third-party products and helps you analyze security trends in your environment to identify the highest priority security issues. For more information about Security Hub, see the [_Security HubUser Guide_](https://docs.aws.amazon.com/securityhub/latest/userguide/what-is-securityhub.html).

When you use operations in the Security Hub API, the requests are executed only in the Amazon Web Services Region that is currently active or in the specific Amazon Web Services Region that you specify in your request. Any configuration or settings change that results from the operation is applied only to that Region. To make the same change in other Regions, execute the same command for each Region to apply the change to.

For example, if your Region is set to us-west-2, when you use CreateMembers to add a member account to Security Hub, the association of the member account with the administrator account is created only in the us-west-2 Region. Security Hub must be enabled for the member account in the same Region that the invitation was sent from.

The following throttling limits apply to using Security Hub API operations.
  - BatchEnableStandards - RateLimit of 1 request per second, BurstLimit of 1 request per second.
  - GetFindings - RateLimit of 3 requests per second. BurstLimit of 6 requests per second.
  - BatchImportFindings - RateLimit of 10 requests per second. BurstLimit of 30 requests per second.
  - BatchUpdateFindings - RateLimit of 10 requests per second. BurstLimit of 30 requests per second.
  - UpdateStandardsControl - RateLimit of 1 request per second, BurstLimit of 5 requests per second.
  - All other operations - RateLimit of 10 requests per second. BurstLimit of 30 requests per second.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-securityhub` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.14.0"
aws-sdk-securityhub = "0.14.0"
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

