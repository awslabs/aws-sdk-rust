# aws-sdk-sagemakergatewayruntime

Use the Amazon SageMaker Gateway Runtime API to run inference through a Global Inference Gateway. A Global Inference Gateway routes each request to an inference gateway that serves the model named in the request. The request body is an OpenAI-compatible JSON object, sent with `Content-Type: application/json`, and the model is selected by the `model` key in that body. The content of the body is passed to the model server unchanged, except for the `stream` flag, which each operation sets. Requests are signed with Signature Version 4 under the `sagemaker` signing name. IAM actions are in the `sagemaker` namespace, for example `sagemaker:InvokeGlobalInferenceGateway`. The default regional endpoint is `gateway-runtime.sagemaker.{region}.amazonaws.com`.

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-sagemakergatewayruntime` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-sagemakergatewayruntime = "1.0.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_sagemakergatewayruntime as sagemakergatewayruntime;

#[::tokio::main]
async fn main() -> Result<(), sagemakergatewayruntime::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_sagemakergatewayruntime::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-sagemakergatewayruntime/latest/aws_sdk_sagemakergatewayruntime/client/struct.Client.html)
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

