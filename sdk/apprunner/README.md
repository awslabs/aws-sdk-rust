# aws-sdk-apprunner

App Runner is an application service that provides a fast, simple, and cost-effective way to go directly from an existing container image or source code to a running service in the Amazon Web Services Cloud in seconds. You don't need to learn new technologies, decide which compute service to use, or understand how to provision and configure Amazon Web Services resources.

App Runner connects directly to your container registry or source code repository. It provides an automatic delivery pipeline with fully managed operations, high performance, scalability, and security.

For more information about App Runner, see the [App Runner Developer Guide](https://docs.aws.amazon.com/apprunner/latest/dg/). For release information, see the [App Runner Release Notes](https://docs.aws.amazon.com/apprunner/latest/relnotes/).

To install the Software Development Kits (SDKs), Integrated Development Environment (IDE) Toolkits, and command line tools that you can use to access the API, see [Tools for Amazon Web Services](http://aws.amazon.com/tools/).

__Endpoints__

For a list of Region-specific endpoints that App Runner supports, see [App Runner endpoints and quotas](https://docs.aws.amazon.com/general/latest/gr/apprunner.html) in the _Amazon Web Services General Reference_.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-apprunner` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-apprunner = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_apprunner as apprunner;

#[::tokio::main]
async fn main() -> Result<(), apprunner::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_apprunner::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-apprunner/latest/aws_sdk_apprunner/client/struct.Client.html)
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

