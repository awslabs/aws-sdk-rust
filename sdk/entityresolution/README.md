# aws-sdk-entityresolution

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

Welcome to the _AWS Entity Resolution API Reference_.

AWS Entity Resolution is an AWS service that provides pre-configured entity resolution capabilities that enable developers and analysts at advertising and marketing companies to build an accurate and complete view of their consumers.

With AWS Entity Resolution, you have the ability to match source records containing consumer identifiers, such as name, email address, and phone number. This holds true even when these records have incomplete or conflicting identifiers. For example, AWS Entity Resolution can effectively match a source record from a customer relationship management (CRM) system, which includes account information like first name, last name, postal address, phone number, and email address, with a source record from a marketing system containing campaign information, such as username and email address.

To learn more about AWS Entity Resolution concepts, procedures, and best practices, see the [AWS Entity Resolution User Guide](https://docs.aws.amazon.com/entityresolution/latest/userguide/what-is-service.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-entityresolution` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.56.0"
aws-sdk-entityresolution = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_entityresolution as entityresolution;

#[::tokio::main]
async fn main() -> Result<(), entityresolution::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_entityresolution::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-entityresolution/latest/aws_sdk_entityresolution/client/struct.Client.html)
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

