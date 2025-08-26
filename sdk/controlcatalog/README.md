# aws-sdk-controlcatalog

Welcome to the Control Catalog API reference. This guide is for developers who need detailed information about how to programmatically identify and filter the common controls and related metadata that are available to Amazon Web Services customers. This API reference provides descriptions, syntax, and usage examples for each of the actions and data types that are supported by Control Catalog.

Use the following links to get started with the Control Catalog API:
  - [Actions](https://docs.aws.amazon.com/controlcatalog/latest/APIReference/API_Operations.html): An alphabetical list of all Control Catalog API operations.
  - [Data types](https://docs.aws.amazon.com/controlcatalog/latest/APIReference/API_Types.html): An alphabetical list of all Control Catalog data types.
  - [Common parameters](https://docs.aws.amazon.com/controlcatalog/latest/APIReference/CommonParameters.html): Parameters that all operations can use.
  - [Common errors](https://docs.aws.amazon.com/controlcatalog/latest/APIReference/CommonErrors.html): Client and server errors that all operations can return.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-controlcatalog` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-controlcatalog = "1.67.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_controlcatalog as controlcatalog;

#[::tokio::main]
async fn main() -> Result<(), controlcatalog::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_controlcatalog::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-controlcatalog/latest/aws_sdk_controlcatalog/client/struct.Client.html)
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

