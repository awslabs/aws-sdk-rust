# aws-sdk-health

The Health API provides access to the Health information that appears in the [Health Dashboard](https://health.aws.amazon.com/health/home). You can use the API operations to get information about events that might affect your Amazon Web Services and resources.

You must have a Business, Enterprise On-Ramp, or Enterprise Support plan from [Amazon Web Services Support](http://aws.amazon.com/premiumsupport/) to use the Health API. If you call the Health API from an Amazon Web Services account that doesn't have a Business, Enterprise On-Ramp, or Enterprise Support plan, you receive a SubscriptionRequiredException error.

For API access, you need an access key ID and a secret access key. Use temporary credentials instead of long-term access keys when possible. Temporary credentials include an access key ID, a secret access key, and a security token that indicates when the credentials expire. For more information, see [Best practices for managing Amazon Web Services access keys](https://docs.aws.amazon.com/general/latest/gr/aws-access-keys-best-practices.html) in the _Amazon Web Services General Reference_.

You can use the Health endpoint health.us-east-1.amazonaws.com (HTTPS) to call the Health API operations. Health supports a multi-Region application architecture and has two regional endpoints in an active-passive configuration. You can use the high availability endpoint example to determine which Amazon Web Services Region is active, so that you can get the latest information from the API. For more information, see [Accessing the Health API](https://docs.aws.amazon.com/health/latest/ug/health-api.html) in the _Health User Guide_.

For authentication of requests, Health uses the [Signature Version 4 Signing Process](https://docs.aws.amazon.com/general/latest/gr/signature-version-4.html).

If your Amazon Web Services account is part of Organizations, you can use the Health organizational view feature. This feature provides a centralized view of Health events across all accounts in your organization. You can aggregate Health events in real time to identify accounts in your organization that are affected by an operational event or get notified of security vulnerabilities. Use the organizational view API operations to enable this feature and return event information. For more information, see [Aggregating Health events](https://docs.aws.amazon.com/health/latest/ug/aggregate-events.html) in the _Health User Guide_.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-health` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.4", features = ["behavior-version-latest"] }
aws-sdk-health = "1.12.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_health as health;

#[::tokio::main]
async fn main() -> Result<(), health::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_health::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-health/latest/aws_sdk_health/client/struct.Client.html)
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

