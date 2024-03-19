# aws-sdk-workspacesthinclient

Amazon WorkSpaces Thin Client is an affordable device built to work with Amazon Web Services End User Computing (EUC) virtual desktops to provide users with a complete cloud desktop solution. WorkSpaces Thin Client is a compact device designed to connect up to two monitors and USB devices like a keyboard, mouse, headset, and webcam. To maximize endpoint security, WorkSpaces Thin Client devices do not allow local data storage or installation of unapproved applications. The WorkSpaces Thin Client device ships preloaded with device management software.

You can use these APIs to complete WorkSpaces Thin Client tasks, such as creating environments or viewing devices. For more information about WorkSpaces Thin Client, including the required permissions to use the service, see the [Amazon WorkSpaces Thin Client Administrator Guide](https://docs.aws.amazon.com/workspaces-thin-client/latest/ag/). For more information about using the Command Line Interface (CLI) to manage your WorkSpaces Thin Client resources, see the [WorkSpaces Thin Client section of the CLI Reference](https://docs.aws.amazon.com/cli/latest/reference/workspaces-thin-client/index.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-workspacesthinclient` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-workspacesthinclient = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_workspacesthinclient as workspacesthinclient;

#[::tokio::main]
async fn main() -> Result<(), workspacesthinclient::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_workspacesthinclient::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-workspacesthinclient/latest/aws_sdk_workspacesthinclient/client/struct.Client.html)
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

