# aws-sdk-sagemakerinferencerouterruntime

Amazon SageMaker Inference Router Runtime is the runtime API for Amazon SageMaker AI inference routers. An inference router sends each request that you make to one of the destinations of a routing target. A destination is a SageMaker AI endpoint or inference component, in your account or in another account, in the router's Amazon Web Services Region or in another Region in the same Amazon Web Services partition. The router chooses a healthy destination with capacity, and prefers destinations in its own Region.

Use InvokeInferenceRouter to get an inference in real time. Use InvokeInferenceRouterAsync to submit a request whose input and response are stored in Amazon S3.

The inference router runtime endpoint supports HTTP/1.1 only. HTTP/2 isn't supported. If you use the Amazon Web Services SDKs or the Amazon Web Services CLI, you don't need to change anything.

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-sagemakerinferencerouterruntime` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-sagemakerinferencerouterruntime = "1.2.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_sagemakerinferencerouterruntime as sagemakerinferencerouterruntime;

#[::tokio::main]
async fn main() -> Result<(), sagemakerinferencerouterruntime::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_sagemakerinferencerouterruntime::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-sagemakerinferencerouterruntime/latest/aws_sdk_sagemakerinferencerouterruntime/client/struct.Client.html)
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

