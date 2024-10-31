# aws-sdk-geomaps

Integrate high-quality base map data into your applications using [MapLibre](https://maplibre.org). Capabilities include:
  - Access to comprehensive base map data, allowing you to tailor the map display to your specific needs.
  - Multiple pre-designed map styles suited for various application types, such as navigation, logistics, or data visualization.
  - Generation of static map images for scenarios where interactive maps aren't suitable, such as:
    - Embedding in emails or documents
    - Displaying in low-bandwidth environments
    - Creating printable maps
    - Enhancing application performance by reducing client-side rendering

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-geomaps` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-geomaps = "1.1.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_geomaps as geomaps;

#[::tokio::main]
async fn main() -> Result<(), geomaps::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_geomaps::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-geomaps/latest/aws_sdk_geomaps/client/struct.Client.html)
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

