# aws-sdk-simspaceweaver

SimSpace Weaver (SimSpace Weaver) is a service that you can use to build and run large-scale spatial simulations in the Amazon Web Services Cloud. For example, you can create crowd simulations, large real-world environments, and immersive and interactive experiences. For more information about SimSpace Weaver, see the _ [SimSpace Weaver User Guide](https://docs.aws.amazon.com/simspaceweaver/latest/userguide/) _.

This API reference describes the API operations and data types that you can use to communicate directly with SimSpace Weaver.

SimSpace Weaver also provides the SimSpace Weaver app SDK, which you use for app development. The SimSpace Weaver app SDK API reference is included in the SimSpace Weaver app SDK documentation. This documentation is part of the SimSpace Weaver app SDK distributable package.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-simspaceweaver` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-simspaceweaver = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_simspaceweaver as simspaceweaver;

#[::tokio::main]
async fn main() -> Result<(), simspaceweaver::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_simspaceweaver::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-simspaceweaver/latest/aws_sdk_simspaceweaver/client/struct.Client.html)
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

