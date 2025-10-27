# aws-sdk-mpa

Multi-party approval is a capability of [Organizations](http://aws.amazon.com/organizations) that allows you to protect a predefined list of operations through a distributed approval process. Use Multi-party approval to establish approval workflows and transform security processes into team-based decisions.

__When to use Multi-party approval__:
  - You need to align with the Zero Trust principle of "never trust, always verify"
  - You need to make sure that the right humans have access to the right things in the right way
  - You need distributed decision-making for sensitive or critical operations
  - You need to protect against unintended operations on sensitive or critical resources
  - You need formal reviews and approvals for auditing or compliance reasons

For more information, see [What is Multi-party approval](https://docs.aws.amazon.com/mpa/latest/userguide/what-is.html) in the _Multi-party approval User Guide_.

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-mpa` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-mpa = "1.14.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_mpa as mpa;

#[::tokio::main]
async fn main() -> Result<(), mpa::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_mpa::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-mpa/latest/aws_sdk_mpa/client/struct.Client.html)
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

