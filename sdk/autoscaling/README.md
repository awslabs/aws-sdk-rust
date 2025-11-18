# aws-sdk-autoscaling

The [DescribeAutoScalingGroups](https://docs.aws.amazon.com/autoscaling/ec2/APIReference/API_DescribeAutoScalingGroups.html) API operation might be throttled when retrieving details for an Auto Scaling group that contains many instances. By default, this operation returns details for all instances in the group. To help prevent throttling, you can set the IncludeInstances parameter to false to exclude instance details from the response.

Amazon EC2 Auto Scaling is designed to automatically launch and terminate EC2 instances based on user-defined scaling policies, scheduled actions, and health checks.

For more information, see the [Amazon EC2 Auto Scaling User Guide](https://docs.aws.amazon.com/autoscaling/ec2/userguide/what-is-amazon-ec2-auto-scaling.html) and the [Amazon EC2 Auto Scaling API Reference](https://docs.aws.amazon.com/autoscaling/ec2/APIReference/Welcome.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-autoscaling` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-autoscaling = "1.101.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_autoscaling as autoscaling;

#[::tokio::main]
async fn main() -> Result<(), autoscaling::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_autoscaling::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-autoscaling/latest/aws_sdk_autoscaling/client/struct.Client.html)
for information on what calls can be made, and the inputs and outputs for each of those calls.

## Using the SDK

Until the SDK is released, we will be adding information about using the SDK to the
[Developer Guide](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/welcome.html). Feel free to suggest
additional sections for the guide by opening an issue and describing what you are trying to do.

## Getting Help

* [GitHub discussions](https://github.com/awslabs/aws-sdk-rust/discussions) - For ideas, RFCs & general questions
* [GitHub issues](https://github.com/awslabs/aws-sdk-rust/issues/new/choose) - For bug reports & feature requests
* [Generated Docs (latest version)](https://awslabs.github.io/aws-sdk-rust/)
* [Usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1)

## License

This project is licensed under the Apache-2.0 License.

