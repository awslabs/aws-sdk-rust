# aws-sdk-devopsagent

AWS DevOps Agent is a frontier agent that resolves and proactively prevents incidents, continuously improving reliability and performance. AWS DevOps Agent investigates incidents and identifies operational improvements as an experienced DevOps engineer. The agent works by: * Learning your resources and their relationships. * Working with your observability tools, runbooks, code repositories, and CI/CD pipelines. * Correlating telemetry, code, and deployment data to understand relationships between your application resources. * Supporting applications in multicloud and hybrid environments.

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-devopsagent` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-devopsagent = "1.4.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_devopsagent as devopsagent;

#[::tokio::main]
async fn main() -> Result<(), devopsagent::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_devopsagent::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-devopsagent/latest/aws_sdk_devopsagent/client/struct.Client.html)
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

