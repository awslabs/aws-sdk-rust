# aws-sdk-synthetics

You can use Amazon CloudWatch Synthetics to continually monitor your services. You can create and manage _canaries_, which are modular, lightweight scripts that monitor your endpoints and APIs from the outside-in. You can set up your canaries to run 24 hours a day, once per minute. The canaries help you check the availability and latency of your web services and troubleshoot anomalies by investigating load time data, screenshots of the UI, logs, and metrics. The canaries seamlessly integrate with CloudWatch ServiceLens to help you trace the causes of impacted nodes in your applications. For more information, see [Using ServiceLens to Monitor the Health of Your Applications](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/ServiceLens.html) in the _Amazon CloudWatch User Guide_.

Before you create and manage canaries, be aware of the security considerations. For more information, see [Security Considerations for Synthetics Canaries](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/servicelens_canaries_security.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-synthetics` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-synthetics = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_synthetics as synthetics;

#[::tokio::main]
async fn main() -> Result<(), synthetics::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_synthetics::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-synthetics/latest/aws_sdk_synthetics/client/struct.Client.html)
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

