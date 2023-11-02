# aws-sdk-arczonalshift

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

This is the API Reference Guide for the zonal shift feature of Amazon Route 53 Application Recovery Controller. This guide is for developers who need detailed information about zonal shift API actions, data types, and errors.

Zonal shift is in preview release for Amazon Route 53 Application Recovery Controller and is subject to change.

Zonal shift in Route 53 ARC enables you to move traffic for a load balancer resource away from an Availability Zone. Starting a zonal shift helps your application recover immediately, for example, from a developer's bad code deployment or from an AWS infrastructure failure in a single Availability Zone, reducing the impact and time lost from an issue in one zone.

Supported AWS resources are automatically registered with Route 53 ARC. Resources that are registered for zonal shifts in Route 53 ARC are managed resources in Route 53 ARC. You can start a zonal shift for any managed resource in your account in a Region. At this time, you can only start a zonal shift for Network Load Balancers and Application Load Balancers with cross-zone load balancing turned off.

Zonal shifts are temporary. You must specify an expiration when you start a zonal shift, of up to three days initially. If you want to still keep traffic away from an Availability Zone, you can update the zonal shift and set a new expiration. You can also cancel a zonal shift, before it expires, for example, if you're ready to restore traffic to the Availability Zone.

For more information about using zonal shift, see the [Amazon Route 53 Application Recovery Controller Developer Guide](https://docs.aws.amazon.com/r53recovery/latest/dg/what-is-route53-recovery.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-arczonalshift` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.57.1"
aws-sdk-arczonalshift = "0.13.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_arczonalshift as arczonalshift;

#[::tokio::main]
async fn main() -> Result<(), arczonalshift::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_arczonalshift::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-arczonalshift/latest/aws_sdk_arczonalshift/client/struct.Client.html)
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

