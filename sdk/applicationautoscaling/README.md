# aws-sdk-applicationautoscaling

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

With Application Auto Scaling, you can configure automatic scaling for the following resources:
  - Amazon AppStream 2.0 fleets
  - Amazon Aurora Replicas
  - Amazon Comprehend document classification and entity recognizer endpoints
  - Amazon DynamoDB tables and global secondary indexes throughput capacity
  - Amazon ECS services
  - Amazon ElastiCache for Redis clusters (replication groups)
  - Amazon EMR clusters
  - Amazon Keyspaces (for Apache Cassandra) tables
  - Lambda function provisioned concurrency
  - Amazon Managed Streaming for Apache Kafka broker storage
  - Amazon Neptune clusters
  - Amazon SageMaker endpoint variants
  - Spot Fleets (Amazon EC2)
  - Custom resources provided by your own applications or services

__API Summary__

The Application Auto Scaling service API includes three key sets of actions:
  - Register and manage scalable targets - Register Amazon Web Services or custom resources as scalable targets (a resource that Application Auto Scaling can scale), set minimum and maximum capacity limits, and retrieve information on existing scalable targets.
  - Configure and manage automatic scaling - Define scaling policies to dynamically scale your resources in response to CloudWatch alarms, schedule one-time or recurring scaling actions, and retrieve your recent scaling activity history.
  - Suspend and resume scaling - Temporarily suspend and later resume automatic scaling by calling the [RegisterScalableTarget](https://docs.aws.amazon.com/autoscaling/application/APIReference/API_RegisterScalableTarget.html) API action for any Application Auto Scaling scalable target. You can suspend and resume (individually or in combination) scale-out activities that are triggered by a scaling policy, scale-in activities that are triggered by a scaling policy, and scheduled scaling.

To learn more about Application Auto Scaling, including information about granting IAM users required permissions for Application Auto Scaling actions, see the [Application Auto Scaling User Guide](https://docs.aws.amazon.com/autoscaling/application/userguide/what-is-application-auto-scaling.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-applicationautoscaling` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.14.0"
aws-sdk-applicationautoscaling = "0.14.0"
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

