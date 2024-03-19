# aws-sdk-sms

__Product update__

We recommend [Amazon Web Services Application Migration Service](http://aws.amazon.com/application-migration-service) (Amazon Web Services MGN) as the primary migration service for lift-and-shift migrations. If Amazon Web Services MGN is unavailable in a specific Amazon Web Services Region, you can use the Server Migration Service APIs through March 2023.

Server Migration Service (Server Migration Service) makes it easier and faster for you to migrate your on-premises workloads to Amazon Web Services. To learn more about Server Migration Service, see the following resources:
  - [Server Migration Service product page](http://aws.amazon.com/server-migration-service/)
  - [Server Migration Service User Guide](https://docs.aws.amazon.com/server-migration-service/latest/userguide/)

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-sms` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-sms = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_sms as sms;

#[::tokio::main]
async fn main() -> Result<(), sms::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_sms::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-sms/latest/aws_sdk_sms/client/struct.Client.html)
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

