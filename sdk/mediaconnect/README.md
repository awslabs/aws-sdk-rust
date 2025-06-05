# aws-sdk-mediaconnect

Welcome to the Elemental MediaConnect API reference.

MediaConnect is a service that lets you ingest live video content into the cloud and distribute it to destinations all over the world, both inside and outside the Amazon Web Services cloud. This API reference provides descriptions, syntax, and usage examples for each of the actions and data types that are supported by MediaConnect.

Use the following links to get started with the MediaConnect API:
  - [Actions](https://docs.aws.amazon.com/mediaconnect/latest/api/API_Operations.html): An alphabetical list of all MediaConnect API operations.
  - [Data types](https://docs.aws.amazon.com/mediaconnect/latest/api/API_Types.html): An alphabetical list of all MediaConnect data types.
  - [Common parameters](https://docs.aws.amazon.com/mediaconnect/latest/api/CommonParameters.html): Parameters that all operations can use.
  - [Common errors](https://docs.aws.amazon.com/mediaconnect/latest/api/CommonErrors.html): Client and server errors that all operations can return.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-mediaconnect` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-mediaconnect = "1.77.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_mediaconnect as mediaconnect;

#[::tokio::main]
async fn main() -> Result<(), mediaconnect::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_mediaconnect::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-mediaconnect/latest/aws_sdk_mediaconnect/client/struct.Client.html)
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

