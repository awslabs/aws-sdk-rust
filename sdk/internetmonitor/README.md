# aws-sdk-internetmonitor

Amazon CloudWatch Internet Monitor provides visibility into how internet issues impact the performance and availability between your applications hosted on Amazon Web Services and your end users. It can reduce the time it takes for you to diagnose internet issues from days to minutes. Internet Monitor uses the connectivity data that Amazon Web Services captures from its global networking footprint to calculate a baseline of performance and availability for internet traffic. This is the same data that Amazon Web Services uses to monitor internet uptime and availability. With those measurements as a baseline, Internet Monitor raises awareness for you when there are significant problems for your end users in the different geographic locations where your application runs.

Internet Monitor publishes internet measurements to CloudWatch Logs and CloudWatch Metrics, to easily support using CloudWatch tools with health information for geographies and networks specific to your application. Internet Monitor sends health events to Amazon EventBridge so that you can set up notifications. If an issue is caused by the Amazon Web Services network, you also automatically receive an Amazon Web Services Health Dashboard notification with the steps that Amazon Web Services is taking to mitigate the problem.

To use Internet Monitor, you create a _monitor_ and associate your application's resources with it - VPCs, NLBs, CloudFront distributions, or WorkSpaces directories - so Internet Monitor can determine where your application's internet traffic is. Internet Monitor then provides internet measurements from Amazon Web Services that are specific to the locations and ASNs (typically, internet service providers or ISPs) that communicate with your application.

For more information, see [Using Amazon CloudWatch Internet Monitor](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-InternetMonitor.html) in the _Amazon CloudWatch User Guide_.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-internetmonitor` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-internetmonitor = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_internetmonitor as internetmonitor;

#[::tokio::main]
async fn main() -> Result<(), internetmonitor::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_internetmonitor::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-internetmonitor/latest/aws_sdk_internetmonitor/client/struct.Client.html)
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

