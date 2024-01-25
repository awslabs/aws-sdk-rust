# aws-sdk-networkmonitor

Amazon CloudWatch Network Monitor is an Amazon Web Services active network monitoring service that identifies if a network issues exists within the Amazon Web Services network or your own company network. Within Network Monitor you'll choose the source VPCs and subnets from the Amazon Web Services network in which you operate and then you'll choose the destination IP addresses from your on-premises network. From these sources and destinations, Network Monitor creates a monitor containing all the possible source and destination combinations, each of which is called a probe, within a single monitor. These probes then monitor network traffic to help you identify where network issues might be affecting your traffic.

For more information, see [Using Amazon CloudWatch Network Monitor](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/what-is-network-monitor.html) in the _Amazon CloudWatch User Guide_.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-networkmonitor` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.4", features = ["behavior-version-latest"] }
aws-sdk-networkmonitor = "1.3.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_networkmonitor as networkmonitor;

#[::tokio::main]
async fn main() -> Result<(), networkmonitor::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_networkmonitor::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-networkmonitor/latest/aws_sdk_networkmonitor/client/struct.Client.html)
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

