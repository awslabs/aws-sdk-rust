# aws-sdk-freetier

You can use the Amazon Web Services Free Tier API to query programmatically your Free Tier usage data.

Free Tier tracks your monthly usage data for all free tier offers that are associated with your Amazon Web Services account. You can use the Free Tier API to filter and show only the data that you want.

Service endpoint

The Free Tier API provides the following endpoint:
  - https://freetier.us-east-1.api.aws

For more information, see [Using the Amazon Web Services Free Tier](https://docs.aws.amazon.com/awsaccountbilling/latest/aboutv2/billing-free-tier.html) in the _Billing User Guide_.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-freetier` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-freetier = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_freetier as freetier;

#[::tokio::main]
async fn main() -> Result<(), freetier::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_freetier::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-freetier/latest/aws_sdk_freetier/client/struct.Client.html)
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

