# aws-sdk-georoutes

With the Amazon Location Routes API you can calculate routes and estimate travel time based on up-to-date road network and live traffic information.

Calculate optimal travel routes and estimate travel times using up-to-date road network and traffic data. Key features include:
  - Point-to-point routing with estimated travel time, distance, and turn-by-turn directions
  - Multi-point route optimization to minimize travel time or distance
  - Route matrices for efficient multi-destination planning
  - Isoline calculations to determine reachable areas within specified time or distance thresholds
  - Map-matching to align GPS traces with the road network

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-georoutes` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-georoutes = "1.36.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_georoutes as georoutes;

#[::tokio::main]
async fn main() -> Result<(), georoutes::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_georoutes::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-georoutes/latest/aws_sdk_georoutes/client/struct.Client.html)
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

