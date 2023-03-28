# aws-sdk-mediastore

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

An AWS Elemental MediaStore container is a namespace that holds folders and objects. You use a container endpoint to create, read, and delete objects.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-mediastore` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.54.1"
aws-sdk-mediastore = "0.25.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust
use aws_sdk_mediastore as mediastore;

#[tokio::main]
async fn main() -> Result<(), mediastore::Error> {
    let config = aws_config::load_from_env().await;
    let client = mediastore::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-mediastore/latest/aws_sdk_mediastore/client/struct.Client.html)
for information on what calls can be made, and the inputs and outputs for each of those calls.

## Using the SDK

Until the SDK is released, we will be adding information about using the SDK to the
[Developer Guide](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/welcome.html). Feel free to suggest
additional sections for the guide by opening an issue and describing what you are trying to do.

## Getting Help

* [GitHub discussions](https://github.com/awslabs/aws-sdk-rust/discussions) - For ideas, RFCs & general questions
* [GitHub issues](https://github.com/awslabs/aws-sdk-rust/issues/new/choose) – For bug reports & feature requests
* [Generated Docs (latest version)](https://awslabs.github.io/aws-sdk-rust/)
* [Usage examples](https://github.com/awslabs/aws-sdk-rust/tree/main/examples)

## License

This project is licensed under the Apache-2.0 License.

