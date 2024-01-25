# aws-sdk-arczonalshift

Welcome to the Zonal Shift API Reference Guide for Amazon Route 53 Application Recovery Controller (Route 53 ARC).

You can start a zonal shift to move traffic for a load balancer resource away from an Availability Zone to help your application recover quickly from an impairment in an Availability Zone. For example, you can recover your application from a developer's bad code deployment or from an Amazon Web Services infrastructure failure in a single Availability Zone.

You can also configure zonal autoshift for a load balancer resource. Zonal autoshift is a capability in Route 53 ARC where Amazon Web Services shifts away application resource traffic from an Availability Zone, on your behalf, to help reduce your time to recovery during events. Amazon Web Services shifts away traffic for resources that are enabled for zonal autoshift whenever Amazon Web Services determines that there's an issue in the Availability Zone that could potentially affect customers.

To ensure that zonal autoshift is safe for your application, you must also configure practice runs when you enable zonal autoshift for a resource. Practice runs start weekly zonal shifts for a resource, to shift traffic for the resource out of an Availability Zone. Practice runs make sure, on a regular basis, that you have enough capacity in all the Availability Zones in an Amazon Web Services Region for your application to continue to operate normally when traffic for a resource is shifted away from one Availability Zone.

You must prescale resource capacity in all Availability Zones in the Region where your application is deployed, before you configure practice runs or enable zonal autoshift for a resource. You should not rely on scaling on demand when an autoshift or practice run starts.

For more information about using zonal shift and zonal autoshift, see the [Amazon Route 53 Application Recovery Controller Developer Guide](https://docs.aws.amazon.com/r53recovery/latest/dg/what-is-route53-recovery.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-arczonalshift` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.4", features = ["behavior-version-latest"] }
aws-sdk-arczonalshift = "1.13.0"
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

