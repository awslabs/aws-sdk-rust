# aws-sdk-eventbridgev2

Amazon EventBridge event bus API. An event bus receives events published by your applications and AWS services, stores them for a configurable retention period, and delivers them to subscribers. A subscriber filters events, optionally transforms them, and invokes a target such as Lambda, SQS, SNS, Kinesis, Step Functions, or an HTTP endpoint. The API manages event buses, subscribers, event sources, resource policies, and tags, and publishes events through PutEvents and PutRawEvents.

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-eventbridgev2` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-eventbridgev2 = "1.0.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_eventbridgev2 as eventbridgev2;

#[::tokio::main]
async fn main() -> Result<(), eventbridgev2::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_eventbridgev2::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-eventbridgev2/latest/aws_sdk_eventbridgev2/client/struct.Client.html)
for information on what calls can be made, and the inputs and outputs for each of those calls.

## Using the SDK

Until the SDK is released, we will be adding information about using the SDK to the
[Developer Guide](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/welcome.html). Feel free to suggest
additional sections for the guide by opening an issue and describing what you are trying to do.

## Getting Help

* [GitHub discussions](https://github.com/awslabs/aws-sdk-rust/discussions) - For ideas, RFCs & general questions
* [GitHub issues](https://github.com/awslabs/aws-sdk-rust/issues/new/choose) - For bug reports & feature requests
* [Generated Docs (latest version)](https://awslabs.github.io/aws-sdk-rust/)
* [Usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1)

## License

This project is licensed under the Apache-2.0 License.

