# aws-sdk-route53recoverycluster

Welcome to the Routing Control (Recovery Cluster) API Reference Guide for Amazon Route 53 Application Recovery Controller.

With Route 53 ARC, you can use routing control with extreme reliability to recover applications by rerouting traffic across Availability Zones or Amazon Web Services Regions. Routing controls are simple on/off switches hosted on a highly available cluster in Route 53 ARC. A cluster provides a set of five redundant Regional endpoints against which you can run API calls to get or update the state of routing controls. To implement failover, you set one routing control to ON and another one to OFF, to reroute traffic from one Availability Zone or Amazon Web Services Region to another.

_Be aware that you must specify a Regional endpoint for a cluster when you work with API cluster operations to get or update routing control states in Route 53 ARC._ In addition, you must specify the US West (Oregon) Region for Route 53 ARC API calls. For example, use the parameter --region us-west-2 with AWS CLI commands. For more information, see [Get and update routing control states using the API](https://docs.aws.amazon.com/r53recovery/latest/dg/routing-control.update.api.html) in the Amazon Route 53 Application Recovery Controller Developer Guide.

This API guide includes information about the API operations for how to get and update routing control states in Route 53 ARC. To work with routing control in Route 53 ARC, you must first create the required components (clusters, control panels, and routing controls) using the recovery cluster configuration API.

For more information about working with routing control in Route 53 ARC, see the following:
  - Create clusters, control panels, and routing controls by using API operations. For more information, see the [Recovery Control Configuration API Reference Guide for Amazon Route 53 Application Recovery Controller](https://docs.aws.amazon.com/recovery-cluster/latest/api/).
  - Learn about the components in recovery control, including clusters, routing controls, and control panels, and how to work with Route 53 ARC in the Amazon Web Services console. For more information, see [Recovery control components](https://docs.aws.amazon.com/r53recovery/latest/dg/introduction-components.html#introduction-components-routing) in the Amazon Route 53 Application Recovery Controller Developer Guide.
  - Route 53 ARC also provides readiness checks that continually audit resources to help make sure that your applications are scaled and ready to handle failover traffic. For more information about the related API operations, see the [Recovery Readiness API Reference Guide for Amazon Route 53 Application Recovery Controller](https://docs.aws.amazon.com/recovery-readiness/latest/api/).
  - For more information about creating resilient applications and preparing for recovery readiness with Route 53 ARC, see the [Amazon Route 53 Application Recovery Controller Developer Guide](https://docs.aws.amazon.com/r53recovery/latest/dg/).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-route53recoverycluster` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-route53recoverycluster = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_route53recoverycluster as route53recoverycluster;

#[::tokio::main]
async fn main() -> Result<(), route53recoverycluster::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_route53recoverycluster::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-route53recoverycluster/latest/aws_sdk_route53recoverycluster/client/struct.Client.html)
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

