# aws-sdk-oam

Use Amazon CloudWatch Observability Access Manager to create and manage links between source accounts and monitoring accounts by using _CloudWatch cross-account observability_. With CloudWatch cross-account observability, you can monitor and troubleshoot applications that span multiple accounts within a Region. Seamlessly search, visualize, and analyze your metrics, logs, traces, and Application Insights applications in any of the linked accounts without account boundaries.

Set up one or more Amazon Web Services accounts as _monitoring accounts_ and link them with multiple _source accounts_. A monitoring account is a central Amazon Web Services account that can view and interact with observability data generated from source accounts. A source account is an individual Amazon Web Services account that generates observability data for the resources that reside in it. Source accounts share their observability data with the monitoring account. The shared observability data can include metrics in Amazon CloudWatch, logs in Amazon CloudWatch Logs, traces in X-Ray, and applications in Amazon CloudWatch Application Insights.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-oam` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.4", features = ["behavior-version-latest"] }
aws-sdk-oam = "1.12.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_oam as oam;

#[::tokio::main]
async fn main() -> Result<(), oam::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_oam::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-oam/latest/aws_sdk_oam/client/struct.Client.html)
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

