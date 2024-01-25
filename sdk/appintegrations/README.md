# aws-sdk-appintegrations

The Amazon AppIntegrations service enables you to configure and reuse connections to external applications.

For information about how you can use external applications with Amazon Connect, see [Set up pre-built integrations](https://docs.aws.amazon.com/connect/latest/adminguide/crm.html) and [Deliver information to agents using Amazon Connect Wisdom](https://docs.aws.amazon.com/connect/latest/adminguide/amazon-connect-wisdom.html) in the _Amazon Connect Administrator Guide_.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-appintegrations` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.4", features = ["behavior-version-latest"] }
aws-sdk-appintegrations = "1.12.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_appintegrations as appintegrations;

#[::tokio::main]
async fn main() -> Result<(), appintegrations::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_appintegrations::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-appintegrations/latest/aws_sdk_appintegrations/client/struct.Client.html)
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

