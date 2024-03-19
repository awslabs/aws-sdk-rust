# aws-sdk-repostspace

AWS re:Post Private is a private version of AWS re:Post for enterprises with Enterprise Support or Enterprise On-Ramp Support plans. It provides access to knowledge and experts to accelerate cloud adoption and increase developer productivity. With your organization-specific private re:Post, you can build an organization-specific developer community that drives efficiencies at scale and provides access to valuable knowledge resources. Additionally, re:Post Private centralizes trusted AWS technical content and offers private discussion forums to improve how your teams collaborate internally and with AWS to remove technical obstacles, accelerate innovation, and scale more efficiently in the cloud.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-repostspace` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-repostspace = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_repostspace as repostspace;

#[::tokio::main]
async fn main() -> Result<(), repostspace::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_repostspace::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-repostspace/latest/aws_sdk_repostspace/client/struct.Client.html)
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

