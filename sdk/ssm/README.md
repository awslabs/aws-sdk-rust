# aws-sdk-ssm

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

Amazon Web Services Systems Manager is a collection of capabilities to help you manage your applications and infrastructure running in the Amazon Web Services Cloud;. Systems Manager simplifies application and resource management, shortens the time to detect and resolve operational problems, and helps you manage your Amazon Web Services resources securely at scale.

This reference is intended to be used with the [Amazon Web Services Systems Manager User Guide](https://docs.aws.amazon.com/systems-manager/latest/userguide/).

To get started, verify prerequisites. For more information, see [Setting up Amazon Web Services Systems Manager](https://docs.aws.amazon.com/systems-manager/latest/userguide/systems-manager-setting-up.html).

__Related resources__
  - For information about how to use a Query API, see [Making API requests](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/making-api-requests.html).
  - For information about other API operations you can perform on EC2 instances, see the [Amazon EC2 API Reference](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/).
  - For information about AppConfig, a capability of Systems Manager, see the [AppConfig User Guide](https://docs.aws.amazon.com/appconfig/latest/userguide/) and the [AppConfig API Reference](https://docs.aws.amazon.com/appconfig/2019-10-09/APIReference/).
  - For information about Incident Manager, a capability of Systems Manager, see the [Incident Manager User Guide](https://docs.aws.amazon.com/incident-manager/latest/userguide/) and the [Incident Manager API Reference](https://docs.aws.amazon.com/incident-manager/latest/APIReference/).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-ssm` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.47.0"
aws-sdk-ssm = "0.17.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust
use aws_sdk_ssm as ssm;

#[tokio::main]
async fn main() -> Result<(), ssm::Error> {
    let config = aws_config::load_from_env().await;
    let client = ssm::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-ssm/latest/aws_sdk_ssm/client/struct.Client.html)
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

