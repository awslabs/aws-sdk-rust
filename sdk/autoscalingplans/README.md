# aws-sdk-autoscalingplans

Use AWS Auto Scaling to create scaling plans for your applications to automatically scale your scalable AWS resources.

__API Summary__

You can use the AWS Auto Scaling service API to accomplish the following tasks:
  - Create and manage scaling plans
  - Define target tracking scaling policies to dynamically scale your resources based on utilization
  - Scale Amazon EC2 Auto Scaling groups using predictive scaling and dynamic scaling to scale your Amazon EC2 capacity faster
  - Set minimum and maximum capacity limits
  - Retrieve information on existing scaling plans
  - Access current forecast data and historical forecast data for up to 56 days previous

To learn more about AWS Auto Scaling, including information about granting IAM users required permissions for AWS Auto Scaling actions, see the [AWS Auto Scaling User Guide](https://docs.aws.amazon.com/autoscaling/plans/userguide/what-is-aws-auto-scaling.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-autoscalingplans` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-autoscalingplans = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_autoscalingplans as autoscalingplans;

#[::tokio::main]
async fn main() -> Result<(), autoscalingplans::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_autoscalingplans::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-autoscalingplans/latest/aws_sdk_autoscalingplans/client/struct.Client.html)
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

