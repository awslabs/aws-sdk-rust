# aws-sdk-wellarchitected

Amazon Web Services Well-Architected helps you evaluate your architectures against Amazon Web Services best practices across operational excellence, security, reliability, performance efficiency, cost optimization, and sustainability. The service includes the Amazon Web Services Well-Architected Agent for AI-powered recommendations tailored to your specific environment, and the [Well-Architected Tool](http://aws.amazon.com/well-architected-tool) for conducting reviews and tracking improvements.

This is the _Amazon Web Services Well-Architected API Reference_. Through this API, you can programmatically access personalized recommendations and automation scripts from the Amazon Web Services Well-Architected Agent, and create and manage workloads, conduct lens reviews, track milestones, manage custom lenses, share workloads across accounts, and manage profiles with the Well-Architected Tool.

For more information about the service, see the [Amazon Web Services Well-Architected User Guide](https://docs.aws.amazon.com/wellarchitected/latest/userguide/intro.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-wellarchitected` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-wellarchitected = "1.111.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_wellarchitected as wellarchitected;

#[::tokio::main]
async fn main() -> Result<(), wellarchitected::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_wellarchitected::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-wellarchitected/latest/aws_sdk_wellarchitected/client/struct.Client.html)
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

