# aws-sdk-paymentcryptography

Amazon Web Services Payment Cryptography Control Plane APIs manage encryption keys for use during payment-related cryptographic operations. You can create, import, export, share, manage, and delete keys. You can also manage Identity and Access Management (IAM) policies for keys. For more information, see [Identity and access management](https://docs.aws.amazon.com/payment-cryptography/latest/userguide/security-iam.html) in the _Amazon Web Services Payment Cryptography User Guide._

To use encryption keys for payment-related transaction processing and associated cryptographic operations, you use the [Amazon Web Services Payment Cryptography Data Plane](https://docs.aws.amazon.com/payment-cryptography/latest/DataAPIReference/Welcome.html). You can perform actions like encrypt, decrypt, generate, and verify payment-related data.

All Amazon Web Services Payment Cryptography API calls must be signed and transmitted using Transport Layer Security (TLS). We recommend you always use the latest supported TLS version for logging API requests.

Amazon Web Services Payment Cryptography supports CloudTrail for control plane operations, a service that logs Amazon Web Services API calls and related events for your Amazon Web Services account and delivers them to an Amazon S3 bucket you specify. By using the information collected by CloudTrail, you can determine what requests were made to Amazon Web Services Payment Cryptography, who made the request, when it was made, and so on. If you don't conﬁgure a trail, you can still view the most recent events in the CloudTrail console. For more information, see the [CloudTrail User Guide](https://docs.aws.amazon.com/awscloudtrail/latest/userguide/).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-paymentcryptography` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-paymentcryptography = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_paymentcryptography as paymentcryptography;

#[::tokio::main]
async fn main() -> Result<(), paymentcryptography::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_paymentcryptography::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-paymentcryptography/latest/aws_sdk_paymentcryptography/client/struct.Client.html)
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

