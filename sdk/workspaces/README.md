# aws-sdk-workspaces

Amazon WorkSpaces enables you to provision virtual, cloud-based Microsoft Windows or Amazon Linux desktops for your users, known as _WorkSpaces_. WorkSpaces eliminates the need to procure and deploy hardware or install complex software. You can quickly add or remove users as your needs change. Users can access their virtual desktops from multiple devices or web browsers.

This API Reference provides detailed information about the actions, data types, parameters, and errors of the WorkSpaces service. For more information about the supported Amazon Web Services Regions, endpoints, and service quotas of the Amazon WorkSpaces service, see [WorkSpaces endpoints and quotas](https://docs.aws.amazon.com/general/latest/gr/wsp.html) in the _Amazon Web Services General Reference_.

You can also manage your WorkSpaces resources using the WorkSpaces console, Command Line Interface (CLI), and SDKs. For more information about administering WorkSpaces, see the [Amazon WorkSpaces Administration Guide](https://docs.aws.amazon.com/workspaces/latest/adminguide/). For more information about using the Amazon WorkSpaces client application or web browser to access provisioned WorkSpaces, see the [Amazon WorkSpaces User Guide](https://docs.aws.amazon.com/workspaces/latest/userguide/). For more information about using the CLI to manage your WorkSpaces resources, see the [WorkSpaces section of the CLI Reference](https://docs.aws.amazon.com/cli/latest/reference/workspaces/index.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-workspaces` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-workspaces = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_workspaces as workspaces;

#[::tokio::main]
async fn main() -> Result<(), workspaces::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_workspaces::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-workspaces/latest/aws_sdk_workspaces/client/struct.Client.html)
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

