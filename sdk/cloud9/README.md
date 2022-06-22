# aws-sdk-cloud9

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

Cloud9 is a collection of tools that you can use to code, build, run, test, debug, and release software in the cloud.

For more information about Cloud9, see the [Cloud9 User Guide](https://docs.aws.amazon.com/cloud9/latest/user-guide).

Cloud9 supports these operations:
  - CreateEnvironmentEC2: Creates an Cloud9 development environment, launches an Amazon EC2 instance, and then connects from the instance to the environment.
  - CreateEnvironmentMembership: Adds an environment member to an environment.
  - DeleteEnvironment: Deletes an environment. If an Amazon EC2 instance is connected to the environment, also terminates the instance.
  - DeleteEnvironmentMembership: Deletes an environment member from an environment.
  - DescribeEnvironmentMemberships: Gets information about environment members for an environment.
  - DescribeEnvironments: Gets information about environments.
  - DescribeEnvironmentStatus: Gets status information for an environment.
  - ListEnvironments: Gets a list of environment identifiers.
  - ListTagsForResource: Gets the tags for an environment.
  - TagResource: Adds tags to an environment.
  - UntagResource: Removes tags from an environment.
  - UpdateEnvironment: Changes the settings of an existing environment.
  - UpdateEnvironmentMembership: Changes the settings of an existing environment member for an environment.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-cloud9` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.14.0"
aws-sdk-cloud9 = "0.14.0"
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

