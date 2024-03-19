# aws-sdk-codestarnotifications

This AWS CodeStar Notifications API Reference provides descriptions and usage examples of the operations and data types for the AWS CodeStar Notifications API. You can use the AWS CodeStar Notifications API to work with the following objects:

Notification rules, by calling the following:
  - CreateNotificationRule, which creates a notification rule for a resource in your account.
  - DeleteNotificationRule, which deletes a notification rule.
  - DescribeNotificationRule, which provides information about a notification rule.
  - ListNotificationRules, which lists the notification rules associated with your account.
  - UpdateNotificationRule, which changes the name, events, or targets associated with a notification rule.
  - Subscribe, which subscribes a target to a notification rule.
  - Unsubscribe, which removes a target from a notification rule.

Targets, by calling the following:
  - DeleteTarget, which removes a notification rule target from a notification rule.
  - ListTargets, which lists the targets associated with a notification rule.

Events, by calling the following:
  - ListEventTypes, which lists the event types you can include in a notification rule.

Tags, by calling the following:
  - ListTagsForResource, which lists the tags already associated with a notification rule in your account.
  - TagResource, which associates a tag you provide with a notification rule in your account.
  - UntagResource, which removes a tag from a notification rule in your account.

For information about how to use AWS CodeStar Notifications, see the [Amazon Web Services Developer Tools Console User Guide](https://docs.aws.amazon.com/dtconsole/latest/userguide/what-is-dtconsole.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-codestarnotifications` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-codestarnotifications = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_codestarnotifications as codestarnotifications;

#[::tokio::main]
async fn main() -> Result<(), codestarnotifications::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_codestarnotifications::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-codestarnotifications/latest/aws_sdk_codestarnotifications/client/struct.Client.html)
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

