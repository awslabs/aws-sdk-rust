# aws-sdk-ram

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

This is the _Resource Access Manager API Reference_. This documentation provides descriptions and syntax for each of the actions and data types in RAM. RAM is a service that helps you securely share your Amazon Web Services resources across Amazon Web Services accounts. If you have multiple Amazon Web Services accounts, you can use RAM to share those resources with other accounts. If you use Organizations to manage your accounts, then you share your resources with your organization or organizational units (OUs). For supported resource types, you can also share resources with individual Identity and Access Management (IAM) roles an users.

To learn more about RAM, see the following resources:
  - [Resource Access Manager product page](http://aws.amazon.com/ram)
  - [Resource Access Manager User Guide](https://docs.aws.amazon.com/ram/latest/userguide/)

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-ram` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.49.0"
aws-sdk-ram = "0.19.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust
use aws_sdk_ram as ram;

#[tokio::main]
async fn main() -> Result<(), ram::Error> {
    let config = aws_config::load_from_env().await;
    let client = ram::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-ram/latest/aws_sdk_ram/client/struct.Client.html)
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

