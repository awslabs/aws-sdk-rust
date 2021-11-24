# aws-sdk-budgets

**Please Note: The SDK is currently released as an alpha and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

The AWS Budgets API enables you to use AWS Budgets to plan your service usage, service costs, and instance reservations. The API reference provides descriptions, syntax, and usage examples for each of the actions and data types for AWS Budgets.

Budgets provide you with a way to see the following information:
  - How close your plan is to your budgeted amount or to the free tier limits
  - Your usage-to-date, including how much you've used of your Reserved Instances (RIs)
  - Your current estimated charges from AWS, and how much your predicted usage will accrue in charges by the end of the month
  - How much of your budget has been used

AWS updates your budget status several times a day. Budgets track your unblended costs, subscriptions, refunds, and RIs. You can create the following types of budgets:
  - __Cost budgets__ - Plan how much you want to spend on a service.
  - __Usage budgets__ - Plan how much you want to use one or more services.
  - __RI utilization budgets__ - Define a utilization threshold, and receive alerts when your RI usage falls below that threshold. This lets you see if your RIs are unused or under-utilized.
  - __RI coverage budgets__ - Define a coverage threshold, and receive alerts when the number of your instance hours that are covered by RIs fall below that threshold. This lets you see how much of your instance usage is covered by a reservation.

Service Endpoint

The AWS Budgets API provides the following endpoint:
  - https://budgets.amazonaws.com

For information about costs that are associated with the AWS Budgets API, see [AWS Cost Management Pricing](https://aws.amazon.com/aws-cost-management/pricing/).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-budgets` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.0.26-alpha"
aws-sdk-budgets = "0.0.26-alpha"
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
* [Usage examples](https://github.com/awslabs/aws-sdk-rust/tree/main/examples)

## License

This project is licensed under the Apache-2.0 License.

