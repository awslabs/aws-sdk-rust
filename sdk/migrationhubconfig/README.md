# aws-sdk-migrationhubconfig

The AWS Migration Hub home region APIs are available specifically for working with your Migration Hub home region. You can use these APIs to determine a home region, as well as to create and work with controls that describe the home region.
  - You must make API calls for write actions (create, notify, associate, disassociate, import, or put) while in your home region, or a HomeRegionNotSetException error is returned.
  - API calls for read actions (list, describe, stop, and delete) are permitted outside of your home region.
  - If you call a write API outside the home region, an InvalidInputException is returned.
  - You can call GetHomeRegion action to obtain the account's Migration Hub home region.

For specific API usage, see the sections that follow in this AWS Migration Hub Home Region API reference.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-migrationhubconfig` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-migrationhubconfig = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_migrationhubconfig as migrationhubconfig;

#[::tokio::main]
async fn main() -> Result<(), migrationhubconfig::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_migrationhubconfig::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-migrationhubconfig/latest/aws_sdk_migrationhubconfig/client/struct.Client.html)
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

