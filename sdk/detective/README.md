# aws-sdk-detective

**Please Note: The SDK is currently released as an alpha and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

Detective uses machine learning and purpose-built visualizations to help you analyze and investigate security issues across your Amazon Web Services (AWS) workloads. Detective automatically extracts time-based events such as login attempts, API calls, and network traffic from AWS CloudTrail and Amazon Virtual Private Cloud (Amazon VPC) flow logs. It also extracts findings detected by Amazon GuardDuty.

The Detective API primarily supports the creation and management of behavior graphs. A behavior graph contains the extracted data from a set of member accounts, and is created and managed by an administrator account.

Every behavior graph is specific to a Region. You can only use the API to manage graphs that belong to the Region that is associated with the currently selected endpoint.

A Detective administrator account can use the Detective API to do the following:
  - Enable and disable Detective. Enabling Detective creates a new behavior graph.
  - View the list of member accounts in a behavior graph.
  - Add member accounts to a behavior graph.
  - Remove member accounts from a behavior graph.

A member account can use the Detective API to do the following:
  - View the list of behavior graphs that they are invited to.
  - Accept an invitation to contribute to a behavior graph.
  - Decline an invitation to contribute to a behavior graph.
  - Remove their account from a behavior graph.

All API actions are logged as CloudTrail events. See [Logging Detective API Calls with CloudTrail](https://docs.aws.amazon.com/detective/latest/adminguide/logging-using-cloudtrail.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/sdk/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-detective` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.0.23-alpha"
aws-sdk-detective = "0.0.23-alpha"
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

