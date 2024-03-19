# aws-sdk-paymentcryptographydata

You use the Amazon Web Services Payment Cryptography Data Plane to manage how encryption keys are used for payment-related transaction processing and associated cryptographic operations. You can encrypt, decrypt, generate, verify, and translate payment-related cryptographic operations in Amazon Web Services Payment Cryptography. For more information, see [Data operations](https://docs.aws.amazon.com/payment-cryptography/latest/userguide/data-operations.html) in the _Amazon Web Services Payment Cryptography User Guide_.

To manage your encryption keys, you use the [Amazon Web Services Payment Cryptography Control Plane](https://docs.aws.amazon.com/payment-cryptography/latest/APIReference/Welcome.html). You can create, import, export, share, manage, and delete keys. You can also manage Identity and Access Management (IAM) policies for keys.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-paymentcryptographydata` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-paymentcryptographydata = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_paymentcryptographydata as paymentcryptographydata;

#[::tokio::main]
async fn main() -> Result<(), paymentcryptographydata::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_paymentcryptographydata::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-paymentcryptographydata/latest/aws_sdk_paymentcryptographydata/client/struct.Client.html)
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

