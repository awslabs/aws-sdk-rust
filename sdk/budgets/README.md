# aws-sdk-budgets

Use the Amazon Web Services Budgets API to plan your service usage, service costs, and instance reservations. This API reference provides descriptions, syntax, and usage examples for each of the actions and data types for the Amazon Web Services Budgets feature.

Budgets provide you with a way to see the following information:
  - How close your plan is to your budgeted amount or to the free tier limits
  - Your usage-to-date, including how much you've used of your Reserved Instances (RIs)
  - Your current estimated charges from Amazon Web Services, and how much your predicted usage will accrue in charges by the end of the month
  - How much of your budget has been used

Amazon Web Services updates your budget status several times a day. Budgets track your unblended costs, subscriptions, refunds, and RIs. You can create the following types of budgets:
  - __Cost budgets__ - Plan how much you want to spend on a service.
  - __Usage budgets__ - Plan how much you want to use one or more services.
  - __RI utilization budgets__ - Define a utilization threshold, and receive alerts when your RI usage falls below that threshold. This lets you see if your RIs are unused or under-utilized.
  - __RI coverage budgets__ - Define a coverage threshold, and receive alerts when the number of your instance hours that are covered by RIs fall below that threshold. This lets you see how much of your instance usage is covered by a reservation.

Service Endpoint

The Amazon Web Services Budgets API provides the following endpoint:
  - https://budgets.amazonaws.com

For information about costs that are associated with the Amazon Web Services Budgets API, see [Amazon Web Services Cost Management Pricing](https://aws.amazon.com/aws-cost-management/pricing/).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-budgets` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-budgets = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_budgets as budgets;

#[::tokio::main]
async fn main() -> Result<(), budgets::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_budgets::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-budgets/latest/aws_sdk_budgets/client/struct.Client.html)
for information on what calls can be made, and the inputs and outputs for each of those calls.

## Using the SDK

Until the SDK is released, we will be adding information about using the SDK to the
[Developer Guide](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/welcome.html). Feel free to suggest
additional sections for the guide by opening an issue and describing what you are trying to do.

## Getting Help

* [GitHub discussions](https://github.com/awslabs/aws-sdk-rust/discussions) - For ideas, RFCs & general questions
* [GitHub issues](https://github.com/awslabs/aws-sdk-rust/issues/new/choose) - For bug reports & feature requests
* [Generated Docs (latest version)](https://awslabs.github.io/aws-sdk-rust/)
* [Usage examples](https://github.com/awslabs/aws-sdk-rust/tree/main/examples)

## License

This project is licensed under the Apache-2.0 License.

