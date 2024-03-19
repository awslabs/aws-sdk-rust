# aws-sdk-elasticsearch

Use the Amazon Elasticsearch Configuration API to create, configure, and manage Elasticsearch domains.

For sample code that uses the Configuration API, see the [Amazon Elasticsearch Service Developer Guide](https://docs.aws.amazon.com/elasticsearch-service/latest/developerguide/es-configuration-samples.html). The guide also contains [sample code for sending signed HTTP requests to the Elasticsearch APIs](https://docs.aws.amazon.com/elasticsearch-service/latest/developerguide/es-request-signing.html).

The endpoint for configuration service requests is region-specific: es._region_.amazonaws.com. For example, es.us-east-1.amazonaws.com. For a current list of supported regions and endpoints, see [Regions and Endpoints](http://docs.aws.amazon.com/general/latest/gr/rande.html#elasticsearch-service-regions).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-elasticsearch` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-elasticsearch = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_elasticsearch as elasticsearch;

#[::tokio::main]
async fn main() -> Result<(), elasticsearch::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_elasticsearch::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-elasticsearch/latest/aws_sdk_elasticsearch/client/struct.Client.html)
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

