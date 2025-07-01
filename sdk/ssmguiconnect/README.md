# aws-sdk-ssmguiconnect

Systems Manager GUI Connect, a component of Fleet Manager, lets you connect to your Window Server-type Amazon Elastic Compute Cloud (Amazon EC2) instances using the Remote Desktop Protocol (RDP). GUI Connect, which is powered by [Amazon DCV](https://docs.aws.amazon.com/dcv/latest/adminguide/what-is-dcv.html), provides you with secure connectivity to your Windows Server instances directly from the Systems Manager console. You can have up to four simultaneous connections in a single browser window. In the console, GUI Connect is also referred to as Fleet Manager Remote Desktop.

This reference is intended to be used with the [_Amazon Web Services Systems Manager User Guide_](https://docs.aws.amazon.com/systems-manager/latest/userguide/). To get started, see the following user guide topics:
  - [Setting up Amazon Web Services Systems Manager](https://docs.aws.amazon.com/systems-manager/latest/userguide/systems-manager-setting-up.html)
  - [Connect to a Windows Server managed instance using Remote Desktop](https://docs.aws.amazon.com/systems-manager/latest/userguide/fleet-rdp.html)

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-ssmguiconnect` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-ssmguiconnect = "1.9.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_ssmguiconnect as ssmguiconnect;

#[::tokio::main]
async fn main() -> Result<(), ssmguiconnect::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_ssmguiconnect::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-ssmguiconnect/latest/aws_sdk_ssmguiconnect/client/struct.Client.html)
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

