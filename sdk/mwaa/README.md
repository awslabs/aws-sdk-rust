# aws-sdk-mwaa

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

This section contains the Amazon Managed Workflows for Apache Airflow (MWAA) API reference documentation. For more information, see [What Is Amazon MWAA?](https://docs.aws.amazon.com/mwaa/latest/userguide/what-is-mwaa.html).

__Endpoints__
  - api.airflow.{region}.amazonaws.com - This endpoint is used for environment management.
    - [CreateEnvironment](https://docs.aws.amazon.com/mwaa/latest/API/API_CreateEnvironment.html)
    - [DeleteEnvironment](https://docs.aws.amazon.com/mwaa/latest/API/API_DeleteEnvironment.html)
    - [GetEnvironment](https://docs.aws.amazon.com/mwaa/latest/API/API_GetEnvironment.html)
    - [ListEnvironments](https://docs.aws.amazon.com/mwaa/latest/API/API_ListEnvironments.html)
    - [ListTagsForResource](https://docs.aws.amazon.com/mwaa/latest/API/API_ListTagsForResource.html)
    - [TagResource](https://docs.aws.amazon.com/mwaa/latest/API/API_TagResource.html)
    - [UntagResource](https://docs.aws.amazon.com/mwaa/latest/API/API_UntagResource.html)
    - [UpdateEnvironment](https://docs.aws.amazon.com/mwaa/latest/API/API_UpdateEnvironment.html)

  - env.airflow.{region}.amazonaws.com - This endpoint is used to operate the Airflow environment.
    - [CreateCliToken](https://docs.aws.amazon.com/mwaa/latest/API/API_CreateCliToken.html )
    - [CreateWebLoginToken](https://docs.aws.amazon.com/mwaa/latest/API/API_CreateWebLoginToken.html)

  - ops.airflow.{region}.amazonaws.com - This endpoint is used to push environment metrics that track environment health.
    - [PublishMetrics](https://docs.aws.amazon.com/mwaa/latest/API/API_PublishMetrics.html )

__Regions__

For a list of regions that Amazon MWAA supports, see [Region availability](https://docs.aws.amazon.com/mwaa/latest/userguide/what-is-mwaa.html#regions-mwaa) in the _Amazon MWAA User Guide_.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-mwaa` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.14.0"
aws-sdk-mwaa = "0.14.0"
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

