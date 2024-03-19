# aws-sdk-supportapp

You can use the Amazon Web Services Support App in Slack API to manage your support cases in Slack for your Amazon Web Services account. After you configure your Slack workspace and channel with the Amazon Web Services Support App, you can perform the following tasks directly in your Slack channel:
  - Create, search, update, and resolve your support cases
  - Request service quota increases for your account
  - Invite Amazon Web Services Support agents to your channel so that you can chat directly about your support cases

For more information about how to perform these actions in Slack, see the following documentation in the _Amazon Web Services Support User Guide_:
  - [Amazon Web Services Support App in Slack](https://docs.aws.amazon.com/awssupport/latest/user/aws-support-app-for-slack.html)
  - [Joining a live chat session with Amazon Web Services Support](https://docs.aws.amazon.com/awssupport/latest/user/joining-a-live-chat-session.html)
  - [Requesting service quota increases](https://docs.aws.amazon.com/awssupport/latest/user/service-quota-increase.html)
  - [Amazon Web Services Support App commands in Slack](https://docs.aws.amazon.com/awssupport/latest/user/support-app-commands.html)

You can also use the Amazon Web Services Management Console instead of the Amazon Web Services Support App API to manage your Slack configurations. For more information, see [Authorize a Slack workspace to enable the Amazon Web Services Support App](https://docs.aws.amazon.com/awssupport/latest/user/authorize-slack-workspace.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-supportapp` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-supportapp = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_supportapp as supportapp;

#[::tokio::main]
async fn main() -> Result<(), supportapp::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_supportapp::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-supportapp/latest/aws_sdk_supportapp/client/struct.Client.html)
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

