# aws-sdk-connect

  - [Connect Customer Customer actions](https://docs.aws.amazon.com/connect/latest/APIReference/API_Operations_Amazon_Connect_Service.html)
  - [Connect Customer Customer data types](https://docs.aws.amazon.com/connect/latest/APIReference/API_Types_Amazon_Connect_Service.html)

Connect Customer Customer engages customers at every touchpoint and creates deeper relationships with AI powered capabilities.

Build and manage customer communication experiences. Connect customers to agents, enable intelligent routing, and track performance in real-time.

There are limits to the number of Connect Customer resources that you can create. There are also limits to the number of requests that you can make per second. For more information, see [Connect Customer Service Quotas](https://docs.aws.amazon.com/connect/latest/adminguide/amazon-connect-service-limits.html) in the _Connect Customer Administrator Guide_.

You can use an endpoint to connect programmatically to an Amazon Web Services service. For a list of Connect Customer endpoints, see [Connect Customer Endpoints](https://docs.aws.amazon.com/general/latest/gr/connect_region.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-connect` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-connect = "1.182.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_connect as connect;

#[::tokio::main]
async fn main() -> Result<(), connect::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_connect::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-connect/latest/aws_sdk_connect/client/struct.Client.html)
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

