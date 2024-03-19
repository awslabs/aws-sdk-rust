# aws-sdk-sagemakera2iruntime

Amazon Augmented AI (Amazon A2I) adds the benefit of human judgment to any machine learning application. When an AI application can't evaluate data with a high degree of confidence, human reviewers can take over. This human review is called a human review workflow. To create and start a human review workflow, you need three resources: a _worker task template_, a _flow definition_, and a _human loop_.

For information about these resources and prerequisites for using Amazon A2I, see [Get Started with Amazon Augmented AI](https://docs.aws.amazon.com/sagemaker/latest/dg/a2i-getting-started.html) in the Amazon SageMaker Developer Guide.

This API reference includes information about API actions and data types that you can use to interact with Amazon A2I programmatically. Use this guide to:
  - Start a human loop with the StartHumanLoop operation when using Amazon A2I with a _custom task type_. To learn more about the difference between custom and built-in task types, see [Use Task Types](https://docs.aws.amazon.com/sagemaker/latest/dg/a2i-task-types-general.html). To learn how to start a human loop using this API, see [Create and Start a Human Loop for a Custom Task Type](https://docs.aws.amazon.com/sagemaker/latest/dg/a2i-start-human-loop.html#a2i-instructions-starthumanloop) in the Amazon SageMaker Developer Guide.
  - Manage your human loops. You can list all human loops that you have created, describe individual human loops, and stop and delete human loops. To learn more, see [Monitor and Manage Your Human Loop](https://docs.aws.amazon.com/sagemaker/latest/dg/a2i-monitor-humanloop-results.html) in the Amazon SageMaker Developer Guide.

Amazon A2I integrates APIs from various AWS services to create and start human review workflows for those services. To learn how Amazon A2I uses these APIs, see [Use APIs in Amazon A2I](https://docs.aws.amazon.com/sagemaker/latest/dg/a2i-api-references.html) in the Amazon SageMaker Developer Guide.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-sagemakera2iruntime` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-sagemakera2iruntime = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_sagemakera2iruntime as sagemakera2iruntime;

#[::tokio::main]
async fn main() -> Result<(), sagemakera2iruntime::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_sagemakera2iruntime::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-sagemakera2iruntime/latest/aws_sdk_sagemakera2iruntime/client/struct.Client.html)
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

