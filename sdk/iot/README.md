# aws-sdk-iot

IoT provides secure, bi-directional communication between Internet-connected devices (such as sensors, actuators, embedded devices, or smart appliances) and the Amazon Web Services cloud. You can discover your custom IoT-Data endpoint to communicate with, configure rules for data processing and integration with other services, organize resources associated with each device (Registry), configure logging, and create and manage policies and credentials to authenticate devices.

The service endpoints that expose this API are listed in [Amazon Web Services IoT Core Endpoints and Quotas](https://docs.aws.amazon.com/general/latest/gr/iot-core.html). You must use the endpoint for the region that has the resources you want to access.

The service name used by [Amazon Web Services Signature Version 4](https://docs.aws.amazon.com/general/latest/gr/signature-version-4.html) to sign the request is: _execute-api_.

For more information about how IoT works, see the [Developer Guide](https://docs.aws.amazon.com/iot/latest/developerguide/aws-iot-how-it-works.html).

For information about how to use the credentials provider for IoT, see [Authorizing Direct Calls to Amazon Web Services Services](https://docs.aws.amazon.com/iot/latest/developerguide/authorizing-direct-aws.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-iot` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-iot = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_iot as iot;

#[::tokio::main]
async fn main() -> Result<(), iot::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_iot::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-iot/latest/aws_sdk_iot/client/struct.Client.html)
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

