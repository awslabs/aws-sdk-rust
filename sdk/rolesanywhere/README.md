# aws-sdk-rolesanywhere

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

AWS Identity and Access Management Roles Anywhere provides a secure way for your workloads such as servers, containers, and applications running outside of AWS to obtain Temporary AWS credentials. Your workloads can use the same IAM policies and roles that you have configured with native AWS applications to access AWS resources. Using IAM Roles Anywhere will eliminate the need to manage long term credentials for workloads running outside of AWS.

To use IAM Roles Anywhere customer workloads will need to use X.509 certificates issued by their Certificate Authority (CA) . The Certificate Authority (CA) needs to be registered with IAM Roles Anywhere as a trust anchor to establish trust between customer PKI and IAM Roles Anywhere. Customers who do not manage their own PKI system can use AWS Certificate Manager Private Certificate Authority (ACM PCA) to create a Certificate Authority and use that to establish trust with IAM Roles Anywhere

This guide describes the IAM rolesanywhere operations that you can call programmatically. For general information about IAM Roles Anywhere see [https://docs.aws.amazon.com/](https://docs.aws.amazon.com/)

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-rolesanywhere` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.49.0"
aws-sdk-rolesanywhere = "0.4.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust
use aws_sdk_rolesanywhere as rolesanywhere;

#[tokio::main]
async fn main() -> Result<(), rolesanywhere::Error> {
    let config = aws_config::load_from_env().await;
    let client = rolesanywhere::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-rolesanywhere/latest/aws_sdk_rolesanywhere/client/struct.Client.html)
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

