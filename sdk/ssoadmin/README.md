# aws-sdk-ssoadmin

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

Amazon Web Services Single Sign On helps you securely create, or connect, your workforce identities and manage their access centrally across Amazon Web Services accounts and applications. Amazon Web Services SSO is the recommended approach for workforce authentication and authorization in Amazon Web Services, for organizations of any size and type.

This reference guide provides information on single sign-on operations which could be used for access management of Amazon Web Services accounts. For information about Amazon Web Services SSO features, see the [Amazon Web Services SSO User Guide](https://docs.aws.amazon.com/singlesignon/latest/userguide/what-is.html).

Many operations in the Amazon Web Services SSO APIs rely on identifiers for users and groups, known as principals. For more information about how to work with principals and principal IDs in Amazon Web Services SSO, see the [Identity Store API Reference](https://docs.aws.amazon.com/singlesignon/latest/IdentityStoreAPIReference/welcome.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-ssoadmin` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.49.0"
aws-sdk-ssoadmin = "0.19.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust
use aws_sdk_ssoadmin as ssoadmin;

#[tokio::main]
async fn main() -> Result<(), ssoadmin::Error> {
    let config = aws_config::load_from_env().await;
    let client = ssoadmin::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-ssoadmin/latest/aws_sdk_ssoadmin/client/struct.Client.html)
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

