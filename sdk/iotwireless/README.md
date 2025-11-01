# aws-sdk-iotwireless

AWS IoT Wireless provides bi-directional communication between internet-connected wireless devices and the AWS Cloud. To onboard both [LoRaWAN](https://docs.aws.amazon.com/iot-wireless/latest/developerguide/iot-lorawan.html) and [Sidewalk](https://docs.aws.amazon.com/iot-wireless/latest/developerguide/iot-sidewalk.html) devices to AWS IoT, use the IoT Wireless API. These wireless devices use the Low Power Wide Area Networking (LPWAN) communication protocol to communicate with AWS IoT.

Using the API, you can perform create, read, update, and delete operations for your wireless devices, gateways, destinations, and profiles. After onboarding your devices, you can use the API operations to set log levels and monitor your devices with CloudWatch.

You can also use the API operations to create multicast groups and schedule a multicast session for sending a downlink message to devices in the group. By using Firmware Updates Over-The-Air (FUOTA) API operations, you can create a FUOTA task and schedule a session to update the firmware of individual devices or an entire group of devices in a multicast group.

To connect to the AWS IoT Wireless Service, use the Service endpoints as described in [IoT Wireless Service endpoints](https://docs.aws.amazon.com/general/latest/gr/iot-lorawan.html#iot-wireless_region). You can use both IPv4 and IPv6 protocols to connect to the endpoints and send requests to the AWS IoT Wireless service. For more information, see [Using IPv6 with AWS IoT Wireless](https://docs.aws.amazon.com/iot-wireless/latest/developerguide/wireless-ipv6-access.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-iotwireless` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-iotwireless = "1.94.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_iotwireless as iotwireless;

#[::tokio::main]
async fn main() -> Result<(), iotwireless::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_iotwireless::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-iotwireless/latest/aws_sdk_iotwireless/client/struct.Client.html)
for information on what calls can be made, and the inputs and outputs for each of those calls.

## Using the SDK

Until the SDK is released, we will be adding information about using the SDK to the
[Developer Guide](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/welcome.html). Feel free to suggest
additional sections for the guide by opening an issue and describing what you are trying to do.

## Getting Help

* [GitHub discussions](https://github.com/awslabs/aws-sdk-rust/discussions) - For ideas, RFCs & general questions
* [GitHub issues](https://github.com/awslabs/aws-sdk-rust/issues/new/choose) - For bug reports & feature requests
* [Generated Docs (latest version)](https://awslabs.github.io/aws-sdk-rust/)
* [Usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1)

## License

This project is licensed under the Apache-2.0 License.

