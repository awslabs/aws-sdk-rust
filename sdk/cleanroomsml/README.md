# aws-sdk-cleanroomsml

Welcome to the _Amazon Web Services Clean Rooms ML API Reference_.

Amazon Web Services Clean Rooms ML provides a privacy-enhancing method for two parties to identify similar users in their data without the need to share their data with each other. The first party brings the training data to Clean Rooms so that they can create and configure an audience model (lookalike model) and associate it with a collaboration. The second party then brings their seed data to Clean Rooms and generates an audience (lookalike segment) that resembles the training data.

To learn more about Amazon Web Services Clean Rooms ML concepts, procedures, and best practices, see the [Clean Rooms User Guide](https://docs.aws.amazon.com/clean-rooms/latest/userguide/machine-learning.html).

To learn more about SQL commands, functions, and conditions supported in Clean Rooms, see the [Clean Rooms SQL Reference](https://docs.aws.amazon.com/clean-rooms/latest/sql-reference/sql-reference.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-cleanroomsml` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-cleanroomsml = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_cleanroomsml as cleanroomsml;

#[::tokio::main]
async fn main() -> Result<(), cleanroomsml::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_cleanroomsml::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-cleanroomsml/latest/aws_sdk_cleanroomsml/client/struct.Client.html)
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

