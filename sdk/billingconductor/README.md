# aws-sdk-billingconductor

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

Amazon Web Services Billing Conductor is a fully managed service that you can use to customize a [pro forma](https://docs.aws.amazon.com/enterprisebilling/6b7c01c5-b592-467e-9769-90052eaf359c/userguide/understanding-eb.html#eb-other-definitions) version of your billing data each month, to accurately show or chargeback your end customers. Amazon Web Services Billing Conductor doesn't change the way you're billed by Amazon Web Services each month by design. Instead, it provides you with a mechanism to configure, generate, and display rates to certain customers over a given billing period. You can also analyze the difference between the rates you apply to your accounting groupings relative to your actual rates from Amazon Web Services. As a result of your Amazon Web Services Billing Conductor configuration, the payer account can also see the custom rate applied on the billing details page of the [Amazon Web Services Billing console](https://console.aws.amazon.com/billing), or configure a cost and usage report per billing group.

This documentation shows how you can configure Amazon Web Services Billing Conductor using its API. For more information about using the [Amazon Web Services Billing Conductor](https://console.aws.amazon.com/enterprisebilling/) user interface, see the [Amazon Web Services Enterprise Billing Console User Guide](https://docs.aws.amazon.com/enterprisebilling/6b7c01c5-b592-467e-9769-90052eaf359c/userguide/what-is-enterprisebilling.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-billingconductor` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.14.0"
aws-sdk-billingconductor = "0.14.0"
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

