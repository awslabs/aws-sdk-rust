# aws-sdk-route53recoverycluster

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

Welcome to the Amazon Route 53 Application Recovery Controller API Reference Guide for Recovery Control Data Plane .

Recovery control in Route 53 Application Recovery Controller includes extremely reliable routing controls that enable you to recover applications by rerouting traffic, for example, across Availability Zones or AWS Regions. Routing controls are simple on/off switches hosted on a cluster. A cluster is a set of five redundant regional endpoints against which you can execute API calls to update or get the state of routing controls. You use routing controls to failover traffic to recover your application across Availability Zones or Regions.

This API guide includes information about how to get and update routing control states in Route 53 Application Recovery Controller.

For more information about Route 53 Application Recovery Controller, see the following:
  - You can create clusters, routing controls, and control panels by using the control plane API for Recovery Control. For more information, see [Amazon Route 53 Application Recovery Controller Recovery Control API Reference](https://docs.aws.amazon.com/recovery-cluster/latest/api/).
  - Route 53 Application Recovery Controller also provides continuous readiness checks to ensure that your applications are scaled to handle failover traffic. For more information about the related API actions, see [Amazon Route 53 Application Recovery Controller Recovery Readiness API Reference](https://docs.aws.amazon.com/recovery-readiness/latest/api/).
  - For more information about creating resilient applications and preparing for recovery readiness with Route 53 Application Recovery Controller, see the [Amazon Route 53 Application Recovery Controller Developer Guide](r53recovery/latest/dg/).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-route53recoverycluster` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.7.0"
aws-sdk-route53recoverycluster = "0.7.0"
tokio = { version = "1", features = ["full"] }
```

## Using the SDK

Until the SDK is released, we will be adding information about using the SDK to the
[Guide](https://github.com/awslabs/aws-sdk-rust/blob/main/Guide.md). Feel free to suggest
additional sections for the guide by opening an issue and describing what you are trying to do.

## Getting Help

* [GitHub discussions](https://github.com/awslabs/aws-sdk-rust/discussions) - For ideas, RFCs & general questions
* [GitHub issues](https://github.com/awslabs/aws-sdk-rust/issues/new/choose) – For bug reports & feature requests
* [Generated Docs (latest version)](https://awslabs.github.io/aws-sdk-rust/)
* [Usage examples](https://github.com/awslabs/aws-sdk-rust/tree/main/examples)

## License

This project is licensed under the Apache-2.0 License.

