# aws-sdk-iotdeviceadvisor

Amazon Web Services IoT Core Device Advisor is a cloud-based, fully managed test capability for validating IoT devices during device software development. Device Advisor provides pre-built tests that you can use to validate IoT devices for reliable and secure connectivity with Amazon Web Services IoT Core before deploying devices to production. By using Device Advisor, you can confirm that your devices can connect to Amazon Web Services IoT Core, follow security best practices and, if applicable, receive software updates from IoT Device Management. You can also download signed qualification reports to submit to the Amazon Web Services Partner Network to get your device qualified for the Amazon Web Services Partner Device Catalog without the need to send your device in and wait for it to be tested.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-iotdeviceadvisor` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-iotdeviceadvisor = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_iotdeviceadvisor as iotdeviceadvisor;

#[::tokio::main]
async fn main() -> Result<(), iotdeviceadvisor::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_iotdeviceadvisor::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-iotdeviceadvisor/latest/aws_sdk_iotdeviceadvisor/client/struct.Client.html)
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

