# aws-sdk-networkflowmonitor

Network Flow Monitor is a feature of Amazon CloudWatch Network Monitoring that provides visibility into the performance of network flows for your Amazon Web Services workloads, between instances in subnets, as well as to and from Amazon Web Services. Lightweight agents that you install on the instances capture performance metrics for your network flows, such as packet loss and latency, and send them to the Network Flow Monitor backend. Then, you can view and analyze metrics from the top contributors for each metric type, to help troubleshoot issues.

In addition, when you create a monitor, Network Flow Monitor provides a network health indicator (NHI) that informs you whether there were Amazon Web Services network issues for one or more of the network flows tracked by a monitor, during a time period that you choose. By using this value, you can independently determine if the Amazon Web Services network is impacting your workload during a specific time frame, to help you focus troubleshooting efforts.

To learn more about Network Flow Monitor, see the [Network Flow Monitor User Guide](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-NetworkFlowMonitor.html) in the Amazon CloudWatch User Guide.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-networkflowmonitor` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-networkflowmonitor = "1.11.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_networkflowmonitor as networkflowmonitor;

#[::tokio::main]
async fn main() -> Result<(), networkflowmonitor::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_networkflowmonitor::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-networkflowmonitor/latest/aws_sdk_networkflowmonitor/client/struct.Client.html)
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

