# aws-sdk-route53recoverycluster

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

Welcome to the Routing Control (Recovery Cluster) API Reference Guide for Amazon Route 53 Application Recovery Controller.

With Amazon Route 53 Application Recovery Controller, you can use routing control with extreme reliability to recover applications by rerouting traffic across Availability Zones or AWS Regions. Routing controls are simple on/off switches hosted on a highly available cluster in Application Recovery Controller. A cluster provides a set of five redundant Regional endpoints against which you can run API calls to get or update the state of routing controls. To implement failover, you set one routing control on and another one off, to reroute traffic from one Availability Zone or Amazon Web Services Region to another.

_Be aware that you must specify the Regional endpoints for a cluster when you work with API cluster operations to get or update routing control states in Application Recovery Controller._ In addition, you must specify the US West (Oregon) Region for Application Recovery Controller API calls. For example, use the parameter region us-west-2 with AWS CLI commands. For more information, see [Get and update routing control states using the API](https://docs.aws.amazon.com/r53recovery/latest/dg/routing-control.update.api.html) in the Amazon Route 53 Application Recovery Controller Developer Guide.

This API guide includes information about the API operations for how to get and update routing control states in Application Recovery Controller. You also must set up the structures to support routing controls: clusters and control panels.

For more information about working with routing control in Application Recovery Controller, see the following:
  - To create clusters, routing controls, and control panels by using the control plane API for routing control, see the [Recovery Control Configuration API Reference Guide for Amazon Route 53 Application Recovery Controller](https://docs.aws.amazon.com/recovery-cluster/latest/api/).
  - Learn about the components in recovery control configuration, including clusters, routing controls, and control panels. For more information, see [Recovery control components](https://docs.aws.amazon.com/r53recovery/latest/dg/introduction-components.html#introduction-components-routing) in the Amazon Route 53 Application Recovery Controller Developer Guide.
  - Application Recovery Controller also provides readiness checks that run continually to help make sure that your applications are scaled and ready to handle failover traffic. For more information about the related API actions, see the [Recovery Readiness API Reference Guide for Amazon Route 53 Application Recovery Controller](https://docs.aws.amazon.com/recovery-readiness/latest/api/).
  - For more information about creating resilient applications and preparing for recovery readiness with Application Recovery Controller, see the [Amazon Route 53 Application Recovery Controller Developer Guide](https://docs.aws.amazon.com/r53recovery/latest/dg/).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-route53recoverycluster` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.9.0"
aws-sdk-route53recoverycluster = "0.9.0"
tokio = { version = "1", features = ["full"] }
```

## Using the SDK

Until the SDK is released, we will be adding information about using the SDK to the
[Developer Guide](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/welcome.html). Feel free to suggest
additional sections for the guide by opening an issue and describing what you are trying to do.

## Getting Help

* [GitHub discussions](https://github.com/awslabs/aws-sdk-rust/discussions) - For ideas, RFCs & general questions
* [GitHub issues](https://github.com/awslabs/aws-sdk-rust/issues/new/choose) – For bug reports & feature requests
* [Generated Docs (latest version)](https://awslabs.github.io/aws-sdk-rust/)
* [Usage examples](https://github.com/awslabs/aws-sdk-rust/tree/main/examples)

## License

This project is licensed under the Apache-2.0 License.

