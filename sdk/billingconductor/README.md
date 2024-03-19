# aws-sdk-billingconductor

Amazon Web Services Billing Conductor is a fully managed service that you can use to customize a [proforma](https://docs.aws.amazon.com/billingconductor/latest/userguide/understanding-eb.html#eb-other-definitions) version of your billing data each month, to accurately show or chargeback your end customers. Amazon Web Services Billing Conductor doesn't change the way you're billed by Amazon Web Services each month by design. Instead, it provides you with a mechanism to configure, generate, and display rates to certain customers over a given billing period. You can also analyze the difference between the rates you apply to your accounting groupings relative to your actual rates from Amazon Web Services. As a result of your Amazon Web Services Billing Conductor configuration, the payer account can also see the custom rate applied on the billing details page of the [Amazon Web Services Billing console](https://console.aws.amazon.com/billing), or configure a cost and usage report per billing group.

This documentation shows how you can configure Amazon Web Services Billing Conductor using its API. For more information about using the [Amazon Web Services Billing Conductor](https://console.aws.amazon.com/billingconductor/) user interface, see the [Amazon Web Services Billing Conductor User Guide](https://docs.aws.amazon.com/billingconductor/latest/userguide/what-is-billingconductor.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-billingconductor` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-billingconductor = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_billingconductor as billingconductor;

#[::tokio::main]
async fn main() -> Result<(), billingconductor::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_billingconductor::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-billingconductor/latest/aws_sdk_billingconductor/client/struct.Client.html)
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

