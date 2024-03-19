# aws-sdk-devopsguru

Amazon DevOps Guru is a fully managed service that helps you identify anomalous behavior in business critical operational applications. You specify the Amazon Web Services resources that you want DevOps Guru to cover, then the Amazon CloudWatch metrics and Amazon Web Services CloudTrail events related to those resources are analyzed. When anomalous behavior is detected, DevOps Guru creates an _insight_ that includes recommendations, related events, and related metrics that can help you improve your operational applications. For more information, see [What is Amazon DevOps Guru](https://docs.aws.amazon.com/devops-guru/latest/userguide/welcome.html).

You can specify 1 or 2 Amazon Simple Notification Service topics so you are notified every time a new insight is created. You can also enable DevOps Guru to generate an OpsItem in Amazon Web Services Systems Manager for each insight to help you manage and track your work addressing insights.

To learn about the DevOps Guru workflow, see [How DevOps Guru works](https://docs.aws.amazon.com/devops-guru/latest/userguide/welcome.html#how-it-works). To learn about DevOps Guru concepts, see [Concepts in DevOps Guru](https://docs.aws.amazon.com/devops-guru/latest/userguide/concepts.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-devopsguru` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-devopsguru = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_devopsguru as devopsguru;

#[::tokio::main]
async fn main() -> Result<(), devopsguru::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_devopsguru::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-devopsguru/latest/aws_sdk_devopsguru/client/struct.Client.html)
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

