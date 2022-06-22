# aws-sdk-emrcontainers

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

Amazon EMR on EKS provides a deployment option for Amazon EMR that allows you to run open-source big data frameworks on Amazon Elastic Kubernetes Service (Amazon EKS). With this deployment option, you can focus on running analytics workloads while Amazon EMR on EKS builds, configures, and manages containers for open-source applications. For more information about Amazon EMR on EKS concepts and tasks, see [What is Amazon EMR on EKS](https://docs.aws.amazon.com/emr/latest/EMR-on-EKS-DevelopmentGuide/emr-eks.html).

_Amazon EMR containers_ is the API name for Amazon EMR on EKS. The emr-containers prefix is used in the following scenarios:
  - It is the prefix in the CLI commands for Amazon EMR on EKS. For example, aws emr-containers start-job-run.
  - It is the prefix before IAM policy actions for Amazon EMR on EKS. For example, "Action": [ "emr-containers:StartJobRun"]. For more information, see [Policy actions for Amazon EMR on EKS](https://docs.aws.amazon.com/emr/latest/EMR-on-EKS-DevelopmentGuide/security_iam_service-with-iam.html#security_iam_service-with-iam-id-based-policies-actions).
  - It is the prefix used in Amazon EMR on EKS service endpoints. For example, emr-containers.us-east-2.amazonaws.com. For more information, see [Amazon EMR on EKS Service Endpoints](https://docs.aws.amazon.com/emr/latest/EMR-on-EKS-DevelopmentGuide/service-quotas.html#service-endpoints).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-emrcontainers` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.14.0"
aws-sdk-emrcontainers = "0.14.0"
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

