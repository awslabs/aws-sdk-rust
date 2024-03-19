# aws-sdk-migrationhubrefactorspaces

This API reference provides descriptions, syntax, and other details about each of the actions and data types for Amazon Web Services Migration Hub Refactor Spaces (Refactor Spaces). The topic for each action shows the API request parameters and the response. Alternatively, you can use one of the Amazon Web Services SDKs to access an API that is tailored to the programming language or platform that you're using. For more information, see [Amazon Web Services SDKs](https://aws.amazon.com/tools/#SDKs).

To share Refactor Spaces environments with other Amazon Web Services accounts or with Organizations and their OUs, use Resource Access Manager's CreateResourceShare API. See [CreateResourceShare](https://docs.aws.amazon.com/ram/latest/APIReference/API_CreateResourceShare.html) in the _Amazon Web Services RAM API Reference_.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-migrationhubrefactorspaces` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-migrationhubrefactorspaces = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_migrationhubrefactorspaces as migrationhubrefactorspaces;

#[::tokio::main]
async fn main() -> Result<(), migrationhubrefactorspaces::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_migrationhubrefactorspaces::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-migrationhubrefactorspaces/latest/aws_sdk_migrationhubrefactorspaces/client/struct.Client.html)
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

