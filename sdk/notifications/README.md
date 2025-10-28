# aws-sdk-notifications

The _User Notifications API Reference_ provides descriptions, API request parameters, and the JSON response for each of the User Notifications API actions.

User Notification control plane APIs are currently available in US East (Virginia) - us-east-1.

[GetNotificationEvent](https://docs.aws.amazon.com/notifications/latest/APIReference/API_GetNotificationEvent.html) and [ListNotificationEvents](https://docs.aws.amazon.com/notifications/latest/APIReference/API_ListNotificationEvents.html) APIs are currently available in [commercial partition Regions](https://docs.aws.amazon.com/notifications/latest/userguide/supported-regions.html) and only return notifications stored in the same Region in which they're called.

The User Notifications console can only be used in US East (Virginia). Your data however, is stored in each Region chosen as a [notification hub](https://docs.aws.amazon.com/notifications/latest/userguide/notification-hubs.html) in addition to US East (Virginia).

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-notifications` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-notifications = "1.40.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_notifications as notifications;

#[::tokio::main]
async fn main() -> Result<(), notifications::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_notifications::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-notifications/latest/aws_sdk_notifications/client/struct.Client.html)
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

